//! Root and caller ownership at Start, Complete and concrete Turn boundaries.
//! Typed Wait/text neighbours keep provider grammar and transport outside this domain measurement.
//! Contract: smith/docs/design/domain/run.md, sections 3, 6 and 12;
//! smith/docs/design/protocol/limits.md, section 11.

use skein_lib::{Duration, Env, Queue, Time, Token, Wall};
use skein_world::domain::heap::Meter;
use smith_agent_world::LIMITS;
use smith_domain::{self as root, Domain, Event, Grant, GrantName, Limits, Request, llm, run, session};

// Content capture capacity for this scenario:
// one parallel tool, sixty-four completion blocks and four jobs.
const ROOT_MEMORY_CONTENT_EDGE_RECORDS: u32 = 283;

const WORKER: Token = Token::new(91);
const PARENT: Token = Token::new(92);
const LARGE: usize = 8192;
const ID: &[u8] = b"call_0000000000000001";
const SAID: &[u8] = b"tiny observed text";
const RESTORED: &[u8] = b"observed restored text";

fn size<T>() -> u64 {
    u64::try_from(size_of::<T>()).expect("fixture cell size fits")
}

fn len(value: &[u8]) -> u64 {
    skein_lib::len_u64(value.len())
}

fn sum(values: impl IntoIterator<Item = u64>) -> u64 {
    values.into_iter().try_fold(0_u64, u64::checked_add).expect("checked fixture ownership sum")
}

fn cells<T>(count: usize) -> u64 {
    u64::try_from(count).expect("bounded fixture cells").checked_mul(size::<T>()).expect("checked fixture array")
}

fn bounds(messages: u32) -> Limits {
    let decoded_call_bytes = 32_768;
    let completion_bytes = 131_072;
    Limits {
        accounts: 1,
        endpoints: 1,
        configured_endpoint_bytes: 4096,
        decoded_call_bytes,
        content_edge_records: ROOT_MEMORY_CONTENT_EDGE_RECORDS,
        run: run::Limits {
            runs: 1,
            conversations: 1,
            run_conversations: 1,
            calls: 1,
            run_bytes: 4096,
            brief_sections: 4,
            host_part: 1 << 20,
            room: 0,
            compaction_threshold: [4, 5],
            compaction_material: 0,
            effort_bytes: 32,
            schema_costs: run::SchemaCosts::NONE,
            directories: 1,
            directory_name_bytes: 64,
            host_path_bytes: 4096,
            conflicts: 64,
            conflict_path_bytes: 4096,
            host_tools: 0,
            host_input_bytes: 64,
            host_reply_bytes: 64,
            answered_calls: 16,
            answered_bytes: 4096,
            answer_bytes: 64,
            guide_bytes: 64,
            outcome_bytes: 256,
            check_tail: 64,

            nudges: messages,
            messages: 1,
            message_bytes: u32::try_from(LARGE).expect("bounded caller message"),
            ..LIMITS.run
        },
        session: session::Limits {
            sessions: 1,
            history_messages: messages,
            // Source generation has independent room for maximum provider and
            // delegate receiving credit plus context. Restore replaces this
            // allowance with the separately observed exact payload/credit cap.
            history_bytes: 1_048_576,
            completion_bytes,
            completion_blocks: 64,
            failure_bytes: LIMITS.session.failure_bytes,
            delegated_result_bytes: 262_144,
            parallel_tools: 1,

            tools: root::tools::Limits {
                shell_output: 0,
                kits: 1,
                calls: 1,
                repos: 1,
                path_bytes: 64,
                host_path_bytes: 4096,
                known_files: 1,
                file_bytes: 64,
                tool_payload: 64,
                read_bytes: 64,
                list_entries: 1,
                list_bytes: 4096,
                match_lines: 1,
                shell_head: 0,
                shell_tail: 0,
                search_hits: 1,
                search_bytes: 64,
                ..LIMITS.session.tools
            },
            ..LIMITS.session
        },
        ..LIMITS
    }
}

fn workspace() -> run::Workspace {
    run::Workspace {
        directories: Box::new([run::Directory {
            host: Box::from(b"/host".as_slice()),
            name: b"work".as_slice().into(),
            root: Token::new(7),
            writable: false,
            git: true,
            conflicts: Box::new([]),
        }]),
    }
}

fn charter(restoring: bool) -> run::Charter {
    run::Charter {
        check_timeout: None,
        mode: run::Mode::Finite,
        reserve: run::charter::Reserve { turns: 0, time: skein_lib::Duration::ZERO },
        instructions: Box::new([]),
        brief: run::Brief {
            sections: Box::new([run::Section {
                title: b"Task".as_slice().into(),
                text: b"@root-memory".as_slice().into(),
            }]),
        },

        grants: run::charter::Grants {
            wait: true,
            deliver: None,
            save: false,
            tools: run::charter::Tools { inspect: false, modify: false, shell: false },
            agents: false,
            host_tools: Box::new([]),
        },
        outcome: run::outcome::OutcomeSpec {
            change: None,
            verdicts: Box::new([]),
            report: Some(run::outcome::TextSpec { max: 64, fields: Box::new([]) }),
            failure: None,
        },
        budget: LIMITS.run.budget,
        llm: run::charter::Llm {
            effort: None,
            prices: run::Prices { input: 1, cached: 1, cache_write: None, output: 1, unit: 1_000_000 },
            account: Some(0),
            endpoint: run::charter::Endpoint(0),
            model: b"fixture-model".as_slice().into(),
            window: 16_384,
            output: 128,
            dialect: 2,
        },
        models: Box::new([]),
        conventions: None,
        resume: restoring,
        waiting: Duration::from_secs(1),
    }
}

/// Independent owning boxes of this fixed public caller Start fixture. Charter
/// and Conventions wrappers are inline in Counted, not heap allocations.
fn workspace_bytes(workspace: Option<&run::Workspace>) -> u64 {
    workspace.map_or(0, |workspace| {
        sum([
            cells::<run::Directory>(workspace.directories.len()),
            sum(workspace.directories.iter().map(|directory| {
                sum([
                    len(&directory.name),
                    len(&directory.host),
                    cells::<Box<[u8]>>(directory.conflicts.len()),
                    sum(directory.conflicts.iter().map(|path| len(path))),
                ])
            })),
        ])
    })
}

fn charter_bytes(charter: &run::Charter) -> u64 {
    assert!(charter.grants.host_tools.is_empty() && charter.models.is_empty());
    assert!(charter.outcome.verdicts.is_empty());
    assert!(charter.outcome.change.as_ref().is_none_or(|spec| spec.fields.is_empty()));
    assert!(charter.outcome.report.as_ref().is_some_and(|spec| spec.fields.is_empty()));
    sum([
        len(&charter.instructions),
        cells::<run::Section>(charter.brief.sections.len()),
        sum(charter.brief.sections.iter().map(|section| sum([len(&section.title), len(&section.text)]))),
        len(&charter.llm.model),
        charter.conventions.as_ref().map_or(0, |conventions| sum([len(&conventions.guide), len(&conventions.checks)])),
    ])
}

