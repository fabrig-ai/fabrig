//! Feed the domain events, inspect the requests that come out.

mod builtin;

mod accounting;
mod program;
mod restore;
mod retry;
mod tell;

use alloc::boxed::Box;
use core::mem::size_of;

use skein_lib::{Duration, Env, List, Queue, Time, Token, Wall, bytes as owned_bytes};
use smith_domain_tools::{
    self as tools, Authority, Call, Done, Effect, Grants, Name, Op, Outcome, Part, Path, Place, Repo, Version,
};

use crate::llm::{
    Block, Completion, Decoded, Descriptor, Endpoint, Failure, Message, Problem, Prompt, Returned, Role, Stop, Usage,
};
use crate::{
    Budget, Dimension, Domain, End, Event, FactKind as Fact, Limits, MAX_PARALLEL, Request, Spec, Yield, fire, resume,
    step, valid,
};

use crate::fixture::{BUDGET, SMALL as LIMITS, TOOLS};

const OUT_OF_TIME: End = End::Budget { spent: Dimension::Time };

/// The domain, its environment, its output, and the `Used`
/// it has reported.
struct Harness {
    observed: Queue<crate::facts::Fact>,
    domain: Domain,
    env: Env<Limits>,
    out: Queue<Request>,
    turns: u32,
    usage: Usage,
    turn_records: List<crate::record::Turn>,
    prices_reported: u32,
}

impl Harness {
    fn keep_facts(&mut self) {
        while let Some(fact) = self.domain.pop_fact() {
            self.observed.push(fact);
        }
    }

    fn pop_fact(&mut self) -> Option<crate::facts::Fact> {
        match self.observed.pop() {
            Some(fact) => Some(fact),
            None => self.domain.pop_fact(),
        }
    }

    fn new(limits: &Limits) -> Harness {
        Harness {
            observed: Queue::uncapped(),
            domain: Domain::new(limits, 1),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits },
            out: Queue::uncapped(),
            turns: 0,
            usage: Usage::ZERO,

            turn_records: List::with_capacity(
                limits.sessions.checked_mul(limits.history_messages).expect("all fixture turns fit"),
            ),
            prices_reported: 0,
        }
    }

    /// Steps the domain with `event`, which emits at most one request besides
    /// `Used`.
    fn step(&mut self, event: Event) -> Option<Request> {
        self.keep_facts();
        step(&mut self.domain, &self.env, event, &mut self.out);
        self.one()
    }

    /// Fires the alarm that is due, which emits at most one request besides
    /// `Used`.
    fn fire(&mut self) -> Option<Request> {
        assert!(self.domain.is_due(self.env.now), "an alarm is due");
        self.keep_facts();
        fire(&mut self.domain, &self.env, &mut self.out);
        self.one()
    }

    /// Starts the session that is ready, which emits at most one request
    /// besides `Used`.
    fn resume(&mut self) -> Option<Request> {
        assert!(self.domain.is_ready(), "a session is ready");
        self.keep_facts();
        resume(&mut self.domain, &self.env, &mut self.out);
        self.one()
    }

    /// The one request out besides `Used`, which is tallied.
    fn one(&mut self) -> Option<Request> {
        let mut one = None;
        while let Some(request) = self.next() {
            match request {
                Request::Turn { .. } | Request::Priced { .. } => unreachable!("next observes records"),
                Request::Used { opener: _, usage, .. } => {
                    assert!(one.is_none(), "a completion's usage comes first");
                    self.turns = self.turns.checked_add(1).expect("bounded completions");
                    self.usage = usage_sum(self.usage, usage).expect("only accepted usage is reported");
                }
                request @ (Request::JobsCancelled { .. }
                | Request::Opened { .. }
                | Request::Yielded { .. }
                | Request::Ended { .. }
                | Request::Complete { .. }
                | Request::Cancel { .. }
                | Request::Yield { .. }
                | Request::Io { .. }
                | Request::Job { .. }
                | Request::CancelIo { .. }
                | Request::Delegate { .. }
                | Request::Withdraw { .. }) => {
                    assert!(one.is_none(), "one request at most besides Used");
                    one = Some(request);
                }
            }
        }
        one
    }

    /// Observe the concrete record outputs independently from lifecycle/effects.
    fn next(&mut self) -> Option<Request> {
        while let Some(request) = self.out.pop() {
            match request {
                Request::Priced { spent, own_spent, .. } => {
                    assert_eq!((spent, own_spent), (0, 0));
                    self.prices_reported = self.prices_reported.checked_add(1).expect("bounded fixture prices");
                }
                Request::Turn { turn, .. } => {
                    assert_eq!(turn.version, crate::record::VERSION);
                    assert_eq!((turn.endpoint, turn.dialect), (Some(spec().endpoint), 1));
                    assert_eq!(turn.spent, 0);
                    assert!(!turn.messages.is_empty());
                    for message in &turn.messages {
                        for block in &message.content {
                            if let Block::ToolCall { call, .. } = block {
                                assert_eq!(call, &Decoded::Historical, "saved turns resolve all live call tickets");
                            }
                        }
                    }
                    self.turn_records.push(turn).expect("configured fixture turn observation bound");
                }
                other @ (Request::JobsCancelled { .. }
                | Request::Opened { .. }
                | Request::Yielded { .. }
                | Request::Used { .. }
                | Request::Ended { .. }
                | Request::Complete { .. }
                | Request::Cancel { .. }
                | Request::Yield { .. }
                | Request::Io { .. }
                | Request::Job { .. }
                | Request::CancelIo { .. }
                | Request::Delegate { .. }
                | Request::Withdraw { .. }) => return Some(other),
            }
        }
        None
    }

    /// Steps the domain with `event`, which starts or cancels a batch of tool
    /// runs, and returns the tokens of the runs it names, in order.
    fn batch(&mut self, event: Event) -> List<Token> {
        self.keep_facts();
        step(&mut self.domain, &self.env, event, &mut self.out);
        let mut runs = List::with_capacity(self.out.len());
        while let Some(request) = self.next() {
            match request {
                Request::Turn { .. } | Request::Priced { .. } => unreachable!("next observes records"),
                Request::Io { owner, .. }
                | Request::Job { owner, .. }
                | Request::CancelIo { owner }
                | Request::Delegate { owner, .. }
                | Request::Withdraw { owner } => {
                    runs.push(owner).expect("room for a batch");
                }
                Request::Used { .. } => {}
                other @ (Request::JobsCancelled { .. }
                | Request::Opened { .. }
                | Request::Yielded { .. }
                | Request::Ended { .. }
                | Request::Complete { .. }
                | Request::Cancel { .. }
                | Request::Yield { .. }) => panic!("expected a batch, not {other:?}"),
            }
        }
        runs
    }

    /// Opens a session for `opener`, returning the session's name, which also
    /// owns its calls, and its first prompt.
    fn open(&mut self, opener: u64) -> (Token, Prompt) {
        self.open_with(opener, spec())
    }

    fn open_with(&mut self, opener: u64, spec: Spec) -> (Token, Prompt) {
        step(
            &mut self.domain,
            &self.env,
            Event::Open { opener: Token::new(opener), opening: opening(spec) },
            &mut self.out,
        );
        let Some(Request::Opened { opener: to, session }) = self.next() else {
            panic!("expected the session to open");
        };
        assert_eq!(to.raw(), opener);
        let (owner, prompt) = calling(self.one());
        assert_eq!(owner, session, "a session's calls are its own");
        (session, prompt)
    }

    /// Opens a session for opener 1, and has the LLM finish its turn.
    fn yielded(&mut self) -> Token {
        self.yielded_with(spec(), done())
    }

    /// Opens a session for `spec` for opener 1, and has the LLM yield with
    /// `answer`.
    fn yielded_with(&mut self, spec: Spec, answer: Completion) -> Token {
        let (session, _) = self.open_with(1, spec);
        drop(yielded(self.step(Event::Completed { owner: session, completion: answer })));
        session
    }

    /// Drains the facts told so far, checking that they are `expected`.
    fn told(&mut self, expected: &[Fact]) {
        for fact in expected {
            assert_eq!(pop_kind(self).as_ref(), Some(fact));
        }
        assert_eq!(self.pop_fact(), None, "nothing more was told");
    }

    fn after(&mut self, span: Duration) {
        self.env.now = self.env.now.checked_add(span).expect("the test stays in range");
    }
}

fn bytes(text: &[u8]) -> Box<[u8]> {
    Box::from(text)
}

fn spec() -> Spec {
    Spec {
        backend: crate::record::Backend::Direct,
        effort: None,
        final_turn: None,
        window: 65_536,
        line: 64_512,
        summary: b"Summarise the work so far.".as_slice().into(),
        preamble: b"Earlier window summary:".as_slice().into(),
        affinity: crate::llm::Affinity { key: [0; 16], thread: 0 },
        protocol_allowance: 0,
        progress: crate::Progress { head: Duration::from_secs(30), idle: Duration::from_secs(30) },
        account: Some(0),
        endpoint: Endpoint(0),
        model: bytes(b"model"),
        system: bytes(b"be brief"),
        authority: authority(1),
        delegated: Box::new([FINISH]),
        prompt: bytes(b"fix the bug"),
        output: 1024,
        budget: BUDGET,
    }
}

/// The root of the checkout's one repository, as io names it.
const ROOT: Token = Token::new(70);

/// What a spec's kit may do: inspect and modify, in `repos` repositories
/// mounted side by side under `/work`, the first at `/work` itself.
fn authority(repos: u8) -> Authority {
    let mut mounted = List::with_capacity(u32::from(repos));
    for repo in 0..repos {
        let mount: Box<[Name]> = match repo.checked_sub(1) {
            None => Box::new([named(b"work")]),
            Some(other) => Box::new([named(b"other"), named(&[b'a'.checked_add(other).expect("few")])]),
        };
        let root = Token::new(ROOT.raw().checked_add(u64::from(repo)).expect("few"));
        mounted.push(Repo { host: Box::new([named(b"host")]), mount, root, writable: true }).expect("room for each");
    }
    Authority {
        cwd: Box::new([named(b"work")]),
        repos: mounted.into_boxed(),
        grants: Grants { inspect: true, modify: true, shell: false },
    }
}

