use super::*;

pub(crate) const PEER: Token = Token::new(100);

pub(crate) const BUDGET: Budget = Budget { turns: 10, spend: 10_000, time: Duration::from_secs(600) };

/// Fixed admission and ownership limits shared by focused run and result tests.
/// Contract: smith/docs/design/domain/run.md, sections 3.1, 7.1 and 13.
pub(crate) const LIMITS: Limits = crate::fixture::SMALL;

/// The domain, its environment, and its pending output.
pub(crate) struct Harness {
    pub(super) observed: Queue<crate::facts::Fact>,
    pub(super) domain: Domain,
    pub(super) env: Env<Limits>,
    pub(super) out: Queue<Request>,
    pub(super) prices: alloc::collections::BTreeMap<Token, u64>,
    pub(super) checks_ended: u32,
    pub(super) peers: alloc::collections::BTreeMap<Token, Token>,
}

impl Harness {
    pub(super) fn keep_facts(&mut self) {
        while let Some(fact) = self.domain.pop_fact() {
            self.observed.push(fact);
        }
    }

    pub(super) fn pop_fact(&mut self) -> Option<crate::facts::Fact> {
        match self.observed.pop() {
            Some(fact) => Some(fact),
            None => self.domain.pop_fact(),
        }
    }

    #[expect(
        clippy::large_types_passed_by_value,
        reason = "programming-model.md, section 10.2: the harness receives a copied immutable profile"
    )]
    pub(super) fn new(limits: Limits) -> Harness {
        Harness {
            observed: Queue::with_capacity(4096),
            domain: Domain::new(&limits, 0),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            out: Queue::uncapped(),
            prices: alloc::collections::BTreeMap::new(),
            checks_ended: 0,
            peers: alloc::collections::BTreeMap::new(),
        }
    }

    /// Steps `event`, returning what it requests, oldest first.
    pub(super) fn step(&mut self, event: Event) -> Box<[Request]> {
        if let Event::Started { conversation, peer } = &event {
            self.peers.insert(*peer, *conversation);
        }
        // Controls supply own units alongside raw usage; the run receives the
        // two independent session notices in protocol order.
        if let Event::Used { conversation, spend } = &event {
            let previous = self.prices.get(conversation).copied().unwrap_or(0);
            let own_spent = previous.checked_add(spend.units).expect("fixture own units");
            self.prices.insert(*conversation, own_spent);
            let priced = Event::Priced { conversation: *conversation, own_spent, subtree_spent: own_spent };
            self.keep_facts();
            step(&mut self.domain, &self.env, priced, &mut self.out);
        }
        if let Event::Priced { conversation, own_spent, .. } = &event {
            self.prices.insert(*conversation, *own_spent);
        }
        self.keep_facts();
        step(&mut self.domain, &self.env, event, &mut self.out);
        self.drain()
    }

    pub(super) fn fire(&mut self) -> Box<[Request]> {
        assert!(self.domain.is_due(self.env.now), "an alarm is due");
        self.keep_facts();
        fire(&mut self.domain, &self.env, &mut self.out);
        self.drain()
    }

    pub(super) fn resume(&mut self) -> Box<[Request]> {
        self.keep_facts();
        self.domain.reclaim();
        crate::resume(&mut self.domain, &self.env, &mut self.out);
        self.drain()
    }

    pub(super) fn drain(&mut self) -> Box<[Request]> {
        #[expect(
            clippy::disallowed_types,
            reason = "programming-model.md, section 10.2: the deterministic harness collects an uncapped output drain"
        )]
        let mut requests = Vec::new();
        // Each cancellation settles the writer's jobs once; its finite follow-up
        // requests are translated before this entrance returns.
        while let Some(request) = self.out.pop() {
            match request {
                Request::CancelJobs { peer } => {
                    let conversation = self.peers[&peer];
                    self.keep_facts();
                    step(&mut self.domain, &self.env, Event::JobsCancelled { conversation }, &mut self.out);
                }
                Request::ChecksEnded { .. } => {
                    self.checks_ended = self.checks_ended.checked_add(1).expect("bounded checks");
                }
                request @ (Request::CompletionReady { .. }
                | Request::MessageRefused { .. }
                | Request::Waiting { .. }
                | Request::Turn { .. }
                | Request::HostCall { .. }
                | Request::WithdrawHost { .. }
                | Request::Admitted { .. }
                | Request::Answer { .. }
                | Request::Open { .. }
                | Request::Say { .. }
                | Request::Yield { .. }
                | Request::Close { .. }
                | Request::Read { .. }
                | Request::Probe { .. }
                | Request::Check { .. }
                | Request::Abort { .. }
                | Request::Checking { .. }
                | Request::Deliver { .. }
                | Request::Return { .. }) => requests.push(request),
            }
        }
        requests.into_boxed_slice()
    }

    /// Starts a run of `charter` for call `call`: what it requests.
    pub(super) fn start(&mut self, call: u64, charter: Charter) -> Box<[Request]> {
        let mounted = if charter.outcome.change.is_some() {
            Some(coding_workspace())
        } else if charter.grants.deliver.is_some() {
            let mut mounted = Some(workspace());
            mounted.as_mut().unwrap().directories[0].writable = true;
            mounted
        } else {
            Some(workspace())
        };
        self.start_workspace(call, charter, mounted)
    }

    pub(super) fn start_workspace(
        &mut self,
        call: u64,
        charter: Charter,
        workspace: Option<Workspace>,
    ) -> Box<[Request]> {
        let reply_to = ReplyTo::new(Token::new(call));
        self.step(Event::Start {
            messages: Box::default(),
            window: crate::Window { turns: u32::MAX, bytes: u64::MAX, largest_turn: 1 },
            reply_to,
            host_run: Token::new(call),
            identity: None,
            activation: 1,
            charter,
            workspace,
            transcript: None,
            resumed: false,
        })
    }

    /// Starts a run of the test charter for call `call`, which is admitted:
    /// the run's token, once it has looked for its one repository's guide.
    pub(super) fn prepare(&mut self, call: u64) -> Token {
        let requests = self.start(call, charter());
        let [Request::Admitted { host_run, run }, Request::Read { owner, .. }] = &*requests else {
            panic!("expected an admitted run, got {requests:?}");
        };
        assert_eq!((*host_run, owner), (Token::new(call), run));
        *run
    }

    /// Starts a run of the test charter for call `call`, which is admitted and
    /// finds no guide: the run's token and its main conversation's.
    pub(super) fn admit(&mut self, call: u64) -> (Token, Token) {
        let run = self.prepare(call);
        let requests = self.step(Event::Read { owner: run, read: Read::Missing });
        let [Request::Open { conversation, opening: _ }] = &*requests else {
            panic!("expected main to open, got {requests:?}");
        };
        (run, *conversation)
    }

    /// Admits a run for call `call` and starts its main conversation as
    /// `peer`: the run's token and main's.
    pub(super) fn running(&mut self, call: u64, peer: u64) -> (Token, Token) {
        let (run, conversation) = self.admit(call);
        assert!(self.step(Event::Started { conversation, peer: Token::new(peer) }).is_empty(), "starting is quiet");
        (run, conversation)
    }

    pub(super) fn after(&mut self, span: Duration) {
        self.env.now = self.env.now.checked_add(span).expect("the test stays in range");
    }

    /// Starts a run of `coding()` for call `call` whose repositories both have
    /// checks, and its main conversation as `peer`: the run's token and main's.
    pub(super) fn coding(&mut self, call: u64, peer: u64) -> (Token, Token) {
        self.coding_policy(call, peer, coding())
    }

    pub(super) fn coding_policy(&mut self, call: u64, peer: u64, policy: Charter) -> (Token, Token) {
        let requests = self.start(call, policy);
        let [Request::Admitted { run, .. }, Request::Read { .. }] = &*requests else {
            panic!("expected a read, got {requests:?}");
        };
        let run = *run;
        drop(self.step(Event::Read { owner: run, read: Read::Missing }));
        drop(self.step(Event::Probed { owner: run, executable: true }));
        drop(self.step(Event::Read { owner: run, read: Read::Missing }));
        let requests = self.step(Event::Probed { owner: run, executable: true });
        let [Request::Open { conversation, .. }] = &*requests else {
            panic!("expected main to open, got {requests:?}");
        };
        let conversation = *conversation;
        drop(self.step(Event::Started { conversation, peer: Token::new(peer) }));
        (run, conversation)
    }

    /// Has main finish with `change()` as call `call`: the owner of the
    /// landing, whose first check is in flight.
    pub(super) fn land(&mut self, conversation: Token, call: u64) -> Token {
        let requests = self.step(finish(conversation, call, Declared::Change(change())));
        let [Request::Check { owner, program, deadline, tail }, Request::Checking { host_run: _, deadline: until }] =
            &*requests
        else {
            panic!("expected the first check, got {requests:?}");
        };
        assert_eq!(program, &Place { root: Token::new(900), path: bytes(b".temper/pre-pr") });
        assert_eq!((*tail, deadline), (LIMITS.check_tail, until));
        assert_eq!(*deadline, self.env.now.saturating_add(LIMITS.check_timeout));
        *owner
    }

    /// Starts a run of `charter` for call `call`, finding no guide, and its
    /// main conversation as `peer`: the run's token and main's.
    pub(super) fn running_on(&mut self, call: u64, peer: u64, charter: Charter) -> (Token, Token) {
        let requests = self.start(call, charter);
        let [Request::Admitted { run, .. }, Request::Read { .. }] = &*requests else {
            panic!("expected a read, got {requests:?}");
        };
        let run = *run;
        let requests = self.step(Event::Read { owner: run, read: Read::Missing });
        let [Request::Open { conversation, .. }] = &*requests else {
            panic!("expected main to open, got {requests:?}");
        };
        let conversation = *conversation;
        drop(self.step(Event::Started { conversation, peer: Token::new(peer) }));
        (run, conversation)
    }

    /// Has `asker` ask for a sub-agent as `call`, started as `peer`: its
    /// opening and its token.
    pub(super) fn child(
        &mut self,
        asker: Token,
        call: u64,
        wanted: crate::charter::Families,
        peer: u64,
    ) -> (Token, Opening) {
        let requests = self.step(ask(asker, call, wanted, None, None));
        let Ok(one) = Box::<[Request; 1]>::try_from(requests) else { panic!("expected one request") };
        let [Request::Open { conversation, opening }] = *one else { panic!("expected the sub-agent to open") };
        drop(self.step(Event::Started { conversation, peer: Token::new(peer) }));
        (conversation, opening)
    }
}

