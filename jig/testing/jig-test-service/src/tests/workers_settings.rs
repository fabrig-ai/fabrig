//! The admission component and core see each settings decision without a restart.
use super::fixtures::{self, Driven};
use jig_domain_core::{self as core, settings as s};
use jig_domain_people as people;
use jig_protocol_engine_workers as engine;
use jig_protocol_worker_wire as wire;
use jig_protocol_worker_wire::Record;
use skein_channel::{Lower, LowerEvent};
use skein_lib::{Duration, Intake, List, Queue, ReplyTo, Secret, Time, Token, Wall, Writer, stream};

const NAME: &[u8] = b"w1";
const SECRET: &[u8] = b"settings-worker";

pub(super) fn setting(places: u32) -> s::WorkerSetting {
    s::WorkerSetting {
        number: 1,
        name: NAME.into(),
        places,
        digest: Some(skein_lib::sha256::digest(SECRET)),
        withdrawn: false,
    }
}
pub(super) fn change(driven: &mut Driven, serial: u128, edit: s::Edit) {
    let against = fixtures::engine(&mut driven.service).domain.settings().newest();
    driven.party(core::Event::ChangeSettings {
        reply_to: ReplyTo::new(Token::new(u64::try_from(serial).unwrap())),
        sign_in: driven.sign_in.raw(),
        key: serial.to_be_bytes(),
        digest: [u8::try_from(serial).unwrap(); 32],
        change: Box::new(s::Change {
            against,
            running: s::Running::Stays,
            items: Box::new([s::Item { setting: s::Setting::Worker(1), edit }]),
        }),
    });
    driven.commit_all();
}
fn component(driven: &mut Driven) -> &mut engine::Component {
    &mut fixtures::engine(&mut driven.service).workers.as_mut().expect("workers").host.component
}
fn lower(driven: &mut Driven, connection: Token, event: LowerEvent, out: &mut Queue<engine::Request>) {
    component(driven).down(Time::ZERO, engine::Event::Lower { connection, event }, out);
}
fn hello(_codec: &wire::v19::Limits) -> skein_channel::Frame {
    let mut bounds = fixtures::agent().bounds;
    bounds.inbox = u32::MAX;
    bounds.unacknowledged = u32::MAX;
    bounds.checks = Duration::from_nanos(1);
    let mut endpoints = List::with_capacity(u32::try_from(bounds.endpoints.len()).unwrap());
    for row in bounds.endpoints {
        endpoints.push(wire::EndpointBound { endpoint: row.endpoint, window: row.window, output: row.output }).unwrap();
    }
    let bounds = wire::Bounds {
        endpoints,
        program_endpoints: List::from_boxed(bounds.program_endpoints),
        turns: bounds.turns,
        spend: bounds.spend,
        time: bounds.time,
        waiting: bounds.waiting,
        checks: bounds.checks,
        message_bytes: bounds.message_bytes,
        inbox: bounds.inbox,
        unacknowledged: bounds.unacknowledged,
        reply_bytes: bounds.reply_bytes,
    };
    let hello = wire::Hello {
        fresh: false,
        slots: 4,
        stop_bound: Duration::ZERO,
        bounds,
        workstreams: List::with_capacity(0),
        hosting: List::with_capacity(0),
        jobs: List::with_capacity(0),
    };
    let mut body = Writer::new(
        usize::try_from(hello.measure(&jig_protocol_worker_wire::CEILINGS).expect("record within limits")).unwrap(),
    );
    hello.write(&jig_protocol_worker_wire::CEILINGS, &mut body).unwrap();
    let mut frame = skein_channel::frame_writer(
        wire::Kind::Hello.number(),
        hello.measure(&jig_protocol_worker_wire::CEILINGS).expect("record within limits"),
    )
    .unwrap();
    frame.put(&body.finish()).unwrap();
    frame.finish().unwrap()
}
#[expect(
    clippy::too_many_lines,
    reason = "one scripted opening keeps its credential, fake stream and root link handoff in the same finite pump"
)]
fn opening(driven: &mut Driven, connection: Token, secret: &[u8]) -> (Option<u16>, Option<Token>) {
    let config = fixtures::config();
    let limits = crate::limits::small(&config.limits).workers;
    let codec = wire::sizes::limits(&limits.sizes).unwrap();
    let mut wires = List::with_capacity(codec.credential_wires);
    for spoken in wire::spoken().into_boxed() {
        wires
            .push(wire::Wire { name: spoken.name.as_bytes().into(), lowest: spoken.lowest, layouts: spoken.layouts })
            .unwrap();
    }
    let credential = wire::Credential {
        opening: wire::Opening::Link,
        name: NAME.into(),
        secret: Secret::new(secret.into()),
        wires,
        output: limits.room,
    };
    let mut bytes = Writer::new(
        usize::try_from(credential.measure(&jig_protocol_worker_wire::CEILINGS).expect("record within limits"))
            .unwrap(),
    );
    credential.write(&jig_protocol_worker_wire::CEILINGS, &mut bytes).unwrap();
    let mut peer = skein_fake_channel::ScriptedPeer::new(
        engine::limits::schema(&limits).unwrap(),
        skein_channel::Role::Initiator,
        jig_protocol_worker_link::channel_limits(&jig_protocol_worker_link::Limits {
            sizes: limits.sizes,
            chunk: limits.chunk,
            room: limits.room,
            handshake: limits.handshake,
            ping: limits.ping,
            silence: limits.silence,
            stall: limits.stall,
            backoff: Duration::from_secs(1),
            ceiling: Duration::from_secs(10),
            proved: Duration::from_secs(10),
            grace: Duration::from_secs(60),
            busy: Duration::from_secs(1),
        })
        .unwrap(),
        wire::VERSION,
        bytes.finish(),
    )
    .unwrap();
    peer.play(hello(&codec));
    let mut out = Queue::uncapped();
    let mut intake = Intake::with_capacity(limits.room);
    let mut read = stream::Read::Nothing;
    let mut refused = None;
    let mut admitted = None;
    component(driven).down(Time::ZERO, engine::Event::Accepted { connection, in_process: false }, &mut out);
    for _ in 0_u16..1000 {
        let mut progress = false;
        while let Some(request) = out.pop() {
            progress = true;
            match request {
                engine::Request::Lower { lower: Lower::Write(stream::OutputDown::Room { right, .. }), .. } => lower(
                    driven,
                    connection,
                    LowerEvent::Write(stream::OutputUp::Settled { right, outcome: stream::OutputOutcome::Granted }),
                    &mut out,
                ),
                engine::Request::Lower { lower: Lower::Write(stream::OutputDown::Send { bytes, .. }), .. } => {
                    peer.feed(&bytes).unwrap();
                }
                engine::Request::Lower { lower: Lower::Read(stream::Down::Demand { read: demand, .. }), .. } => {
                    read = demand;
                }
                engine::Request::Refused { reason, .. } => refused = Some(reason),
                engine::Request::Hello { channel, worker, hello } => {
                    admitted = Some(channel);
                    fixtures::engine(&mut driven.service).events.push(jig_test_domain::Event::Link {
                        channel,
                        link: jig_test_domain::link::Event::Hello { worker, hello },
                    });
                }
                engine::Request::Close { .. } | engine::Request::Abort { .. } => {
                    component(driven).down(Time::ZERO, engine::Event::Gone { connection }, &mut out);
                }
                engine::Request::Lost { channel } => fixtures::engine(&mut driven.service)
                    .events
                    .push(jig_test_domain::Event::Link { channel, link: jig_test_domain::link::Event::Lost }),
                engine::Request::Told { .. }
                | engine::Request::Take { .. }
                | engine::Request::TurnAway { .. }
                | engine::Request::Opened { .. }
                | engine::Request::FiguresTaken { .. }
                | engine::Request::Ended { .. }
                | engine::Request::Up { .. }
                | engine::Request::Unsent { .. }
                | engine::Request::Closed
                | engine::Request::Lower { .. } => {}
            }
        }
        if let Some(bytes) = peer.pop_output() {
            intake.append(&bytes).unwrap();
            progress = true;
        }
        if let Some(bytes) = intake.meet(read) {
            read = stream::Read::Nothing;
            lower(driven, connection, LowerEvent::Read(stream::Up::Bytes(bytes)), &mut out);
            progress = true;
        }
        if !progress {
            driven.commit_all();
            return (refused, admitted);
        }
    }
    panic!("opening settles");
}
fn deployment(driven: &mut Driven) -> Box<core::documents::Deployment> {
    let engine = fixtures::engine(&mut driven.service);
    let core::documents::Document::Deployment(document) = core::documents::make(
        engine.domain.core(),
        &engine.env.limits.core,
        1,
        &core::documents::Subject::Deployment,
        &core::documents::Window { rows: 4, after: None },
        Wall::EPOCH,
    )
    .unwrap() else {
        panic!("deployment")
    };
    document
}