fn named(text: &[u8]) -> Name {
    Name::new(bytes(text)).expect("a test name is a name")
}

/// Where an operation loads, for a read.
fn loading(op: &Op) -> Place {
    let Op::Load { at, max: _ } = op else {
        panic!("expected a load, not {op:?}");
    };
    at.clone()
}

/// Where the tools find the file `name` of the repository.
fn place(name: &[u8]) -> Place {
    Place { root: ROOT, path: bytes(name) }
}

fn budget(budget: Budget) -> Spec {
    Spec { budget, ..spec() }
}

fn text(text: &[u8]) -> Block {
    Block::Text { text: bytes(text), replay: None }
}

/// A relative path of one name.
fn path(name: &[u8]) -> Path {
    Path { absolute: false, parts: Box::new([Part::Name { name: named(name) }]) }
}

/// A path outside the checkout.
fn outside() -> Path {
    Path { absolute: true, parts: Box::new([Part::Name { name: named(b"etc") }]) }
}

/// Lists the working directory.
fn list() -> Call {
    Call::List {
        path: Path { absolute: false, parts: Box::new([Part::Current]) },
        depth: smith_domain_tools::Depth::Whole,
        glob: None,
    }
}

/// Reads `name`.
fn cat(name: &[u8]) -> Call {
    Call::Read { path: path(name), offset: 1, window: tools::Budget::Default }
}

/// The LLM's call `id` of `call`, as the protocol layer decoded it.
fn tool_call(id: &[u8], call: Call) -> Block {
    Block::ToolCall {
        id: bytes(id),
        name: bytes(b"tool"),
        input: bytes(b"{}"),
        call: Decoded::Owned { call },
        replay: None,
    }
}

/// The LLM's call `id`, which the protocol layer could not decode.
fn invalid(id: &[u8], problem: Problem) -> Block {
    Block::ToolCall {
        id: bytes(id),
        name: bytes(b"tool"),
        input: bytes(b"{"),
        call: Decoded::Invalid { descriptor: None, problem },
        replay: None,
    }
}

/// Writes `name`.
fn write(name: &[u8]) -> Call {
    Call::Write { path: path(name), content: bytes(b"x") }
}

/// The tool the opener serves in every spec: a finish, which writes.
const FINISH: Descriptor = Descriptor { ticket: Token::new(7), effect: Effect::Write };

/// The LLM's call `id` to a tool the opener serves, kept under `ticket`.
fn delegated(id: &[u8], ticket: u64, effect: Effect) -> Block {
    let call = Decoded::Delegated {
        descriptor: Token::new(ticket),
        source: crate::ToolSource::Run,
        ticket: Token::new(ticket),
        effect,
    };
    Block::ToolCall { id: bytes(id), name: bytes(b"finish"), input: bytes(b"{}"), call, replay: None }
}

/// A concrete answer preserves the old ticket's distinct payload and length.
fn answer(ticket: u64, length: u64) -> Box<[u8]> {
    let mut payload = owned_bytes::zeroed(usize::try_from(length).expect("fixture length"));
    let fill = u8::try_from(ticket).expect("fixture byte");
    payload.fill(fill);
    payload
}

fn answered(owner: Token, ticket: u64, length: u64, error: bool) -> Event {
    Event::Answered { owner, text: answer(ticket, length), error, spent: 0 }
}

fn answer_cancelled(owner: Token) -> Event {
    Event::AnswerCancelled { owner, spent: 0 }
}

fn opening(spec: Spec) -> Box<crate::record::Opening> {
    Box::new(crate::record::Opening {
        window_llm: None,
        spec,
        dialect: 1,
        prices: crate::record::Prices { input: 0, cached: 0, cache_write: None, output: 0, unit: 1 },
        budget: 100,
        transcript: None,
        answered: Box::default(),
    })
}

/// Preserve the pressure bytes and add this scene's provider/result
/// reservation envelope; all payload lengths remain unchanged.
fn receiving_limits(pressure: u64, content: &[Block], result_credit: u64) -> Limits {
    let limits = Limits {
        completion_bytes: crate::session::content_cost(content).expect("fixture completion fits"),
        completion_blocks: u32::try_from(content.len()).expect("fixture blocks fit"),
        ..LIMITS
    };
    let envelope = crate::completion_reserve(&limits).expect("provider bounds fit").max(result_credit);
    let cells = u64::try_from(size_of::<Block>())
        .expect("block")
        .checked_add(64_u64.checked_mul(u64::from(limits.completion_blocks)).expect("bounded calls"))
        .expect("headroom");
    Limits {
        history_bytes: pressure
            .checked_add(envelope)
            .expect("fixture envelope")
            .checked_add(cells)
            .expect("summary and future results"),
        ..limits
    }
}

fn initial_bytes() -> u64 {
    let spec = spec();
    let model = u64::try_from(spec.model.len()).expect("fixture model fits");
    let system = u64::try_from(spec.system.len()).expect("fixture system fits");
    let prompt = u64::try_from(spec.prompt.len()).expect("fixture prompt fits");
    let descriptor = u64::try_from(size_of::<Descriptor>()).expect("descriptor fits");
    let descriptors = descriptor
        .checked_mul(u64::try_from(spec.delegated.len()).expect("descriptor count fits"))
        .expect("descriptors fit");
    let block = u64::try_from(size_of::<Block>()).expect("block fits");
    let names = model.checked_add(system).expect("fixture names fit");
    let payload = names
        .checked_add(
            prompt
                .checked_mul(2)
                .expect("task copied")
                .checked_add(u64::try_from(spec.summary.len()).expect("summary fits"))
                .expect("summary fits")
                .checked_add(u64::try_from(spec.preamble.len()).expect("preamble fits"))
                .expect("retained instructions fit"),
        )
        .expect("fixture opening payload fits");
    let described = payload.checked_add(descriptors).expect("fixture descriptors fit");
    described.checked_add(block).expect("fixture initial block fits")
}

fn usage_sum(previous: Usage, usage: Usage) -> Option<Usage> {
    Some(Usage {
        input_tokens: Some(previous.input_tokens.unwrap_or(0).checked_add(usage.input_tokens.unwrap_or(0))?),
        output_tokens: Some(previous.output_tokens.unwrap_or(0).checked_add(usage.output_tokens.unwrap_or(0))?),
        cache_read_tokens: Some(
            previous.cache_read_tokens.unwrap_or(0).checked_add(usage.cache_read_tokens.unwrap_or(0))?,
        ),
        cache_write_tokens: Some(
            previous.cache_write_tokens.unwrap_or(0).checked_add(usage.cache_write_tokens.unwrap_or(0))?,
        ),
        reasoning_tokens: Some(0),
    })
}

/// A read that found `content`.
fn read(content: &[u8]) -> Outcome {
    Outcome::Read {
        content: bytes(content),
        first: 1,
        lines: 1,
        total: 1,
        cut: tools::Cut::Whole,
        next: tools::Next::End,
        window: tools::BytesClamp::Within,
    }
}

/// io loaded `content` for the operation of `owner`.
fn loaded(content: &[u8]) -> Done {
    Done::Loaded { content: bytes(content), version: Version::new([1, 0, 0, 0]) }
}

/// io loaded `content` for the operation of `owner`, the tools' read of a file.
fn ran(owner: Token, content: &[u8]) -> Event {
    Event::Done { owner, done: loaded(content) }
}

/// A fact the tools told, as the session for `opener` passes it on.
fn by_tools(opener: Token, fact: tools::FactKind) -> Fact {
    Fact::Tools { opener, fact }
}

fn result(id: &[u8], outcome: Outcome) -> Block {
    Block::ToolResult { id: bytes(id), result: Returned::Owned { outcome } }
}

/// What every completion uses, unless a test says otherwise.
const USAGE: Usage = Usage {
    input_tokens: Some(10),
    output_tokens: Some(5),
    cache_read_tokens: Some(3),
    cache_write_tokens: Some(2),
    reasoning_tokens: Some(0),
};

fn completion(content: Box<[Block]>, stop: Stop) -> Completion {
    Completion { reasoning_dropped: Box::default(), content, stop, usage: USAGE }
}

fn done() -> Completion {
    completion(Box::new([text(b"done")]), Stop::EndTurn)
}

/// The LLM reads `main.rs`.
fn reading() -> Completion {
    completion(Box::new([tool_call(b"c1", cat(b"main.rs"))]), Stop::ToolUse)
}

fn calling(request: Option<Request>) -> (Token, Prompt) {
    let Some(Request::Complete { owner, prompt, .. }) = request else {
        panic!("expected a call, not {request:?}");
    };
    (owner, prompt)
}

/// The operation the tools ask of io for a call, named by its owner.
fn running(request: Option<Request>) -> (Token, Op) {
    let Some(Request::Io { owner, op, deadline: _ }) = request else {
        panic!("expected an operation for the tools, not {request:?}");
    };
    (owner, op)
}

fn yielded(request: Option<Request>) -> (u64, Yield, Box<[u8]>) {
    let Some(Request::Yielded { opener, stop, text }) = request else {
        panic!("expected a yield, not {request:?}");
    };
    (opener.raw(), stop, text)
}

/// The end of opener 1's session, after `turns` completions that each used
/// `USAGE`.
fn ended(end: End, turns: u32) -> Request {
    let mut usage = Usage::ZERO;
    for _ in 0..turns {
        usage = usage_sum(usage, USAGE).expect("bounded fixture usage");
    }
    Request::Ended {
        opener: Token::new(1),
        end,
        turns,
        usage,
        reported: if turns == 0 { Usage::NONE } else { usage },
        detail: Box::default(),
    }
}