pub(crate) fn bytes(text: &[u8]) -> Box<[u8]> {
    Box::from(text)
}

pub(crate) fn rule(name: &[u8], min: u32, max: u32) -> VerdictRule {
    VerdictRule {
        name: bytes(name),
        text_max: 1024,
        fields: Box::new([]),
        items: crate::outcome::ItemSpec {
            min,
            max,
            kinds: Box::new([
                crate::outcome::ItemRule {
                    kind: bytes(b"blocking"),
                    fields: Box::new([
                        FieldRule { name: bytes(b"path"), max: 1024 },
                        FieldRule { name: bytes(b"body"), max: 1024 },
                    ]),
                },
                crate::outcome::ItemRule {
                    kind: bytes(b"nit"),
                    fields: Box::new([
                        FieldRule { name: bytes(b"path"), max: 1024 },
                        FieldRule { name: bytes(b"body"), max: 1024 },
                    ]),
                },
            ]),
        },
    }
}

pub(crate) fn workspace() -> Workspace {
    Workspace {
        directories: Box::new([Directory {
            host: Box::from(b"/host".as_slice()),
            name: bytes(b"temper"),
            root: Token::new(900),
            writable: false,
            git: true,
            conflicts: Box::new([]),
        }]),
    }
}

pub(crate) fn charter() -> Charter {
    Charter {
        mode: crate::Mode::Finite,
        reserve: crate::charter::Reserve { turns: 0, time: Duration::ZERO },
        instructions: Box::new([]),
        brief: crate::Brief {
            sections: Box::new([crate::Section {
                title: b"Task".as_slice().into(),
                text: bytes(b"Review the change."),
            }]),
        },

        grants: Grants {
            wait: true,
            deliver: None,
            tools: Tools { inspect: true, modify: false, shell: true },

            agents: false,
            host_tools: Box::new([HostTool {
                name: bytes(b"comment"),
                description: b"Host action".as_slice().into(),
                schema: b"{}".as_slice().into(),
                effect: crate::HostEffect::Read,
                timeout: Duration::from_secs(5),
            }]),
        },
        outcome: OutcomeSpec {
            change: None,
            verdicts: Box::new([rule(b"approve", 0, 0), rule(b"request", 1, 8)]),
            report: None,
            failure: None,
        },
        budget: BUDGET,
        llm: Llm {
            effort: None,
            prices: crate::Prices { input: 1, cached: 1, cache_write: None, output: 1, unit: 1_000_000 },
            account: Some(0),
            endpoint: Endpoint(1),
            model: bytes(b"model-a"),
            window: 8192,
            output: 1024,
            dialect: 1,
        },
        models: Box::new([]),
        conventions: Some(crate::Conventions {
            guide: b"AGENTS.md".as_slice().into(),
            checks: b".temper/pre-pr".as_slice().into(),
        }),
        resume: false,
        waiting: Duration::from_secs(30),
    }
}

