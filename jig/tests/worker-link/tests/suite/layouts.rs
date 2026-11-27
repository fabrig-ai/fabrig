//! Both codecs see their own layout; the pipe sees the derived worker's layout.
use crate::joined_world::{Joined, layouts::Profile, received};
use jig_domain_host as host;
use jig_domain_link as link;
use jig_protocol_engine_workers as engine;
use jig_protocol_worker_link::Request;
use jig_protocol_worker_wire::{self as wire, Kind, Part};
use jig_worker_link_world::fixtures::{self, ATTEMPT_TWO as ATTEMPT, RUN_SEVEN as RUN};
use skein_lib::{Duration, List, Rng, Secret, Token};

pub(super) fn limits() -> jig_protocol_worker_link::Limits {
    let mut limits = fixtures::joined_limits();
    limits.sizes.charter = 512;
    limits.sizes.transcript = 512;
    limits.sizes.turn = 256;
    limits.sizes.result = 256;
    limits
}

#[derive(Clone, Copy)]
struct Sample(u32);
impl Sample {
    fn app(self, note: u8) -> Box<[u8]> {
        let mut bytes = vec![0]; // FromEngine/FromWorker fixtures use Form::specimen.
        bytes.extend(self.0.to_be_bytes());
        bytes.push(note);
        bytes.into_boxed_slice()
    }
    fn text(self) -> Box<[u8]> {
        format!("{:08x}", self.0).into_bytes().into_boxed_slice()
    }
    fn duration(self) -> Duration {
        Duration::from_millis(u64::from(self.0))
    }
    fn token(self) -> Token {
        Token::new(u64::from(self.0) + 1)
    }
    fn flag(self) -> bool {
        self.0 & 1 != 0
    }
    fn charter(self) -> Box<[u8]> {
        use smith_charter::v5::{self as c, Record};
        let charter = c::Charter {
            instructions: self.text(),
            brief: List::uncapped(),
            tools: c::Tools {
                families: c::Families { inspect: false, modify: false, shell: false, agents: false },
                wait: self.flag(),
                deliver: None,
                save: false,
                host: List::uncapped(),
                contract: 1,
            },
            contract: c::Contract { report: None, verdicts: List::uncapped(), change: None, failure: None },
            conventions: None,
            budget: c::Budget {
                turns: 1,
                spend: u64::from(self.0),
                time: self.duration(),
                reserve: c::Reserve { turns: 0, time: Duration::ZERO },
            },
            main: c::Llm {
                endpoint: b"endpoint".as_slice().into(),
                account: None,
                model: self.text(),
                effort: None,
                speed: None,
                window: 32,
                output: 8,
                prices: c::Prices { input: 1, cached: 0, cache_write: None, output: 1, unit: 1 },
            },
            models: List::uncapped(),
            waiting: Duration::ZERO,
            resume: true,
            mode: c::Mode::Finite,
            check_timeout: None,
        };
        charter.encode(&c::CEILINGS).expect("fixture charter")
    }
    fn turn(self) -> Box<[u8]> {
        use smith_transcript::v8::{self as t, Record};
        t::Turn {
            kind: t::TurnKind::Ordinary,
            opening: None,
            endpoint: b"endpoint".as_slice().into(),
            model: self.text(),
            effort: None,
            dialect: 1,
            place: 1,
            usage: t::Usage {
                input: Some(u64::from(self.0)),
                output: None,
                cache_read: None,
                cache_write: None,
                reasoning: None,
            },
            spent: u64::from(self.0),
            jobs: 0,
            messages: List::uncapped(),
            system: None,
        }
        .encode(&t::CEILINGS)
        .expect("fixture turn")
    }
    fn result(self) -> Box<[u8]> {
        use smith_charter::v5::{self as c, Record};
        c::RunResult {
            form: c::Form::Report,
            label: None,
            text: self.text(),
            fields: List::uncapped(),
            items: List::uncapped(),
        }
        .encode(&c::CEILINGS)
        .expect("fixture result")
    }
    fn grant_value(self) -> engine::GrantValue {
        engine::GrantValue::Application { secret: Secret::new(self.app(0)) }
    }
    fn grant(self) -> Box<[Option<engine::GrantValue>]> {
        Box::new([Some(self.grant_value())])
    }
    fn workspace(self, note: u8) -> Box<[Part]> {
        Box::new([Part { token: Token::new(10), bytes: self.app(note) }])
    }
    fn assignment(self) -> host::LinkDown {
        let mut assignment = fixtures::assignment();
        assignment.assignment.charter = self.charter();
        assignment.assignment.save = self.flag();
        assignment.turns = Box::new([self.turn()]);
        assignment.parts = Box::new([link::KeptPart {
            attempt: ATTEMPT.raw(),
            turn: 2,
            thread: 1,
            call: 1,
            kind: link::PartKind::Answer,
            body: self.turn(),
        }]);
        host::LinkDown::Assign { assignment }
    }
    fn job(self, note: u8) -> host::LinkDown {
        host::LinkDown::AssignJob {
            job: host::JobAssignment {
                run: RUN,
                attempt: ATTEMPT,
                kind: link::JobKind(self.0),
                spec: self.app(note),
                deadline: self.duration(),
                workspace: Some(host::Workspace { workstream: 3, items: Token::new(10) }),
                grants: Box::new([fixtures::grant(link::Form::Application)]),
            },
        }
    }
    fn message(self, refused: bool) -> host::LinkDown {
        let words = if refused { b"13 characters".as_slice().into() } else { self.text() };
        host::LinkDown::Message {
            run: RUN,
            attempt: ATTEMPT,
            name: self.token(),
            sender: self.text(),
            words,
            finish_requested: self.flag(),
        }
    }
    fn turn_notice(self) -> host::LinkUp {
        let host::LinkUp::Turn { run, attempt, mut turn } = fixtures::turn() else { panic!("turn fixture") };
        turn.body = self.turn();
        turn.spent = u64::from(self.0);
        host::LinkUp::Turn { run, attempt, turn }
    }
    fn part_notice(self) -> host::LinkUp {
        host::LinkUp::Part {
            run: RUN,
            attempt: ATTEMPT,
            part: link::Part {
                number: 1,
                turn: 2,
                thread: 1,
                call: 1,
                kind: link::PartKind::Answer,
                body: self.turn(),
            },
        }
    }
}