struct Complete {
    owner: Token,
    prompt: llm::Prompt,
    maximum: u64,
}

fn terminal_bytes(event: &Event) -> u64 {
    let Event::Below(smith_domain::Below::Completed { completion, .. }) = event else {
        panic!("observed typed successful terminal")
    };
    let [said] = completion.content.as_ref() else { panic!("one observed scripted assistant block") };
    let payload = match said {
        llm::Said::Text { text, replay } => {
            assert!(text.as_ref() == SAID || text.as_ref() == RESTORED);
            assert_eq!(completion.stop, llm::Stop::EndTurn);
            sum([len(text), replay.as_ref().map_or(0, |value| len(&value.bytes))])
        }
        llm::Said::ToolCall { id, name, input, call: llm::Decoded::Served { ask: run::Ask::Wait }, replay } => {
            assert_eq!(id.as_ref(), ID);
            assert_eq!(name.as_ref(), b"wait");
            assert_eq!(input.as_ref(), b"{}");
            assert_eq!(completion.stop, llm::Stop::ToolUse);
            sum([len(id), len(name), len(input), replay.as_ref().map_or(0, |value| len(&value.bytes))])
        }
        llm::Said::Refusal { .. } | llm::Said::Opaque { .. } | llm::Said::ToolCall { .. } => {
            panic!("only observed literal Text and decoded Wait terminals belong to this measured fixture")
        }
    };
    sum([size::<llm::Said>(), payload])
}

// The fixture ledger counts observed public cells and owning byte fields. It
// neither calls owned_bytes/content_cost nor recreates admission or rendering.
fn record_blocks(blocks: &[session::llm::Block]) -> u64 {
    let mut bytes = skein_lib::len_u64(blocks.len()) * size::<session::llm::Block>();
    for block in blocks {
        bytes += match block {
            session::llm::Block::Text { text, replay } => {
                len(text) + replay.as_ref().map_or(0, |value| len(&value.bytes))
            }
            session::llm::Block::ToolCall(session::llm::ToolCall {
                id,
                name,
                input,
                call: session::llm::Decoded::Historical,
                replay,
            }) => len(id) + len(name) + len(input) + replay.as_ref().map_or(0, |value| len(&value.bytes)),
            session::llm::Block::ToolResult { id, result: session::llm::Returned::Text { text, replay, .. } } => {
                len(id) + len(text) + replay.as_ref().map_or(0, |value| len(&value.bytes))
            }
            session::llm::Block::Refusal { .. }
            | session::llm::Block::Opaque { .. }
            | session::llm::Block::ToolCall(_)
            | session::llm::Block::ToolResult { .. } => {
                panic!("generated fixture has only literal text, historical Wait and concrete feedback")
            }
        };
    }
    bytes
}

fn messages_bytes(messages: &[session::llm::Message]) -> u64 {
    skein_lib::len_u64(messages.len()) * size::<session::llm::Message>()
        + messages.iter().map(|message| record_blocks(&message.content)).sum::<u64>()
}

fn turn_bytes(turn: &session::record::Turn) -> u64 {
    use session::record::{Backend, Kind, Opener};
    let opener = match &turn.kind {
        Kind::First(opener) | Kind::Window(opener) => {
            size::<Opener>()
                + opener.system.as_ref().map_or(0, |text| len(text))
                + match &opener.backend {
                    Backend::Direct => 0,
                    Backend::Program { name } => len(name),
                }
        }
        Kind::Ordinary => 0,
    };
    opener + messages_bytes(&turn.messages) + len(&turn.model) + turn.effort.as_ref().map_or(0, |effort| len(effort))
}

fn transcript_bytes(history: &root::Transcript) -> u64 {
    skein_lib::len_u64(history.turns.len()) * size::<session::record::Turn>()
        + history.turns.iter().map(turn_bytes).sum::<u64>()
}

fn prompt_bytes(prompt: &llm::Prompt) -> u64 {
    let mut bytes = len(&prompt.model)
        + len(&prompt.system)
        + skein_lib::len_u64(prompt.served.len()) * size::<llm::Served>()
        + skein_lib::len_u64(prompt.messages.len()) * size::<llm::Message>();
    bytes += match &prompt.namespace {
        root::tools::Namespace::NoWorkspace | root::tools::Namespace::One => 0,
        root::tools::Namespace::Several { working } => len(working.as_bytes()),
    };
    for served in &prompt.served {
        assert!(matches!(served, llm::Served::Finish | llm::Served::Wait));
    }
    for message in &prompt.messages {
        bytes += skein_lib::len_u64(message.content.len()) * size::<llm::Block>();
        for block in &message.content {
            bytes += match block {
                llm::Block::Text { text, replay } => len(text) + replay.as_ref().map_or(0, |value| len(&value.bytes)),
                llm::Block::ToolCall { id, name, input, replay } => {
                    len(id) + len(name) + len(input) + replay.as_ref().map_or(0, |value| len(&value.bytes))
                }
                llm::Block::ToolResult { id, result: llm::Returned::Text { text, replay, .. } } => {
                    len(id) + len(text) + replay.as_ref().map_or(0, |value| len(&value.bytes))
                }
                llm::Block::Refusal { .. } | llm::Block::Opaque { .. } | llm::Block::ToolResult { .. } => {
                    panic!("literal supported root fixture fields")
                }
            };
        }
    }
    bytes
}

// Prompt deliberately has no Clone; the outside caller copies only the whole
// observed fixture values. This does not translate records or classify calls.
fn copy_prompt(prompt: &llm::Prompt) -> llm::Prompt {
    let expected = prompt_bytes(prompt);
    let copy_meter = Meter::new();
    copy_meter.start();
    let messages = prompt
        .messages
        .iter()
        .map(|message| llm::Message {
            role: message.role,
            content: message
                .content
                .iter()
                .map(|block| match block {
                    llm::Block::Text { text, replay } => {
                        llm::Block::Text { text: text.clone(), replay: replay.clone() }
                    }
                    llm::Block::ToolCall { id, name, input, replay } => llm::Block::ToolCall {
                        id: id.clone(),
                        name: name.clone(),
                        input: input.clone(),
                        replay: replay.clone(),
                    },
                    llm::Block::ToolResult { id, result: llm::Returned::Text { text, error, replay } } => {
                        llm::Block::ToolResult {
                            id: id.clone(),
                            result: llm::Returned::Text { text: text.clone(), error: *error, replay: replay.clone() },
                        }
                    }
                    llm::Block::Refusal { .. } | llm::Block::Opaque { .. } | llm::Block::ToolResult { .. } => {
                        panic!("outside copies only the observed supported fixture fields")
                    }
                })
                .collect(),
        })
        .collect();
    let copied = llm::Prompt {
        effort: None,
        purpose: prompt.purpose,
        choice: prompt.choice.clone(),
        namespace: prompt.namespace.clone(),
        affinity: prompt.affinity,
        asker: prompt.asker,
        endpoint: prompt.endpoint,
        model: prompt.model.clone(),
        system: prompt.system.clone(),
        tools: prompt.tools,
        served: prompt.served.clone(),
        messages,
        max_tokens: prompt.max_tokens,
    };
    let measured = copy_meter.end();
    assert_eq!(measured.held(), expected, "whole prompt arrays and payload/replay fields are owned independently");
    assert!(measured.peak() <= expected, "caller prompt copy has no unpriced construction transient");
    copied
}

