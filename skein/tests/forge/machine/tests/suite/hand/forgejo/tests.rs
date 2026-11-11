use skein_forge::api;
use skein_forge_connection::{Answered, Event, Limits, Request, start::Start};
use skein_forge_machine_world::hand as fixtures;
use skein_lib::{Duration, Time, Token};
#[test]
fn a_call_waiting_for_its_forge_is_unavailable_at_its_deadline() {
    let mut h = fixtures::Harness::new(fixtures::settings());
    assert_eq!(h.start(fixtures::FORGE), Start::Checking { next: Duration::from_secs(1) }, "start checking");
    h.call(1, fixtures::REPO, api::Op::Read(api::Read::Settings));
    assert_eq!(h.component.next_deadline(), Some(Time::from_nanos(10_000_000_000)), "arrival plus deadline");
    h.run(Duration::from_secs(9));
    assert_eq!(h.answered(1), None, "before deadline");
    h.run(Duration::from_secs(1));
    assert_eq!(
        h.answered(1),
        Some(&Answered { call: Token::new(1), cost: 0, free: 0, rate: None, result: Err(api::Error::Unavailable) }),
        "at deadline"
    );
    h.run(Duration::from_secs(1));
    assert_eq!(h.answers.len(), 1, "answered once");
    assert_eq!(h.component.next_deadline(), Some(Time::from_nanos(21_000_000_000)), "only the new start waits");
    assert_eq!(h.io.len(), 0, "nothing sent");
}

#[test]
fn refused_answers_are_offered_once_and_returned_answers_stay_until_close() {
    let mut h = fixtures::Harness::new(fixtures::settings());
    let answer = Answered { call: Token::new(1), cost: 0, free: 0, rate: None, result: Err(api::Error::Unavailable) };
    h.send(Request::Refused(answer.clone()));
    h.send(Request::Offer);
    let Some(Event::Answered(offered)) = h.up.pop() else {
        unreachable!("one offered answer");
    };
    assert_eq!(offered, answer, "same answer offered");
    h.send(Request::Returned(offered));
    h.send(Request::Offer);
    assert_eq!(h.up.len(), 0, "returned answer never offered");
    h.send(Request::Close);
    assert_eq!(h.up.len(), 0, "connections close before the terminal");
    h.run(Duration::ZERO);
    assert!(h.closed, "closed emitted after io closed");
    h.send(Request::Offer);
    h.send(Request::Close);
    assert_eq!(h.up.len(), 0, "nothing after closed");
}

#[test]
fn closing_waiting_calls_emits_only_the_close_terminal() {
    let mut h = fixtures::Harness::new(fixtures::settings());
    h.call(1, fixtures::REPO, api::Op::Read(api::Read::Settings));
    h.send(Request::Close);
    assert_eq!(h.up.len(), 0, "close waits for io");
    h.run(Duration::ZERO);
    assert!(h.closed, "close terminal after last socket");
    h.run(Duration::from_secs(20));
    assert_eq!(h.answered(1), None, "no call answer after root stopped");
    assert_eq!(h.component.next_deadline(), None, "no live deadlines");
}

#[test]
fn all_due_calls_finish_in_the_same_timer_turn() {
    let limits =
        Limits { connections: 20, kept_bytes: 20 * u64::from(fixtures::shared().answer_bytes), ..fixtures::limits() };
    let shared = fixtures::shared();
    let shared = skein_forge_connection::Shared { calls: 20, ..shared };
    let mut h = fixtures::Harness::with(limits, shared, fixtures::settings());
    for call in 0..20 {
        h.call(call, fixtures::REPO, api::Op::Read(api::Read::Settings));
    }
    for (name, span, count, deadline) in [
        ("all due", Duration::from_secs(10), 20, Some(Time::from_nanos(11_000_000_000))),
        ("rest", Duration::ZERO, 20, Some(Time::from_nanos(11_000_000_000))),
    ] {
        h.run(span);
        assert_eq!(h.answers.len(), count, "{name}: calls ended");
        assert_eq!(h.component.next_deadline(), deadline, "{name}: earliest remaining");
    }
}

#[test]
fn one_offer_drains_all_eligible_held_answers() {
    let limits =
        Limits { connections: 20, kept_bytes: 20 * u64::from(fixtures::shared().answer_bytes), ..fixtures::limits() };
    let shared = fixtures::shared();
    let shared = skein_forge_connection::Shared { calls: 20, ..shared };
    let mut h = fixtures::Harness::with(limits, shared, fixtures::settings());
    for call in 0..20 {
        h.send(Request::Refused(Answered {
            call: Token::new(call),
            cost: 0,
            free: 0,
            rate: None,
            result: Err(api::Error::Unavailable),
        }));
    }
    for (name, count) in [("all held", 20), ("empty", 0)] {
        h.send(Request::Offer);
        assert_eq!(h.up.len(), count, "{name}: offer batch");
        h.up.clear();
    }
}

#[test]
fn a_github_forge_without_access_is_inert() {
    let mut settings = fixtures::github_settings();
    settings.forges.get_mut(0).expect("forge").access = None;
    let mut h = fixtures::Harness::new(settings);
    h.run(Duration::from_secs(20));
    assert_eq!(h.start(fixtures::FORGE), Start::Unconnected);
    assert_eq!(h.connections_opened(), 0);
}