pub(crate) fn spend(input: u64) -> Spend {
    Spend {
        units: input,
        turns: 1,
        input: Some(input),
        output: Some(10),
        cache_read: Some(0),
        cache_write: Some(0),
        reasoning: None,
    }
}

/// What a run answered, and to which call.
pub(crate) fn answered(requests: Box<[Request]>) -> (u64, Answer) {
    let Ok(one) = Box::<[Request; 1]>::try_from(requests) else {
        panic!("expected one request");
    };
    let [Request::Answer { to, answer, read: _, detail: _ }] = *one else {
        panic!("expected an answer");
    };
    (to.into_token().raw(), answer)
}

pub(crate) fn failed(failure: Failure, spent: Spend) -> Answer {
    Answer::Failed { failure, spent, turns: 0 }
}

pub(crate) fn repository(name: &[u8]) -> Directory {
    Directory {
        host: Box::from(b"/host".as_slice()),
        name: bytes(name),
        root: Token::new(1),
        writable: true,
        git: true,
        conflicts: Box::new([]),
    }
}

pub(crate) fn spec(verdicts: Box<[VerdictRule]>) -> OutcomeSpec {
    OutcomeSpec { change: None, verdicts, report: None, failure: None }
}

pub(crate) fn end_turn(conversation: Token) -> Event {
    Event::Yielded { conversation, stop: Stop::EndTurn, text: bytes(b"done, I think") }
}

