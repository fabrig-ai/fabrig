//! Renewal uses the reserved connection independently of ordinary GitHub calls.
use super::github_installation_loss::{public_ready, requests};
use skein_forge::github::request::Scope;
use skein_forge_connection::seam::Access;
use skein_forge_connection::{Event, Request, Stopped, seam::Tokens};
use skein_forge_machine_world::hand::{
    self as fixtures, CONTENTS, CONTENTS_DUE, CONTENTS_EXPIRY, EXPIRES, FORGE, FULL, FULL_DUE, Harness, LATER, LIVE,
    NEW_CONTENTS, NEW_FULL, SUSPENDED, VIEWER, accepted, advance, commit, contains, contents_renewing, mint,
    minted_pair, ready, renewed_contents, renewed_full, renewing, row, tokens,
};
use skein_lib::{Duration, Queue, Time, Wall};
use std::boxed::Box;

const SECOND: Duration = Duration::from_secs(1);
const INITIAL_REQUESTS: u32 = 4; // listing, two scopes and Viewer
const REPEATED_FIRES: u32 = 3; // one consumed timer must withstand repeated delivery
// The fixture wall stays at the epoch; thirty seconds is half renew_before.
const SHORT_EXPIRY: &[u8] = b"1970-01-01T00:00:30Z";
const NORMAL_EXPIRY: &[u8] = b"1970-01-01T00:02:00Z";
const NORMAL_DUE: Duration = Duration::from_secs(60);
const MINT_REQUEST: &[u8] = b"POST /app/installations/1/access_tokens HTTP/1.1\r\n";
fn usable(h: &Harness, value: &[u8]) {
    let row = h.component.test_github().rows.get(&1).expect("eligible row");
    assert!(row.credential(h.env.now).expect("usable Full").expose() == value);
}
fn expiry(h: &Harness, index: usize) -> Wall {
    match tokens(h).get(index).expect("Contents event") {
        Event::GitToken { expires, .. } => *expires,
        Event::Installation { .. }
        | Event::Fact { .. }
        | Event::Answered(_)
        | Event::Hint(_)
        | Event::Listening { .. }
        | Event::ListenerLost
        | Event::Connected { .. }
        | Event::Stopped { .. }
        | Event::Closed => unreachable!("token history"),
    }
}
#[test]
fn each_scope_renews_at_its_own_due_time() {
    let mut h = ready();
    let row = h.component.test_github().rows.get(&1).expect("initial row");
    assert_eq!(
        (row.full.expiry, row.full.due),
        (Time::from_nanos(3_600_000_000_000), Some(Time::from_nanos(3_540_000_000_000)))
    );
    assert_eq!(
        (row.contents.expiry, row.contents.due),
        (Time::from_nanos(4_200_000_000_000), Some(Time::from_nanos(4_140_000_000_000)))
    );
    advance(&mut h, FULL_DUE - 1);
    assert_eq!(requests(&h), INITIAL_REQUESTS);
    assert_eq!(h.connections_opened(), 1);
    advance(&mut h, FULL_DUE);
    h.connected(FORGE);
    assert_eq!(h.component.test_github().mint, Some((1, Scope::Full)));
    renewed_full(&mut h);
    assert_eq!(requests(&h), INITIAL_REQUESTS + 1);
    h.run(h.env.limits.idle);
    advance(&mut h, CONTENTS_DUE - 1);
    assert_eq!(requests(&h), INITIAL_REQUESTS + 1);
    advance(&mut h, CONTENTS_DUE);
    h.connected(FORGE);
    assert_eq!(h.component.test_github().mint, Some((1, Scope::Contents)));
    renewed_contents(&mut h);
    assert_eq!(requests(&h), INITIAL_REQUESTS + 2);
    assert_eq!(tokens(&h).len(), 2);
    assert_eq!(expiry(&h, 1), Wall::from_nanos(10_800_000_000_000));
    assert_eq!(
        h.component.test_github().rows.get(&1).expect("row").contents.due,
        Some(Time::from_nanos(14_880_000_000_000))
    );
    public_ready(&h);
}
#[test]
fn ready_time_mints_fix_both_scope_clocks() {
    let mut h = Harness::new(fixtures::github_settings());
    h.connected(FORGE);
    h.answer(FORGE, &fixtures::ok(LIVE));
    h.run(Duration::from_secs(100));
    accepted(&mut h, &[1]);
    h.run(Duration::ZERO);
    h.connected(FORGE);
    h.answer(FORGE, &mint(FULL, EXPIRES));
    h.answer(FORGE, &mint(CONTENTS, CONTENTS_EXPIRY));
    h.answer(FORGE, &fixtures::ok(VIEWER));
    h.run(Duration::ZERO);
    let row = h.component.test_github().rows.get(&1).expect("Ready-time row");
    assert_eq!(
        (row.full.expiry, row.full.due),
        (Time::from_nanos(3_700_000_000_000), Some(Time::from_nanos(3_640_000_000_000)))
    );
    assert_eq!(
        (row.contents.expiry, row.contents.due),
        (Time::from_nanos(4_300_000_000_000), Some(Time::from_nanos(4_240_000_000_000)))
    );
    public_ready(&h);
}
#[test]
fn later_wall_jumps_leave_due_and_expiry_fixed() {
    let mut h = ready();
    h.env.wall = Wall::from_nanos(u64::MAX);
    advance(&mut h, FULL_DUE - 1);
    usable(&h, FULL);
    assert_eq!(h.component.next_deadline(), Some(Time::from_nanos(3_540_000_000_000)));
    h.env.wall = Wall::EPOCH;
    advance(&mut h, FULL_DUE);
    h.connected(FORGE);
    usable(&h, FULL);
    assert_eq!(h.component.test_github().mint, Some((1, Scope::Full)));
}
#[test]
fn renewal_keeps_old_full_while_minting() {
    let h = renewing();
    usable(&h, FULL);
    public_ready(&h);
}
type MintReplies = std::vec::Vec<(u64, Box<[u8]>)>;