fn copy_transcript(history: &root::Transcript) -> root::Transcript {
    let expected = transcript_bytes(history);
    let copy_meter = Meter::new();
    copy_meter.start();
    let copied = history.clone();
    let measured = copy_meter.end();
    assert_eq!(measured.held(), expected, "copied Turn/Message/Block arrays and payloads are independently owned");
    assert!(measured.peak() <= expected, "caller history copy has no unpriced construction transient");
    copied
}

fn copy_turn(turn: &session::record::Turn) -> session::record::Turn {
    let expected = turn_bytes(turn);
    let copy_meter = Meter::new();
    copy_meter.start();
    let copied = turn.clone();
    let measured = copy_meter.end();
    assert_eq!(measured.held(), expected, "copied concrete Turn fields are independently owned");
    assert!(measured.peak() <= expected, "caller Turn copy has no unpriced construction transient");
    copied
}

fn assert_rewritten(history: &root::Transcript, prompt: &llm::Prompt) {
    let expected = history.turns.iter().flat_map(|turn| &turn.messages);
    for (record, message) in expected.zip(&prompt.messages) {
        assert_eq!(message.role, record.role);
        assert_eq!(message.content.len(), record.content.len());
        for (original, block) in record.content.iter().zip(&message.content) {
            match (original, block) {
                (session::llm::Block::Text { text: original, replay: envelope }, llm::Block::Text { text, replay }) => {
                    assert_eq!(text, original);
                    assert_eq!(replay, envelope);
                }
                (
                    session::llm::Block::ToolCall(session::llm::ToolCall {
                        id: original_id,
                        name: original_name,
                        input: original_input,
                        replay: envelope,
                        ..
                    }),
                    llm::Block::ToolCall { id, name, input, replay },
                ) => {
                    assert_eq!(id, original_id);
                    assert_eq!(name, original_name);
                    assert_eq!(input, original_input);
                    assert_eq!(replay, envelope);
                }
                (
                    session::llm::Block::ToolResult {
                        id: original_id,
                        result: session::llm::Returned::Text { text: original, error: original_error, replay: envelope },
                    },
                    llm::Block::ToolResult { id, result: llm::Returned::Text { text, error, replay } },
                ) => {
                    assert_eq!(id, original_id);
                    assert_eq!(text, original);
                    assert_eq!(error, original_error);
                    assert_eq!(replay, envelope);
                }
                _ => panic!("observed root rewrite preserves the independent observed fields"),
            }
        }
    }
    let waking = prompt.messages.last().expect("observed waking User");
    assert_eq!(waking.role, llm::Role::User);
    let [llm::Block::Text { text, replay: None }] = waking.content.as_ref() else { panic!("waking text") };
    assert_eq!(text.as_ref(), smith_agent_world::fixture::RESUME);
}

struct Counted {
    meter: Meter,
    ceiling: u64,
    domain: Option<Domain>,
    env: Env<Limits>,
    out: Queue<Request>,
    caller_charter: Option<run::Charter>,
    caller_workspace: Option<run::Workspace>,
    records: Vec<session::record::Turn>,
    prefix: Option<root::Transcript>,
    saved: Option<root::Transcript>,
    initial: Option<llm::Prompt>,
    held_prompt: Option<llm::Prompt>,
    prompt_copy: Option<llm::Prompt>,
    turn_copy: Option<session::record::Turn>,
    complete: Option<Complete>,
    admitted: Option<Token>,
    read: Option<Token>,
    probe: Option<Token>,
    waiting: bool,
    answer: Option<run::Answer>,
    calls: u32,
    long_open: u32,
    restored_copies: bool,
}

impl Counted {
    fn new(limits: &Limits, ceiling: u64) -> Self {
        Self::with_window(limits, 16_384, ceiling)
    }

