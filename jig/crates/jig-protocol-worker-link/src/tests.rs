mod backoff;
mod busy;
mod fixtures;
mod relays;
mod translate;

#[expect(clippy::wildcard_enum_match_arm, reason = "the test refuses every unexpected opening request")]
fn opening(component: &mut crate::Component) -> skein_lib::List<skein_lib::stream::OutputDown> {
    use crate::{Event, Request};
    use skein_channel::{Lower, LowerEvent};
    use skein_lib::{List, Queue, Time, Token, stream};
    let mut out = Queue::uncapped();
    component.fire(Time::ZERO, &mut out);
    let Request::Dial = out.pop().expect("first dial") else { panic!("first dial") };
    assert!(out.is_empty());
    let connection = Token::new(1);
    component.down(Time::ZERO, Event::Dialled { connection }, &mut out);
    let mut writes = List::uncapped();
    for _ in 0_u8..16 {
        let Some(request) = out.pop() else { break };
        match request {
            Request::Lower { connection: token, lower: Lower::Write(write) } => {
                assert_eq!(token, connection);
                if let stream::OutputDown::Room { right, .. } = &write {
                    component.down(
                        Time::ZERO,
                        Event::Lower {
                            connection,
                            event: LowerEvent::Write(stream::OutputUp::Settled {
                                right: *right,
                                outcome: stream::OutputOutcome::Granted,
                            }),
                        },
                        &mut out,
                    );
                }
                writes.push(write).expect("uncapped writes");
            }
            Request::Lower { lower: Lower::Read(_), .. } => {}
            other => panic!("unexpected opening request: {other:?}"),
        }
    }
    assert!(out.is_empty());
    let mut sent = false;
    for write in &writes {
        match write {
            stream::OutputDown::Send { .. } => sent = true,
            stream::OutputDown::Room { .. }
            | stream::OutputDown::Cancel { .. }
            | stream::OutputDown::Release { .. } => {}
        }
    }
    assert!(sent);
    writes
}

#[test]
fn a_prepared_link_opens_as_a_new_link() {
    use jig_protocol_worker_wire as wire;
    let table = wire::schema(&fixtures::codec()).expect("table");
    let mut prepared =
        crate::Component::with_schema(&fixtures::LIMITS, wire::spoken(), fixtures::credential(), 1, table)
            .expect("prepared link");
    let mut new =
        crate::Component::new(&fixtures::LIMITS, wire::spoken(), fixtures::credential(), 1).expect("new link");
    assert_eq!(opening(&mut prepared), opening(&mut new));
}

#[test]
fn a_prepared_link_refuses_an_invalid_table() {
    use jig_protocol_worker_wire as wire;
    let mut table = wire::schema(&fixtures::codec()).expect("table");
    table.versions.get_mut(0).expect("floor").kinds.get_mut(0).expect("first kind").largest = u32::MAX;
    assert!(!crate::valid_with_schema(&fixtures::LIMITS, &table));
    assert_eq!(
        crate::Component::with_schema(&fixtures::LIMITS, wire::spoken(), fixtures::credential(), 1, table).err(),
        Some(crate::Unusable::Limits),
    );
}

#[test]
fn a_prepared_link_requires_the_stated_worker_wire_range() {
    use jig_protocol_worker_wire as wire;
    use skein_lib::List;
    for lowest in [None, Some(wire::wire::FLOOR.saturating_sub(1)), Some(wire::VERSION.saturating_add(1))] {
        let mut wires = List::uncapped();
        for spoken in &wire::spoken() {
            if spoken.name != wire::wire::FAMILY {
                wires.push(spoken.clone()).expect("uncapped wires");
            } else if let Some(lowest) = lowest {
                wires.push(wire::Spoken::current(spoken.name, lowest, [0; 32])).expect("uncapped wires");
            }
        }
        let table = wire::schema(&fixtures::codec()).expect("table");
        assert_eq!(
            crate::Component::with_schema(&fixtures::LIMITS, wires, fixtures::credential(), 1, table).err(),
            Some(crate::Unusable::Limits),
        );
    }
}

#[test]
fn valid_refuses_each_unrepresentable_limit() {
    for (name, limits, accepted) in [
        ("fixture", fixtures::LIMITS, true),
        ("handshake", crate::Limits { handshake: skein_lib::Duration::ZERO, ..fixtures::LIMITS }, false),
        ("ping", crate::Limits { ping: skein_lib::Duration::ZERO, ..fixtures::LIMITS }, false),
        ("stall", crate::Limits { stall: skein_lib::Duration::ZERO, ..fixtures::LIMITS }, false),
        ("silence", crate::Limits { silence: fixtures::LIMITS.ping, ..fixtures::LIMITS }, false),
        ("chunk", crate::Limits { chunk: 0, ..fixtures::LIMITS }, false),
        ("grace", crate::Limits { grace: skein_lib::Duration::ZERO, ..fixtures::LIMITS }, false),
        ("busy", crate::Limits { busy: skein_lib::Duration::ZERO, ..fixtures::LIMITS }, false),
        ("backoff", crate::Limits { backoff: skein_lib::Duration::ZERO, ..fixtures::LIMITS }, false),
        ("proved", crate::Limits { proved: skein_lib::Duration::ZERO, ..fixtures::LIMITS }, false),
        ("ceiling", crate::Limits { ceiling: skein_lib::Duration::ZERO, ..fixtures::LIMITS }, false),
        (
            "name edge",
            crate::Limits {
                sizes: jig_protocol_worker_wire::sizes::Sizes { name: u32::MAX, ..fixtures::SIZES },
                ..fixtures::LIMITS
            },
            false,
        ),
        (
            "program_sources edge",
            crate::Limits {
                sizes: jig_protocol_worker_wire::sizes::Sizes { program_sources: u32::MAX, ..fixtures::SIZES },
                ..fixtures::LIMITS
            },
            false,
        ),
        (
            "slots edge",
            crate::Limits {
                sizes: jig_protocol_worker_wire::sizes::Sizes { slots: u32::MAX, ..fixtures::SIZES },
                ..fixtures::LIMITS
            },
            false,
        ),
        (
            "turns edge",
            crate::Limits {
                sizes: jig_protocol_worker_wire::sizes::Sizes { turns: u32::MAX, ..fixtures::SIZES },
                ..fixtures::LIMITS
            },
            false,
        ),
        (
            "calls edge",
            crate::Limits {
                sizes: jig_protocol_worker_wire::sizes::Sizes { calls: u32::MAX, ..fixtures::SIZES },
                ..fixtures::LIMITS
            },
            false,
        ),
        (
            "accounts edge",
            crate::Limits {
                sizes: jig_protocol_worker_wire::sizes::Sizes { accounts: u32::MAX, ..fixtures::SIZES },
                ..fixtures::LIMITS
            },
            false,
        ),
        (
            "credentials edge",
            crate::Limits {
                sizes: jig_protocol_worker_wire::sizes::Sizes { credentials: u32::MAX, ..fixtures::SIZES },
                ..fixtures::LIMITS
            },
            false,
        ),
    ] {
        assert_eq!(crate::valid(&limits), accepted, "{name}");
    }
}