fn row(driven: &mut Driven) -> core::documents::WorkerState {
    deployment(driven).worker_settings.get(0).unwrap().clone()
}

#[test]
fn a_worker_joins_after_its_setting_and_places_wait_for_its_next_opening() {
    let mut driven = Driven::started(&fixtures::workers_document());
    assert_eq!(opening(&mut driven, Token::new(100), SECRET), (Some(wire::Reason::NotAdmitted.code()), None));
    change(&mut driven, 101, s::Edit::Set(s::Value::Worker(Box::new(setting(1)))));
    assert_eq!(driven.reply, Some(people::Reply::Outcome(people::Outcome::SettingsChanged { revision: 2 })));
    assert_eq!(opening(&mut driven, Token::new(102), b"wrong"), (Some(wire::Reason::NotAdmitted.code()), None));
    let (_, channel) = opening(&mut driven, Token::new(103), SECRET);
    assert!(channel.is_some(), "adding the setting admitted the next opening, without a restart");
    assert!(row(&mut driven).linked, "hello refusals: {:?}", deployment(&mut driven).hello_refusals);
    change(&mut driven, 104, s::Edit::Set(s::Value::Worker(Box::new(setting(2)))));
    assert_eq!(deployment(&mut driven).hosts.get(0).unwrap().slots, 1);
    change(&mut driven, 105, s::Edit::Removed);
    assert_eq!(
        driven.reply,
        Some(people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NotWellFormed {
            at: 0,
            why: people::Malformed::Settings { setting: s::Setting::Worker(1), fault: s::Fault::Linked, detail: None },
        })))
    );
    // Feed the component's loss through the same root link door as its host.
    let mut out = Queue::uncapped();
    component(&mut driven).down(Time::ZERO, engine::Event::Gone { connection: Token::new(103) }, &mut out);
    while let Some(request) = out.pop() {
        if let engine::Request::Lost { channel } = request {
            fixtures::engine(&mut driven.service)
                .events
                .push(jig_test_domain::Event::Link { channel, link: jig_test_domain::link::Event::Lost });
        }
    }
    driven.commit_all();
    let lost = row(&mut driven);
    assert!(!lost.linked);
    assert_eq!(lost.since, Some(Wall::from_nanos(1)));
    assert!(opening(&mut driven, Token::new(106), SECRET).1.is_some());
    let hosts = deployment(&mut driven).hosts;
    let mut took_places = false;
    for host in &hosts {
        took_places |= host.connected && host.slots == 2;
    }
    assert!(took_places);
}