    fn with_window(limits: &Limits, window: u32, ceiling: u64) -> Self {
        let meter = Meter::new();
        meter.start();
        let out = Queue::uncapped();
        let domain = Domain::new(
            limits,
            root::Config {
                programs: Box::new([]),
                endpoints: Box::new([run::charter::Endpoint(0)]),
                limits: Box::new([smith_domain::EndpointLimits {
                    protocol_allowance: 0,
                    endpoint: smith_domain::run::charter::Endpoint(0),
                    window,
                    output: 4096,
                    head: Duration::from_secs(60),
                    idle: Duration::from_secs(30),
                }]),
            },
            41,
        );
        let measured = meter.end();
        meter.check(measured, ceiling, limits);
        Self {
            meter,
            ceiling,
            domain: Some(domain),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits },
            out,
            caller_charter: None,
            caller_workspace: None,
            records: Vec::new(),
            prefix: None,
            saved: None,
            initial: None,
            held_prompt: None,
            prompt_copy: None,
            turn_copy: None,
            complete: None,
            admitted: None,
            read: None,
            probe: None,
            waiting: false,
            answer: None,
            calls: 0,
            long_open: 0,
            restored_copies: false,
        }
    }

    fn drain(&mut self, measured: skein_world::domain::heap::Measured) {
        while let Some(request) = self.out.pop() {
            match request {
                Request::Admitted { host_run, run } => {
                    assert_eq!(host_run, WORKER);
                    assert!(self.admitted.replace(run).is_none());
                }
                Request::Lower(smith_domain::Lower::Read { owner, at, .. }) => {
                    if let Some(caller) = &self.caller_charter {
                        let selected = caller.conventions.as_ref().expect("maximum path Start");
                        assert_eq!(at.path.as_ref(), selected.guide.as_ref());
                        assert_eq!(at.path.len(), run::Conventions::PATH_CAPACITY);
                    }
                    assert!(self.read.replace(owner).is_none());
                }
                Request::Lower(smith_domain::Lower::Probe { owner, at, .. }) => {
                    if let Some(caller) = &self.caller_charter {
                        let selected = caller.conventions.as_ref().expect("maximum path Start");
                        assert_eq!(at.path.as_ref(), selected.checks.as_ref());
                        assert_eq!(at.path.len(), run::Conventions::PATH_CAPACITY);
                        assert_eq!(at.root, Token::new(7), "only the writable mount is probed");
                    }
                    assert!(self.probe.replace(owner).is_none());
                }
                Request::Lower(smith_domain::Lower::Complete { owner, prompt, max_completion_bytes, .. }) => {
                    self.calls += 1;
                    assert!(self.complete.replace(Complete { owner, prompt, maximum: max_completion_bytes }).is_none());
                }
                Request::Turn { host_run, number, turn, .. } => {
                    assert_eq!(host_run, WORKER);
                    assert_eq!(number, u32::try_from(self.records.len()).expect("bounded records") + 1);
                    self.records.push(turn);
                }
                Request::Waiting { host_run, .. } => {
                    assert_eq!(host_run, WORKER);
                    self.waiting = true;
                }
                Request::Answer { to, answer, read: _, detail: _ } => {
                    assert_eq!(to.into_token(), PARENT);
                    assert_eq!(self.long_open, 0, "provider work ended before the original Start terminal");
                    assert!(self.answer.replace(answer).is_none(), "one original Start terminal");
                }
                Request::Long { host_run, operation, .. } => {
                    assert_eq!(host_run, WORKER);
                    assert_eq!(operation, root::Operation::Completion, "this fixture only issues provider work");
                    self.long_open = self.long_open.checked_add(1).expect("bounded observed provider spans");
                    assert_eq!(self.long_open, 1, "one observed issued provider call per session");
                }
                Request::LongDone { host_run } => {
                    assert_eq!(host_run, WORKER);
                    self.long_open = self.long_open.checked_sub(1).expect("terminal belongs to observed provider work");
                }
                Request::Lower(smith_domain::Lower::Yield { .. } | smith_domain::Lower::Close { .. }) => {}
                Request::Lower(smith_domain::Lower::Cancel { owner }) => {
                    let complete = self.complete.as_ref().expect("pending compaction is cancelled before typed work");
                    assert_eq!(complete.owner, owner);
                    assert!(matches!(complete.prompt.purpose, llm::Purpose::Compaction { .. }));
                }
                Request::HostCall { .. }
                | Request::WithdrawHost { .. }
                | Request::Save { .. }
                | Request::Deliver { .. }
                | Request::Unauthorized { .. }
                | Request::Refused { .. }
                | Request::Limited { .. }
                | Request::MessageRefused { .. }
                | Request::Lower(
                    smith_domain::Lower::Io { .. }
                    | smith_domain::Lower::Job { .. }
                    | smith_domain::Lower::CancelIo { .. }
                    | smith_domain::Lower::Check { .. }
                    | smith_domain::Lower::Abort { .. },
                ) => {
                    panic!("this positive Wait/text fixture owes only discovery, provider and parent observations")
                }
            }
        }
        self.meter.check(measured, self.ceiling, &self.env.limits);
    }

    fn step(&mut self, event: Event) {
        self.meter.start();
        let domain = self.domain.as_mut().expect("live root");
        domain.reclaim();
        root::step(domain, &self.env, event, &mut self.out);
        while domain.pop_fact().is_some() {}
        while domain.pop_content().is_some() {}
        let measured = self.meter.end();
        self.drain(measured);
    }

    fn resume(&mut self) {
        self.meter.start();
        let domain = self.domain.as_mut().expect("live root");
        domain.reclaim();
        root::resume(domain, &self.env, &mut self.out);
        while domain.pop_fact().is_some() {}
        while domain.pop_content().is_some() {}
        let measured = self.meter.end();
        self.drain(measured);
    }

    fn start(&mut self, history: Option<root::Transcript>, restoring: bool) {
        self.step(Event::Start(smith_domain::Start {
            workspace: Some(workspace()),
            host_run: WORKER,
            activation: if restoring { 2 } else { 1 },
            window: root::Window { turns: u32::MAX, bytes: u64::MAX },
            transcript: history,
            grants: Box::new([Grant {
                renewable: true,
                name: GrantName { account: 0, generation: 1 },
                valid: Duration::from_secs(3600),
            }]),
            ..smith_domain_fixtures::start(PARENT, charter(restoring))
        }));
        assert!(self.admitted.is_some() && self.read.is_some());
        assert_eq!(self.calls, 0, "selected Start ownership survives outstanding discovery");
        assert!(self.complete.is_none());
        assert_eq!(self.long_open, 0, "observed provider work settled before outside waiting");
    }

    fn discover(&mut self) {
        let owner = self.read.take().expect("observed root discovery request");
        self.step(Event::Below(smith_domain::Below::Read { owner, read: run::Read::Missing }));
        if let Some(owner) = self.probe.take() {
            self.step(Event::Below(smith_domain::Below::Probed { owner, executable: false }));
        }
    }

    fn cycle(&mut self, cycles: u32) {
        for _ in 0..100 {
            if let Some(complete) = self.complete.take() {
                if self.initial.is_none() {
                    self.initial = Some(copy_prompt(&complete.prompt));
                }
                if self.held_prompt.is_some() && self.turn_copy.is_none() {
                    self.turn_copy = self.records.last().map(copy_turn);
                }
                let event = self.completion(complete, cycles);
                self.step(event);
            } else {
                self.resume();
            }
            if self.complete.is_none() && (self.waiting || self.answer.is_some()) {
                break;
            }
        }
        assert!(self.complete.is_none());
        assert!(!self.domain.as_ref().expect("live root").is_ready());
        assert!(
            self.waiting || self.answer.is_some(),
            "bounded root loop reached an outside wait or observed terminal"
        );
    }

    fn until_compaction(&mut self, cycles: u32) {
        let complete = self.complete.take().expect("observed restored ordinary request");
        assert!(matches!(complete.prompt.purpose, llm::Purpose::Ordinary { guard: Some(_) }));
        let event = self.completion(complete, cycles);
        self.step(event);
        for _ in 0..100 {
            if self.complete.is_some() {
                break;
            }
            self.resume();
        }
        let complete = self.complete.as_ref().expect("the next request is the compaction");
        assert!(matches!(complete.prompt.purpose, llm::Purpose::Compaction { .. }));
        assert!(self.answer.is_none());
        assert!(!self.domain.as_ref().expect("live root").is_ready());
    }

    fn wake(&mut self, name: Token, text: Box<[u8]>) {
        self.waiting = false;
        let run = self.admitted.expect("observed admitted handle");
        self.step(Event::Message { finish_requested: false, label: b"host".as_slice().into(), run, name, text });
    }

    fn completion(&mut self, complete: Complete, cycles: u32) -> Event {
        let round = complete.prompt.messages.iter().filter(|message| message.role == llm::Role::Assistant).count();
        let waiting = round < usize::try_from(cycles * 2).expect("bounded rounds") && round % 2 == 0;
        if self.prefix.is_some()
            && self.saved.is_some()
            && self.held_prompt.is_some()
            && self.prompt_copy.is_some()
            && self.turn_copy.is_some()
        {
            self.restored_copies = true;
        }
        let content = if waiting {
            llm::Said::ToolCall {
                id: ID.into(),
                name: b"wait".as_slice().into(),
                input: b"{}".as_slice().into(),
                call: llm::Decoded::Served { ask: run::Ask::Wait },
                replay: None,
            }
        } else {
            let text = if round >= usize::try_from(cycles * 2).expect("bounded rounds") { RESTORED } else { SAID };
            llm::Said::Text { text: text.into(), replay: None }
        };
        drop(complete.prompt);
        let event = Event::Below(smith_domain::Below::Completed {
            owner: complete.owner,
            completion: llm::Completion {
                reasoning_dropped: Box::new([]),
                content: Box::new([content]),
                stop: if waiting { llm::Stop::ToolUse } else { llm::Stop::EndTurn },
                usage: llm::Usage { input_tokens: Some(0), output_tokens: Some(1), ..llm::Usage::NONE },
            },
        });
        assert!(terminal_bytes(&event) <= complete.maximum, "typed terminal fits its receiving contract");
        event
    }

    fn replace_root(&mut self, limits: &Limits) {
        assert!(self.answer.is_some(), "previous original Start settled");
        self.meter.start();
        drop(self.domain.take());
        let measured = self.meter.end();
        self.meter.check(measured, self.ceiling, &self.env.limits);
        self.env.limits = *limits;
        self.answer = None;
        self.admitted = None;
        self.read = None;
        self.probe = None;
        self.waiting = false;
        self.calls = 0;
        self.records.clear();
        self.meter.start();
        self.domain = Some(Domain::new(
            limits,
            root::Config {
                programs: Box::new([]),
                endpoints: Box::new([run::charter::Endpoint(0)]),
                limits: Box::new([smith_domain::EndpointLimits {
                    protocol_allowance: 0,
                    endpoint: smith_domain::run::charter::Endpoint(0),
                    window: 16_384,
                    output: 4096,
                    head: Duration::from_secs(60),
                    idle: Duration::from_secs(30),
                }]),
            },
            42,
        ));
        let measured = self.meter.end();
        self.drain(measured);
    }
}