fn sample(rng: &mut Rng) -> Sample {
    Sample(u32::try_from(rng.below(u64::from(u32::MAX))).expect("u32 sample"))
}

fn heard(world: &Joined) -> Vec<(&host::LinkUp, &[Part])> {
    world
        .engine_root
        .iter()
        .filter_map(|request| match request {
            engine::Request::Up { up, parts, .. } => Some((up, parts.as_ref())),
            _ => None,
        })
        .collect()
}
fn hello(world: &Joined) -> Option<&link::Hello> {
    world.engine_root.iter().find_map(|r| match r {
        engine::Request::Hello { hello, .. } => Some(hello),
        _ => None,
    })
}
fn handed(world: &Joined) -> Vec<(&host::LinkDown, &[Part])> {
    world
        .worker_root
        .iter()
        .filter_map(|request| match request {
            Request::Down { down, parts } => Some((down, parts.as_ref())),
            _ => None,
        })
        .collect()
}

#[expect(clippy::too_many_lines, reason = "the fixture lists one case per worker kind together")]
fn up_samples(world: &mut Joined, rng: &mut Rng) {
    let source = link::SourceName::new(b"source").expect("source");
    let window = link::UsageWindow {
        label: sample(rng).text(),
        window: link::FactWindow { length: Some(sample(rng).duration()), used: Some(25), resets: None, state: 1 },
    };
    let work = host::Work { left: Some(Token::new(10)), saved: Some(Token::new(11)) };
    let parts = |rng: &mut Rng| -> Box<[Part]> {
        Box::new([
            Part { token: Token::new(10), bytes: sample(rng).app(0) },
            Part { token: Token::new(11), bytes: sample(rng).app(0) },
        ])
    };
    let up = [
        (sample(rng).turn_notice(), Box::<[Part]>::default()),
        (fixtures::call(&sample(rng).0.to_be_bytes()), Box::default()),
        (
            host::LinkUp::Answer {
                run: RUN,
                attempt: ATTEMPT,
                answer: host::RunAnswer {
                    read: Some(sample(rng).token()),
                    refused: Box::new([]),
                    turns: 1,
                    spent: u64::from(sample(rng).0),
                    ending: host::Ending::Ended { outcome: sample(rng).result(), work: work.clone() },
                },
            },
            parts(rng),
        ),
        (
            host::LinkUp::Activity {
                run: RUN,
                attempt: ATTEMPT,
                activity: host::AgentActivity::Long {
                    operation: link::Operation::Command,
                    span: sample(rng).duration(),
                },
            },
            Box::default(),
        ),
        (
            host::LinkUp::Unauthorized {
                run: RUN,
                attempt: ATTEMPT,
                account: 1,
                generation: sample(rng).token().raw(),
            },
            Box::default(),
        ),
        (
            host::LinkUp::Limited { run: RUN, attempt: ATTEMPT, account: 1, reset: Some(sample(rng).duration()) },
            Box::default(),
        ),
        (
            host::LinkUp::AccountRefused {
                run: RUN,
                attempt: ATTEMPT,
                account: 1,
                code: sample(rng).text(),
                reason: sample(rng).text(),
            },
            Box::default(),
        ),
        (
            host::LinkUp::Logins {
                logins: Box::new([link::Login {
                    name: source,
                    state: link::LoginState::Usable,
                    email: Some(sample(rng).text()),
                }]),
            },
            Box::default(),
        ),
        (host::LinkUp::SignedOut { name: source, result: link::SignOutResult::SignedOut }, Box::default()),
        (
            host::LinkUp::ListRead {
                read: sample(rng).token().raw(),
                outcome: link::ListOutcome::Listed(Box::new([])),
            },
            Box::default(),
        ),
        (host::LinkUp::Draining, Box::default()),
        (
            host::LinkUp::JobAnswer {
                run: RUN,
                attempt: ATTEMPT,
                answer: host::JobEnding::Finished { result: sample(rng).app(0), work },
            },
            parts(rng),
        ),
        (
            host::LinkUp::Ask {
                run: RUN,
                attempt: ATTEMPT,
                number: sample(rng).token().raw(),
                bytes: sample(rng).app(0),
            },
            Box::default(),
        ),
        (host::LinkUp::Usage { run: RUN, attempt: ATTEMPT, account: 1, window: window.clone() }, Box::default()),
        (
            host::LinkUp::UsageRead {
                read: sample(rng).token().raw(),
                outcome: link::UsageOutcome::Read(Box::new([window])),
            },
            Box::default(),
        ),
        (
            host::LinkUp::Paused {
                state: if sample(rng).flag() { link::PauseState::Paused } else { link::PauseState::Taking },
            },
            Box::default(),
        ),
        (sample(rng).part_notice(), Box::default()),
    ];
    for (up, parts) in up {
        world.worker_up(up, parts);
        world.run();
    }
}

