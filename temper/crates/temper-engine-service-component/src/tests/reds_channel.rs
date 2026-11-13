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
}
impl Channel {
    fn new() -> Self {
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
        let engine = engine::Component::new(
            &limits,
            Box::new([engine::Worker { name: b"reader".as_slice().into(), digest: skein_io::digest::digest(secret) }]),
            jig_protocol_worker_wire::spoken(),
            payload::application(&fixtures::wire_limits()).unwrap(),
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
        let Some(engine::Request::Hello { channel: name, .. }) = channel.up.pop() else { panic!("hello crosses") };
        assert_eq!(name, CONNECTION);
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
                    other @ (worker::Request::Lower { .. }
                    | worker::Request::CannotSendWhole { .. }
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

#[test]
fn a_hosted_run_gets_its_goals_reds_through_jigs_channel() {
    let (mut service, run) = fixtures::claimed_chat_on(CONNECTION);
    reds::seed_reds(&mut service, run.task);
    let commits = service.engine.as_ref().unwrap().domain.deployment().commits;
    let mut channel = Channel::new();
    let mut limits = fixtures::engine().jig.limits.hub;
    limits.application_ask_bytes = 1;
    limits.application_answer_bytes = 141_317;
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
            bytes: temper_worker_service::translate::ask(temper_worker_domain::wire::Ask::Reds),
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
    assert_eq!(
        answer,
        Some(temper_worker_domain::wire::Answer::Reds {
            reds: Box::new([
                temper_worker_domain::wire::Red { key: b"lint p c f".as_slice().into(), head: Box::new([1; 20]) },
                temper_worker_domain::wire::Red { key: b"test a::t x".as_slice().into(), head: Box::new([2; 32]) },
            ])
        })
    );
    assert!(out.is_empty());
    assert_eq!(service.engine.as_ref().unwrap().domain.deployment().commits, commits);
}