fn assert_generated(history: &root::Transcript, messages: u32) {
    assert_eq!(history.turns.len(), usize::try_from(2 * ((messages - 4) / 5)).expect("reachable Turns"));
    let mut previous = None;
    let mut count = 0_u32;
    let mut large = 0;
    for (index, turn) in history.turns.iter().enumerate() {
        assert_eq!(turn.sequence, u32::try_from(index).expect("bounded sequence") + 1);
        assert_eq!(turn.version, session::record::VERSION);
        assert_eq!(turn.endpoint, Some(session::llm::Endpoint(0)));
        assert_eq!(turn.dialect, 2);
        assert_eq!(turn.messages.len(), if index % 2 == 0 { 3 } else { 2 });
        assert_eq!(turn.messages.iter().filter(|message| message.role == session::llm::Role::Assistant).count(), 1);
        for message in &turn.messages {
            if message.role == session::llm::Role::Assistant {
                assert_eq!(previous, Some(session::llm::Role::User), "whole-history predecessor, not per-Turn minimum");
            } else if previous.is_none() {
                assert_eq!(message.role, session::llm::Role::User);
            }
            previous = Some(message.role);
            count += 1;
            for block in &message.content {
                match block {
                    session::llm::Block::Text { text, .. } => {
                        if text.len() == LARGE {
                            assert_eq!(&text[..6], b"host: ");
                            assert!(text[6..].iter().all(|byte| *byte == b'x'));
                            large += 1;
                        }
                    }
                    session::llm::Block::ToolCall(session::llm::ToolCall { id, name, input, call, .. }) => {
                        assert_eq!(id.as_ref(), ID);
                        assert_eq!(name.as_ref(), b"wait");
                        assert_eq!(input.as_ref(), b"{}");
                        assert_eq!(call, &session::llm::Decoded::Historical);
                    }
                    session::llm::Block::ToolResult {
                        id,
                        result: session::llm::Returned::Text { text, error, replay },
                    } => {
                        assert_eq!(id.as_ref(), ID);
                        assert_eq!(text.as_ref(), b"Messages arrived. Read them before continuing.");
                        assert!(!*error);
                        assert!(replay.is_none());
                    }
                    session::llm::Block::Refusal { .. }
                    | session::llm::Block::Opaque { .. }
                    | session::llm::Block::ToolResult { .. } => panic!("supported fixture records"),
                }
            }
        }
    }
    assert_eq!(count, messages - 4, "maximum reachable history leaves waking and provider slots");
    assert_eq!(large, 1);
}

fn cap(source: &Counted, waking: &[u8]) -> u64 {
    // Fixed SUMMARY and PREAMBLE texts in smith-domain-run/src/prompt.rs;
    // domain/run.md, section 3.3. The summary includes the 1,000-word limit.
    const SUMMARY_BYTES: u64 = 415;
    const PREAMBLE_BYTES: u64 = 97;

    let first = source.initial.as_ref().expect("observed fresh root prompt");
    let [message] = first.messages.as_ref() else { panic!("fresh observed waking message") };
    let [llm::Block::Text { text, replay: None }] = message.content.as_ref() else { panic!("plain charter brief") };
    assert_eq!(text.as_ref(), b"Begin the work your brief describes.");
    assert_eq!(first.served.len(), 2);
    // Independent public opening ownership plus observed concrete record fields.
    // Message/Turn envelopes are deliberately separate from this payload cap.
    let opening = len(&first.model)
        + len(&first.system)
        + 2 * size::<session::llm::Descriptor>()
        + size::<session::llm::Block>()
        + len(text)
        + len(waking)
        + SUMMARY_BYTES
        + PREAMBLE_BYTES; // Retained summary instruction and preamble beside the task and history.
    let prefix = source
        .prefix
        .as_ref()
        .expect("observed prefix")
        .turns
        .iter()
        .flat_map(|turn| &turn.messages)
        .map(|message| record_blocks(&message.content))
        .sum::<u64>();
    opening
        + prefix
        + session::completion_reserve(&source.env.limits.session).expect("public required receiving credit")
        + size::<session::llm::Block>()
        + u64::from(source.env.limits.session.tools.path_bytes) * u64::from(source.env.limits.session.completion_blocks)
}