fn down_samples(world: &mut Joined, rng: &mut Rng, call: &[u8], bad_message: bool) {
    type Down = (host::LinkDown, Box<[Part]>, Box<[Option<engine::GrantValue>]>);
    let source = link::SourceName::new(b"source").expect("source");

    let mut grant = fixtures::grant(link::Form::Application);
    grant.generation = sample(rng).token().raw();
    let channel = world.channel();
    world.engine_event(engine::Event::PartBusy { channel, run: RUN, attempt: ATTEMPT, part: 1 });
    world.run();
    world.engine_event(engine::Event::TurnBusy { channel, run: RUN, attempt: ATTEMPT, turn: 1 });
    world.run();
    let samples: [Down; 16] = [
        (
            sample(rng).assignment(),
            sample(rng).workspace(0),
            Box::new([Some(crate::joined_world::credential(engine::Scheme::Bearer)), Some(sample(rng).grant_value())]),
        ),
        (sample(rng).message(bad_message), Box::default(), Box::default()),
        (fixtures::relay_answer(call), Box::default(), Box::default()),
        (host::LinkDown::TurnKept { run: RUN, attempt: ATTEMPT, turn: 1 }, Box::default(), Box::default()),
        (host::LinkDown::PartKept { run: RUN, attempt: ATTEMPT, part: 1 }, Box::default(), Box::default()),
        (host::LinkDown::Acknowledge { run: RUN, attempt: ATTEMPT }, Box::default(), Box::default()),
        (host::LinkDown::Cancel { run: RUN, attempt: ATTEMPT }, Box::default(), Box::default()),
        (host::LinkDown::Grant { run: RUN, attempt: ATTEMPT, grant }, Box::default(), sample(rng).grant()),
        (host::LinkDown::Sources { sources: Box::new([source]) }, Box::default(), Box::default()),
        (host::LinkDown::Check { name: source }, Box::default(), Box::default()),
        (host::LinkDown::SignOut { name: source }, Box::default(), Box::default()),
        (sample(rng).job(0), sample(rng).workspace(0), sample(rng).grant()),
        (
            host::LinkDown::ReadList {
                read: sample(rng).token().raw(),
                endpoint: 1,
                grant: fixtures::grant(link::Form::Credential(link::Scheme::Bearer)),
            },
            Box::default(),
            Box::new([Some(crate::joined_world::credential(engine::Scheme::Bearer))]),
        ),
        (
            host::LinkDown::Asked {
                run: RUN,
                attempt: ATTEMPT,
                number: sample(rng).token().raw(),
                answer: host::AskAnswer::Bytes(sample(rng).app(0)),
            },
            Box::default(),
            Box::default(),
        ),
        (host::LinkDown::ReadUsage { read: sample(rng).token().raw(), source }, Box::default(), Box::default()),
        (
            host::LinkDown::Pause {
                mode: if sample(rng).flag() { link::PauseMode::Pause } else { link::PauseMode::Drain },
            },
            Box::default(),
            Box::default(),
        ),
    ];
    for (record, parts, grants) in samples {
        world.engine_down(record, parts, grants);
        world.run();
    }
    world.engine_event(engine::Event::Restarting { wait: sample(rng).duration() });
    world.run();
}

