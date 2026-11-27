//! Real root command effects cross both framed halves into the real watchdog.
use super::*;

struct LongWorld {
    domain: agent::Domain,
    env: Env<agent::Limits>,
    out: Queue<AgentRequest>,
    wire: World,
    host: smith_host_world::World,
    run: Option<Token>,
    completion: Option<Token>,
    prompt: Option<agent::llm::Prompt>,
    command: Option<Token>,
    cancelled: bool,
    notices: Vec<Option<agent::Operation>>,
}

impl LongWorld {
    fn new() -> Self {
        let limits = smith_agent_world::LIMITS;
        let mut bounds = smith_host_world::limits();
        bounds.turn_bytes = agent::max_turn_bytes(&limits).expect("declared turn envelope");
        bounds.unacknowledged_bytes = bounds.turn_bytes;
        Self::with(&limits, configuration(), bounds)
    }

    fn with(limits: &agent::Limits, configuration: agent::Config, bounds: host::Limits) -> Self {
        let mut host = smith_host_world::World::new(97, bounds);
        host.live();
        let wire = World::started();
        Self {
            domain: agent::Domain::new(limits, configuration, 97),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits },
            out: Queue::uncapped(),
            wire,
            host,
            run: None,
            completion: None,
            prompt: None,
            command: None,
            cancelled: false,
            notices: Vec::new(),
        }
    }

    fn start(&mut self, charter: agent::run::Charter) {
        self.step(AgentEvent::Start(smith_domain::Start {
            host_run: Token::new(7),
            window: agent::Window { turns: 100, bytes: u64::MAX },
            workspace: Some(agent::run::Workspace {
                directories: Box::from([agent::run::Directory {
                    results: None,
                    host: Box::from(b"/host".as_slice()),
                    name: b"work".as_slice().into(),
                    root: Token::new(900),
                    writable: true,
                    git: true,
                    conflicts: Box::default(),
                }]),
            }),
            grants: Box::from([agent::Grant {
                name: agent::GrantName { account: 0, generation: 1 },
                renewable: true,
                valid: Duration::from_secs(7200),
            }]),
            ..smith_domain_fixtures::start(Token::new(1), charter)
        }));
    }

    fn shell(&mut self, command: &[u8], input: &[u8]) {
        let owner = self.completion.take().expect("observed provider request");
        self.step(AgentEvent::Below(smith_domain::Below::Completed {
            owner,
            completion: agent::llm::Completion {
                served: smith_domain::llm::ServedSpeed::Unreported,
                reasoning_dropped: Box::default(),
                stop: agent::llm::Stop::ToolUse,
                usage: agent::llm::Usage::NONE,
                content: Box::from([agent::llm::Said::ToolCall {
                    id: b"command".as_slice().into(),
                    name: b"shell".as_slice().into(),
                    input: input.into(),
                    replay: None,
                    call: agent::llm::Decoded::Owned {
                        call: agent::tools::Call::Shell {
                            after: agent::tools::Yield::Never,
                            command: command.into(),
                            timeout: agent::tools::Timeout::Asked(Duration::from_secs(30)),
                            output: agent::tools::Budget::Default,
                        },
                    },
                }]),
            },
        }));
    }

    fn at(&mut self, seconds: u64) {
        self.env.now = Time::ZERO.saturating_add(Duration::from_secs(seconds));
        self.host.at(seconds);
        while self.domain.is_due(self.env.now) {
            while self.domain.pop_fact().is_some() {}
            agent::fire(&mut self.domain, &self.env, &mut self.out);
            self.outputs();
        }
        self.ready();
    }

    fn step(&mut self, event: AgentEvent) {
        while self.domain.pop_fact().is_some() {}
        agent::step(&mut self.domain, &self.env, event, &mut self.out);
        self.outputs();
    }

    fn ready(&mut self) {
        for _ in 0..100 {
            self.domain.reclaim();
            if !self.domain.is_ready() {
                return;
            }
            while self.domain.pop_fact().is_some() {}
            agent::resume(&mut self.domain, &self.env, &mut self.out);
            self.outputs();
        }
        panic!("bounded ready handoffs settle");
    }

    fn outputs(&mut self) {
        while let Some(request) = self.out.pop() {
            match request {
                AgentRequest::Admitted { run, .. } => {
                    self.run = Some(run);
                    self.wire.agent_admits();
                    self.wire.settle();
                }
                AgentRequest::Lower(smith_domain::Lower::Read { owner, .. }) => {
                    self.step(AgentEvent::Below(smith_domain::Below::Read { owner, read: agent::run::Read::Missing }));
                }
                AgentRequest::Lower(smith_domain::Lower::Probe { owner, .. }) => {
                    self.step(AgentEvent::Below(smith_domain::Below::Probed { owner, executable: false }));
                }
                AgentRequest::Long { operation, until, .. } => {
                    let span = until.saturating_since(self.env.now);
                    self.notices.push(Some(operation));
                    self.wire.agent_starts_long(operation, span);
                    self.wire.settle();
                    assert!(self.wire.observations().contains(&Observation::HostLong { operation, span }));
                    self.host.up(Up::Long { operation, span });
                }
                AgentRequest::LongDone { .. } => {
                    self.notices.push(None);
                    self.wire.agent_ends_long();
                    self.wire.settle();
                    self.host.up(Up::LongDone);
                }
                AgentRequest::Lower(smith_domain::Lower::Complete { owner, prompt, .. }) => {
                    self.completion = Some(owner);
                    self.prompt = Some(prompt);
                }
                AgentRequest::Lower(smith_domain::Lower::Io { owner, op: agent::tools::Op::Spawn { .. }, .. }) => {
                    self.command = Some(owner);
                }
                AgentRequest::Lower(smith_domain::Lower::Job { .. }) => unreachable!("the tools issue no job act yet"),
                AgentRequest::Lower(smith_domain::Lower::CancelIo { owner }) => {
                    assert_eq!(Some(owner), self.command);
                    self.cancelled = true;
                }
                AgentRequest::Turn { number, turn, read, spent, opens_window, .. } => {
                    self.wire.agent_tells_turn(number, read, &turn);
                    self.wire.settle();
                    self.host.up(Up::Turn {
                        turn: host::Turn {
                            number,
                            read,
                            spent: spent.units,
                            opens_window,
                            body: host::TurnValue::new(turn, self.host.stage.env.limits.turn_bytes)
                                .expect("observed turn fits declared credit"),
                        },
                    });
                    self.host.event(host::Input::Parent(host::parent::Event::Acknowledge {
                        agent: self.host.agent(),
                        turn: number,
                    }));
                    self.host.sent();
                    self.wire.host_acknowledges(number);
                    self.wire.settle();
                    self.step(AgentEvent::Acknowledge { run: self.run.expect("admitted root"), turn: number });
                }
                AgentRequest::Answer { answer, read, .. } => {
                    assert!(self.run.is_some(), "scripted start refused: {answer:?}");
                    self.wire.agent_answers_with_fence(answer, read);
                    self.wire.settle();
                    let answer = self.wire.take_host_answer().expect("observed answer crosses the framed halves");
                    self.host.up(Up::Answer { answer });
                }
                other => panic!("unexpected command story request: {other:?}"),
            }
        }
    }
}

