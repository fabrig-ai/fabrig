//! The stream's room and starts refused locally (jig/docs/design/protocol/channel.md, 9, 11 and 12).
use super::fixtures;
use crate::{Bound, Component, Event, Limits, Request, Unsent, host};
use jig_domain_host as hub;
use jig_protocol_worker_wire::{self as wire, sizes};
use skein_lib::{Duration, List, Queue, Time, Token};

fn config() -> jig_protocol_config::Engine {
    use jig_protocol_config as config;
    let document = config::document(
        br#"{
        "limits":{"profile":"small"},
        "llm":{"providers":[],"endpoints":[]},
        "hosts":{"workers":[],"listener":[{"socket":"127.0.0.1:7100"}]},
        "clients":{"heartbeat_ms":10000,"link":true},
        "deployment":{"directory":"state","seed":7},
        "owners":[{"provider":1,"subject":"owner"}],
        "providers":[{"number":1,"kind":"link"},{"number":2,"kind":"deployment"}],
        "connectors":[],"memory_bytes":34359738368,"projects":[]
    }"#,
        &config::DocumentLimits { bytes: 1024, tokens: 256, depth: 8 },
    )
    .expect("configuration document");
    config::engine(&document, 0, &config::Directories::working(), config::Defaults::Passed).expect("configuration")
}

#[test]
#[expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "the boundary table collects every diagnostic in ordinary Rust test code"
)]
fn the_room_holds_control_and_every_non_start_frame() {
    let config = config();
    let files = jig_protocol_config::Files { entries: List::with_capacity(0) };
    let mut failures = Vec::new();
    for (name, sizes, bytes) in [
        ("refuse", fixtures::SIZES, 270),
        ("message", sizes::Sizes { message: 1024, ..fixtures::SIZES }, 0),
        ("relayed", sizes::Sizes { answer: 1024, ..fixtures::SIZES }, 0),
        ("grant", sizes::Sizes { grant: 1024, ..fixtures::SIZES }, 0),
    ] {
        let kind = match name {
            "message" => wire::Kind::Message,
            "relayed" => wire::Kind::Relayed,
            "grant" => wire::Kind::Grant,
            _ => wire::Kind::Acknowledge,
        };
        let bytes = if bytes == 0 { sizes::largest_body(&sizes, kind).expect("bounded body") + 8 } else { bytes };
        for (room, accepted) in [(bytes, true), (bytes - 1, false)] {
            let limits = Limits { sizes, room, ..fixtures::LIMITS };
            let valid = crate::valid(&limits);
            let component = Component::new(&limits, fixtures::workers(), fixtures::PAYLOAD).is_ok();
            let host = host::Workers::new(
                &config,
                &files,
                host::Tokens { listeners: 1, connections: 2 },
                fixtures::PAYLOAD,
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
fn an_unencodable_job_is_answered_with_bounded_words() {
    for detail in [0, 16, 128] {
        let limits = Limits { sizes: sizes::Sizes { detail, ..fixtures::SIZES }, ..fixtures::LIMITS };
        let mut component = Component::new(&limits, fixtures::workers(), fixtures::PAYLOAD).expect("limits");
        let connection = Token::new(100);
        let channel = Token::new(1);
        component.down(Time::ZERO, Event::Accepted { connection }, &mut Queue::uncapped());
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
        let words = b"job cannot encode";
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
        let mut component = Component::new(&limits, fixtures::workers(), fixtures::PAYLOAD).expect("limits");
        let connection = Token::new(100);
        let channel = Token::new(1);
        component.down(Time::ZERO, Event::Accepted { connection }, &mut Queue::uncapped());
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
        let bytes = record.frame().wire_len();
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
            other @ (hub::LinkUp::Draining
            | hub::LinkUp::ListRead { .. }
            | hub::LinkUp::Piece { .. }
            | hub::LinkUp::Logins { .. }
            | hub::LinkUp::SignedOut { .. }
            | hub::LinkUp::Activity { .. }
            | hub::LinkUp::Unauthorized { .. }
            | hub::LinkUp::Limited { .. }
            | hub::LinkUp::AccountRefused { .. }
            | hub::LinkUp::Turn { .. }
            | hub::LinkUp::Call { .. }) => panic!("start answer: {other:?}"),
        }
        assert!(out.is_empty());
        let state = component.connections.get(&connection).expect("link kept");
        assert!(!state.queued());
        assert_eq!(state.ping, Some(Time::ZERO.saturating_add(limits.ping)));
    }
}