fn run_samples(world: &mut Joined, seed: u64, bad_message: bool) {
    let mut rng = Rng::new(seed);
    world.set_hello_stop_bound(sample(&mut rng).duration());
    let mut figures = fixtures::figures();
    figures.limits = List::from_boxed(Box::new([wire::Limit {
        name: b"sample".as_slice().into(),
        value: u64::from(sample(&mut rng).0),
    }]));
    world.figures(figures);
    world.acknowledge(false);
    world.run();
    up_samples(world, &mut rng);
    let call = heard(world)
        .into_iter()
        .find_map(|(up, _)| match up {
            host::LinkUp::Call { call, .. } => Some(call.clone()),
            _ => None,
        })
        .expect("a call crossed");
    down_samples(world, &mut rng, &call, bad_message);
}

/// Compare owner-visible data against the same records crossing unchanged.
/// The wire observations also cover frames consumed by the components.
pub(super) fn exchange(seed: u64, bad_message: bool) {
    let limits = limits();
    let mut current = Joined::new(&limits, seed);
    let mut derived = Joined::with_layouts(&limits, seed, Profile::Derived);
    run_samples(&mut current, seed, bad_message);
    run_samples(&mut derived, seed, bad_message);
    assert_eq!(heard(&derived), heard(&current), "up records and parts");
    let mut expected = handed(&current);
    if bad_message {
        expected.retain(|(down, _)| !matches!(down, host::LinkDown::Message { .. }));
        message_refusal(&derived);
    }
    assert_eq!(handed(&derived), expected, "down records and parts");
    let values = |world: &Joined| {
        world
            .values
            .iter()
            .map(|request| match request {
                Request::Value { account, generation, expires, value } => {
                    (*account, *generation, *expires, received(value))
                }
                _ => panic!("grant value"),
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(values(&derived), values(&current));
    let kinds = |kinds: Vec<u16>| kinds.into_iter().collect::<std::collections::BTreeSet<_>>();
    let current_up = kinds(current.worker_records());
    let current_down = kinds(current.engine_records());
    let derived_up = kinds(derived.worker_records());
    let derived_down = kinds(derived.engine_records());
    for rule in wire::KINDS {
        if rule.direction == skein_channel::Direction::FromInitiator {
            assert!(current_up.contains(&rule.kind), "missing source {:?}", rule.body);
            assert!(derived_up.contains(&(rule.kind + 1)), "missing derived {:?}", rule.body);
        } else {
            assert!(current_down.contains(&rule.kind), "missing source {:?}", rule.body);
            assert_eq!(
                derived_down.contains(&(rule.kind + 1)),
                !bad_message || rule.body != Kind::Message,
                "derived {:?}",
                rule.body
            );
        }
    }
    let figures = |world: &Joined| {
        world
            .worker_root
            .iter()
            .find_map(|r| match r {
                Request::Figures { figures } => Some(figures.clone()),
                _ => None,
            })
            .expect("figures cross before hello")
    };
    let restart = |world: &Joined| {
        world
            .worker_root
            .iter()
            .find_map(|r| match r {
                Request::Restarting { wait } => Some(*wait),
                _ => None,
            })
            .expect("restart word crosses")
    };
    assert_eq!(figures(&derived), figures(&current));
    assert_eq!(restart(&derived), restart(&current));
    assert_eq!(hello(&derived), hello(&current));
    assert_eq!(derived.engine_channels(), current.engine_channels());
    for world in [&mut current, &mut derived] {
        assert!(
            !world
                .engine_root
                .iter()
                .any(|r| matches!(r, engine::Request::Unread { .. } | engine::Request::Lost { .. }))
        );
        world.finish();
    }
}

#[test]
fn all_worker_kinds_cross_at_derived_layouts() {
    exchange(fixtures::SEED_ONE, false);
}

fn start(profile: Profile) -> Joined {
    let mut world = Joined::with_layouts(&limits(), fixtures::SEED_ONE, profile);
    world.acknowledge(false);
    world.run();
    assert_eq!(world.engine_channels(), [Token::new(1)]);
    world
}

#[test]
fn untranslated_starts_send_nothing_and_answer_with_the_plan_words() {
    let mut world = start(Profile::Derived);
    let sample = Sample(7);
    let words = b"the worker's layout cannot keep test-worker-wire Test.note: field";
    let mut grants = sample.grant().into_vec();
    grants.insert(0, Some(crate::joined_world::credential(engine::Scheme::Bearer)));
    world.engine_down(sample.assignment(), sample.workspace(1), grants.into_boxed_slice());
    world.run();
    world.engine_down(sample.job(1), sample.workspace(0), sample.grant());
    world.run();
    assert_eq!(world.engine_records(), []);
    assert_eq!(handed(&world), []);
    assert!(world.values.is_empty());
    let up = heard(&world);
    let [
        (host::LinkUp::Answer { run, attempt, answer }, []),
        (host::LinkUp::JobAnswer { run: job, attempt: job_attempt, answer: job_answer }, []),
    ] = up.as_slice()
    else {
        panic!("local answers for both starts: {up:?}")
    };
    assert_eq!((*run, *attempt, *job, *job_attempt), (RUN, ATTEMPT, RUN, ATTEMPT));
    assert_eq!((answer.turns, answer.spent), (0, 0));
    assert_eq!(answer.ending, host::Ending::Refused(host::Refusal::Untranslated(words.as_slice().into())));
    assert_eq!(*job_answer, host::JobEnding::Refused(host::JobRefusal::Untranslated(words.as_slice().into())));
    for request in &world.engine_root {
        if let engine::Request::Unsent { unsent, .. } = request {
            assert!(!unsent.mismatches(), "only the task stays off this channel");
        }
    }
    world.finish();
}

fn message_refusal(world: &Joined) {
    let complaints: Vec<_> = world
        .engine_root
        .iter()
        .filter_map(|r| match r {
            engine::Request::Unsent { worker, channel, unsent } => Some((worker, channel, unsent)),
            _ => None,
        })
        .collect();
    let [(worker, channel, unsent)] = complaints.as_slice() else { panic!("one refusal: {complaints:?}") };
    assert_eq!((**worker, **channel, unsent.kind), (0, world.channel(), Kind::Message));
    assert!(unsent.mismatches());
    let engine::Bound::Translation(problem) = &unsent.bound else { panic!("Keep refusal") };
    assert_eq!(&*problem.kept(), b"the worker's layout cannot keep jig-worker-wire Inbound.words: narrowed");
}

#[test]
fn a_running_message_refusal_discards_the_frame_and_marks_the_channel() {
    exchange(fixtures::SEED_ONE, true);
}

#[test]
fn a_worker_only_transcript_variant_is_an_undecodable_turn() {
    let mut world = start(Profile::Derived);
    world.next_turn_worker_only();
    world.worker_up(Sample(7).turn_notice(), Box::new([]));
    world.run();
    assert_eq!(heard(&world), []);
    let problem = world
        .engine_root
        .iter()
        .find_map(|r| match r {
            engine::Request::Unread { worker: 0, kind: Kind::Turn, problem } => Some(problem),
            _ => None,
        })
        .expect("cannot read turn diagnostic");
    assert_eq!(&*problem.read(), b"smith-transcript TurnKind.worker_only: variant");
    assert!(
        world
            .engine_ended
            .iter()
            .any(|r| matches!(r, engine::Request::Ended { why: engine::Loss::Refused { reason: 3 }, .. }))
    );
    world.finish();
}

#[test]
fn optional_kinds_absent_from_the_worker_layout_are_not_sent() {
    let mut world = start(Profile::MissingKinds);
    let channel = world.channel();
    world.engine_event(engine::Event::PartBusy { channel, run: RUN, attempt: ATTEMPT, part: 1 });
    world.worker_up(Sample(7).part_notice(), Box::new([]));
    world.run();
    assert_eq!(world.engine_records(), []);
    assert_eq!(world.worker_records(), [Kind::Hello.number() + 1]);
    assert_eq!(heard(&world), []);
    assert!(world.engine_root.iter().all(|r| matches!(r, engine::Request::Hello { .. })));
    world.finish();
}