// Finishing.

/// When a conversation of a run started at zero expires: the run's deadline.
pub(crate) const EXPIRY: Time = Time::from_nanos(BUDGET.time.as_nanos());

/// What a run's main conversation asks when it finishes with `outcome`, as
/// its call `call`.
pub(crate) fn finish(conversation: Token, call: u64, outcome: Declared) -> Event {
    finish_by(conversation, call, outcome, EXPIRY)
}

/// The same, with the call due by `deadline`.
pub(crate) fn finish_by(conversation: Token, call: u64, outcome: Declared, deadline: Time) -> Event {
    Event::Delegated {
        name: crate::CallName { activation: 1, completion: 1, position: 0 },
        conversation,
        call: Token::new(call),
        ask: Ask::Finish { outcome },
        deadline,
    }
}

pub(crate) fn returned(call: u64, result: Returned) -> Request {
    Request::Return { spent: 0, call: Token::new(call), result }
}

pub(crate) fn verdict(name: &[u8], children: Box<[Item]>) -> Declared {
    Declared::Verdict(Verdict { name: bytes(name), text: bytes(b"Looks good."), items: children, fields: Box::new([]) })
}

pub(crate) fn comment() -> Item {
    let fields = Box::new([
        Field { name: bytes(b"path"), value: bytes(b"a.rs") },
        Field { name: bytes(b"body"), value: bytes(b"Nit.") },
    ]);
    Item { kind: bytes(b"nit"), fields }
}

pub(crate) fn change() -> Change {
    Change {
        fields: Box::new([
            Field { name: b"title".as_slice().into(), value: bytes(b"Fix the parser") },
            Field { name: b"body".as_slice().into(), value: bytes(b"It accepts tabs now.") },
        ]),
    }
}