#[test]
fn a_session_runs_the_tools_it_is_asked_for_and_yields_once_the_llm_is_done() {
    let mut h = Harness::new(&LIMITS);
    let (owner, prompt) = h.open(1);
    assert_eq!(prompt.messages.len(), 1);
    assert_eq!(prompt.tools, spec().authority.grants);

    let content =
        Box::new([text(b"let me look"), tool_call(b"c1", cat(b"main.rs")), tool_call(b"c2", cat(b"gone.rs"))]);
    // Two reads: they run together, each the tools' load of its file.
    step(&mut h.domain, &h.env, Event::Completed { owner, completion: completion(content, Stop::ToolUse) }, &mut h.out);
    h.next().expect("the completion's usage");
    let (first, op) = running(h.next());
    assert_eq!(loading(&op), place(b"main.rs"));
    let (second, op) = running(h.one());
    assert_eq!(loading(&op), place(b"gone.rs"));
    assert_ne!(first, second, "each operation has a token of its own");
    // The second ends first: the results still go back in call order.
    assert_eq!(h.step(Event::Done { owner: second, done: Done::Missing }), None);
    let (_, prompt) = calling(h.step(ran(first, b"main.rs")));
    assert_eq!(prompt.messages.len(), 3);
    let results = [
        result(b"c1", read(b"main.rs")),
        result(b"c2", Outcome::NotFound { path: tools::Resolved::Working { path: bytes(b"gone.rs") } }),
    ];
    assert_eq!(&*prompt.messages[2].content, &results);

    let done = completion(Box::new([text(b"fixed "), text(b"it")]), Stop::EndTurn);
    assert_eq!(yielded(h.step(Event::Completed { owner, completion: done })), (1, Yield::Done, bytes(b"fixed it")));
    let expiry = Time::ZERO.saturating_add(BUDGET.time);
    assert_eq!(h.domain.next_deadline(), Some(expiry), "a yielded session still expires");

    // Its kit closes with it, at once, as it runs nothing.
    assert_eq!(h.step(Event::Close { session: owner }), Some(ended(End::Closed, 2)));
    assert_eq!(h.domain.next_deadline(), None);
    assert_eq!((h.domain.sessions(), h.domain.kits()), (1, 1));
    h.domain.reclaim();
    assert_eq!((h.domain.sessions(), h.domain.kits(), h.domain.runs()), (0, 0, 0));
}

#[test]
fn a_yielded_session_goes_on_with_the_openers_message() {
    let mut h = Harness::new(&LIMITS);
    let session = h.yielded();
    let (owner, prompt) = calling(h.step(Event::Continue { session, content: bytes(b"you have not finished") }));
    assert_eq!(owner, session);
    let messages = [
        Message { role: Role::User, content: Box::new([text(b"fix the bug")]) },
        Message { role: Role::Assistant, content: Box::new([text(b"done")]) },
        Message { role: Role::User, content: Box::new([text(b"you have not finished")]) },
    ];
    assert_eq!(&*prompt.messages, &messages);

    let again = completion(Box::new([text(b"finished")]), Stop::EndTurn);
    assert_eq!(yielded(h.step(Event::Completed { owner, completion: again })), (1, Yield::Done, bytes(b"finished")));
}

/// Opens a session whose LLM answers `content` with `stop`, and returns how it
/// yielded.
fn yield_on(stop: Stop, content: Box<[Block]>) -> (u64, Yield, Box<[u8]>) {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    yielded(h.step(Event::Completed { owner, completion: completion(content, stop) }))
}

#[test]
fn an_invalid_call_is_answered_with_its_problem_and_nothing_runs_for_it() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    let missing = Problem::Missing { field: bytes(b"path") };
    let content = Box::new([
        invalid(b"c1", Problem::NotAnObject),
        tool_call(b"c2", cat(b"main.rs")),
        invalid(b"c3", missing.clone()),
    ]);
    let (run, op) = running(h.step(Event::Completed { owner, completion: completion(content, Stop::ToolUse) }));
    assert_eq!(loading(&op), place(b"main.rs"));
    let (_, prompt) = calling(h.step(ran(run, b"main.rs")));
    let results = [
        Block::ToolResult { id: bytes(b"c1"), result: Returned::Invalid { problem: Problem::NotAnObject } },
        result(b"c2", read(b"main.rs")),
        Block::ToolResult { id: bytes(b"c3"), result: Returned::Invalid { problem: missing } },
    ];
    assert_eq!(&*prompt.messages[2].content, &results, "every call gets its result, in call order");

    let opener = Token::new(1);
    let started = Fact::CompletionStarted {
        window: 1,
        opener,
        attempt: 0,
        messages: 1,
        max_tokens: 1024,
        choice: crate::facts::Choice::Auto,
    };
    let kit = Fact::Tools { opener, fact: tools::FactKind::Opened { session: owner } };
    let opening = [pop_kind(&mut h), pop_kind(&mut h), pop_kind(&mut h)];
    assert_eq!(opening, [Some(Fact::Opened { opener }), Some(started), Some(kit)]);
    let answered = Fact::CompletionAnswered { opener, stop: Stop::ToolUse, blocks: 3, calls: 3, invalid: 2 };
    assert_eq!(pop_kind(&mut h), Some(answered));
}

#[test]
fn a_message_of_invalid_calls_waits_on_the_ready_list() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    let content = Box::new([invalid(b"c1", Problem::UnknownTool)]);
    assert_eq!(h.step(Event::Completed { owner, completion: completion(content, Stop::ToolUse) }), None);
    assert!(!h.domain.is_ready(), "the ready list waits for reclaim");
    h.domain.reclaim();
    let (_, prompt) = calling(h.resume());
    let answer = Block::ToolResult { id: bytes(b"c1"), result: Returned::Invalid { problem: Problem::UnknownTool } };
    assert_eq!(&*prompt.messages[2].content, &[answer]);
}

#[test]
fn a_call_too_large_for_the_agent_is_answered_so_like_any_invalid_call() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    let content = Box::new([invalid(b"c1", Problem::TooLarge)]);
    assert_eq!(h.step(Event::Completed { owner, completion: completion(content, Stop::ToolUse) }), None);
    h.domain.reclaim();
    let (_, prompt) = calling(h.resume());
    let answer = Block::ToolResult { id: bytes(b"c1"), result: Returned::Invalid { problem: Problem::TooLarge } };
    assert_eq!(&*prompt.messages[2].content, &[answer]);
}

#[test]
fn the_calls_a_yield_leaves_are_answered_when_the_opener_continues() {
    let mut h = Harness::new(&LIMITS);
    let cut = completion(Box::new([text(b"let me"), invalid(b"c1", Problem::NotAnObject)]), Stop::MaxTokens);
    let session = h.yielded_with(spec(), cut);
    let (_, prompt) = calling(h.step(Event::Continue { session, content: bytes(b"go on") }));
    let last = prompt.messages.last().expect("the opener's message goes last");
    let content = [
        Block::ToolResult { id: bytes(b"c1"), result: Returned::NotRun { why: crate::llm::NotRun::Stopped } },
        text(b"go on"),
    ];
    assert_eq!((last.role, &*last.content), (Role::User, &content[..]));

    // A turn that ended with a call it did not wait for: the call does not run.
    let mut h = Harness::new(&LIMITS);
    let ended = completion(Box::new([tool_call(b"c1", list())]), Stop::EndTurn);
    let session = h.yielded_with(spec(), ended);
    let (_, prompt) = calling(h.step(Event::Continue { session, content: bytes(b"go on") }));
    let last = prompt.messages.last().expect("the opener's message goes last");
    assert_eq!(
        last.content.first(),
        Some(&Block::ToolResult { id: bytes(b"c1"), result: Returned::NotRun { why: crate::llm::NotRun::Stopped } })
    );
}

#[test]
fn a_tool_call_carries_its_deadline_and_one_that_runs_out_of_time_goes_back_to_the_llm() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    h.after(Duration::from_secs(1));
    step(&mut h.domain, &h.env, Event::Completed { owner, completion: reading() }, &mut h.out);
    drop(h.next());
    let Some(Request::Io { owner: run, op: _, deadline }) = h.one() else {
        panic!("expected an operation for the tools");
    };
    assert_eq!(
        deadline,
        h.env.now.saturating_add(LIMITS.tools.file_timeout),
        "the file duration bounds the operation within its common call deadline"
    );
    // Whoever runs the call runs the race: the session arms nothing for it.
    assert_eq!(h.domain.next_deadline(), Some(Time::ZERO.saturating_add(BUDGET.time)));
    h.env.now = deadline;
    let (_, prompt) = calling(h.step(Event::Done { owner: run, done: Done::TimedOut }));
    assert_eq!(&*prompt.messages[2].content, &[result(b"c1", Outcome::TimedOut)]);
}

#[test]
fn adjacent_reads_run_together_and_a_write_runs_alone_and_results_go_back_in_order() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    let content = Box::new([
        tool_call(b"a", cat(b"a")),
        tool_call(b"b", cat(b"b")),
        tool_call(b"c", cat(b"c")),
        tool_call(b"d", write(b"d")),
        tool_call(b"e", cat(b"e")),
    ]);
    // Two reads at a time, as the limits allow.
    let first = h.batch(Event::Completed { owner, completion: completion(content, Stop::ToolUse) });
    let &[run_a, run_b] = first.as_slice() else { panic!("two reads, not {first:?}") };
    assert_eq!(h.step(ran(run_b, b"b")), None, "the batch waits for all its runs");
    // The read before the write, alone; the write; the read after it.
    let second = h.batch(ran(run_a, b"a"));
    let &[run_c] = second.as_slice() else { panic!("one read, not {second:?}") };
    let third = h.batch(ran(run_c, b"c"));
    let &[run_d] = third.as_slice() else { panic!("one write, not {third:?}") };
    let fourth = h.batch(Event::Done { owner: run_d, done: Done::Stored { version: Version::new([2, 0, 0, 0]) } });
    let &[run_e] = fourth.as_slice() else { panic!("one read, not {fourth:?}") };
    let (_, prompt) = calling(h.step(ran(run_e, b"e")));
    let results = [
        result(b"a", read(b"a")),
        result(b"b", read(b"b")),
        result(b"c", read(b"c")),
        result(b"d", Outcome::Written { created: true }),
        result(b"e", read(b"e")),
    ];
    assert_eq!(&*prompt.messages[2].content, &results);
}