fn generate_prefix(counted: &mut Counted, messages: u32, cycles: u32) -> Limits {
    counted.start(None, false);
    counted.discover();
    counted.cycle(cycles + 1);
    assert!(counted.waiting);
    for at in 0..cycles {
        counted.waiting = false;
        let run = counted.admitted.expect("observed admitted handle");
        let text = if at == 0 { vec![b'x'; LARGE - 6].into_boxed_slice() } else { b"x".as_slice().into() };
        counted.step(Event::Message {
            finish_requested: false,
            label: Box::from(&b"host"[..]),
            run,
            name: Token::new(u64::from(at)),
            text,
        });
        counted.cycle(cycles + 1);
        assert!(counted.waiting, "wake {at}: records={} answer={:?}", counted.records.len(), counted.answer);
    }
    let copy_meter = Meter::new();
    copy_meter.start();
    let prefix = root::Transcript {
        version: session::record::VERSION,
        endpoint: Some(session::llm::Endpoint(0)),
        dialect: 2,
        turns: counted.records.clone().into(),
    };
    let copied = copy_meter.end();
    let expected = transcript_bytes(&prefix);
    assert_eq!(copied.held(), expected, "concrete source-record handoff owns all public envelopes and payloads");
    assert!(copied.peak() <= expected, "record handoff has no unpriced construction transient");
    counted.prefix = Some(prefix);
    assert_generated(counted.prefix.as_ref().expect("observed selected history"), messages);
    counted.saved = counted.prefix.as_ref().map(copy_transcript);
    let capped = cap(counted, smith_agent_world::fixture::RESUME);
    let tight = Limits {
        session: session::Limits { history_bytes: capped, ..counted.env.limits.session },
        ..counted.env.limits
    };
    let run = counted.admitted.expect("observed source run");
    counted.waiting = false;
    counted.step(Event::Cancel { run });
    counted.cycle(cycles + 1);
    assert!(
        matches!(counted.answer, Some(run::Answer::Failed { failure: run::RunFailure::Cancelled, turns, .. }) if turns == 2 * cycles + 1)
    );

    tight
}

fn restore_overlap(counted: &mut Counted, messages: u32, cycles: u32) {
    // The full prefix owns messages - 4 slots. Restored waking, the observed
    // Wait/result, the offered message and text, and the next nudge occupy six
    // more. Two provider slots and one summary slot remain for the next held
    // wait, so this overlap uses messages + 5. Payload caps retain the source contract.
    let overlapping = bounds(messages.checked_add(5).expect("second receiving entrance and its summary slot"));
    counted.replace_root(&overlapping);
    counted.start(counted.prefix.as_ref().map(copy_transcript), true);
    counted.discover();
    assert_eq!(counted.calls, 1, "full source history starts one observed restored callback");
    let complete = counted.complete.as_ref().expect("observed restored Complete before caller-held copies");
    assert_eq!(complete.prompt.messages.len(), usize::try_from(messages - 3).expect("full restored plus waking"));
    assert_rewritten(counted.prefix.as_ref().expect("maximum generated prefix"), &complete.prompt);
    counted.held_prompt = Some(copy_prompt(&complete.prompt));
    counted.prompt_copy = counted.held_prompt.as_ref().map(copy_prompt);
    counted.cycle(cycles + 2);
    assert!(counted.waiting && counted.records.is_empty(), "first restored typed wait remains held");
    counted.wake(Token::new(100), b"wake".as_slice().into());
    counted.cycle(cycles + 2);
    assert_eq!(counted.calls, 3, "two settled callbacks and the next held wait use three observed neighbours");
    assert_eq!(counted.records.len(), 2, "both restored terminals produced concrete Turns");
    assert_eq!(counted.records[0].sequence, 2 * cycles + 1);
    assert_eq!(counted.records[1].sequence, 2 * cycles + 2);
    assert!(
        counted.restored_copies,
        "full generated histories, both restored Prompt copies and a Turn copy coexist with caller copies"
    );
    assert!(counted.waiting, "observed restored Wait/text pair reaches Waiting after both typed terminals");
    let run = counted.admitted.expect("observed overlapping restore activation");
    counted.waiting = false;
    counted.step(Event::Cancel { run });
    counted.cycle(cycles + 2);
    assert!(matches!(counted.answer, Some(run::Answer::Failed { failure: run::RunFailure::Cancelled, turns: 3, .. })));
    counted.resume();
}

fn restore_exact(counted: &mut Counted, messages: u32, cycles: u32, tight: &Limits) {
    let prompt = counted.held_prompt.as_ref().expect("observed overlapping restore");
    let waking = prompt.messages.last().expect("observed resumed waking message");
    let [llm::Block::Text { text: waking, replay: None }] = waking.content.as_ref() else {
        panic!("plain resumed waking text")
    };
    // The unchanged source history and independently specified resumed wake
    // attain the receiving payload cap, preserving the LARGE message and envelopes.
    assert_eq!(cap(counted, waking), tight.session.history_bytes, "history plus observed wake attains the payload cap");
    counted.replace_root(tight);
    counted.start(counted.prefix.as_ref().map(copy_transcript), true);
    counted.discover();
    assert_eq!(counted.calls, 1, "exact reservation-inclusive payload is admitted before typed work");
    let complete = counted.complete.as_ref().expect("observed rewritten Complete");
    assert_eq!(complete.prompt.messages.len(), usize::try_from(messages - 3).expect("restored plus waking"));
    assert_eq!(complete.prompt.model, counted.initial.as_ref().expect("original observation").model);
    assert_eq!(complete.prompt.system, counted.initial.as_ref().expect("original observation").system);
    assert_rewritten(counted.prefix.as_ref().expect("original generated prefix"), &complete.prompt);
    counted.held_prompt = Some(copy_prompt(&complete.prompt));
    counted.prompt_copy = counted.held_prompt.as_ref().map(copy_prompt);
    counted.until_compaction(cycles);
    let [turn] = counted.records.as_slice() else { panic!("one observed concrete restored Turn") };
    assert_eq!(turn.sequence, 2 * cycles + 1);
    assert_eq!(turn.messages.len(), 2);
    let [_, assistant] = turn.messages.as_ref() else { panic!("observed wake/assistant Turn") };
    let [session::llm::Block::Text { text, .. }] = assistant.content.as_ref() else {
        panic!("observed restored typed text")
    };
    assert_eq!(text.as_ref(), RESTORED);
    counted.turn_copy = Some(copy_turn(turn));
    let owner = counted.complete.as_ref().expect("held compaction callback").owner;
    let run = counted.admitted.expect("restored activation");
    counted.step(Event::Cancel { run });
    counted.resume();
    drop(counted.complete.take());
    counted.step(Event::Below(smith_domain::Below::Cancelled { owner }));
    assert!(matches!(counted.answer, Some(run::Answer::Failed { failure: run::RunFailure::Cancelled, turns: 1, .. })));
    counted.resume();
}