/// The test charter, finishing with a change whose checks must pass, in two
/// writable repositories.
pub(crate) fn coding_workspace() -> Workspace {
    Workspace {
        directories: Box::new([
            Directory {
                host: Box::from(b"/host".as_slice()),
                name: bytes(b"temper"),
                root: Token::new(900),
                writable: true,
                git: true,
                conflicts: Box::new([]),
            },
            Directory {
                host: Box::from(b"/host".as_slice()),
                name: bytes(b"docs"),
                root: Token::new(901),
                writable: true,
                git: true,
                conflicts: Box::new([]),
            },
        ]),
    }
}

pub(crate) fn coding() -> Charter {
    Charter {
        reserve: crate::charter::Reserve { turns: 0, time: LIMITS.check_timeout },
        outcome: OutcomeSpec {
            change: Some(ChangeSpec {
                checks_must_pass: true,
                fields: Box::new([
                    FieldRule { name: b"title".as_slice().into(), max: 1024 },
                    FieldRule { name: b"body".as_slice().into(), max: 1024 },
                ]),
            }),
            verdicts: Box::new([]),
            report: None,
            failure: None,
        },
        ..charter()
    }
}

pub(crate) fn ran(code: u8, output: &[u8]) -> Ran {
    Ran { exit: Exit::Code { code }, output: bytes(output), cut: 0 }
}

// A cancel in every state.

pub(crate) fn cancelled() -> Answer {
    failed(Failure::Cancelled, Spend::ZERO)
}

// Sub-agents.

pub(crate) fn refused(refusal: AskRefusal) -> Returned {
    Returned::Refused { refusal }
}

pub(crate) fn families(inspect: bool, modify: bool, agents: bool) -> crate::charter::Families {
    crate::charter::Families { tools: Tools { inspect, modify, shell: false }, agents }
}

/// The test charter, granting sub-agents and listing one LLM for them.
pub(crate) fn agents() -> Charter {
    let grants = Grants { agents: true, ..charter().grants };
    let models = Box::new([Llm {
        effort: None,
        prices: crate::Prices { input: 1, cached: 1, cache_write: None, output: 1, unit: 1_000_000 },
        account: Some(0),
        endpoint: Endpoint(2),
        model: bytes(b"model-b"),
        window: 8192,
        output: 512,
        dialect: 1,
    }]);
    Charter { grants, models, ..charter() }
}

pub(crate) fn ask(
    conversation: Token,
    call: u64,
    wanted: crate::charter::Families,
    llm: Option<Box<[u8]>>,
    share: Option<crate::Share>,
) -> Event {
    let ask = Ask::SubAgent {
        brief: bytes(b"Find the parser."),
        families: wanted,
        llm,
        share: share.unwrap_or(crate::Share::UNASKED),
    };
    Event::Delegated {
        name: crate::CallName { activation: 1, completion: 1, position: 0 },
        conversation,
        call: Token::new(call),
        ask,
        deadline: EXPIRY,
    }
}

/// What `conversation` asks for a sub-agent that may inspect, as its call
/// `call` due by `deadline`.
pub(crate) fn ask_by(conversation: Token, call: u64, deadline: Time) -> Event {
    let ask = Ask::SubAgent {
        brief: bytes(b"Find it."),
        families: families(true, false, false),
        llm: None,
        share: crate::Share::UNASKED,
    };
    Event::Delegated {
        name: crate::CallName { activation: 1, completion: 1, position: 0 },
        conversation,
        call: Token::new(call),
        ask,
        deadline,
    }
}

// Facts.

/// The facts the domain holds, oldest first.
pub(crate) fn facts(harness: &mut Harness) -> Box<[Fact]> {
    let pending = harness.observed.len().checked_add(harness.domain.pending_facts()).expect("held facts fit u32");
    let mut facts = List::with_capacity(pending);
    for _ in 0..pending {
        let fact = harness.pop_fact().expect("entry count includes every held fact");
        facts.push(fact.kind).expect("room for every fact kept");
    }
    facts.into_boxed()
}

pub(crate) fn text_charter(failure: bool) -> Charter {
    let contract =
        crate::outcome::TextSpec { max: 8, fields: Box::new([FieldRule { name: bytes(b"source"), max: 4 }]) };
    Charter {
        outcome: OutcomeSpec {
            change: None,
            verdicts: Box::new([]),
            report: if failure { None } else { Some(contract.clone()) },
            failure: if failure { Some(contract) } else { None },
        },
        ..charter()
    }
}