struct BackoffRow {
    responses: MintReplies,
    due: u64,
    credential: &'static [u8],
    deadline: Time,
}
#[test]
fn renewal_backoff_keeps_the_current_full_credential_usable() {
    skein_rows::check(
        [
            (
                "renewal_keeps_old_full_during_backoff",
                BackoffRow {
                    responses: vec![],
                    due: FULL_DUE,
                    credential: FULL,
                    deadline: Time::from_nanos(3_541_000_000_000),
                },
            ),
            (
                "contents_backoff_leaves_full_usable",
                BackoffRow {
                    responses: vec![(FULL_DUE, mint(NEW_FULL, LATER))],
                    due: CONTENTS_DUE,
                    credential: NEW_FULL,
                    deadline: Time::from_nanos(4_141_000_000_000),
                },
            ),
        ],
        |row| {
            let responses: std::vec::Vec<_> =
                row.responses.iter().map(|(at, response)| (*at, response.as_ref())).collect();
            let mut h = renewal_after(&responses, row.due);
            h.answer(FORGE, &fixtures::status(503));
            usable(&h, row.credential);
            assert_eq!(h.component.test_github().deadline(), Some(row.deadline));
            public_ready(&h);
        },
    );
}
#[test]
fn expired_full_stays_eligible_without_a_credential() {
    let mut h = renewing();
    h.answer(FORGE, &fixtures::response_with_fields(429, b"{}", b"Retry-After: 120\r\n"));
    advance(&mut h, 3600);
    let row = h.component.test_github().rows.get(&1).expect("eligible expired row");
    assert!(row.accepted && !row.suspended);
    assert!(row.credential(h.env.now).is_none());
    assert!(row.api.is_none());
    assert_eq!(h.component.next_deadline(), Some(Time::from_nanos(3_660_000_000_000)));
    h.run(Duration::ZERO);
    assert_eq!(h.component.next_deadline(), Some(Time::from_nanos(3_660_000_000_000)));
    assert_eq!(requests(&h), INITIAL_REQUESTS + 1);
    public_ready(&h);
}
#[test]
fn successful_full_replaces_old_value_and_revision() {
    let mut h = renewing();
    renewed_full(&mut h);
    usable(&h, NEW_FULL);
    let row = h.component.test_github().rows.get(&1).expect("row");
    assert_eq!(row.token_revision, 2);
    assert_eq!(
        (row.full.expiry, row.full.due),
        (Time::from_nanos(14_340_000_000_000), Some(Time::from_nanos(14_280_000_000_000)))
    );
    assert_eq!(tokens(&h).len(), 1);
    public_ready(&h);
}
#[test]
fn contents_only_renewal_keeps_full_usable() {
    let mut h = contents_renewing();
    usable(&h, NEW_FULL);
    renewed_contents(&mut h);
    usable(&h, NEW_FULL);
    assert_eq!(h.component.test_github().rows.get(&1).expect("row").token_revision, 2);
}
fn scope_renewing(scope: Scope) -> Harness {
    match scope {
        Scope::Full => renewing(),
        Scope::Contents => contents_renewing(),
    }
}
fn heard_mints(h: &Harness) -> u32 {
    skein_lib::bytes::count(&h.all_request_bytes(FORGE), MINT_REQUEST, u32::MAX)
}
fn just_before(h: &mut Harness, due: Time) {
    let before = Time::from_nanos(due.as_nanos().saturating_sub(Duration::from_millis(1).as_nanos()));
    h.run(before.saturating_since(h.env.now));
}
fn scope_clock(h: &Harness, scope: Scope) -> skein_forge_connection::seam::Clock {
    let row = h.component.test_github().rows.get(&1).expect("renewing row");
    match scope {
        Scope::Full => row.full,
        Scope::Contents => row.contents,
    }
}
fn short_mints_at_cap(scope: Scope) -> Harness {
    let mut h = scope_renewing(scope);
    for wait in [1, 2, 4, 8] {
        h.answer(FORGE, &mint(NEW_FULL, SHORT_EXPIRY));
        h.run(Duration::from_secs(wait));
        h.connected(FORGE);
    }
    h
}
#[test]
fn already_due_full_is_sent_on_viewer() {
    // Half the lead and exactly the lead both give an already-due mint.
    for expires in [SHORT_EXPIRY, b"1970-01-01T00:01:00Z"] {
        let mut h = minted_pair(expires, CONTENTS_EXPIRY);
        let wire = h.request_bytes(FORGE);
        let viewer = skein_lib::bytes::find(&wire, b"POST /graphql HTTP/1.1\r\n").expect("Viewer sent");
        assert!(contains(
            wire.get(viewer..).expect("Viewer request"),
            b"Authorization: Bearer artificial-full-api-secret\r\n"
        ));
        usable(&h, FULL);
        h.answer(FORGE, &fixtures::ok(VIEWER));
        h.run(Duration::ZERO);
        public_ready(&h);
    }
}
#[test]
fn short_mints_double_backoff_then_hold_at_cap() {
    for scope in [Scope::Full, Scope::Contents] {
        let mut h = scope_renewing(scope);
        let initial = heard_mints(&h);
        // Four cap windows, beyond the initial thirty-second token life.
        for (wait, extra) in [(1, 1), (2, 2), (4, 3), (8, 4), (8, 5), (8, 6), (8, 7)] {
            h.answer(FORGE, &mint(NEW_FULL, SHORT_EXPIRY));
            let due = h.env.now.saturating_add(Duration::from_secs(wait));
            assert_eq!(scope_clock(&h, scope).due, Some(due), "{scope:?}");
            for _ in 0..REPEATED_FIRES {
                h.run(Duration::ZERO);
            }
            just_before(&mut h, due);
            assert_eq!(heard_mints(&h), initial + extra - 1, "{scope:?}: before {wait}s");
            h.run(Duration::from_millis(1));
            h.connected(FORGE);
            assert_eq!(heard_mints(&h), initial + extra, "{scope:?}: after {wait}s");
            assert_eq!(h.component.test_github().mint, Some((1, scope)));
        }
    }
}
#[test]
fn normal_life_resets_short_mint_backoff() {
    for scope in [Scope::Full, Scope::Contents] {
        let mut h = short_mints_at_cap(scope);
        h.answer(FORGE, &mint(NEW_FULL, NORMAL_EXPIRY));
        let due = h.env.now.saturating_add(NORMAL_DUE);
        assert_eq!(scope_clock(&h, scope).due, Some(due), "{scope:?}");
        let count = heard_mints(&h);
        just_before(&mut h, due);
        assert_eq!(heard_mints(&h), count, "{scope:?}: before normal due");
        h.run(Duration::from_millis(1));
        h.connected(FORGE);
        assert_eq!(heard_mints(&h), count + 1, "{scope:?}: at normal due");
        h.answer(FORGE, &mint(NEW_FULL, SHORT_EXPIRY));
        assert_eq!(scope_clock(&h, scope).due, Some(due.saturating_add(SECOND)), "{scope:?}: reset");
    }
}
#[test]
fn short_mints_preserve_the_other_scope_clock_and_backoff() {
    for (scope, other) in [(Scope::Full, Scope::Contents), (Scope::Contents, Scope::Full)] {
        let mut h = scope_renewing(scope);
        // The other scope's four-second next wait (two doublings) survives these answers.
        h.component.test_github_mut().rows.get_mut(&1).expect("row").clock(other).next = Duration::from_secs(4);
        let before = scope_clock(&h, other);
        for wait in [1, 2] {
            h.answer(FORGE, &mint(NEW_FULL, SHORT_EXPIRY));
            let after = scope_clock(&h, other);
            assert_eq!(
                (after.expiry, after.due, after.pending, after.next),
                (before.expiry, before.due, before.pending, before.next),
                "{scope:?}"
            );
            h.run(Duration::from_secs(wait));
            h.connected(FORGE);
        }
    }
}
#[test]
fn short_mints_and_failures_share_backoff() {
    for scope in [Scope::Full, Scope::Contents] {
        let mut h = scope_renewing(scope);
        for (failed, wait) in [(true, 1), (false, 2), (true, 4), (false, 8)] {
            if failed {
                h.answer(FORGE, &fixtures::status(503));
            } else {
                h.answer(FORGE, &mint(NEW_FULL, SHORT_EXPIRY));
            }
            assert_eq!(
                h.component.test_github().deadline(),
                Some(h.env.now.saturating_add(Duration::from_secs(wait))),
                "{scope:?}"
            );
            h.run(Duration::from_secs(wait));
            h.connected(FORGE);
        }
    }
}
fn waits(mut h: Harness, label: &str) {
    h.run(Duration::ZERO);
    let due = Time::from_nanos(3_541_000_000_000);
    assert_eq!(h.component.test_github().deadline(), Some(due), "{label}");
    h.run(Duration::from_millis(999));
    assert_eq!(requests(&h), INITIAL_REQUESTS + 1, "{label}");
    h.run(Duration::from_millis(1));
    h.connected(FORGE);
    for _ in 0..REPEATED_FIRES {
        h.run(Duration::ZERO);
    }
    assert_eq!(requests(&h), INITIAL_REQUESTS + 2, "{label}");
    renewed_full(&mut h);
    assert_eq!(requests(&h), INITIAL_REQUESTS + 2, "{label}");
    public_ready(&h);
}
#[test]
fn renewal_failure_waits_before_retry() {
    for (label, status, body) in [("rate", 429, b"{}".as_slice()), ("malformed", 201, b"{")] {
        let mut h = renewing();
        h.answer(FORGE, &fixtures::response(status, body));
        waits(h, label);
    }
}
#[test]
fn renewal_transport_failure_waits_before_retry() {
    let mut h = renewing();
    h.reset(FORGE);
    waits(h, "transport");
}
#[test]
fn renewal_timeout_waits_before_retry() {
    let mut h = renewing();
    h.run(h.env.limits.deadline);
    assert_eq!(h.component.test_github().deadline(), Some(Time::from_nanos(3_551_000_000_000)));
    usable(&h, FULL);
}
#[test]
fn invalid_renewed_value_keeps_old_full_and_waits() {
    let mut h = renewing();
    h.answer(FORGE, &mint(b"", LATER));
    usable(&h, FULL);
    assert_eq!(h.component.test_github().deadline(), Some(Time::from_nanos(3_541_000_000_000)));
}
#[test]
fn renewal_backoff_doubles_to_cap() {
    let mut h = renewing();
    for (wait, until) in [(1, 3541), (2, 3543), (4, 3547), (8, 3555), (8, 3563)] {
        h.answer(FORGE, &fixtures::status(503));
        assert_eq!(h.component.test_github().deadline(), Some(Time::from_nanos(Duration::from_secs(until).as_nanos())));
        h.run(Duration::from_secs(wait));
        h.connected(FORGE);
    }
    renewed_full(&mut h);
    h.run(h.env.limits.idle);
    advance(&mut h, CONTENTS_DUE);
    h.connected(FORGE);
    h.answer(FORGE, &fixtures::status(503));
    assert_eq!(h.component.test_github().deadline(), Some(Time::from_nanos(4_141_000_000_000)));
}
#[test]
fn renewal_long_retry_after_wins() {
    let mut h = renewing();
    h.answer(FORGE, &fixtures::response_with_fields(429, b"{}", b"Retry-After: 20\r\n"));
    assert_eq!(h.component.test_github().deadline(), Some(Time::from_nanos(3_560_000_000_000)));
    advance(&mut h, 3559);
    assert_eq!(requests(&h), INITIAL_REQUESTS + 1);
    advance(&mut h, 3560);
    h.connected(FORGE);
    assert_eq!(requests(&h), INITIAL_REQUESTS + 2);
}
#[test]
fn renewal_terminal_mint_status_stops_token_refused() {
    let mut h = renewing();
    h.answer(FORGE, &fixtures::status(422));
    h.run(Duration::ZERO);
    assert_eq!(h.stopped(), Some((FORGE, Stopped::TokenRefused { installation: 1, status: 422 })));
    assert_eq!(h.connected_count, 1);
}
fn refusal_refresh(status: u16, body: &[u8]) {
    let mut h = renewing();
    h.answer(FORGE, &fixtures::status(status));
    let row = h.component.test_github().rows.get(&1).expect("refused row");
    assert!(row.api.is_none());
    assert_eq!((row.full.due, row.contents.due), (None, None));
    h.run(Duration::ZERO);
    assert_eq!(requests(&h), INITIAL_REQUESTS + 1);
    h.run(SECOND);
    h.connected(FORGE);
    h.answer(FORGE, &fixtures::ok(body));
    h.run(Duration::ZERO);
    assert_eq!(requests(&h), INITIAL_REQUESTS + 2);
    assert_eq!(tokens(&h).len(), 1);
    public_ready(&h);
}
#[test]
fn renewal_refusals_wait_before_refreshing_lost_installation_eligibility() {
    skein_rows::check(
        [
            ("renewal_403_waits_then_rereads_suspension", (403, SUSPENDED)),
            ("renewal_404_waits_then_rereads_deletion", (404, b"[]")),
        ],
        |(v0, v1)| refusal_refresh(v0, v1),
    );
}

