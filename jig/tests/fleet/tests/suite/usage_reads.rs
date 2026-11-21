//! Usage placing uses the last usable inventory and the negotiated capability.
use super::{FIRST, RETURNED, SECOND, World, hello_endpoints};
use jig_domain_fleet::{Event, Request};
use jig_domain_link::{Login, LoginState, SourceName, UsageOutcome};
use skein_lib::Token;

fn source() -> SourceName {
    SourceName::new(b"personal").expect("fixture source")
}
fn declared() -> Event {
    Event::Sources { sources: Box::new([source()]) }
}
fn inventory(channel: Token, state: LoginState) -> Event {
    Event::Logins { channel, logins: Box::new([Login { name: source(), state, email: None }]) }
}
fn hello(channel: Token, capable: bool) -> Event {
    let mut event = hello_endpoints(channel, 0, &[]);
    if let Event::Hello { hello, .. } = &mut event {
        hello.reads_usage = capable;
    }
    event
}
fn read(read: u64) -> Event {
    Event::ReadUsage { read, source: source() }
}
fn sent(channel: Token, read: u64) -> Request {
    Request::ReadUsage { channel, read, source: source() }
}
fn answer(channel: Token, read: u64) -> Event {
    Event::UsageRead { channel, read, outcome: UsageOutcome::NotSignedIn }
}
fn answered(read: u64) -> Request {
    Request::UsageRead { read, outcome: UsageOutcome::NotSignedIn }
}
fn ready(world: &mut World, channel: Token) {
    world.send(hello(channel, true));
    world.send(inventory(channel, LoginState::Usable));
}

#[test]
fn usage_waits_for_a_ready_undrained_capable_login_without_a_slot_or_endpoint() {
    for (label, capable, login, drained, places) in [
        ("capable usable", true, Some(LoginState::Usable), false, true),
        ("old wire", false, Some(LoginState::Usable), false, false),
        ("not ready", true, None, false, false),
        ("not signed in", true, Some(LoginState::NotSignedIn), false, false),
        (
            "other method",
            true,
            Some(LoginState::SignedInOtherwise(jig_domain_link::LoginMethod::new(b"key").expect("fixture method"))),
            false,
            false,
        ),
        ("draining", true, Some(LoginState::Usable), true, false),
    ] {
        let mut world = World::new();
        world.send(declared());
        assert!(world.send(read(1)).is_empty(), "{label}: held while disconnected");
        assert_eq!(
            world.send(hello(FIRST, capable)),
            [Request::Sources { channel: FIRST, sources: Box::new([source()]) }]
        );
        if drained {
            assert_eq!(world.send(Event::Draining { channel: FIRST }), []);
        }
        let out = login.map(|state| world.send(inventory(FIRST, state))).unwrap_or_default();
        assert_eq!(out, if places { vec![sent(FIRST, 1)] } else { vec![] }, "{label}");
        if !places {
            let out = world.send(inventory(FIRST, LoginState::Usable));
            assert_eq!(out, if capable && !drained { vec![sent(FIRST, 1)] } else { vec![] }, "{label}: new inventory");
        }
    }
    let mut world = World::new();
    let other = SourceName::new(b"other").expect("other source");
    world.send(Event::Sources { sources: Box::new([source(), other]) });
    world.send(hello(FIRST, true));
    world.send(read(1));
    assert!(
        world
            .send(Event::Logins {
                channel: FIRST,
                logins: Box::new([
                    Login { name: source(), state: LoginState::NotSignedIn, email: None },
                    Login { name: other, state: LoginState::Usable, email: None },
                ])
            })
            .is_empty(),
        "another usable login does not cover the read's source"
    );
}

#[test]
fn cancelled_usage_reads_release_queue_capacity_and_workers_and_fence_answers() {
    let mut world = World::new();
    world.send(declared());
    assert_eq!(world.send(read(1)), []);
    assert_eq!(world.send(Event::CancelUsage { read: 1 }), []);
    assert_eq!(world.send(Event::CancelUsage { read: 99 }), [], "unknown cancellation changes nothing");
    world.send(hello(FIRST, true));
    assert_eq!(world.send(inventory(FIRST, LoginState::Usable)), [], "a cancelled queued read is never sent");
    assert_eq!(world.send(read(2)), [sent(FIRST, 2)]);
    assert_eq!(world.send(read(3)), []);
    assert_eq!(world.send(Event::CancelUsage { read: 2 }), [sent(FIRST, 3)], "cancellation frees the worker");
    assert_eq!(world.send(answer(FIRST, 2)), [], "a late answer cannot release the next flight");
    assert_eq!(world.send(read(4)), [], "cancellation frees account capacity, not the next flight's worker");
    assert_eq!(world.send(answer(FIRST, 3)), [answered(3), sent(FIRST, 4)]);
    assert_eq!(world.send(answer(FIRST, 4)), [answered(4)]);
    assert_eq!(world.send(read(5)), [sent(FIRST, 5)], "the same source can be read after cancellation");
}

#[test]
fn usage_serializes_retries_and_fences_answers_without_attempts() {
    let mut world = World::new();
    world.send(declared());
    ready(&mut world, FIRST);
    assert_eq!(world.send(read(1)), [sent(FIRST, 1)]);
    assert!(world.send(read(2)).is_empty(), "one usage read per worker");
    assert!(world.send(answer(SECOND, 1)).is_empty(), "another channel cannot answer");
    assert!(world.send(answer(FIRST, 99)).is_empty(), "an unknown read cannot answer");
    world.send(Event::Lost { channel: FIRST });
    assert!(world.send(answer(FIRST, 1)).is_empty(), "the lost channel cannot answer");
    world.send(hello(RETURNED, true));
    assert_eq!(world.send(inventory(RETURNED, LoginState::Usable)), [sent(RETURNED, 1)]);
    assert_eq!(world.send(answer(FIRST, 1)), []);
    assert_eq!(world.send(answer(RETURNED, 1)), [answered(1), sent(RETURNED, 2)]);
    assert!(world.send(answer(RETURNED, 1)).is_empty(), "a completed read is no longer held");
    assert_eq!(world.send(answer(RETURNED, 2)), [answered(2)]);
    ready(&mut world, SECOND);
    let first = world.send(read(3));
    let second = world.send(read(4));
    assert_eq!(first.len(), 1, "one eligible worker, not a broadcast");
    assert_eq!(second.len(), 1, "a second worker can read concurrently");
    let Request::ReadUsage { channel: a, .. } = first[0] else { panic!("usage read") };
    let Request::ReadUsage { channel: b, .. } = second[0] else { panic!("usage read") };
    assert_ne!(a, b);
}
