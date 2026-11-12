//! Connection machinery and each forge's start.
use skein_forge_connection::seam::{Access, ConnectionAccess};
use skein_forge_machine_world::hand as fixtures;
use skein_lib::{Duration, Time};
use std::boxed::Box;

#[test]
fn the_start_reads_user_then_settings_on_one_connection() {
    let mut h = fixtures::Harness::new(fixtures::settings());
    h.connected(fixtures::FORGE);
    h.answer(fixtures::FORGE, &fixtures::user_ok());
    h.answer(fixtures::FORGE, &fixtures::settings_ok());
    h.run(Duration::from_millis(10));
    assert_eq!(
        h.request_lines(fixtures::FORGE).as_slice(),
        [Box::from(&b"GET /api/v1/user HTTP/1.1"[..]), Box::from(&b"GET /api/v1/settings/api HTTP/1.1"[..])],
        "start request order"
    );
    assert_eq!(h.start(fixtures::FORGE), skein_forge_connection::start::Start::Ready { cap: 2 }, "page cap");
    assert_eq!(h.connections_opened(), 1, "one reused connection");
    assert_eq!(h.stopped(), None, "start accepted");
}

#[test]
fn a_refused_token_emits_stopped_once_naming_the_forge() {
    let mut h = fixtures::Harness::new(fixtures::settings());
    h.connected(fixtures::FORGE);
    h.answer(fixtures::FORGE, &fixtures::status(401));
    h.run(Duration::ZERO);
    assert_eq!(
        h.stopped(),
        Some((
            1,
            skein_forge_connection::Stopped::Refused { read: skein_forge_connection::StartRead::User, status: 401 }
        )),
        "refused forge"
    );
    h.run(Duration::from_secs(20));
    assert_eq!(h.connections_opened(), 1, "stopped never restarted");
}

#[test]
fn an_idle_connection_closes_at_idle() {
    let mut h = fixtures::Harness::ready();
    assert_eq!(h.component.next_deadline(), Some(Time::from_nanos(5_000_000_000)), "fixture idle");
    h.run(Duration::from_secs(4));
    assert_eq!(h.connections_opened(), 1, "idle before deadline");
    h.run(Duration::from_secs(1));
    assert_eq!(h.component.next_deadline(), None, "idle connection ended");
    h.send(skein_forge_connection::Request::Close);
    h.run(Duration::ZERO);
    assert!(h.closed, "component no longer holds the socket");
}

#[test]
fn a_start_read_unanswered_at_its_deadline_waits_and_asks_again() {
    let mut h = fixtures::Harness::new(fixtures::settings());
    h.connected(fixtures::FORGE);
    assert_eq!(
        h.request_lines(fixtures::FORGE).as_slice(),
        [Box::from(&b"GET /api/v1/user HTTP/1.1"[..])],
        "user sent"
    );
    h.run(Duration::from_secs(10));
    assert_eq!(
        h.start(fixtures::FORGE),
        skein_forge_connection::start::Start::Waiting {
            until: Time::from_nanos(11_000_000_000),
            next: Duration::from_secs(2)
        },
        "read deadline waits"
    );
    h.run(Duration::from_millis(999));
    assert_eq!(h.connections_opened(), 1, "not before backoff");
    h.run(Duration::from_millis(1));
    h.connected(fixtures::FORGE);
    assert_eq!(h.connections_opened(), 2, "timed out connection replaced");
    assert_eq!(
        h.request_lines(fixtures::FORGE).as_slice(),
        [Box::from(&b"GET /api/v1/user HTTP/1.1"[..])],
        "new user read"
    );
}

