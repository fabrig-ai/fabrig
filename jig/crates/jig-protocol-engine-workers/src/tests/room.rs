//! The stream's room and starts refused locally (jig/docs/design/protocol/channel/liveness.md, 1, and jig/docs/design/protocol/channel/README.md, 10, and jig/docs/design/protocol/channel/testing.md, 1).
use super::fixtures;
use crate::{Bound, Component, Event, Limits, Request, Unsent, host};
use jig_domain_host as hub;
use jig_protocol_worker_wire::{self as wire, sizes};
use skein_lib::{Duration, List, Queue, Time, Token};

pub(super) fn config() -> jig_protocol_config::Engine {
    use jig_protocol_config as config;
    let document = config::document(
        br#"{
        "limits":{"profile":"standard"},
        "llm":{"providers":[],"endpoints":[]},
        "hosts":{"listener":[{"socket":"127.0.0.1:7100"}]},
        "clients":{"heartbeat_ms":10000,"link":true},
        "deployment":{"directory":"state","seed":7},
        "owners":[{"provider":1,"subject":"owner"}],
        "providers":[{"number":1,"kind":"link"},{"number":2,"kind":"deployment"}],
        "connectors":[],"memory_bytes":34359738368,"projects":[]
    }"#,
        &config::DocumentLimits { bytes: 1024, tokens: 256, depth: 8 },
    )
    .expect("configuration document");
    config::engine_over(
        &document,
        0,
        &config::Directories::working(),
        config::Defaults::Passed,
        &config::Fixed::NONE,
        &jig_protocol_config_fixtures::small(),
    )
    .expect("configuration")
}

fn frame_bytes(sizes: &sizes::Sizes, kind: wire::Kind) -> u32 {
    const HEADER: u32 = 8;
    sizes::largest_body(sizes, kind).expect("bounded body").checked_add(HEADER).expect("bounded frame")
}