#[test]
fn a_batch_the_tools_answer_at_once_goes_on_from_the_ready_list_after_the_reclaim_point() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    // Outside the checkout: the tools answer these at their entrance.
    let away = Call::Read { path: outside(), offset: 1, window: tools::Budget::Default };
    let over = Call::Write { path: outside(), content: bytes(b"x") };
    let content = Box::new([tool_call(b"c1", away), tool_call(b"c2", over), tool_call(b"c3", cat(b"main.rs"))]);
    // The first batch is answered within the step that started it: the
    // session rests, and is ready once the iteration's reclaim point passed.
    assert_eq!(h.step(Event::Completed { owner, completion: completion(content, Stop::ToolUse) }), None);
    assert!(!h.domain.is_ready(), "not before the reclaim point");
    assert_eq!(h.domain.next_deadline(), Some(Time::ZERO.saturating_add(BUDGET.time)), "no alarm but expiry");
    h.domain.reclaim();
    assert_eq!(h.domain.runs(), 0, "the runs answered at once are reclaimed before the next batch");
    assert_eq!(h.resume(), None, "the write is answered at once too");
    assert!(!h.domain.is_ready(), "it rests again until the next reclaim point");
    h.domain.reclaim();
    let (load, op) = running(h.resume());
    assert!(!h.domain.is_ready());
    assert_eq!(loading(&op), place(b"main.rs"));
    let (_, prompt) = calling(h.step(ran(load, b"main.rs")));
    let results = [
        result(b"c1", Outcome::Outside { path: tools::Resolved::Outside { path: bytes(b"etc") } }),
        result(b"c2", Outcome::Outside { path: tools::Resolved::Outside { path: bytes(b"etc") } }),
        result(b"c3", read(b"main.rs")),
    ];
    assert_eq!(&*prompt.messages[2].content, &results);
}

#[test]
fn a_session_closed_while_it_rests_leaves_the_ready_list() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    let away = Call::Read { path: outside(), offset: 1, window: tools::Budget::Default };
    let over = Call::Write { path: outside(), content: bytes(b"x") };
    let content = Box::new([tool_call(b"c1", away), tool_call(b"c2", over)]);
    assert_eq!(h.step(Event::Completed { owner, completion: completion(content, Stop::ToolUse) }), None);
    assert_eq!(h.step(Event::Close { session: owner }), Some(ended(End::Closed, 1)));
    h.domain.reclaim();
    assert!(!h.domain.is_ready(), "a closed session is not resumed");
}

#[test]
fn a_maximum_legal_batch_dispatches_every_call() {
    let limits = Limits { parallel_tools: MAX_PARALLEL, ..LIMITS };
    let mut h = Harness::new(&limits);
    let (owner, _) = h.open(1);
    let mut content = List::with_capacity(MAX_PARALLEL);
    for _ in 0..MAX_PARALLEL {
        content.push(tool_call(b"c", list())).expect("room for every call");
    }
    let completion = completion(content.into_boxed(), Stop::ToolUse);
    h.keep_facts();
    step(&mut h.domain, &h.env, Event::Completed { owner, completion }, &mut h.out);
    assert_eq!(h.out.len(), 2 + MAX_PARALLEL, "the price, usage, and an operation for each call");
    while h.out.pop().is_some() {}
    assert!(!valid(&Limits { parallel_tools: 0, ..LIMITS }));
    assert!(!valid(&Limits { parallel_tools: MAX_PARALLEL + 1, ..LIMITS }));
    // A batch the tools could not take whole, and a session without a kit.
    assert!(!valid(&Limits { tools: tools::Limits { calls: 1, ..TOOLS }, ..LIMITS }));
    assert!(!valid(&Limits { tools: tools::Limits { kits: 1, ..TOOLS }, ..LIMITS }));
}

#[test]
fn closing_a_session_with_a_batch_in_flight_closes_its_kit_and_waits_for_every_run() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    let content = Box::new([tool_call(b"a", cat(b"a")), tool_call(b"b", cat(b"b"))]);
    let runs = h.batch(Event::Completed { owner, completion: completion(content, Stop::ToolUse) });
    // The kit's close cancels the operation of each call it runs.
    let cancels = h.batch(Event::Close { session: owner });
    assert_eq!(cancels.as_slice(), runs.as_slice());
    let &[a, b] = runs.as_slice() else { panic!("two reads, not {runs:?}") };
    assert_eq!(h.step(Event::Done { owner: a, done: Done::Cancelled }), None);
    // The other won its race; with it, the kit has closed.
    assert_eq!(h.step(ran(b, b"late")), Some(ended(End::Closed, 1)));
    assert_eq!(h.domain.next_deadline(), None);
}

#[test]
fn maximum_owned_results_fit_when_they_win_cancellation() {
    let content = Box::new([tool_call(b"a", cat(b"a")), tool_call(b"b", cat(b"b"))]);
    let batch_credit = u64::from(TOOLS.read_bytes).checked_mul(2).expect("two full read results fit");
    let mut h = Harness::new(&receiving_limits(2048, &content[..], batch_credit));
    let (owner, _) = h.open(1);
    let runs = h.batch(Event::Completed { owner, completion: completion(content, Stop::ToolUse) });
    let &[a, b] = runs.as_slice() else { panic!("two reads, not {runs:?}") };
    assert_eq!(h.batch(Event::Close { session: owner }).as_slice(), runs.as_slice());
    assert_eq!(h.step(ran(a, &[b'x'; 4096])), None, "the admitted maximum result wins cancellation");
    assert_eq!(h.step(Event::Done { owner: b, done: Done::Cancelled }), Some(ended(End::Closed, 1)));
    let turn = &h.turn_records.get(0).expect("observed turn");
    assert_eq!(turn.messages[2].content[0], result(b"a", read(&[b'x'; 4096])));
}

#[test]
fn a_result_is_charged_its_id_with_its_call_so_a_refusal_at_the_entrance_always_fits() {
    let away = Call::Read { path: outside(), offset: 1, window: tools::Budget::Default };
    let large = Box::new([tool_call(b"a", cat(b"main.rs")), tool_call(&[b'i'; 2100], away.clone())]);
    let credit = u64::from(TOOLS.read_bytes).checked_mul(2).expect("two read credits fit");
    let limits = receiving_limits(4096, &large[..], credit);
    let mut h = Harness::new(&limits);
    let (owner, _) = h.open(1);
    let id = [b'i'; 1500];
    let content = Box::new([tool_call(b"a", cat(b"main.rs")), tool_call(&id, away)]);
    let runs = h.batch(Event::Completed { owner, completion: completion(content, Stop::ToolUse) });
    let &[a] = runs.as_slice() else { panic!("the read alone runs, not {runs:?}") };
    let (_, prompt) = calling(h.step(ran(a, b"main.rs")));
    let expected = [
        result(b"a", read(b"main.rs")),
        result(&id, Outcome::Outside { path: tools::Resolved::Outside { path: bytes(b"etc") } }),
    ];
    assert_eq!(&*prompt.messages[2].content, &expected);
    assert_eq!(
        &*h.turn_records.get(0).expect("observed turn").messages[2].content,
        &expected,
        "full IDs survive concrete Turn emission"
    );
}

#[test]
fn the_tools_tell_of_each_call_and_the_session_passes_it_on() {
    let mut h = Harness::new(&LIMITS);
    let opener = Token::new(1);
    let (owner, _) = h.open(1);
    let content = Box::new([text(b"two"), tool_call(b"a", cat(b"a")), tool_call(b"b", cat(b"b"))]);
    let runs = h.batch(Event::Completed { owner, completion: completion(content, Stop::ToolUse) });
    assert_eq!(h.step(ran(runs.as_slice()[1], b"b")), None);
    let read = tools::Tool::Read;
    h.told(&[
        Fact::Opened { opener },
        Fact::CompletionStarted {
            window: 1,
            opener,
            attempt: 0,
            messages: 1,
            max_tokens: 1024,
            choice: crate::facts::Choice::Auto,
        },
        by_tools(opener, tools::FactKind::Opened { session: owner }),
        Fact::CompletionAnswered { opener, stop: Stop::ToolUse, blocks: 3, calls: 2, invalid: 0 },
        Fact::Used { opener, usage: USAGE },
        by_tools(opener, tools::FactKind::Started { session: owner, tool: read }),
        by_tools(opener, tools::FactKind::Started { session: owner, tool: read }),
        by_tools(
            opener,
            tools::FactKind::Answered {
                session: owner,
                tool: read,
                verdict: tools::Verdict::Read,
                bytes: 1,
                omitted_bytes: 0,
                job: None,
                clamped: false,
            },
        ),
    ]);
}

#[test]
fn a_delegated_call_goes_to_the_opener_and_its_answer_goes_back_to_the_llm() {
    let mut h = Harness::new(&LIMITS);
    let (owner, prompt) = h.open(1);
    assert_eq!(&*prompt.delegated, &[FINISH], "the prompt offers what the opener serves");
    let content = Box::new([delegated(b"c1", 9, Effect::Write)]);
    step(&mut h.domain, &h.env, Event::Completed { owner, completion: completion(content, Stop::ToolUse) }, &mut h.out);
    drop(h.next());
    let Some(Request::Delegate { owner: run, opener, call, deadline, origin: _ }) = h.one() else {
        panic!("expected the call delegated");
    };
    assert_eq!((opener, call), (Token::new(1), Token::new(9)));
    assert_eq!(deadline, h.env.now.saturating_add(BUDGET.time), "the opener races it against the session's time");
    let (_, prompt) = calling(h.step(answered(run, 11, 20, true)));
    let result = Block::ToolResult {
        id: bytes(b"c1"),
        result: Returned::Text { text: answer(11, 20), error: true, replay: None },
    };
    assert_eq!(&*prompt.messages[2].content, &[result]);
}

