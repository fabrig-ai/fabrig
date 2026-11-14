//! A focused engine/worker channel world, with a scripted worker root asking its real hub.
//! The worker's check-start behaviour is not involved (domain/forge/reds.md, 5.4).
use super::{fixtures, reds};
use jig_domain_host as hub;
use jig_protocol_engine_workers as engine;
use jig_protocol_worker_link as worker;
use skein_channel::{Lower, LowerEvent};
use skein_lib::{Duration, Env, Intake, Queue, Secret, Time, Token, Wall, stream};
use temper_engine_domain as root;
use temper_protocol_worker_wire::{self as payload, v1};

const CONNECTION: Token = Token::new(1);

struct Channel {
    engine: engine::Component,
    worker: worker::Component,
    engine_out: Queue<engine::Request>,
    worker_out: Queue<worker::Request>,
    up: Queue<engine::Request>,
    down: Queue<hub::LinkDown>,
    to_engine: Intake,
    to_worker: Intake,
    engine_read: Option<stream::Read>,
    worker_read: Option<stream::Read>,
    hello: Option<jig_domain_link::Hello>,
    cannot_send: Option<(&'static str, u16)>,
}
impl Channel {
    fn new() -> Self {
        Self::at_version(payload::wire::VERSION)
    }
    fn at_version(version: u16) -> Self {
        let settings = fixtures::engine();
        let limits = crate::workers::limits(&settings).unwrap();
        let largest = payload::largest(&v1::CEILINGS).unwrap();
        let worker_limits = worker::Limits {
            sizes: jig_protocol_worker_wire::sizes::Sizes {
                application_ask: largest.ask,
                application_answer: largest.answer,
                ..limits.sizes
            },
            chunk: limits.chunk,
            room: limits.room,
            handshake: limits.handshake,
            ping: limits.ping,
            silence: limits.silence,
            stall: limits.stall,
            backoff: Duration::from_millis(100),
            ceiling: Duration::from_secs(2),
            proved: Duration::from_secs(2),
            grace: Duration::from_secs(10),
            busy: Duration::from_millis(100),
        };
        let secret = b"reader-secret".as_slice();
        let mut application = payload::application(&fixtures::wire_limits()).unwrap();
        if version != payload::wire::VERSION {
            let spoken = application.spoken.get_mut(0).unwrap();
            *spoken = jig_protocol_worker_wire::Spoken::current(spoken.name, version, spoken.layout(version).unwrap());
        }
        let engine = engine::Component::new(
            &limits,
            Box::new([engine::Worker { name: b"reader".as_slice().into(), digest: skein_io::digest::digest(secret) }]),
            jig_protocol_worker_wire::spoken(),
            application,
        )
        .unwrap();
        let worker = worker::Component::new(
            &worker_limits,
            jig_protocol_worker_wire::spoken(),
            worker::Credential {
                name: b"reader".as_slice().into(),
                secret: Secret::new(secret.into()),
                application: payload::application(&v1::CEILINGS).unwrap(),
            },
            7,
        )
        .unwrap();
        let mut channel = Self {
            engine,
            worker,
            engine_out: Queue::uncapped(),
            worker_out: Queue::uncapped(),
            up: Queue::uncapped(),
            down: Queue::uncapped(),
            to_engine: Intake::with_capacity(limits.room),
            to_worker: Intake::with_capacity(limits.room),
            engine_read: None,
            worker_read: None,
            cannot_send: None,
            hello: Some(jig_domain_link::Hello {
                jobs: Box::new([]),
                slots: 1,
                stop_bound: Duration::from_millis(100),
                workstreams: Box::new([]),
                hosting: Box::new([]),
                bounds: fixtures::worker_agent().bounds,
            }),
        };
        channel.engine.down(Time::ZERO, engine::Event::Accepted { connection: CONNECTION }, &mut channel.engine_out);
        channel.worker.fire(Time::ZERO, &mut channel.worker_out);
        channel.settle();
        if version == payload::wire::VERSION {
            let Some(engine::Request::Hello { channel: name, .. }) = channel.up.pop() else { panic!("hello crosses") };
            assert_eq!(name, CONNECTION);
        }
        channel
    }
    fn engine_lower(&mut self, event: LowerEvent) {
        self.engine.down(Time::ZERO, engine::Event::Lower { connection: CONNECTION, event }, &mut self.engine_out);
    }
    fn worker_lower(&mut self, event: LowerEvent) {
        self.worker.down(Time::ZERO, worker::Event::Lower { connection: CONNECTION, event }, &mut self.worker_out);
    }
    fn settle(&mut self) {
        for _ in 0_u16..1000 {
            let mut progressed = false;
            while let Some(request) = self.engine_out.pop() {
                progressed = true;
                match request {
                    engine::Request::Lower { lower: Lower::Write(stream::OutputDown::Room { right, .. }), .. } => self
                        .engine_lower(LowerEvent::Write(stream::OutputUp::Settled {
                            right,
                            outcome: stream::OutputOutcome::Granted,
                        })),
                    engine::Request::Lower { lower: Lower::Write(stream::OutputDown::Send { bytes, .. }), .. } => {
                        self.to_worker.append(&bytes).unwrap();
                    }
                    engine::Request::Lower { lower: Lower::Read(stream::Down::Demand { read, .. }), .. } => {
                        self.engine_read = Some(read);
                    }
                    engine::Request::Opened { .. } | engine::Request::Take { .. } => {}
                    request @ (engine::Request::Hello { .. } | engine::Request::Up { .. }) => self.up.push(request),
                    other @ (engine::Request::Lower { .. }
                    | engine::Request::Refused { .. }
                    | engine::Request::Unsent { .. }
                    | engine::Request::TurnAway { .. }
                    | engine::Request::Close { .. }
                    | engine::Request::Abort { .. }
                    | engine::Request::Lost { .. }
                    | engine::Request::Ended { .. }
                    | engine::Request::Closed) => panic!("engine channel: {other:?}"),
                }
            }
            while let Some(request) = self.worker_out.pop() {
                progressed = true;
                match request {
                    worker::Request::Dial => self.worker.down(
                        Time::ZERO,
                        worker::Event::Dialled { connection: CONNECTION },
                        &mut self.worker_out,
                    ),
                    worker::Request::Ready => self.worker.down(
                        Time::ZERO,
                        worker::Event::Hello { hello: self.hello.take().expect("one opening") },
                        &mut self.worker_out,
                    ),
                    worker::Request::Lower { lower: Lower::Write(stream::OutputDown::Room { right, .. }), .. } => self
                        .worker_lower(LowerEvent::Write(stream::OutputUp::Settled {
                            right,
                            outcome: stream::OutputOutcome::Granted,
                        })),
                    worker::Request::Lower { lower: Lower::Write(stream::OutputDown::Send { bytes, .. }), .. } => {
                        self.to_engine.append(&bytes).unwrap();
                    }
                    worker::Request::Lower { lower: Lower::Read(stream::Down::Demand { read, .. }), .. } => {
                        self.worker_read = Some(read);
                    }
                    worker::Request::Down { down, parts } => {
                        assert!(parts.is_empty());
                        self.down.push(down);
                    }
                    worker::Request::CannotSendWhole { wire, version } => {
                        self.cannot_send = Some((wire, version));
                        return;
                    }
                    other @ (worker::Request::Lower { .. }
                    | worker::Request::Untranslated { .. }
                    | worker::Request::Refused { .. }
                    | worker::Request::RelayCancelled { .. }
                    | worker::Request::Resend { .. }
                    | worker::Request::Lost
                    | worker::Request::GracePassed
                    | worker::Request::Closed
                    | worker::Request::Value { .. }
                    | worker::Request::Close { .. }
                    | worker::Request::Abort { .. }
                    | worker::Request::Fact { .. }) => panic!("worker channel: {other:?}"),
                }
            }
            if let Some(read) = self.engine_read
                && let Some(bytes) = self.to_engine.meet(read)
            {
                self.engine_read = None;
                self.engine_lower(LowerEvent::Read(stream::Up::Bytes(bytes)));
                progressed = true;
            }
            if let Some(read) = self.worker_read
                && let Some(bytes) = self.to_worker.meet(read)
            {
                self.worker_read = None;
                self.worker_lower(LowerEvent::Read(stream::Up::Bytes(bytes)));
                progressed = true;
            }
            if !progressed {
                return;
            }
        }
        panic!("channel settles");
    }
    fn send_down(&mut self, down: hub::LinkDown) {
        self.engine.down(
            Time::ZERO,
            engine::Event::Down { channel: CONNECTION, down, parts: Box::new([]), values: Box::new([]) },
            &mut self.engine_out,
        );
        self.settle();
    }
}

fn exchange(
    ask: temper_worker_domain::wire::Ask,
    expected: temper_worker_domain::wire::Answer,
) -> (crate::Service, u64) {
    let (mut service, run) = fixtures::claimed_chat_on(CONNECTION);
    let results = match &ask {
        temper_worker_domain::wire::Ask::Reds => {
            reds::seed_reds(&mut service, run.task);
            false
        }
        temper_worker_domain::wire::Ask::Results(_) => {
            reds::seed_check_change(&mut service, run.task);
            true
        }
    };
    let commits = service.engine.as_ref().unwrap().domain.deployment().commits;
    let mut channel = Channel::new();
    let mut limits = fixtures::engine().jig.limits.hub;
    let largest = payload::largest(&v1::CEILINGS).unwrap();
    limits.application_ask_bytes = largest.ask;
    limits.application_answer_bytes = largest.answer;
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut host = hub::Hub::new(&limits);
    let mut out = Queue::uncapped();
    channel.send_down(hub::LinkDown::Assign {
        assignment: hub::Assignment {
            assignment: hub::RunAssignment {
                identity: [0; 16],
                run: Token::new(run.task),
                attempt: Token::new(run.attempt),
                workspace: None,
                save: true,
                live: hub::LiveAsk::NONE,
                charter: b"charter".as_slice().into(),
                grants: Box::new([]),
            },
            messages: Box::new([]),
            turns: Box::new([]),
            answered: Box::new([]),
        },
    });
    hub::step(&mut host, &env, hub::Event::Down(channel.down.pop().expect("assignment crossed")), &mut out);
    let Some(hub::Request::Agent(hub::ToAgent::Start { .. })) = out.pop() else { panic!("run hosted") };
    hub::step(
        &mut host,
        &env,
        hub::Event::Ask {
            run: Token::new(run.task),
            attempt: Token::new(run.attempt),
            bytes: temper_worker_service::translate::ask(ask),
        },
        &mut out,
    );
    let Some(hub::Request::Up(up)) = out.pop() else { panic!("hosted ask") };
    channel.worker.down(Time::ZERO, worker::Event::Up { up, parts: Box::new([]) }, &mut channel.worker_out);
    channel.settle();
    let Some(engine::Request::Up { channel: name, up, parts }) = channel.up.pop() else { panic!("ask crossed") };
    assert_eq!(name, CONNECTION);
    let engine = service.engine.as_mut().unwrap();
    engine
        .workers
        .route(jig_protocol_engine_workers::host::Output::Up { channel: name, up, parts }, &mut engine.events);
    let mut delivered = Queue::uncapped();
    fixtures::settle_root(&mut service, &mut delivered);
    let Some(root::Delivery::Link { channel: name, down }) = delivered.pop() else { panic!("answer") };
    assert_eq!(name, CONNECTION);
    channel.send_down(down);
    hub::step(&mut host, &env, hub::Event::Down(channel.down.pop().expect("answer crossed")), &mut out);
    let Some(hub::Request::Asked { run: task, attempt, number, answer: hub::AskAnswer::Bytes(bytes) }) = out.pop()
    else {
        panic!("numbered answer")
    };
    assert_eq!((task.raw(), attempt.raw(), number), (run.task, run.attempt, 1));
    let answer = temper_worker_service::translate::answer(&bytes, &v1::CEILINGS);
    assert_eq!(answer, Some(expected));
    assert!(out.is_empty());
    if results {
        assert!(service.engine.as_ref().unwrap().domain.deployment().commits > commits);
    } else {
        assert_eq!(service.engine.as_ref().unwrap().domain.deployment().commits, commits);
    }
    (service, run.task)
}

#[test]
fn a_hosted_run_gets_its_goals_reds_through_jigs_channel() {
    use temper_worker_domain::wire::{Answer, Ask, Red};
    exchange(
        Ask::Reds,
        Answer::Reds {
            reds: Box::new([
                Red { key: b"lint p c f".as_slice().into(), head: Box::new([1; 20]) },
                Red { key: b"test a::t x".as_slice().into(), head: Box::new([2; 32]) },
            ]),
        },
    );
}

#[test]
fn a_hosted_runs_results_are_kept_through_jigs_channel() {
    use temper_engine_domain_forge as forge;
    let mut report = super::results::report();
    report.tag = forge::tag(1, 41);
    report.passed = Box::new([b"test a::t x".as_slice().into()]);
    let (mut service, task) =
        exchange(temper_worker_domain::wire::Ask::Results(report), temper_worker_domain::wire::Answer::Kept);
    let rows = fixtures::load(&mut service.engine.as_mut().unwrap().store, root::Range::Forge);
    let mut pass = None;
    let mut finding = None;
    for record in &rows {
        if let root::Record::Forge { row } = record {
            if let forge::Stored::RedPass(row) = row.as_ref() {
                assert!(pass.replace(row).is_none());
            }
            if let forge::Stored::Finding(row) = row.as_ref() {
                assert!(finding.replace(row).is_none());
            }
        }
    }
    assert_eq!(
        pass.expect("the report's pass is durable"),
        &forge::RedPass { change: task, key: b"test a::t x".as_slice().into(), generation: 2 }
    );
    let finding = finding.expect("the report's flake is durable");
    assert_eq!(
        (finding.project, finding.filer, finding.kind, finding.seen),
        (1, forge::Filer::Task(task), forge::FindingKind::Flake, Some(1))
    );
    assert_eq!(finding.title.as_ref(), b"Flake in a delivery's check: test fabrig-x::t b");
    assert!(finding.body.ends_with(b"Its log under load:\nboom"));
}

#[test]
fn a_v9_worker_closes_at_a_v8_engines_statement_before_hello() {
    let channel = Channel::at_version(8);
    assert_eq!(channel.cannot_send, Some((payload::wire::FAMILY, 8)));
    assert!(channel.hello.is_some(), "the worker never became ready to send hello");
    assert!(channel.up.is_empty(), "the engine sees no worker record");
}