#[test]
#[expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "the boundary table collects every diagnostic in ordinary Rust test code"
)]
fn the_room_holds_control_and_every_non_start_frame() {
    // A full Refuse: 8-byte header, 2-byte reason, 4-byte length, 256 bytes of text.
    const CONTROL_ROOM: u32 = 270;
    let config = config();
    let _files = jig_protocol_config::Files { entries: List::with_capacity(0) };
    let fixture = fixtures::SIZES;
    let resumed = sizes::smallest_resumed(fixture.answered, fixture.call_name, fixture.tool_name, fixture.answer)
        .expect("bounded resumed start");
    let (mut binding, mut floor) =
        if resumed > CONTROL_ROOM { ("resumed".to_owned(), resumed) } else { ("control".to_owned(), CONTROL_ROOM) };
    for rule in wire::KINDS {
        if rule.direction != skein_channel::Direction::FromResponder
            || rule.body == wire::Kind::Assignment
            || rule.body == wire::Kind::Job
            || rule.body == wire::Kind::Figures
        {
            continue;
        }
        let bytes = frame_bytes(&fixture, rule.body);
        if bytes > floor {
            floor = bytes;
            binding = format!("{:?}", rule.body);
        }
    }

    let message = sizes::Sizes { message: 1024, ..fixture };
    let relayed = sizes::Sizes { answer: 1024, ..fixture };
    let grant = sizes::Sizes { grant: 1024, ..fixture };
    let mut failures = Vec::new();
    for (name, sizes, bytes) in [
        (binding.as_str(), fixture, floor),
        ("message", message, frame_bytes(&message, wire::Kind::Message)),
        ("relayed", relayed, frame_bytes(&relayed, wire::Kind::Relayed)),
        ("grant", grant, frame_bytes(&grant, wire::Kind::Grant)),
    ] {
        for (room, accepted) in [(bytes, true), (bytes - 1, false)] {
            let limits = Limits { sizes, room, ..fixtures::LIMITS };
            let valid = crate::valid(&limits);
            let component = Component::new(
                &limits,
                fixtures::workers(),
                jig_protocol_worker_wire::spoken(),
                jig_protocol_worker_wire::Application::opaque(fixtures::application()),
            )
            .is_ok();
            let host = host::Workers::new(
                &config,
                host::Tokens { listeners: 1, connections: 2 },
                jig_protocol_worker_wire::Application::opaque(fixtures::application()),
                &limits,
            );
            if (valid, component, host.is_ok()) != (accepted, accepted, accepted) {
                failures.push(format!(
                    "{name}, room {room}: valid={valid}, component={component}, host={host:?}, expected {accepted}"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn retiming_the_host_reaches_its_component_without_io() {
    let mut workers = host::Workers::new(
        &config(),
        host::Tokens { listeners: 1, connections: 2 },
        wire::Application::opaque(fixtures::application()),
        &fixtures::LIMITS,
    )
    .expect("host");
    let timers = crate::Timers {
        handshake: Duration::from_secs(2),
        hello: fixtures::LIMITS.hello,
        stall: fixtures::LIMITS.stall,
    };
    let pending = workers.pending();
    let io = workers.io.len();
    assert!(workers.retime(timers));
    assert_eq!(workers.pending(), pending);
    assert_eq!(workers.io.len(), io);
    assert!(!workers.retime(crate::Timers { handshake: Duration::from_secs(3), hello: Duration::ZERO, ..timers }));
    workers.down(Time::ZERO, Event::Accepted { connection: Token::new(1), in_process: false });
    assert_eq!(workers.next_deadline(), Some(Time::ZERO.saturating_add(timers.handshake)));
}

fn job(bad: bool) -> hub::LinkDown {
    hub::LinkDown::AssignJob {
        job: hub::JobAssignment {
            run: Token::new(7),
            attempt: Token::new(2),
            kind: jig_domain_link::JobKind(1),
            spec: if bad { Box::from([0; 17]) } else { Box::from([0; 16]) },
            deadline: Duration::from_secs(60),
            workspace: None,
            grants: Box::new([]),
        },
    }
}

#[test]
fn a_resumed_start_is_fitted_to_the_engines_room() {
    const ENGINE: u32 = 620;
    const WORKER: u32 = 2048;
    // Header8 + fixed57 + charter7 + four (fields14 + name3 + tool4 + body256).
    const WHOLE: u32 = 1180;
    let limits = Limits {
        room: ENGINE,
        sizes: sizes::Sizes { answer: 256, answered: 4, ..fixtures::SIZES },
        ..fixtures::LIMITS
    };
    let mut component = Component::new(
        &limits,
        fixtures::workers(),
        jig_protocol_worker_wire::spoken(),
        jig_protocol_worker_wire::Application::opaque(fixtures::application()),
    )
    .expect("engine limits");
    let connection = Token::new(100);
    let channel = Token::new(1);
    component.down(Time::ZERO, Event::Accepted { connection, in_process: false }, &mut Queue::uncapped());
    // Bypass the opening guard to isolate fitting from refusal of unequal rooms.
    // The unopened channel refuses the fitted frame, exposing its measured bytes.
    let state = component.connections.get_mut(&connection).expect("accepted");
    state.phase = crate::connection::Phase::Open;
    state.heard = true;
    state.worker = Some(0);
    state.channel = channel;
    let mut assignment = fixtures::assignment();
    assignment.assignment.workspace = None;
    assignment.assignment.grants = Box::new([]);
    assignment.turns = Box::new([]);
    assignment.messages = Box::new([]);
    let mut calls = List::with_capacity(4);
    for _ in 0_u32..4 {
        calls
            .push(hub::AnsweredCall {
                name: Box::new([0, 2, 7]),
                tool: Box::from(b"read".as_slice()),
                answer: hub::SettledAnswer::Host { error: false, body: Box::new([b'a'; 256]) },
            })
            .expect("four calls");
    }
    assignment.answered = calls.into_boxed();
    let mut out = Queue::uncapped();
    component.down(
        Time::ZERO,
        Event::Down { channel, down: hub::LinkDown::Assign { assignment }, parts: Box::new([]), values: Box::new([]) },
        &mut out,
    );
    let Some(Request::Unsent { unsent, .. }) = out.pop() else { panic!("fitted frame refused by unopened channel") };
    // One answer stays whole; three become 67-byte cut words.
    assert_eq!(unsent.bytes, 617);
    assert_eq!(unsent.bound, Bound::Channel(skein_channel::Unsent::Closed));
    assert!(unsent.bytes <= ENGINE && ENGINE < WHOLE && WHOLE < WORKER);
}

#[test]
fn an_unencodable_job_is_answered_with_bounded_words() {
    for detail in [0, 16, 128] {
        let limits = Limits { sizes: sizes::Sizes { detail, ..fixtures::SIZES }, ..fixtures::LIMITS };
        let mut component = Component::new(
            &limits,
            fixtures::workers(),
            jig_protocol_worker_wire::spoken(),
            jig_protocol_worker_wire::Application::opaque(fixtures::application()),
        )
        .expect("limits");
        let connection = Token::new(100);
        let channel = Token::new(1);
        component.down(Time::ZERO, Event::Accepted { connection, in_process: false }, &mut Queue::uncapped());
        let state = component.connections.get_mut(&connection).expect("accepted");
        state.phase = crate::connection::Phase::Open;
        state.heard = true;
        state.worker = Some(0);
        state.channel = channel;
        let mut out = Queue::uncapped();
        component.down(
            Time::ZERO,
            Event::Down { channel, down: job(true), parts: Box::new([]), values: Box::new([]) },
            &mut out,
        );
        let Some(Request::Up { channel: found, up: hub::LinkUp::JobAnswer { run, attempt, answer }, parts }) =
            out.pop()
        else {
            panic!("job answer")
        };
        assert_eq!((found, run, attempt), (channel, Token::new(7), Token::new(2)));
        assert!(parts.is_empty());
        let words = b"job cannot encode: Job.spec too large";
        let why = words[..words.len().min(usize::try_from(detail).expect("detail"))].into();
        assert_eq!(answer, hub::JobEnding::Refused(hub::JobRefusal::Invalid(hub::JobInvalid::Start { why })));
        assert!(out.is_empty(), "no channel send or loss");
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the test compares the diagnostic with independently formatted words")]
fn a_channels_closed_entrance_answers_the_current_record_once() {
    for assignment in [true, false] {
        let limits = Limits { sizes: sizes::Sizes { detail: 128, ..fixtures::SIZES }, ..fixtures::LIMITS };
        let mut component = Component::new(
            &limits,
            fixtures::workers(),
            jig_protocol_worker_wire::spoken(),
            jig_protocol_worker_wire::Application::opaque(fixtures::application()),
        )
        .expect("limits");
        let connection = Token::new(100);
        let channel = Token::new(1);
        component.down(Time::ZERO, Event::Accepted { connection, in_process: false }, &mut Queue::uncapped());
        // A broken contract: the component believes an unopened channel is open.
        let state = component.connections.get_mut(&connection).expect("accepted");
        state.phase = crate::connection::Phase::Open;
        state.heard = true;
        state.worker = Some(0);
        state.channel = channel;
        let down = if assignment { hub::LinkDown::Assign { assignment: fixtures::assignment() } } else { job(false) };
        let record =
            crate::translate::down(down, &fixtures::parts(), fixtures::values(), &component.codec).expect("record");
        let kind = record.kind();
        let bytes = record.frame(&component.codec).unwrap().wire_len();
        let mut out = Queue::uncapped();
        let down = if assignment { hub::LinkDown::Assign { assignment: fixtures::assignment() } } else { job(false) };
        component.down(
            Time::ZERO,
            Event::Down { channel, down, parts: fixtures::parts(), values: fixtures::values() },
            &mut out,
        );
        let Some(Request::Unsent { worker, channel: found, unsent }) = out.pop() else { panic!("unsent first") };
        assert_eq!(
            (worker, found, unsent),
            (0, channel, Unsent { kind, bytes, bound: Bound::Channel(skein_channel::Unsent::Closed) })
        );
        let Some(Request::Up { up, parts, .. }) = out.pop() else { panic!("answer second") };
        assert!(parts.is_empty());
        let words = format!(
            "{} of {bytes} bytes refused by the channel: closed",
            if assignment { "assignment" } else { "job" }
        );
        match up {
            hub::LinkUp::Answer { answer, .. } => assert_eq!(
                answer.ending,
                hub::Ending::Refused(hub::Refusal::Invalid(hub::Invalid::Start {
                    cause: hub::StartCause::Other,
                    why: words.as_bytes().into()
                }))
            ),
            hub::LinkUp::JobAnswer { answer, .. } => assert_eq!(
                answer,
                hub::JobEnding::Refused(hub::JobRefusal::Invalid(hub::JobInvalid::Start {
                    why: words.as_bytes().into()
                }))
            ),
            // jig/docs/design/domain/engine/fleet.md, A paused worker leaves placement too.
            other @ (hub::LinkUp::Paused { .. }
            | hub::LinkUp::Draining
            | hub::LinkUp::UsageRead { .. }
            | hub::LinkUp::ListRead { .. }
            | hub::LinkUp::Piece { .. }
            | hub::LinkUp::Logins { .. }
            | hub::LinkUp::SignedOut { .. }
            | hub::LinkUp::Activity { .. }
            | hub::LinkUp::Unauthorized { .. }
            | hub::LinkUp::Usage { .. }
            | hub::LinkUp::Limited { .. }
            | hub::LinkUp::AccountRefused { .. }
            | hub::LinkUp::Part { .. }
            | hub::LinkUp::Turn { .. }
            | hub::LinkUp::Ask { .. }
            | hub::LinkUp::Call { .. }) => panic!("start answer: {other:?}"),
        }
        assert!(out.is_empty());
        let state = component.connections.get(&connection).expect("link kept");
        assert!(!state.queued());
        assert_eq!(state.ping, Some(Time::ZERO.saturating_add(limits.ping)));
    }
}