#[test]
fn owned_and_delegated_calls_run_in_one_order_of_batches() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    let content = Box::new([
        tool_call(b"a", cat(b"a")),
        delegated(b"b", 9, Effect::Read),
        delegated(b"c", 10, Effect::Write),
        tool_call(b"d", cat(b"d")),
    ]);
    // A read of the tools and one the opener serves, together.
    step(&mut h.domain, &h.env, Event::Completed { owner, completion: completion(content, Stop::ToolUse) }, &mut h.out);
    drop(h.next());
    let Some(Request::Io { owner: read_a, .. }) = h.next() else { panic!("expected the tools' read") };
    let Some(Request::Delegate { owner: read_b, call, .. }) = h.one() else { panic!("expected the opener's read") };
    assert_eq!(call, Token::new(9));
    assert_eq!(h.step(answered(read_b, 20, 3, false)), None);
    // The write alone, then the last read.
    let Some(Request::Delegate { owner: write_c, call, .. }) = h.step(ran(read_a, b"a")) else {
        panic!("expected the opener's write");
    };
    assert_eq!(call, Token::new(10));
    let (read_d, _) = running(h.step(answered(write_c, 21, 3, false)));
    let (_, prompt) = calling(h.step(ran(read_d, b"d")));
    let results = [
        result(b"a", read(b"a")),
        Block::ToolResult {
            id: bytes(b"b"),
            result: Returned::Text { text: answer(20, 3), error: false, replay: None },
        },
        Block::ToolResult {
            id: bytes(b"c"),
            result: Returned::Text { text: answer(21, 3), error: false, replay: None },
        },
        result(b"d", read(b"d")),
    ];
    assert_eq!(&*prompt.messages[2].content, &results);
}

#[test]
fn closing_withdraws_the_delegated_calls_in_flight_and_waits_for_their_terminals() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    let content = Box::new([tool_call(b"a", cat(b"a")), delegated(b"b", 9, Effect::Read)]);
    let runs = h.batch(Event::Completed { owner, completion: completion(content, Stop::ToolUse) });
    let &[read_a, read_b] = runs.as_slice() else { panic!("two runs, not {runs:?}") };
    // The withdraw, then the kit's close, which cancels the tools' read.
    step(&mut h.domain, &h.env, Event::Close { session: owner }, &mut h.out);
    assert_eq!(h.next(), Some(Request::Withdraw { owner: read_b }));
    assert_eq!(h.next(), Some(Request::CancelIo { owner: read_a }));
    // The answer won its race with the withdraw; the session still waits.
    assert_eq!(h.step(answered(read_b, 20, 3, false)), None);
    let end = h.step(Event::Done { owner: read_a, done: Done::Cancelled });
    assert_eq!(end, Some(ended(End::Closed, 1)));

    // And as the session runs out of time.
    let (owner, _) = h.open(1);
    let content = Box::new([delegated(b"b", 9, Effect::Read)]);
    let runs = h.batch(Event::Completed { owner, completion: completion(content, Stop::ToolUse) });
    h.after(BUDGET.time);
    assert_eq!(h.fire(), Some(Request::Withdraw { owner: runs.as_slice()[0] }));
    assert_eq!(h.step(answer_cancelled(runs.as_slice()[0])), Some(ended(OUT_OF_TIME, 1)));
}

#[test]
fn maximum_delegated_results_fit_when_they_win_cancellation() {
    let content = Box::new([delegated(b"b", 9, Effect::Read), tool_call(b"a", cat(b"a"))]);
    let credit = LIMITS.delegated_result_bytes.checked_add(u64::from(TOOLS.read_bytes)).expect("mixed batch fits");
    for length in [4096, LIMITS.delegated_result_bytes] {
        let mut h = Harness::new(&receiving_limits(2048, &content[..], credit));
        let (owner, _) = h.open(1);
        let runs = h.batch(Event::Completed { owner, completion: completion(content.clone(), Stop::ToolUse) });
        let &[read_b, read_a] = runs.as_slice() else { panic!("two runs, not {runs:?}") };
        let cancels = h.batch(Event::Close { session: owner });
        assert_eq!(cancels.as_slice(), &[read_b, read_a]);
        assert_eq!(h.step(answered(read_b, 20, length, false)), None);
        assert_eq!(h.step(Event::Done { owner: read_a, done: Done::Cancelled }), Some(ended(End::Closed, 1)));
        assert_eq!(
            h.turn_records.get(0).expect("observed turn").messages[2].content[0],
            Block::ToolResult {
                id: bytes(b"b"),
                result: Returned::Text { text: answer(20, length), error: false, replay: None }
            }
        );
    }
}

#[test]
fn delegated_calls_are_told_as_they_start_and_end() {
    let mut h = Harness::new(&LIMITS);
    let opener = Token::new(1);
    let (owner, _) = h.open(1);
    let content = Box::new([delegated(b"a", 9, Effect::Read), delegated(b"b", 10, Effect::Read)]);
    let runs = h.batch(Event::Completed { owner, completion: completion(content, Stop::ToolUse) });
    assert_eq!(h.step(answered(runs.as_slice()[0], 20, 3, true)), None);
    drop(h.batch(Event::Close { session: owner }));
    drop(h.step(answer_cancelled(runs.as_slice()[1])));
    h.told(&[
        Fact::Opened { opener },
        Fact::CompletionStarted {
            window: 1,
            opener,
            attempt: 0,
            messages: 1,
            max_tokens: 1024,
            choice: crate::facts::Choice::Auto,
        },
        Fact::Tools { opener, fact: tools::FactKind::Opened { session: owner } },
        Fact::CompletionAnswered { opener, stop: Stop::ToolUse, blocks: 2, calls: 2, invalid: 0 },
        Fact::Used { opener, usage: USAGE },
        Fact::DelegateStarted { opener, block: 0 },
        Fact::DelegateStarted { opener, block: 1 },
        Fact::DelegateAnswered { opener, bytes: 3, error: true },
        // The kit closes as the session does, with nothing of its own to settle.
        Fact::Tools { opener, fact: tools::FactKind::Closed { session: owner } },
        Fact::DelegateCancelled { opener },
        Fact::Ended { opener, end: End::Closed, turns: 1, usage: USAGE },
    ]);
}

#[test]
fn the_llm_yields_whenever_it_stops_calling_tools() {
    let truncated = yield_on(Stop::MaxTokens, Box::new([text(b"I was say")]));
    assert_eq!(truncated, (1, Yield::Truncated, bytes(b"I was say")));
    assert_eq!(yield_on(Stop::Refusal, Box::new([text(b"no")])), (1, Yield::Refused, bytes(b"no")));
    let malformed = yield_on(Stop::ToolUse, Box::new([text(b"I will call a tool")]));
    assert_eq!(malformed, (1, Yield::Malformed, bytes(b"I will call a tool")));
    assert_eq!(yield_on(Stop::EndTurn, Box::new([])), (1, Yield::Done, bytes(b"")));
}

#[test]
fn opens_beyond_the_session_slots_are_refused_as_busy() {
    let mut h = Harness::new(&Limits { sessions: 1, ..LIMITS });
    drop(h.open(1));
    let refused = h.step(Event::Open { opener: Token::new(2), opening: opening(spec()) });
    assert_eq!(
        refused,
        Some(Request::Ended {
            opener: Token::new(2),
            end: End::Busy,
            turns: 0,
            usage: Usage::ZERO,
            reported: Usage::NONE,
            detail: Box::default()
        })
    );
}

#[test]
fn a_spec_whose_authority_the_tools_refuse_is_refused_as_invalid() {
    // More repositories than the tools' limits let a kit name.
    let mut h = Harness::new(&LIMITS);
    let refused =
        h.step(Event::Open { opener: Token::new(1), opening: opening(Spec { authority: authority(3), ..spec() }) });
    assert_eq!(refused, Some(ended(End::Invalid, 0)));
    h.domain.reclaim();
    assert_eq!((h.domain.sessions(), h.domain.kits()), (0, 0));
}

#[test]
fn closing_a_calling_session_cancels_its_call_and_ends_once_the_call_has_ended() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    assert_eq!(h.step(Event::Close { session: owner }), Some(Request::Cancel { owner }));
    assert_eq!(h.domain.next_deadline(), None, "a closing session waits for nothing but its call");
    assert_eq!(h.step(Event::Cancelled { owner }), Some(ended(End::Closed, 0)));
}

#[test]
fn a_completion_that_wins_the_race_with_a_close_still_ends_the_session() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    assert_eq!(h.step(Event::Close { session: owner }), Some(Request::Cancel { owner }));
    // Its tokens were spent all the same.
    assert_eq!(h.step(Event::Completed { owner, completion: reading() }), Some(ended(End::Closed, 1)));
    assert_eq!((h.turns, h.usage), (1, USAGE));

    let (owner, _) = h.open(1);
    assert_eq!(h.step(Event::Close { session: owner }), Some(Request::Cancel { owner }));
    assert_eq!(
        h.step(Event::Failed {
            retry_ok: true,
            owner,
            failure: Failure::Overloaded,
            evidence: crate::llm::Evidence::Unknown,
            detail: Box::default()
        }),
        Some(ended(End::Closed, 0))
    );
}

#[test]
fn closing_a_tooling_session_cancels_its_tool() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    let content = Box::new([tool_call(b"c1", cat(b"main.rs"))]);
    let (run, _) = running(h.step(Event::Completed { owner, completion: completion(content, Stop::ToolUse) }));
    assert_eq!(h.step(Event::Close { session: owner }), Some(Request::CancelIo { owner: run }));
    assert_eq!(h.step(Event::Done { owner: run, done: Done::Cancelled }), Some(ended(End::Closed, 1)));

    // The tool won the race.
    let (owner, _) = h.open(1);
    let content = Box::new([tool_call(b"c1", cat(b"main.rs"))]);
    let (run, _) = running(h.step(Event::Completed { owner, completion: completion(content, Stop::ToolUse) }));
    assert_eq!(h.step(Event::Close { session: owner }), Some(Request::CancelIo { owner: run }));
    assert_eq!(h.step(ran(run, b"main.rs")), Some(ended(End::Closed, 1)));
}

#[test]
fn closing_a_session_with_nothing_in_flight_ends_it_at_once() {
    let mut h = Harness::new(&LIMITS);
    let session = h.yielded();
    assert_eq!(h.step(Event::Close { session }), Some(ended(End::Closed, 1)));
    assert_eq!(h.domain.next_deadline(), None);

    let (owner, _) = h.open(1);
    assert_eq!(
        h.step(Event::Failed {
            retry_ok: true,
            owner,
            failure: Failure::Unavailable,
            evidence: crate::llm::Evidence::Unknown,
            detail: Box::default()
        }),
        None
    );
    assert_eq!(h.step(Event::Close { session: owner }), Some(ended(End::Closed, 0)));
    assert_eq!(h.domain.next_deadline(), None, "the retry is called off");
}