pub(crate) fn text_result(failure: bool, text: &[u8]) -> Declared {
    let fields = Box::new([Field { name: bytes(b"source"), value: bytes(b"ref") }]);
    if failure {
        Declared::Failure(crate::outcome::DeclaredFailure { reason: bytes(text), fields })
    } else {
        Declared::Report(crate::outcome::Report { text: bytes(text), fields })
    }
}

pub(crate) fn receipts() -> crate::Delivered {
    crate::Delivered::new(Box::new([crate::Receipt::new(0, bytes(b"host receipt")).expect("bounded receipt")]))
        .expect("one changed directory")
}

pub(crate) fn delivered() -> Delivery {
    Delivery::Delivered(receipts())
}

pub(crate) fn mid_report() -> Charter {
    let mut charter = text_charter(false);
    charter.reserve.time = LIMITS.check_timeout;
    charter.grants.deliver = Some(ChangeSpec { checks_must_pass: true, fields: Box::new([]) });
    charter
}

pub(crate) fn mid_running(harness: &mut Harness) -> (Token, Token) {
    mid_running_policy(harness, mid_report())
}

pub(crate) fn mid_running_policy(harness: &mut Harness, policy: Charter) -> (Token, Token) {
    let mut mounted = workspace();
    mounted.directories[0].writable = true;
    let requests = harness.start_workspace(1, policy, Some(mounted));
    let [Request::Admitted { run, .. }, Request::Read { .. }] = &*requests else { panic!("admitted read") };
    let run = *run;
    let requests = harness.step(Event::Read { owner: run, read: Read::Missing });
    let [Request::Probe { .. }] = requests.as_ref() else {
        panic!("prepare writable check");
    };
    let requests = harness.step(Event::Probed { owner: run, executable: true });
    let [Request::Open { conversation, opening }] = &*requests else { panic!("prepared main") };
    assert!(opening.deliver);
    let conversation = *conversation;
    drop(harness.step(Event::Started { conversation, peer: PEER }));
    (run, conversation)
}

pub(crate) fn mid_ask(conversation: Token, completion: u32) -> Event {
    Event::Delegated {
        conversation,
        call: Token::new(50),
        name: crate::CallName { activation: 1, completion, position: 2 },
        ask: Ask::Deliver { change: Change { fields: Box::new([]) } },
        deadline: EXPIRY,
    }
}

pub(crate) fn submit_mid(harness: &mut Harness, conversation: Token) -> Token {
    let requests = harness.step(mid_ask(conversation, 3));
    let [Request::Check { owner, .. }, Request::Checking { .. }] = &*requests else { panic!("all writable checks") };
    let owner = *owner;
    let requests = harness.step(Event::Checked { owner, ran: ran(0, b"passed") });
    let [Request::Deliver { name, deadline, .. }] = &*requests else { panic!("bounded submission") };
    assert_eq!(*name, crate::CallName { activation: 1, completion: 3, position: 2 });
    assert_eq!(*deadline, harness.env.now.saturating_add(LIMITS.delivery_timeout).min(EXPIRY));
    owner
}

pub(crate) fn report() -> Declared {
    text_result(false, b"done")
}