fn refuse_one_over(counted: &mut Counted, cycles: u32, tight: &Limits) {
    for extra_message in [true, false] {
        let mut negative = *tight;
        if extra_message {
            negative.session.history_bytes += 16_384;
            assert!(size::<session::llm::Block>() + len(b"x") < 16_384, "count control has independent byte headroom");
        } else {
            // Pin the byte-only refusal to its prior receiving cap, independent
            // of the new minimum-result and summary admission headroom.
            negative.session.history_bytes -= size::<session::llm::Block>()
                + u64::from(negative.session.tools.path_bytes) * u64::from(negative.session.completion_blocks);
        }
        counted.replace_root(&negative);
        let mut history = copy_transcript(counted.prefix.as_ref().expect("saved observed records"));
        if extra_message {
            let last = history.turns.last_mut().expect("saved turns");
            let mut messages = last.messages.to_vec();
            messages.push(session::llm::Message {
                role: session::llm::Role::User,
                content: Box::new([session::llm::Block::Text { text: b"x".as_slice().into(), replay: None }]),
            });
            last.messages = messages.into();
        } else {
            let text = history
                .turns
                .iter_mut()
                .flat_map(|turn| &mut turn.messages)
                .flat_map(|message| &mut message.content)
                .find_map(|block| match block {
                    session::llm::Block::Text { text, .. } if text.len() == LARGE => Some(text),
                    session::llm::Block::Text { .. }
                    | session::llm::Block::Refusal { .. }
                    | session::llm::Block::Opaque { .. }
                    | session::llm::Block::ToolCall(_)
                    | session::llm::Block::ToolResult { .. } => None,
                })
                .expect("literal generated large message");
            // The prefix and resumed wake attain the receiving cap, so one
            // more byte is exactly over receiving credit.
            *text = vec![b'x'; LARGE + 1].into_boxed_slice();
        }
        counted.start(Some(history), true);
        counted.discover();
        counted.cycle(cycles);
        assert_eq!(counted.calls, 0, "one-over valid history starts no provider or owned tool");
        assert_eq!(counted.records.len(), 0);
        assert!(matches!(
            counted.answer,
            Some(run::Answer::Failed {
                failure: run::RunFailure::Transcript(run::TranscriptRefusal::TooLarge),
                turns: 0,
                ..
            })
        ));
    }
}

fn reclaim_every_owner(mut counted: Counted, tight: &Limits) {
    counted.meter.start();
    drop(counted.domain.take());
    drop(counted.prefix.take());
    drop(counted.saved.take());
    drop(counted.initial.take());
    drop(counted.caller_charter.take());
    drop(counted.caller_workspace.take());
    drop(counted.held_prompt.take());
    drop(counted.prompt_copy.take());
    drop(counted.turn_copy.take());
    counted.records.clear();
    counted.records.shrink_to_fit();
    let measured = counted.meter.end();
    counted.meter.check(measured, counted.ceiling, tight);
    let Counted { meter, out, .. } = counted;
    drop(out);
    assert_eq!(meter.held(), 0, "all root, retained caller copies reclaimed");
}

#[test]
fn restored_root_arrays_payload_rewrites_and_turn_copies_stay_within_the_attained_bound() {
    // Root, restored history and caller copies share this scenario ceiling.
    const RESTORE_CEILING: u64 = 7_500_000;
    for messages in [19, 39] {
        let cycles = (messages - 4) / 5;
        let mut counted = Counted::new(&bounds(messages), RESTORE_CEILING);
        let tight = generate_prefix(&mut counted, messages, cycles);
        restore_overlap(&mut counted, messages, cycles);
        restore_exact(&mut counted, messages, cycles, &tight);
        refuse_one_over(&mut counted, cycles, &tight);
        reclaim_every_owner(counted, &tight);
    }
}

fn maximum_workspace(limits: &run::Limits) -> run::Workspace {
    let mounted = run::Workspace {
        directories: Box::new([
            run::Directory {
                host: format!(
                    "/{:h<width$}",
                    "",
                    width = usize::try_from(limits.host_path_bytes - 1).expect("positive host cap")
                )
                .into_bytes()
                .into(),
                name: vec![b'w'; 64].into_boxed_slice(),
                root: Token::new(7),
                writable: true,
                git: true,
                conflicts: (0..limits.conflicts)
                    .map(|index| {
                        if index == 0 {
                            vec![
                                b'p';
                                usize::try_from(limits.conflict_path_bytes).expect("bounded initial conflict path cap")
                            ]
                            .into_boxed_slice()
                        } else {
                            format!("p{index:02}").into_bytes().into_boxed_slice()
                        }
                    })
                    .collect(),
            },
            run::Directory {
                host: format!(
                    "/{:h<width$}",
                    "",
                    width = usize::try_from(limits.host_path_bytes - 1).expect("positive host cap")
                )
                .into_bytes()
                .into(),
                name: vec![b'r'; 64].into_boxed_slice(),
                root: Token::new(8),
                writable: false,
                git: true,
                conflicts: (0..limits.conflicts)
                    .map(|index| format!("q{index:02}").into_bytes().into_boxed_slice())
                    .collect(),
            },
        ]),
    };
    assert_eq!(mounted.directories.len(), usize::try_from(limits.directories).expect("bounded directory count"));
    for directory in &mounted.directories {
        assert_eq!(directory.name.len(), usize::try_from(limits.directory_name_bytes).expect("bounded mount name cap"));
        assert_eq!(
            directory.conflicts.len(),
            usize::try_from(limits.conflicts).expect("bounded initial conflict count")
        );
        assert!(directory.conflicts.iter().all(|path| path.len()
            <= usize::try_from(limits.conflict_path_bytes).expect("bounded initial conflict path cap")));
    }
    assert_eq!(
        mounted.directories[0].conflicts[0].len(),
        usize::try_from(limits.conflict_path_bytes).expect("bounded initial conflict path cap")
    );
    mounted
}

/// Fill the existing Start aggregate, pricing independent public Section cells
/// before redistributing its remaining bytes among role, titles and bodies.
fn fill_main_context(charter: &mut run::Charter, workspace: Option<&run::Workspace>, limits: &run::Limits) {
    let previous = sum([
        len(&charter.instructions),
        cells::<run::Section>(charter.brief.sections.len()),
        sum(charter.brief.sections.iter().map(|section| sum([len(&section.title), len(&section.text)]))),
    ]);
    let fixed = charter_bytes(charter) - previous + workspace_bytes(workspace);
    let count = usize::try_from(limits.brief_sections).expect("bounded receiving section count");
    let payload = limits.run_bytes - fixed - cells::<run::Section>(count);
    let instruction_bytes = usize::try_from(payload / 4).expect("bounded role payload");
    let mut instructions = vec![b'i'; instruction_bytes];
    instructions[..b"@root-memory".len()].copy_from_slice(b"@root-memory");
    charter.instructions = instructions.into_boxed_slice();
    let mut remainder = payload - len(&charter.instructions);
    let mut sections = Vec::with_capacity(count);
    for section in 0..count {
        let remaining = u64::try_from(count - section).expect("bounded section suffix");
        let title_bytes = usize::try_from(remainder / remaining / 3).expect("bounded title payload");
        let text_bytes =
            usize::try_from(remainder / remaining).expect("bounded Section payload fits usize") - title_bytes;
        let title = vec![b't'; title_bytes].into_boxed_slice();
        let text = vec![b's'; text_bytes].into_boxed_slice();
        remainder -= len(&title) + len(&text);
        sections.push(run::Section { title, text });
    }
    assert_eq!(remainder, 0, "every original aggregate byte is allocated exactly");
    assert!(sections.iter().all(|section| !section.title.is_empty() && !section.text.is_empty()));
    assert_eq!(sections.len(), count, "maximum receiving Section count is attained");
    charter.brief = run::Brief { sections: sections.into_boxed_slice() };
    assert_eq!(charter_bytes(charter) + workspace_bytes(workspace), limits.run_bytes);
}