#[test]
fn closing_a_session_that_is_already_closing_changes_nothing() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    h.after(BUDGET.time);
    assert_eq!(h.fire(), Some(Request::Cancel { owner }));
    assert_eq!(h.step(Event::Close { session: owner }), None);
    assert_eq!(h.step(Event::Cancelled { owner }), Some(ended(OUT_OF_TIME, 0)));
}

#[test]
fn a_handle_to_a_session_that_has_ended_is_dropped() {
    let mut h = Harness::new(&Limits { sessions: 1, ..LIMITS });
    let session = h.yielded();
    assert_eq!(h.step(Event::Close { session }), Some(ended(End::Closed, 1)));
    // Ended, and not yet reclaimed.
    assert_eq!(h.step(Event::Continue { session, content: bytes(b"go on") }), None);
    assert_eq!(h.step(Event::Close { session }), None);
    // Reclaimed, and its slot taken by another session.
    h.domain.reclaim();
    let (other, _) = h.open(2);
    assert_ne!(other, session, "a handle is never reused");
    assert_eq!(h.step(Event::Close { session }), None);
    assert_eq!(h.step(Event::Continue { session, content: bytes(b"go on") }), None);
    assert_eq!(h.domain.sessions(), 1, "the other session lives on");
}

#[test]
fn an_expiring_session_cancels_its_call_and_ends_once_the_call_has_ended() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    h.after(BUDGET.time);
    assert_eq!(h.fire(), Some(Request::Cancel { owner }));
    assert_eq!(h.domain.next_deadline(), None);
    // The completion won the race with the cancel; the session still expires.
    assert_eq!(h.step(Event::Completed { owner, completion: done() }), Some(ended(OUT_OF_TIME, 1)));
}

#[test]
fn an_expiring_session_cancels_its_tool() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    let content = Box::new([tool_call(b"c1", cat(b"main.rs"))]);
    let (run, _) = running(h.step(Event::Completed { owner, completion: completion(content, Stop::ToolUse) }));
    h.after(BUDGET.time);
    assert_eq!(h.fire(), Some(Request::CancelIo { owner: run }));
    assert_eq!(h.step(Event::Done { owner: run, done: Done::Cancelled }), Some(ended(OUT_OF_TIME, 1)));
}

#[test]
fn a_session_whose_time_is_up_starts_no_completion_even_before_its_alarm_fires() {
    let time = Duration::from_secs(60);
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open_with(1, budget(Budget { time, ..BUDGET }));
    let (run, _) = running(h.step(Event::Completed { owner, completion: reading() }));
    // The tool's result and the alarm fall due in the same iteration: the
    // result is handled first, and the session ends there.
    h.after(time);
    let end = h.step(ran(run, b"main.rs"));
    assert_eq!(end, Some(ended(OUT_OF_TIME, 1)));
    assert_eq!(h.domain.next_deadline(), None);
}

#[test]
fn oversized_provider_completions_end_before_calls_or_turns() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    let huge = Box::from([b'x'; 4097].as_slice());
    let oversized = completion(Box::new([text(&huge)]), Stop::EndTurn);
    assert_eq!(
        h.step(Event::Completed { owner, completion: oversized }),
        Some(ended(
            End::Failed {
                detail: Box::default(),
                failure: Failure::Limit { which: crate::llm::Limit::Path(crate::llm::Cap::Answer), bound: 4096 },
                evidence: crate::llm::Evidence::Response { status: None },
                attempts: 1
            },
            1
        ))
    );
    assert_eq!(h.turn_records.len(), 0);

    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    let call = Block::ToolCall {
        id: bytes(b"oversized"),
        name: bytes(b"finish"),
        input: huge,
        call: Decoded::Delegated {
            descriptor: FINISH.ticket,
            source: crate::ToolSource::Run,
            ticket: FINISH.ticket,
            effect: Effect::Write,
        },
        replay: None,
    };
    let completion = completion(Box::new([call]), Stop::ToolUse);
    assert_eq!(
        h.step(Event::Completed { owner, completion }),
        Some(ended(
            End::Failed {
                detail: Box::default(),
                failure: Failure::Limit { which: crate::llm::Limit::Path(crate::llm::Cap::Answer), bound: 4096 },
                evidence: crate::llm::Evidence::Response { status: None },
                attempts: 1
            },
            1
        ))
    );
    assert_eq!((h.domain.runs(), h.turn_records.len()), (0, 0));
}

#[test]
fn late_oversized_completion_replaces_the_closing_reason() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    assert_eq!(h.step(Event::Close { session: owner }), Some(Request::Cancel { owner }));
    let huge = [b'x'; 4097];
    let completion = completion(Box::new([text(&huge)]), Stop::EndTurn);
    assert_eq!(
        h.step(Event::Completed { owner, completion }),
        Some(ended(
            End::Failed {
                detail: Box::default(),
                failure: Failure::Limit { which: crate::llm::Limit::Path(crate::llm::Cap::Answer), bound: 4096 },
                evidence: crate::llm::Evidence::Response { status: None },
                attempts: 1
            },
            1
        ))
    );
    assert_eq!(h.turn_records.len(), 0);
}

/// Opens a session, has the LLM call a tool and the tool run, and returns
/// the session's name and what came out.
fn drive(h: &mut Harness) -> (Token, Prompt, Option<Request>, Option<Request>) {
    let (owner, prompt) = h.open(1);
    let tool = h.step(Event::Completed { owner, completion: reading() });
    let Some(Request::Io { owner: run, .. }) = tool else {
        panic!("expected a tool run, not {tool:?}");
    };
    let call = h.step(ran(run, b"main.rs"));
    (owner, prompt, tool, call)
}

#[test]
fn completion_name_exhaustion_ends_before_any_owned_or_delegated_effect() {
    {
        let mut harness = Harness::new(&LIMITS);
        let (session, _) = harness.open_with(1, spec());
        crate::session::exhaust_origin_for_test(&mut harness.domain, session);
        let answer = completion(
            Box::new([
                delegated(b"repeated-provider-id", 7, Effect::Write),
                tool_call(b"repeated-provider-id", cat(b"main.rs")),
            ]),
            Stop::ToolUse,
        );
        step(
            &mut harness.domain,
            &harness.env,
            Event::Completed { owner: session, completion: answer },
            &mut harness.out,
        );
        let mut used = false;
        let mut ended = false;
        while let Some(request) = harness.out.pop() {
            match request {
                Request::Used { usage, .. } => {
                    assert_eq!(usage, USAGE);
                    used = true;
                }
                Request::Priced { spent, own_spent, .. } => {
                    assert_eq!((spent, own_spent), (0, 0));
                }
                Request::Ended { end, .. } => {
                    assert_eq!(
                        end,
                        End::Failed {
                            detail: Box::default(),
                            failure: Failure::ApplicationLimit,
                            evidence: crate::llm::Evidence::Unsent,
                            attempts: 0
                        }
                    );
                    ended = true;
                }
                Request::JobsCancelled { .. }
                | Request::Opened { .. }
                | Request::Yielded { .. }
                | Request::Complete { .. }
                | Request::Cancel { .. }
                | Request::Yield { .. }
                | Request::Io { .. }
                | Request::Job { .. }
                | Request::CancelIo { .. }
                | Request::Delegate { .. }
                | Request::Withdraw { .. }
                | Request::Turn { .. } => panic!("exhaustion emits no effect or new turn"),
            }
        }
        assert!(used && ended);
    }
}

/// A concrete recording entrance owns its provider reservation.
/// This fixture retains the original raw receiving caps and tools authority.
fn reserved_recorded_call(harness: &mut Harness) -> Token {
    step(
        &mut harness.domain,
        &harness.env,
        Event::Open {
            opener: Token::new(1),
            opening: Box::new(crate::record::Opening {
                window_llm: None,
                spec: spec(),
                dialect: 1,
                prices: crate::record::Prices { input: 0, cached: 0, cache_write: None, output: 0, unit: 1 },
                budget: 100,
                transcript: None,
                answered: Box::default(),
            }),
        },
        &mut harness.out,
    );
    let Some(Request::Opened { opener, session }) = harness.out.pop() else {
        panic!("the concrete recording session is admitted");
    };
    assert_eq!(opener, Token::new(1));
    let (owner, _) = calling(harness.one());
    assert_eq!(owner, session);
    owner
}

#[test]
fn unsent_close_releases_provider_credit_without_requesting_a_cancel() {
    let mut harness = Harness::new(&Limits { spend: 100, ..LIMITS });
    let owner = reserved_recorded_call(&mut harness);
    let (reserved, credit) = crate::session::provider_credit_for_test(&harness.domain, owner);
    assert!(reserved > 0 && credit == Some(reserved));
    assert_eq!(harness.step(Event::UnsentClosed { owner }), Some(ended(End::Closed, 0)));
    assert_eq!(crate::session::provider_credit_for_test(&harness.domain, owner), (0, None));
    assert_eq!(harness.domain.next_deadline(), None);
    assert_eq!(harness.step(Event::UnsentClosed { owner }), None);
    harness.domain.reclaim();
    assert_eq!((harness.domain.sessions(), harness.domain.kits()), (0, 0));
}

#[test]
fn exact_provider_envelope_admits_and_one_byte_short_refuses_before_effects() {
    let reservation = crate::completion_reserve(&LIMITS).expect("configured provider bounds fit");
    let headroom = u64::try_from(size_of::<Block>()).expect("block") + 64 * u64::from(LIMITS.completion_blocks);
    let exact = initial_bytes().checked_add(reservation + headroom).expect("exact entrance fits");
    let mut admitted = Harness::new(&Limits { history_bytes: exact, ..LIMITS });
    let (owner, _) = admitted.open(1);
    assert_eq!(crate::session::provider_credit_for_test(&admitted.domain, owner), (reservation, Some(reservation)));
    assert_eq!(admitted.step(Event::UnsentClosed { owner }), Some(ended(End::Closed, 0)));
    let mut refused = Harness::new(&Limits { history_bytes: exact.checked_sub(1).expect("positive cap"), ..LIMITS });
    step(
        &mut refused.domain,
        &refused.env,
        Event::Open { opener: Token::new(1), opening: opening(spec()) },
        &mut refused.out,
    );
    let Some(Request::Opened { .. }) = refused.next() else { panic!("the fresh window opens before its limit") };
    assert_eq!(
        refused.one(),
        Some(ended(
            End::Failed {
                detail: Box::default(),
                failure: Failure::Limit { which: crate::llm::Limit::History, bound: exact - 1 },
                evidence: crate::llm::Evidence::Unsent,
                attempts: 0
            },
            0
        ))
    );
    assert_eq!((refused.domain.sessions(), refused.domain.kits(), refused.domain.runs()), (1, 1, 0));
    refused.domain.reclaim();
    assert_eq!((refused.domain.sessions(), refused.domain.kits(), refused.domain.runs()), (0, 0, 0));
}