#[test]
fn another_current_row_survives_transient_renewal_failure() {
    let mut h = ready();
    commit(&mut h, Box::from([row(1, false), row(2, false)]));
    accepted(&mut h, &[1, 2]);
    h.run(Duration::ZERO);
    h.connected(FORGE);
    h.answer(FORGE, &mint(FULL, LATER));
    h.answer(FORGE, &mint(CONTENTS, LATER));
    h.run(h.env.limits.idle);
    advance(&mut h, FULL_DUE);
    h.connected(FORGE);
    h.answer(FORGE, &fixtures::status(503));
    let row = h.component.test_github().rows.get(&2).expect("other Current");
    assert_eq!(row.state, Tokens::Current);
    assert!(row.credential(h.env.now).is_some());
    public_ready(&h);
}
#[test]
fn installation_loss_discards_results_of_already_written_renewals() {
    skein_rows::check(
        [
            ("loss_discards_a_written_renewal_result", (vec![], FULL_DUE, NEW_FULL)),
            (
                "loss_discards_a_written_contents_renewal",
                (vec![(FULL_DUE, mint(NEW_FULL, LATER))], CONTENTS_DUE, NEW_CONTENTS),
            ),
        ],
        |(responses, due, token): (MintReplies, u64, &[u8])| {
            let responses: std::vec::Vec<_> = responses.iter().map(|(at, response)| (*at, response.as_ref())).collect();
            let mut h = renewal_after(&responses, due);
            accepted(&mut h, &[]);
            h.answer(FORGE, &mint(token, LATER));
            h.run(Duration::ZERO);
            let row = h.component.test_github().rows.get(&1).expect("Gone");
            assert_eq!(row.state, Tokens::Gone);
            assert!(row.api.is_none());
            assert_eq!(tokens(&h).len(), 1);
            assert!(h.component.test_github().held.is_none());
        },
    );
}
#[derive(Clone, Copy)]
enum Stage {
    Minting,
    Waiting,
    Held,
}
fn stage(stage: Stage) -> Harness {
    let mut h = renewing();
    match stage {
        Stage::Minting => {}
        Stage::Waiting => h.answer(FORGE, &fixtures::status(503)),
        Stage::Held => {
            renewed_full(&mut h);
            h.run(h.env.limits.idle);
            advance(&mut h, CONTENTS_DUE);
            h.connected(FORGE);
            let gh = h.component.test_github_mut();
            gh.held = Some(skein_forge_connection::seam::HeldToken {
                selected: gh.selected.expect("eligible renewal"),
                token: skein_lib::Secret::new(Box::from(NEW_CONTENTS)),
                expires: Wall::from_nanos(10_800_000_000_000),
            });
        }
    }
    h
}
fn close(value: Stage) {
    let mut h = stage(value);
    h.send(Request::Close);
    h.run(Duration::ZERO);
    assert!(h.closed);
    assert!(h.component.test_github().held.is_none());
    assert!(h.component.test_github().rows.get(&1).expect("row").api.is_none());
    assert_eq!(h.component.next_deadline(), None);
    assert_eq!(tokens(&h).len(), 1);
}
fn replace(value: Stage) {
    let mut h = stage(value);
    h.send(Request::Connect {
        forge: FORGE,
        access: fixtures::app_access(skein_fake_forge_github::keys::KEYS.first().expect("public key"), 7, 9),
    });
    h.run(Duration::ZERO);
    h.connected(FORGE);
    h.answer(FORGE, &fixtures::ok(SUSPENDED));
    h.run(Duration::ZERO);
    assert_eq!(h.connected_count, 2);
    assert_eq!(tokens(&h).len(), 1);
    assert!(h.component.test_github().held.is_none());
}
#[test]
fn closing_renewal_phases_discards_tokens_and_pending_work() {
    skein_rows::check(
        [
            ("close_while_renewing_discards_work", Stage::Minting),
            ("close_during_renewal_backoff_discards_work", Stage::Waiting),
            ("close_drops_held_renewed_contents", Stage::Held),
        ],
        close,
    );
}