#[test]
fn each_start_read_keeps_its_deadline_through_connect_and_starts_afresh() {
    for (name, before_connect, answer_user, expected) in [
        ("connect uses holder", Duration::from_secs(9), false, Time::from_nanos(10_000_000_000)),
        ("settings gets fresh deadline", Duration::from_secs(5), true, Time::from_nanos(15_000_000_000)),
    ] {
        let mut h = fixtures::Harness::new(fixtures::settings());
        h.run(before_connect);
        h.connected(fixtures::FORGE);
        if answer_user {
            h.answer(fixtures::FORGE, &fixtures::user_ok());
        }
        assert_eq!(h.component.next_deadline(), Some(expected), "{name}");
    }
}

#[test]
fn a_reset_while_the_start_reads_waits_before_a_new_connection() {
    let mut h = fixtures::Harness::new(fixtures::settings());
    h.connected(fixtures::FORGE);
    h.reset(fixtures::FORGE);
    assert_eq!(
        h.start(fixtures::FORGE),
        skein_forge_connection::start::Start::Waiting {
            until: Time::from_nanos(1_000_000_000),
            next: Duration::from_secs(2)
        },
        "reset backs off"
    );
    h.run(Duration::from_secs(1));
    assert_eq!(h.connections_opened(), 2, "new connect at backoff");
}

#[test]
fn start_retry_after_wins_over_the_backoff_cap() {
    use skein_forge::api;
    use skein_forge_connection::{Answered, start::Start};
    let limits = fixtures::limits();
    let waiting = Start::Waiting { until: Time::from_nanos(21_000_000_000), next: Duration::from_secs(2) };
    let mut h = fixtures::Harness::new(fixtures::settings());
    h.run(Duration::from_secs(1));
    h.connected(fixtures::FORGE);
    h.answer(fixtures::FORGE, b"HTTP/1.1 429 Refused\r\nContent-Length: 0\r\nRetry-After: 20\r\n\r\n");
    assert_eq!(h.start(fixtures::FORGE), waiting);
    h.call(1, fixtures::REPO, api::Op::Read(api::Read::Settings));
    h.run(Duration::from_nanos(9_999_999_999));
    assert_eq!((h.answered(1), h.connections_opened()), (None, 1));
    h.run(Duration::from_nanos(1));
    assert_eq!(
        h.answered(1),
        Some(&Answered {
            call: skein_lib::Token::new(1),
            cost: 0,
            free: 0,
            rate: None,
            result: Err(api::Error::Unavailable)
        })
    );
    assert_eq!(h.answers.len(), 1);
    h.run(Duration::from_nanos(9_999_999_999));
    assert_eq!(h.connections_opened(), 1);
    h.run(Duration::from_nanos(1));
    assert_eq!(h.connections_opened(), 2);
    retry_after_states(&limits, waiting);
}
fn retry_after_states(limits: &skein_forge_connection::Limits, waiting: skein_forge_connection::start::Start) {
    use skein_forge::forgejo::plans::start::Heard as StartHeard;
    use skein_forge_connection::start::{self, Start};
    for state in [Start::Checking { next: limits.start_backoff }, Start::Settings { next: limits.start_backoff }] {
        for (now, expected) in [
            (Time::from_nanos(1_000_000_000), waiting),
            (
                Time::from_nanos(u64::MAX - 1),
                Start::Waiting { until: Time::from_nanos(u64::MAX), next: Duration::from_secs(2) },
            ),
        ] {
            assert_eq!(
                start::heard(
                    state,
                    StartHeard::Status { status: 429, retry: Some(Duration::from_secs(20)) },
                    fixtures::WRITER,
                    now,
                    limits
                ),
                expected,
                "{state:?}/{now:?}"
            );
        }
    }
}