#[test]
fn an_observed_command_crosses_the_channel_and_outlasts_no_progress_until_its_terminal() {
    let mut world = LongWorld::new();
    let mut charter = composed_charter().into_value();
    charter.grants.tools.shell = true;
    world.start(charter);
    world.shell(b"sleep 12", br#"{"command":"sleep 12"}"#);
    let command = world.command.expect("observed command IO was issued");
    assert_eq!(world.notices, [Some(agent::Operation::Completion), None, Some(agent::Operation::Command)]);
    world.env.now = Time::ZERO.saturating_add(Duration::from_secs(11));
    world.host.at(11);
    assert_eq!(world.host.seen.fault, None, "observed command stretches the ten-second no-progress watchdog");
    world.step(AgentEvent::Cancel { run: world.run.expect("observed admitted run") });
    world.ready();
    assert!(world.cancelled);
    assert_eq!(world.notices.len(), 3, "a cancel request is not the command terminal");
    world.step(AgentEvent::Below(smith_domain::Below::Done { owner: command, done: agent::tools::Done::Cancelled }));
    world.ready();
    assert_eq!(world.notices, [Some(agent::Operation::Completion), None, Some(agent::Operation::Command), None]);
    assert!(matches!(
        world.host.seen.answer.as_ref().map(|answer| &answer.result),
        Some(host::RunResult::Failed { failure: host::RunFailure::Cancelled })
    ));
    world.host.cleanup(Some(smith_host_domain::Exit::Code(0)));
    world.host.settled();
    world.domain.reclaim();
    assert_eq!((world.domain.peers(), world.domain.flights(), world.domain.tickets()), (0, 0, 0));
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one story follows the command, streaming provider and framed answer to settlement"
)]
fn a_quick_shell_then_a_slow_stream_answers_under_derived_host_limits() {
    use skein_fake_llm_domain::{self as provider, api};
    let mut configured = smith_agent_process_world::configuration();
    let endpoint = configured.domain.limits.first_mut().expect("declared endpoint");
    endpoint.head = Duration::from_secs(60);
    endpoint.idle = Duration::from_secs(120);
    let bounds = smith_agent_profile::host_limits(
        &smith_agent_process_world::profile(),
        &configured.limits,
        &configured.domain.limits,
    )
    .expect("derived host bounds");
    assert!(
        bounds.no_progress >= configured.limits.domain.session.tools.tool_deadline,
        "the composed host uses the run's derived watchdog"
    );
    let old_bound = bounds.silence.checked_add(configured.limits.io.close_timeout).expect("old watchdog");
    let mut world = LongWorld::with(&configured.limits.domain, configured.domain, bounds);
    let mut charter = composed_charter().into_value();
    charter.budget.turns = 3;
    charter.budget.spend = 100;
    charter.budget.time = Duration::from_secs(600);
    charter.llm.window = 1_048_576;
    charter.llm.output = 512;
    charter.grants.tools.shell = true;
    world.start(charter);
    world.ready();
    world.shell(b"true", br#"{"command":"true"}"#);
    let command = world.command.take().expect("scripted machine receives shell");
    world.step(AgentEvent::Below(smith_domain::Below::Done {
        owner: command,
        done: agent::tools::Done::Exited {
            exit: agent::tools::Exit::Code { code: 0 },
            head: Box::default(),
            tail: Box::default(),
            dropped: 0,
        },
    }));
    world.ready();
    assert_eq!(
        world.notices,
        [
            Some(agent::Operation::Completion),
            None,
            Some(agent::Operation::Command),
            None,
            Some(agent::Operation::Completion),
        ]
    );
    let owner = world.completion.take().expect("completion after the quick shell");
    let prompt = world.prompt.take().expect("real agent prompt");
    let grants = prompt.tools;
    let served = prompt.served.clone();
    let asker = prompt.asker;
    let mut limits = smith_agent_world::Settings::calm(98).provider;
    limits.strict = true;
    limits.latency_min = Duration::from_secs(59);
    limits.latency_max = limits.latency_min;
    let scripts = api::Scripts {
        cued: Box::default(),
        asked: Box::default(),
        otherwise: Box::new([
            // The preceding scripted shell completion occupies turn zero.
            api::Turn { lines: Box::default(), finish: api::Finish::Stop, tokens: 0 },
            api::Turn {
                lines: Box::new([
                    api::Line::Pieces {
                        pieces: Box::new([
                            api::Piece { text: b"still ".as_slice().into(), hold: None },
                            api::Piece { text: b"working ".as_slice().into(), hold: None },
                            api::Piece { text: b"done".as_slice().into(), hold: None },
                        ]),
                    },
                    api::Line::Call {
                        name: b"finish".as_slice().into(),
                        arguments: br#"{"report":"answer"}"#.as_slice().into(),
                    },
                ]),
                finish: api::Finish::ToolCalls,
                tokens: 4,
            },
        ]),
    };
    let mut llm = provider::Domain::played(
        &limits,
        98,
        scripts,
        api::Menu { arguments: Box::default(), invalid: Box::default(), slots: Box::new([]) },
    )
    .expect("script fits");
    let mut env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = Queue::uncapped();
    let channel_turns = world.host.seen.turn_metadata.len();
    provider::step(
        &mut llm,
        &env,
        provider::Event::Call {
            call: owner,
            reply_to: ReplyTo::new(owner),
            query: smith_agent_world::translate::query(prompt),
        },
        &mut out,
    );
    assert!(out.is_empty(), "no first byte yet");
    // The first byte is one second inside head; subsequent events are one
    // second inside idle. None is a record on the agent-host channel.
    env.limits.latency_min = Duration::from_secs(119);
    env.limits.latency_max = env.limits.latency_min;
    for seconds in [59, 178, 297] {
        env.now = Time::ZERO.saturating_add(Duration::from_secs(seconds));
        assert_eq!(llm.next_deadline(), Some(env.now));
        world.at(seconds);
        assert!(Duration::from_secs(seconds) > old_bound);
        assert_eq!(world.host.seen.fault, None, "streaming silence does not stop the run");
        assert_eq!(world.host.seen.turn_metadata.len(), channel_turns, "no channel turn while streaming");
        assert_eq!(world.notices.len(), 5, "no new long-operation report while streaming");
        provider::fire(&mut llm, &env, &mut out);
        match out.pop().expect("scripted provider event") {
            provider::Request::Piece { call, .. } => assert_eq!(call, owner),
            provider::Request::Reply { to, result } => {
                assert_eq!(to.into_token(), owner);
                let completion = smith_agent_world::translate::completion(
                    result.expect("scripted completion"),
                    grants,
                    &served,
                    asker,
                );
                world.step(AgentEvent::Below(smith_domain::Below::Completed { owner, completion }));
                world.ready();
            }
            provider::Request::Hold { .. } => panic!("this script has no holds"),
        }
        assert!(out.is_empty());
    }
    assert_eq!(world.host.seen.fault, None);
    assert!(matches!(
        world.host.seen.answer.as_ref().map(|answer| &answer.result),
        Some(host::RunResult::Accepted { .. })
    ));
    assert!(!world.cancelled);
    world.host.clean_end();
    world.domain.reclaim();
    assert_eq!((world.domain.peers(), world.domain.flights(), world.domain.tickets()), (0, 0, 0));
    llm.reclaim();
    assert_eq!(llm.calls(), 0);
}