fn maximum_convention_charter() -> run::Charter {
    let path_bytes = run::Conventions::PATH_CAPACITY;
    let make_path = |byte| {
        let mut path = vec![byte; path_bytes];
        for slash in (63..path_bytes - 1).step_by(64) {
            path[slash] = b'/';
        }
        path.into_boxed_slice()
    };
    run::Charter {
        reserve: run::charter::Reserve { turns: 0, time: bounds(16).run.check_timeout },
        conventions: Some(run::Conventions { guide: make_path(b'g'), checks: make_path(b'c') }),
        outcome: run::outcome::OutcomeSpec {
            change: Some(run::outcome::ChangeSpec { checks_must_pass: true, fields: Box::new([]) }),
            ..charter(false).outcome
        },
        ..charter(false)
    }
}

#[test]
fn a_restored_run_keeps_full_brief_and_custom_paths_with_caller_copies() {
    // Full brief, workspace paths and independent caller copies share this ceiling.
    const BRIEF_CEILING: u64 = 7_500_000;
    let path_bytes = run::Conventions::PATH_CAPACITY;
    let mut receiving = bounds(16);
    receiving.run.directories = 2;
    // Host alias metadata is retained independently of the unchanged maximum
    // Brief/convention text; give this attained fixture its additional bytes.
    receiving.run.run_bytes = 16_384 + u64::from(receiving.run.directories) * u64::from(receiving.run.host_path_bytes);
    receiving.session.tools.repos = 2;
    // The receiving root must retain every rendered observed run result. Select
    // the public checked feedback bound exactly, rather than widening the
    // original history/observed completion caps or bypassing root admission.
    receiving.session.delegated_result_bytes = root::feedback_bound(&receiving.run)
        .expect("maximum path charter has a finite exact receiving feedback bound")
        .max(receiving.session.delegated_result_bytes);
    let mut counted = Counted::with_window(&receiving, 65_536, BRIEF_CEILING);
    let mut caller = maximum_convention_charter();
    caller.llm.window = 65_536;
    let mounted = Some(maximum_workspace(&receiving.run));
    fill_main_context(&mut caller, mounted.as_ref(), &receiving.run);
    let caller_bytes = charter_bytes(&caller);
    let both_paths = path_bytes.checked_mul(2).expect("two bounded convention paths fit a fixture size");
    assert!(caller_bytes >= u64::try_from(both_paths).expect("bounded maximum convention payload fits u64"));
    assert_eq!(caller_bytes + workspace_bytes(mounted.as_ref()), receiving.run.run_bytes);
    counted.caller_workspace = mounted.clone();
    let active = maximum_caller_start(&caller);
    counted.caller_charter = Some(caller);
    counted.step(Event::Start(smith_domain::Start {
        host_run: WORKER,
        window: root::Window { turns: u32::MAX, bytes: u64::MAX },
        workspace: mounted,
        grants: Box::new([Grant {
            renewable: true,
            name: GrantName { account: 0, generation: 1 },
            valid: Duration::from_secs(3600),
        }]),
        ..smith_domain_fixtures::start(PARENT, active)
    }));
    assert!(counted.admitted.is_some() && counted.read.is_some());
    assert_eq!(counted.calls, 0, "maximum retained paths and caller Start coexist before any provider effect");
    let owner = counted.read.take().expect("observed maximum-path discovery");
    counted.step(Event::Below(smith_domain::Below::Read {
        owner,
        read: run::Read::Text { text: b"Maximum host-selected guide".as_slice().into(), whole: true },
    }));
    let owner = counted.probe.take().expect("observed maximum check-path Probe");
    counted.step(Event::Below(smith_domain::Below::Probed { owner, executable: true }));
    let owner = counted.read.take().expect("observed maximum guide-path Read for the readonly mount");
    counted.step(Event::Below(smith_domain::Below::Read {
        owner,
        read: run::Read::Text { text: b"Maximum readonly guide".as_slice().into(), whole: true },
    }));
    let first = counted.complete.as_ref().expect("observed first root Complete");
    assert!(first.prompt.system.len() > 16_384, "combined metadata exceeds the original typed string cap");
    assert!(first.prompt.system.len() <= 32_768, "explicit typed string cap fits the observed combined prompt");
    let source = counted.caller_charter.as_ref().expect("original caller Start remains independently owned");
    assert!(first.prompt.system.starts_with(&source.instructions), "observed typed main receives full literal role");
    assert_eq!(
        source.brief.sections.len(),
        usize::try_from(receiving.run.brief_sections).expect("bounded receiving Section count fits usize")
    );
    let selected = counted
        .caller_charter
        .as_ref()
        .expect("caller retains its original maximum-path Start")
        .conventions
        .as_ref()
        .expect("caller explicitly selected both maximum convention paths");
    assert_eq!(selected.guide.len(), path_bytes);
    assert_eq!(selected.checks.len(), path_bytes);
    assert_eq!(
        first.prompt.system.windows(path_bytes).filter(|path| *path == selected.guide.as_ref()).count(),
        2,
        "both observed guide headings own the maximum selected path"
    );
    assert_eq!(
        first.prompt.system.windows(path_bytes).filter(|path| *path == selected.checks.as_ref()).count(),
        1,
        "the writable mount owns the maximum selected check label"
    );
    assert!(counted.probe.is_none(), "the readonly mount adds no second check probe");
    counted.cycle(1);
    assert!(counted.waiting && counted.records.is_empty(), "observed wait is held before the typed text callback");
    counted.wake(Token::new(100), b"wake".as_slice().into());
    counted.cycle(2);
    assert!(counted.waiting, "observed Wait/text terminals settle with independent caller copies");
    assert!(counted.caller_charter.is_some(), "caller owns both maximum paths throughout observed typed handoffs");
    let run = counted.admitted.expect("maximum-path activation was admitted");
    counted.waiting = false;
    counted.step(Event::Cancel { run });
    counted.cycle(1);
    assert!(matches!(counted.answer, Some(run::Answer::Failed { failure: run::RunFailure::Cancelled, turns: 3, .. })));
    reclaim_every_owner(counted, &receiving);
}

/// Copy caller-owned maximum context into an independent observed Start.
fn maximum_caller_start(caller: &run::Charter) -> run::Charter {
    run::Charter {
        check_timeout: None,
        instructions: caller.instructions.clone(),
        brief: run::Brief {
            sections: caller
                .brief
                .sections
                .iter()
                .map(|section| run::Section { title: section.title.clone(), text: section.text.clone() })
                .collect(),
        },
        conventions: caller.conventions.clone(),
        outcome: caller.outcome.clone(),
        reserve: caller.reserve,
        llm: caller.llm.clone(),
        ..charter(false)
    }
}