#[test]
fn an_unconfigured_forge_is_unavailable_at_zero_cost() {
    use skein_forge::api;
    let mut settings = fixtures::settings();
    settings.forges = skein_lib::List::with_capacity(0);
    let mut h = fixtures::Harness::new(settings);
    h.call(1, api::Repository { forge: 99, repository: 1 }, api::Op::Read(api::Read::Settings));
    h.run(Duration::ZERO);
    assert_eq!(
        h.answered(1),
        Some(&skein_forge_connection::Answered {
            call: skein_lib::Token::new(1),
            cost: 0,
            free: 0,
            rate: None,
            result: Err(api::Error::Unavailable)
        })
    );
    assert_eq!((h.answers.len(), h.connections_opened()), (1, 0));
}
#[test]
fn a_stopped_forge_does_not_stop_another() {
    use skein_forge::api;
    let mut h = fixtures::Harness::new(fixtures::two_forges());
    h.connected(fixtures::FORGE);
    h.answer(fixtures::FORGE, &fixtures::ok(b"{\"id\":1,\"login\":\"other\"}"));
    h.connected(fixtures::OTHER_FORGE);
    h.answer(fixtures::OTHER_FORGE, &fixtures::user_ok());
    h.answer(fixtures::OTHER_FORGE, &fixtures::settings_ok());
    h.call(1, fixtures::OTHER_REPO, api::Op::Read(api::Read::Branch { branch: Box::from(fixtures::BRANCH) }));
    fixtures::branch_answers(&mut h, fixtures::OTHER_FORGE);
    assert_eq!(h.stopped(), Some((fixtures::FORGE, skein_forge_connection::Stopped::OtherUser { user: 1 })));
    assert_eq!(h.start(fixtures::OTHER_FORGE), skein_forge_connection::start::Start::Ready { cap: 2 });
    assert_eq!(
        h.answered(1),
        Some(&skein_forge_connection::Answered {
            call: skein_lib::Token::new(1),
            cost: 2,
            free: 0,
            rate: None,
            result: Ok(api::Answer::Commit(fixtures::BRANCH_HEAD)),
        })
    );
    assert_eq!(h.answers.len(), 1);
    h.run(Duration::from_secs(20));
    assert_eq!(h.stopped(), Some((fixtures::FORGE, skein_forge_connection::Stopped::OtherUser { user: 1 })));
}
use skein_forge_connection::seam::Phase;

#[test]
fn an_unheld_connect_keeps_the_request_deadline_in_idle() {
    let mut h = fixtures::Harness::ready();
    h.component.test_open_call(&h.env, &mut h.io);
    h.run(Duration::ZERO);
    h.connected(fixtures::FORGE);
    assert_eq!(h.component.test_last_connection().phase(), Phase::Idle);
    assert_eq!(h.component.test_last_connection().test_deadline(), Some(Time::from_nanos(10_000_000_000)));
    h.run(Duration::from_secs(5));
    assert_eq!(h.connections_for(fixtures::FORGE), 1, "unheld connect outlives the idle clock");
    h.run(Duration::from_secs(5));
    assert_eq!(h.connections_for(fixtures::FORGE), 0, "unheld connect ends at the request clock");
}

#[test]
fn an_idle_connection_without_a_deadline_closes_now() {
    let mut h = fixtures::Harness::ready();
    h.component.test_last_connection().test_until(None);
    h.run(Duration::ZERO);
    assert_eq!(h.connections_for(fixtures::FORGE), 0, "None is already due");
}

#[test]
fn a_completed_call_releases_its_connection_on_the_idle_clock() {
    use skein_forge::api;
    let mut h = fixtures::known();
    h.call(1, fixtures::REPO, api::Op::Write(api::Write::DeleteBranch { branch: Box::from(b"work".as_slice()) }));
    h.run(Duration::ZERO);
    h.run(Duration::from_secs(2));
    h.answer(fixtures::FORGE, b"HTTP/1.1 204 No Content\r\n\r\n");
    h.run(Duration::ZERO);
    assert_eq!(h.answered(1).expect("call completed").result, Ok(api::Answer::Done));
    assert_eq!(h.component.test_last_connection().phase(), Phase::Idle);
    assert_eq!(h.component.test_last_connection().test_deadline(), Some(Time::from_nanos(7_000_000_000)));
}