#[test]
fn replacing_renewal_phases_discards_tokens_and_pending_work() {
    skein_rows::check(
        [
            ("replacement_while_renewing_discards_work", Stage::Minting),
            ("replacement_during_renewal_backoff_discards_work", Stage::Waiting),
            ("replacement_drops_held_renewed_contents", Stage::Held),
        ],
        replace,
    );
}

#[test]
fn renewed_contents_follows_changes_once() {
    let mut h = stage(Stage::Held);
    let mut renamed = row(1, false);
    renamed.login = Box::from(b"new-account".as_slice());
    commit(&mut h, Box::from([renamed, row(2, false)]));
    let mut up = Queue::uncapped();
    let mut io = Queue::uncapped();
    h.component.test_local(&h.env, &mut up);
    assert_eq!(up.len(), 1);
    assert!(h.component.test_github().held.is_some());
    drop(up.pop());
    h.component.test_local(&h.env, &mut up);
    drop(up.pop());
    h.component.test_local(&h.env, &mut up);
    assert!(h.component.test_github().held.is_none());
    assert_eq!(up.len(), 1);
    match up.pop().expect("held renewal") {
        Event::GitToken { account, token, .. } => {
            assert_eq!(account.login.as_ref(), b"new-account");
            assert!(token.expose() == NEW_CONTENTS);
        }
        Event::Installation { .. }
        | Event::Fact { .. }
        | Event::Answered(_)
        | Event::Hint(_)
        | Event::Listening { .. }
        | Event::ListenerLost
        | Event::Connected { .. }
        | Event::Stopped { .. }
        | Event::Closed => unreachable!("held token"),
    }
    h.component.fire(&h.env, &mut up, &mut io);
    assert!(up.is_empty());
}
#[test]
fn loss_suppresses_held_renewed_contents() {
    let mut h = stage(Stage::Held);
    accepted(&mut h, &[]);
    h.run(Duration::ZERO);
    assert_eq!(tokens(&h).len(), 1);
    assert!(h.component.test_github().held.is_none());
}
#[test]
fn renewal_debug_redacts_old_new_and_held_tokens() {
    for value in [Stage::Minting, Stage::Waiting, Stage::Held] {
        let h = stage(value);
        let debug = std::format!("{:?} {:?} {:?}", h.component, h.up, h.io);
        for token in [FULL, CONTENTS, NEW_FULL, NEW_CONTENTS] {
            assert!(!contains(debug.as_bytes(), token));
        }
    }
}

#[test]
fn renewal_send_arms_a_fresh_request_deadline() {
    let mut h = renewing();
    h.run(Duration::from_secs(9));
    h.answer(FORGE, &fixtures::status(503));
    h.run(SECOND);
    h.connected(FORGE);
    assert_eq!(h.component.next_deadline(), Some(Time::from_nanos(3_560_000_000_000)));
}
#[test]
fn waiting_for_a_socket_consumes_the_due_alarm() {
    let mut h = ready();
    advance(&mut h, FULL_DUE);
    for _ in 0..REPEATED_FIRES {
        h.run(Duration::ZERO);
    }
    assert_eq!(h.component.next_deadline(), Some(Time::from_nanos(3_550_000_000_000)));
    assert_eq!(h.connections_opened(), 2);
    assert_eq!(requests(&h), INITIAL_REQUESTS);
}

fn renewal_after(responses: &[(u64, &[u8])], due: u64) -> Harness {
    let mut h = ready();
    for &(at, response) in responses {
        advance(&mut h, at);
        h.connected(FORGE);
        h.answer(FORGE, response);
        h.run(Duration::ZERO);
        h.run(h.env.limits.idle);
    }
    advance(&mut h, due);
    h.connected(FORGE);
    h
}