pub(crate) fn host_ask(conversation: Token, call: u64, deadline: Time) -> Event {
    Event::Delegated {
        conversation,
        call: Token::new(call),
        name: crate::CallName { activation: 1, completion: 7, position: 3 },
        ask: Ask::Host {
            tool: bytes(b"comment"),
            effect: crate::HostEffect::Read,
            input: crate::HostInput::attested(bytes(br#" {"whole":{"opaque":[true,1]},"verbatim":"\u0041"} "#))
                .expect("protocol-attested fixture"),
        },
        deadline,
    }
}

pub(crate) fn host_submission(requests: &[Request]) -> crate::RelayName {
    let [Request::HostCall { relay, host_run, name, input, deadline, tool, effect }] = requests else {
        panic!("one opaque relay, got {requests:?}");
    };
    assert_eq!(*host_run, Token::new(71));
    assert_eq!(*name, crate::CallName { activation: 1, completion: 7, position: 3 });
    assert_eq!(tool.as_ref(), b"comment");
    assert_eq!(*effect, crate::HostEffect::Read);
    assert_eq!(input.bytes(), br#" {"whole":{"opaque":[true,1]},"verbatim":"\u0041"} "#);
    assert!(*deadline > Time::ZERO);
    *relay
}

/// Independent host workspace fixtures exercise admission before any IO.
pub(crate) fn workspace_directory(name: &[u8], root: u64, writable: bool, git: bool, conflicts: &[&[u8]]) -> Directory {
    let mut paths = List::with_capacity(u32::try_from(conflicts.len()).unwrap());
    for path in conflicts {
        paths.push(bytes(path)).expect("bounded fixture conflict count");
    }
    Directory {
        host: Box::from(b"/host".as_slice()),
        name: bytes(name),
        root: Token::new(root),
        writable,
        git,
        conflicts: paths.into_boxed(),
    }
}

pub(crate) fn mixed_workspace() -> Workspace {
    Workspace {
        directories: Box::new([
            workspace_directory(b"work", 41, true, false, &[]),
            workspace_directory(b"reference", 42, false, true, &[b"src/merge.rs"]),
        ]),
    }
}

#[expect(
    clippy::large_types_passed_by_value,
    reason = "programming-model.md, section 10.2: the harness receives a copied immutable profile"
)]
pub(crate) fn refuse_workspace(mounted: Option<Workspace>, limits: Limits, expected: Invalid) {
    let mut harness = Harness::new(limits);
    assert_eq!(
        answered(harness.start_workspace(53, charter(), mounted)),
        (53, Answer::Refused(Refusal::Invalid(expected)))
    );
    assert_eq!(harness.domain.runs(), 0, "invalid metadata is refused before retained state or effects");
}

pub(crate) fn priced(conversation: Token, own_spent: u64, subtree_spent: u64) -> Event {
    Event::Priced { conversation, own_spent, subtree_spent }
}

pub(crate) fn observed_facts(harness: &mut Harness) -> Box<[crate::facts::Fact]> {
    let pending = harness.observed.len().checked_add(harness.domain.pending_facts()).expect("held facts fit u32");
    let mut facts = List::with_capacity(pending);
    for _ in 0..pending {
        if let Some(fact) = harness.pop_fact() {
            facts.push(fact).expect("room for all observations");
        }
    }
    facts.into_boxed()
}

pub(crate) fn carried_start(charter: Charter, messages: Box<[crate::Message]>, workspace: Option<Workspace>) -> Event {
    Event::Start {
        messages,
        window: crate::Window { turns: u32::MAX, bytes: u64::MAX, largest_turn: 1 },
        reply_to: ReplyTo::new(Token::new(1)),
        host_run: Token::new(1),
        identity: None,
        activation: 1,
        charter,
        workspace,
        transcript: None,
        resumed: false,
    }
}

pub(crate) fn carried(name: u64, text: &[u8]) -> crate::Message {
    crate::Message { finish_requested: false, name: Token::new(name), label: bytes(b"person"), text: bytes(text) }
}

pub(crate) fn message_driven_charter() -> Charter {
    Charter { brief: crate::Brief { sections: Box::default() }, ..charter() }
}

pub(crate) fn identity_open(harness: &mut Harness, identity: Option<crate::Identity>) -> (Token, Token, Opening) {
    let requests = harness.step(Event::Start {
        identity,
        messages: Box::default(),
        window: crate::Window { turns: u32::MAX, bytes: u64::MAX, largest_turn: 1 },
        reply_to: ReplyTo::new(Token::new(1)),
        host_run: Token::new(1),
        activation: 1,
        charter: agents(),
        workspace: None,
        transcript: None,
        resumed: false,
    });
    let Ok(pair) = Box::<[Request; 2]>::try_from(requests) else { panic!("admission and main opening") };
    let [Request::Admitted { run, .. }, Request::Open { conversation, opening }] = *pair else {
        panic!("admission and main opening");
    };
    drop(harness.step(Event::Started { conversation, peer: PEER }));
    (run, conversation, opening)
}

pub(crate) fn ended(conversation: Token, end: End, spend: Spend) -> Event {
    Event::Ended { conversation, end, spend, detail: Box::default() }
}

pub(crate) fn pass_checks(owner: Token) -> Event {
    Event::Checked { owner, ran: ran(0, b"") }
}