#[test]
fn a_full_owned_batch_leaves_a_summary_instruction_and_compacts() {
    let content = Box::new([tool_call(b"a", cat(b"file")), tool_call(b"b", cat(b"file"))]);
    let limits = exact_read_batch_limits(&content[..], false);
    let mut h = Harness::new(&limits);
    let (owner, _) = h.open(1);
    let runs = h.batch(Event::Completed { owner, completion: completion(content, Stop::ToolUse) });
    assert_eq!(runs.len(), 2);
    assert_eq!(h.step(ran(runs.as_slice()[1], &[b'y'; 4096])), None);
    let (_, prompt) = calling(h.step(ran(runs.as_slice()[0], &[b'x'; 4096])));
    match prompt.purpose {
        crate::llm::Purpose::Compaction { .. } => {}
        crate::llm::Purpose::Ordinary { .. } => panic!("history compacts"),
    }
    let expected = [result(b"a", read(&[b'x'; 4096])), result(b"b", read(&[b'y'; 4096]))];
    assert_eq!(&*h.turn_records.get(0).expect("ordinary turn").messages[2].content, &expected);
}

#[test]
fn a_batch_one_byte_short_runs_every_read_with_a_smaller_cap() {
    let content = Box::new([tool_call(b"a", cat(b"file")), tool_call(b"b", cat(b"file"))]);
    let limits = exact_read_batch_limits(&content[..], true);
    let mut h = Harness::new(&limits);
    let (owner, _) = h.open(1);
    let runs = h.batch(Event::Completed { owner, completion: completion(content, Stop::ToolUse) });
    assert_eq!(runs.len(), 2);
    assert_eq!(h.batch(Event::Close { session: owner }).as_slice(), runs.as_slice());
    assert_eq!(h.step(ran(runs.as_slice()[0], &[b'x'; 4096])), None);
    assert_eq!(h.step(ran(runs.as_slice()[1], &[b'y'; 4096])), Some(ended(End::Closed, 1)));
    for block in &h.turn_records.get(0).expect("ordinary turn").messages[2].content {
        let Block::ToolResult {
            result: Returned::Owned { outcome: Outcome::Read { content, window, cut, .. } }, ..
        } = block
        else {
            panic!("read result")
        };
        assert_eq!(content.len(), 4095);
        assert_eq!(*window, tools::BytesClamp::Clamped { asked: 4096, ceiling: 4095 });
        assert_eq!(*cut, tools::Cut::Line);
    }
}

fn exact_read_batch_limits(content: &[Block], short: bool) -> Limits {
    let limits = receiving_limits(2048, content, 8192);
    let mut held = crate::session::content_cost(content).expect("fixture content");
    for block in content {
        let Block::ToolCall { id, .. } = block else { panic!("fixture calls") };
        held = held
            .checked_add(
                u64::try_from(size_of::<Block>().checked_add(id.len()).expect("result skeleton"))
                    .expect("skeleton fits"),
            )
            .expect("both result skeletons");
    }
    let bytes = initial_bytes()
        .checked_add(held)
        .expect("prefix")
        .checked_add(8192)
        .expect("results")
        .checked_add(u64::try_from(size_of::<Block>()).expect("summary block"))
        .expect("instruction");
    Limits { history_bytes: bytes.checked_sub(u64::from(short)).expect("one-short control"), ..limits }
}

#[expect(clippy::disallowed_types, reason = "ordinary test observation drains the uncapped fact queue whole")]
fn observed_facts(harness: &mut Harness) -> Box<[crate::Fact]> {
    let mut facts = Vec::new();
    while let Some(fact) = harness.pop_fact() {
        facts.push(fact);
    }
    facts.into_boxed_slice()
}

fn fact_times(facts: &[crate::Fact]) -> Box<[Time]> {
    let mut times = List::with_capacity(u32::try_from(facts.len()).expect("observed fact count fits"));
    for fact in facts {
        times.push(fact.at).expect("room for all observations");
    }
    times.into_boxed()
}

fn pop_kind(harness: &mut Harness) -> Option<Fact> {
    let fact = harness.pop_fact()?;
    Some(fact.kind)
}

#[test]
fn a_workspace_refusal_keeps_its_provider_identity_without_starting_work() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    h.env.now = Time::from_nanos(11);
    let call = Call::Read { path: outside(), offset: 1, window: tools::Budget::Default };
    let content = Box::new([tool_call(b"refused", call)]);
    assert_eq!(h.step(Event::Completed { owner, completion: completion(content, Stop::ToolUse) }), None);
    h.env.now = Time::from_nanos(99);
    let facts = observed_facts(&mut h);
    assert_eq!(facts.len(), 6);
    assert_eq!(facts[5].at, Time::from_nanos(11));
    let call = facts[5].call.as_ref().expect("refusal metadata");
    assert_eq!(&*call.id, b"refused");
    assert_eq!(call.source, crate::ToolSource::Workspace);
    assert_eq!(call.input_bytes, 2);
    assert_eq!(
        facts[5].kind,
        Fact::Tools {
            opener: Token::new(1),
            fact: tools::FactKind::Answered {
                session: owner,
                tool: tools::Tool::Read,
                verdict: tools::Verdict::Outside,
                bytes: 0,
                omitted_bytes: 0,
                job: None,
                clamped: false
            }
        }
    );
}

#[test]
fn the_openers_output_cap_replaces_the_old_fixed_token_ceiling() {
    let mut harness = Harness::new(&LIMITS);
    let (_, prompt) = harness.open_with(1, Spec { output: 4097, ..spec() });
    assert_eq!(prompt.max_tokens, 4097);
}

#[test]
fn absent_cache_write_rate_charges_writes_at_input_once() {
    let rates = crate::record::Prices { input: 7, cached: 3, cache_write: None, output: 11, unit: 10 };
    let usage = Usage { input_tokens: Some(1), output_tokens: Some(1), cache_write_tokens: Some(3), ..Usage::ZERO };
    assert_eq!(rates.price(usage), Some(4), "(7 + 21 + 11)/10");
    assert_eq!(crate::record::Prices { cache_write: Some(7), ..rates }.price(usage), Some(4));
    assert_eq!(crate::record::Prices { cache_write: Some(13), ..rates }.price(usage), Some(6));
}

#[test]
fn a_call_outside_the_choice_is_not_run_and_needs_no_result_credit() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    drop(calling(h.step(Event::Narrowed {
        owner,
        note: bytes(b"Finish now."),
        choice: crate::llm::ToolChoice::Only(Box::new([FINISH.ticket])),
        names: Some(Box::new([bytes(b"finish")])),
    })));
    let content = Box::new([tool_call(b"outside", cat(b"main.rs"))]);
    assert_eq!(h.step(Event::Completed { owner, completion: completion(content, Stop::ToolUse) }), None);
    assert_eq!(h.domain.runs(), 0, "the disallowed call has no live tool identity");
    h.domain.reclaim();
    let Some(Request::Complete { prompt, .. }) = h.resume() else {
        panic!("one not-run result precedes the next request");
    };
    assert_eq!(
        prompt.messages.last().unwrap().content.as_ref(),
        &[Block::ToolResult {
            id: bytes(b"outside"),
            result: Returned::NotRun {
                why: crate::llm::NotRun::OutsideChoice { choice: Some(Box::new([bytes(b"finish")])) }
            },
        }]
    );
    assert_eq!(prompt.choice, crate::llm::ToolChoice::Only(Box::new([FINISH.ticket])));
    let mut notes = 0_u32;
    for message in &prompt.messages {
        for block in &message.content {
            match block {
                Block::Text { text, .. } => {
                    if text.as_ref() == b"Finish now." {
                        notes += 1;
                    }
                }
                Block::Refusal { .. } | Block::Opaque { .. } | Block::ToolCall { .. } | Block::ToolResult { .. } => {}
            }
        }
    }
    assert_eq!(notes, 1);
}

#[test]
fn a_selected_descriptor_can_have_a_distinct_live_call_ticket() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    drop(calling(h.step(Event::Narrowed {
        owner,
        note: bytes(b"Finish now."),
        choice: crate::llm::ToolChoice::Only(Box::new([FINISH.ticket])),
        names: Some(Box::new([bytes(b"finish")])),
    })));
    let mut call = delegated(b"allowed", 1234, Effect::Write);
    let Block::ToolCall { call: Decoded::Delegated { descriptor, .. }, .. } = &mut call else { unreachable!() };
    *descriptor = FINISH.ticket;
    let Some(Request::Delegate { call, .. }) =
        h.step(Event::Completed { owner, completion: completion(Box::new([call]), Stop::ToolUse) })
    else {
        panic!("the chosen descriptor must dispatch its call");
    };
    assert_eq!(call, Token::new(1234));
}

#[test]
fn a_narrowing_note_follows_the_latest_settled_results() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    let (run, _) = running(h.step(Event::Completed { owner, completion: reading() }));
    let Some(Request::Complete { prompt: before, .. }) = h.step(ran(run, b"main.rs")) else {
        panic!("settled read reasks");
    };
    let Some(Request::Complete { prompt: after, .. }) = h.step(Event::Narrowed {
        owner,
        note: bytes(b"Finish now."),
        choice: crate::llm::ToolChoice::Only(Box::new([FINISH.ticket])),
        names: Some(Box::new([bytes(b"finish")])),
    }) else {
        panic!("narrowed completion reasks");
    };
    assert_eq!(&after.messages[..before.messages.len()], before.messages.as_ref());
    assert_eq!(after.messages.last().unwrap().content.as_ref(), &[text(b"Finish now.")]);
    assert_eq!(h.turns, 1, "the unsent narrowing spends no extra turn");
}

#[test]
fn a_narrowing_note_follows_a_yielded_session_waking_message() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    drop(yielded(
        h.step(Event::Completed { owner, completion: completion(Box::new([text(b"done for now")]), Stop::EndTurn) }),
    ));
    let Some(Request::Complete { prompt: before, .. }) =
        h.step(Event::Continue { session: owner, content: bytes(b"continue") })
    else {
        panic!("waking message asks");
    };
    let Some(Request::Complete { prompt: after, .. }) = h.step(Event::Narrowed {
        owner,
        note: bytes(b"Finish now."),
        choice: crate::llm::ToolChoice::Only(Box::new([FINISH.ticket])),
        names: Some(Box::new([bytes(b"finish")])),
    }) else {
        panic!("narrowed completion reasks");
    };
    assert_eq!(&after.messages[..before.messages.len()], before.messages.as_ref());
    assert_eq!(after.messages.last().unwrap().content.as_ref(), &[text(b"Finish now.")]);
    assert_eq!(h.turns, 1);
}

#[test]
fn an_allowed_descriptor_keeps_its_argument_problem_during_narrowing() {
    let mut h = Harness::new(&LIMITS);
    let (owner, _) = h.open(1);
    drop(calling(h.step(Event::Narrowed {
        owner,
        note: bytes(b"Finish now."),
        choice: crate::llm::ToolChoice::Only(Box::new([FINISH.ticket])),
        names: Some(Box::new([bytes(b"finish")])),
    })));
    let problem = Problem::Missing { field: bytes(b"report") };
    let call = Block::ToolCall {
        id: bytes(b"allowed-invalid"),
        name: bytes(b"finish"),
        input: bytes(b"{}"),
        replay: None,
        call: Decoded::Invalid { descriptor: Some(FINISH.ticket), problem: problem.clone() },
    };
    assert_eq!(h.step(Event::Completed { owner, completion: completion(Box::new([call]), Stop::ToolUse) }), None);
    h.domain.reclaim();
    let Some(Request::Complete { prompt, .. }) = h.resume() else { panic!("the typed problem reasks") };
    assert_eq!(
        prompt.messages.last().unwrap().content.as_ref(),
        &[Block::ToolResult { id: bytes(b"allowed-invalid"), result: Returned::Invalid { problem } }]
    );
    assert_eq!(h.domain.runs(), 0, "a malformed allowed call still starts no operation");
}

#[test]
fn an_exact_quarter_allowance_keeps_187_point_6ms_ordinary_and_187_point_5ms_final() {
    for (left, final_now) in [(187_600_000, false), (187_500_000, true), (187_250_000, true)] {
        let mut harness = Harness::new(&LIMITS);
        let mut selected = spec();
        selected.budget.time = Duration::from_millis(750);
        selected.final_turn =
            Some(crate::FinalTurn { note: bytes(b"Answer now."), allowance: Duration::from_nanos(187_500_000) });
        let (owner, first) = harness.open_with(1, selected);
        assert_eq!(first.choice, crate::llm::ToolChoice::Auto);
        assert_eq!(yielded(harness.step(Event::Completed { owner, completion: done() })).1, Yield::Done);
        harness.env.now = Time::ZERO.saturating_add(Duration::from_nanos(750_000_000 - left));
        let (_, prompt) = calling(harness.step(Event::Continue { session: owner, content: bytes(b"Go on.") }));
        let expected = if final_now { crate::llm::ToolChoice::None } else { crate::llm::ToolChoice::Auto };
        assert_eq!(prompt.choice, expected);
        assert_eq!(
            prompt.messages.last().unwrap().content.as_ref(),
            &[text(if final_now { b"Answer now." } else { b"Go on." })]
        );
    }
}

#[test]
fn a_due_compaction_counts_two_turns_and_twice_the_final_allowance() {
    let final_turn = crate::FinalTurn { note: bytes(b"Answer."), allowance: Duration::from_millis(100) };
    let roomy = Duration::from_secs(1);
    assert!(!final_turn.due(2, roomy, false));
    assert!(final_turn.due(2, roomy, true));
    assert!(!final_turn.due(3, Duration::from_nanos(200_000_001), true));
    assert!(final_turn.due(3, Duration::from_millis(200), true));
    assert!(!final_turn.due(3, Duration::from_millis(200), false));
    assert!(final_turn.due(3, Duration::from_millis(100), false));
}

fn history_derivation() -> crate::Derivation {
    const LINE_BYTES: u64 = 800;
    const COMPLETION_BYTES: u64 = 600;
    const ANSWER_BYTES: u64 = 256;
    const BATCH_BYTES: u64 = 512;
    const DETAIL_BYTES: u32 = 128;
    const ALLOWANCE_BYTES: u64 = 100;
    crate::Derivation {
        conversations: 1,
        calls_per_response: 2,
        parallel_tools: 4,
        max_turns: 20,
        max_spend: 100,
        max_time: Duration::from_secs(60),
        tools: LIMITS.tools,
        line_bytes: LINE_BYTES,
        completion_bytes: COMPLETION_BYTES,
        answer_bytes: ANSWER_BYTES,
        batch_result_bytes: BATCH_BYTES,
        delegated_result_bytes: ANSWER_BYTES,
        output_items: 6,
        detail: DETAIL_BYTES,
        protocol_allowance: ALLOWANCE_BYTES,
        threshold: [4, 5],
        token_bytes: 8,
    }
}

mod window;

fn later_owned_call(call: Call) -> (Harness, Token, Token, Op) {
    let first = Call::Read { path: path(b"file"), offset: 1, window: tools::Budget::Asked(u32::MAX) };
    let content = Box::new([tool_call(b"first", first), tool_call(b"later", call)]);
    let limits = Limits { history_bytes: 16_384, parallel_tools: 1, ..receiving_limits(2048, &content[..], 0) };
    let mut selected = spec();
    selected.authority.grants.shell = true;
    let mut h = Harness::new(&limits);
    let (session, _) = h.open_with(1, selected);
    let (first, _) =
        running(h.step(Event::Completed { owner: session, completion: completion(content, Stop::ToolUse) }));
    let (later, operation) = running(h.step(ran(first, &answer(u64::from(b'x'), 65_536))));
    (h, session, later, operation)
}

fn close_later_call(h: &mut Harness, session: Token, owner: Token) {
    assert_eq!(h.step(Event::Close { session }), Some(Request::CancelIo { owner }));
    assert_eq!(h.step(Event::Done { owner, done: Done::Cancelled }), Some(ended(End::Closed, 1)));
}

#[test]
fn a_later_search_keeps_its_hit_cells_and_text_within_its_share() {
    let call = Call::Search { path: path(b"file"), pattern: bytes(b"word"), glob: None };
    let (mut h, session, owner, operation) = later_owned_call(call);
    let Op::Search { hits, bytes, .. } = operation else { panic!("search") };
    let cells = u64::try_from(size_of::<tools::Hit>()).expect("cell").checked_mul(u64::from(hits)).expect("cells");
    assert!(cells.checked_add(u64::from(bytes)).expect("result") <= 64);
    assert!(hits < TOOLS.search_hits && bytes < TOOLS.search_bytes);
    close_later_call(&mut h, session, owner);
}

#[test]
fn a_later_command_keeps_its_head_and_tail_within_its_share() {
    let call = Call::Shell {
        after: smith_domain_tools::Yield::Never,
        command: bytes(b"echo output"),
        timeout: tools::Timeout::Default,
        output: tools::Budget::Asked(u32::MAX),
    };
    let (mut h, session, owner, operation) = later_owned_call(call);
    let Op::Spawn { head, tail, .. } = operation else { panic!("command") };
    assert_eq!((head, tail), (32, 32));
    close_later_call(&mut h, session, owner);
}

#[test]
fn an_ordinary_completion_after_a_prompt_carries_the_guard() {
    let mut harness = Harness::new(&LIMITS);
    let owner = harness.yielded();
    let (_, prompt) = calling(harness.step(Event::Continue { session: owner, content: bytes(b"continue") }));
    assert_eq!(
        prompt.purpose,
        crate::llm::Purpose::Ordinary {
            // USAGE's input + cache read + cache write = 10 + 3 + 2.
            guard: Some(crate::llm::Guard { line: 64_512, last_prompt: 15, from: 1 }),
        }
    );
}

#[test]
fn a_windows_first_completion_carries_no_guard() {
    let mut harness = Harness::new(&LIMITS);
    let (_, prompt) = harness.open(1);
    assert_eq!(prompt.purpose, crate::llm::Purpose::Ordinary { guard: None });
}

#[test]
fn a_compaction_carries_no_guard() {
    let mut harness = Harness::new(&LIMITS);
    let owner = harness.yielded();
    let (_, prompt) = calling(harness.step(Event::Continue { session: owner, content: bytes(b"continue") }));
    assert_eq!(
        prompt.purpose,
        crate::llm::Purpose::Ordinary { guard: Some(crate::llm::Guard { line: 64_512, last_prompt: 15, from: 1 }) }
    );
    let (_, compacted) = calling(harness.step(Event::Failed {
        retry_ok: true,
        owner,
        failure: Failure::ContextTooLong,
        evidence: crate::llm::Evidence::Unsent,
        detail: bytes(b"past the line"),
    }));
    // Window - last prompt - output - summary instruction.
    assert_eq!(compacted.purpose, crate::llm::Purpose::Compaction { room: 65_490 });
}

mod batched_change;

#[test]
fn opens_window_is_true_of_first_and_window_turns_only() {
    use crate::{
        llm::Affinity,
        record::{Backend, Kind},
    };
    let affinity = Affinity { key: [0; 16], thread: 0 };
    for (kind, expected) in [
        (Kind::First(Box::new(crate::record::Opener { affinity, account: None, backend: Backend::Direct })), true),
        (
            Kind::Window(Box::new(crate::record::Opener {
                affinity,
                account: Some(7),
                backend: Backend::Program { name: b"claude_code".as_slice().into() },
            })),
            true,
        ),
        (Kind::Ordinary, false),
    ] {
        assert_eq!(kind.opens_window(), expected, "{kind:?}");
    }
}

mod line;
