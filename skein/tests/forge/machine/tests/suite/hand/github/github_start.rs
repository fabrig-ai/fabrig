//! GitHub start transcripts; the only credentials here are artificial fixture values.
use skein_forge_connection::{Event, Request, StartRead, Stopped, start::Start};
use skein_forge_machine_world::hand::{
    self as fixtures, CONTENTS, EXPIRES, EXPIRY, FORGE, FULL, Harness, LIVE, NEXT_PAGE, SUSPENDED, VIEWER, accepted,
    contains, contents, full, mint, tokens,
};
use skein_lib::{Duration, Time};
use std::boxed::Box;

const SECOND: Duration = Duration::from_secs(1);
const PAGE_ONE: &[u8] = br#"[{"id":3,"account":{"id":24,"login":"other","type":"User"},"created_at":"1970-01-01T00:00:01Z","suspended_at":"1970-01-01T00:00:02Z"},{"id":1,"account":{"id":22,"login":"acme","type":"Organization"},"created_at":"1970-01-01T00:00:01Z","suspended_at":null}]"#;
const PAGE_TWO: &[u8] = br#"[{"id":2,"account":{"id":23,"login":"stray","type":"User"},"created_at":"1970-01-01T00:00:01Z","suspended_at":null}]"#;
// Default token_bytes is 64; its name bound is 128. Exercise them independently.
const TOKEN_BOUND: usize = 64;
// Below the 20-byte timestamp, to prove the mint decoder reserves its room independently.
const SMALL_TOKEN_BOUND: u32 = 8;
// Two listing pages, two mint scopes, then Viewer.
const START_REQUESTS: u32 = 5;

#[derive(Clone, Copy, Debug)]
enum Stage {
    Listing,
    Full,
    Contents,
    Identity,
}
fn harness() -> Harness {
    let mut h = Harness::new(fixtures::github_settings());
    accepted(&mut h, &[1]);
    h.connected(FORGE);
    h
}
fn listed(h: &mut Harness) {
    h.answer(FORGE, &fixtures::ok(LIVE));
}
fn at(stage: Stage) -> Harness {
    let mut h = harness();
    match stage {
        Stage::Listing => {}
        Stage::Full => listed(&mut h),
        Stage::Contents => {
            listed(&mut h);
            full(&mut h);
        }
        Stage::Identity => {
            listed(&mut h);
            full(&mut h);
            contents(&mut h);
        }
    }
    h
}
fn waiting(h: &Harness, until: u64, next: u64) {
    assert_eq!(
        h.start(FORGE),
        Start::Waiting {
            until: Time::ZERO.saturating_add(Duration::from_secs(until)),
            next: Duration::from_secs(next)
        }
    );
}

fn request_content_length(head: &[u8]) -> usize {
    const PREFIX: &[u8] = b"\r\nContent-Length: ";
    let Some(at) = skein_lib::bytes::find(head, PREFIX) else {
        return 0;
    };
    let rest = head.get(at.checked_add(PREFIX.len()).expect("header start")..).expect("header value");
    let end = skein_lib::bytes::find(rest, b"\r\n").expect("header end");
    let length = skein_lib::bytes::decimal(rest.get(..end).expect("length digits")).expect("decimal length");
    usize::try_from(length).expect("fixture length")
}

#[test]
fn github_starts_list_mint_both_scopes_then_check_identity() {
    let mut h = harness();
    accepted(&mut h, &[1, 3]);
    h.answer(FORGE, &fixtures::ok(b"[]")); // Finish the prior generation's written page.
    h.answer(FORGE, &fixtures::response_with_fields(200, PAGE_ONE, NEXT_PAGE));
    h.answer(FORGE, &fixtures::ok(PAGE_TWO));
    full(&mut h);
    contents(&mut h);
    h.answer(FORGE, &fixtures::ok(VIEWER));
    let wire = h.request_bytes(FORGE);
    let head_end =
        skein_lib::bytes::find(&wire, b"\r\n\r\n").expect("discarded read head").checked_add(4).expect("head framing");
    let mut rest = wire.get(head_end..).expect("new generation's requests");
    let mut requests = skein_lib::List::with_capacity(START_REQUESTS);
    let mut lines = skein_lib::List::with_capacity(START_REQUESTS);
    for _ in 0..START_REQUESTS {
        let head_end = skein_lib::bytes::find(rest, b"\r\n\r\n")
            .expect("complete request head")
            .checked_add(4)
            .expect("head framing");
        let head = rest.get(..head_end).expect("head");
        let line_end = skein_lib::bytes::find(head, b"\r\n").expect("request line");
        let length = request_content_length(head);
        let body_end = head_end.checked_add(length).expect("request length");
        let body = rest.get(head_end..body_end).expect("complete request body");
        let line = head.get(..line_end).expect("request line");
        lines.push(line).expect("five request lines");
        requests.push((line, head, body)).expect("five requests");
        rest = rest.get(body_end..).expect("next request");
    }
    assert!(rest.is_empty(), "no further request");
    assert_eq!(
        lines.as_slice(),
        [
            b"GET /app/installations?per_page=100&page=1 HTTP/1.1".as_slice(),
            b"GET /app/installations?per_page=100&page=2 HTTP/1.1",
            b"POST /app/installations/1/access_tokens HTTP/1.1",
            b"POST /app/installations/1/access_tokens HTTP/1.1",
            b"POST /graphql HTTP/1.1"
        ]
    );
    let (_, full_head, full_body) = requests.get(2).expect("Full");
    assert!(full_body.is_empty(), "Full has no body");
    assert!(!contains(full_head, b"Content-Type:"));
    let (_, _, contents_body) = requests.get(3).expect("Contents");
    assert_eq!(*contents_body, b"{\"permissions\":{\"contents\":\"write\"}}");
    let (_, viewer_head, _) = requests.get(4).expect("Viewer");
    let authorization =
        skein_forge::github::credential::bearer(&skein_lib::Secret::new(Box::from(FULL))).expect("Full bearer");
    assert!(fixtures::header(viewer_head, b"Authorization: ") == authorization.expose(), "Viewer uses Full");
    h.run(Duration::ZERO);
    assert_eq!(h.start(FORGE), Start::Ready { cap: 100 });
    assert_eq!(h.connected_count, 1);
    assert_eq!(tokens(&h).len(), 1);
    match tokens(&h).first().expect("git token") {
        Event::GitToken { forge, installation, account, token, expires } => {
            assert_eq!(
                (*forge, *installation, account.id, account.login.as_ref(), *expires),
                (FORGE, 1, 22, b"acme".as_slice(), EXPIRY)
            );
            assert!(token.expose() == CONTENTS, "only Contents goes up");
        }
        Event::Installation { .. }
        | Event::Fact { .. }
        | Event::Answered(_)
        | Event::Hint(_)
        | Event::Listening { .. }
        | Event::ListenerLost
        | Event::Connected { .. }
        | Event::Stopped { .. }
        | Event::Closed => unreachable!("token collection"),
    }
}

#[test]
fn github_with_no_accepted_installation_is_ready_without_identity() {
    let mut h = harness();
    accepted(&mut h, &[]);
    h.answer(FORGE, &fixtures::ok(b"[]")); // The changed set discards the written page.
    listed(&mut h);
    h.run(Duration::ZERO);
    assert_eq!(h.start(FORGE), Start::Ready { cap: 100 });
    assert_eq!(h.connected_count, 1);
    assert!(tokens(&h).is_empty());
    assert!(!contains(&h.request_bytes(FORGE), b"POST "));
}
#[test]
fn github_either_mint_401_stops_with_key_refused() {
    for stage in [Stage::Full, Stage::Contents] {
        let mut h = at(stage);
        h.answer(FORGE, &fixtures::status(401));
        h.run(Duration::ZERO);
        assert_eq!(h.stopped(), Some((FORGE, Stopped::KeyRefused)), "{stage:?}");
        assert!(tokens(&h).is_empty());
    }
}
#[test]
fn mint_access_refusals_wait_before_reading_installations_again() {
    skein_rows::check(
        [
            ("github_mint_403_or_404_reads_installations_again", 403),
            ("github_mint_404_reads_installations_again", 404),
        ],
        mint_reread,
    );
}

fn mint_reread(status: u16) {
    let mut h = at(Stage::Full);
    h.answer(FORGE, &fixtures::status(status));
    waiting(&h, 1, 2);
    h.run(Duration::from_millis(999));
    assert_eq!(h.connections_opened(), 1);
    h.run(Duration::from_millis(1));
    assert_eq!(h.start(FORGE), Start::Listing { next: Duration::from_secs(2) });
    h.connected(FORGE);
    listed(&mut h);
    assert!(contains(&h.request_bytes(FORGE), b"POST /app/installations/1/access_tokens"));
    assert_eq!(h.start(FORGE), Start::Minting { next: Duration::from_secs(2) });
}
#[test]
fn fresh_installation_listings_suppress_minting_for_ineligible_rows() {
    skein_rows::check(
        [
            ("github_mint_refusal_fresh_table_removes_or_suspends_the_row", (404, b"[]".as_slice())),
            ("github_mint_refusal_fresh_table_suspends_the_row", (403, SUSPENDED)),
        ],
        |(v0, v1)| fresh_table(v0, v1),
    );
}

fn fresh_table(status: u16, table: &[u8]) {
    let mut h = at(Stage::Contents);
    h.answer(FORGE, &fixtures::status(status));
    h.run(SECOND);
    h.connected(FORGE);
    h.answer(FORGE, &fixtures::ok(table));
    h.run(Duration::ZERO);
    assert_eq!(h.start(FORGE), Start::Ready { cap: 100 });
    assert!(!contains(&h.request_bytes(FORGE), b"POST "));
    assert!(tokens(&h).is_empty());
}
#[test]
fn github_accepted_ids_past_the_bound_are_ignored() {
    let limits = skein_forge_connection::Limits { installations: 2, ..fixtures::limits() };
    let mut h = Harness::with(limits, fixtures::shared(), fixtures::github_settings());
    accepted(&mut h, &[0, 1, 1, 2, 3]);
    h.connected(FORGE);
    // Row 1 is absent: 2 remains accepted, and 3 was past the accepted-set bound.
    h.answer(FORGE, &fixtures::response_with_fields(200, PAGE_TWO, NEXT_PAGE));
    let live_third = br#"[{"id":3,"account":{"id":24,"login":"other","type":"User"},"created_at":"1970-01-01T00:00:01Z","suspended_at":null}]"#;
    h.answer(FORGE, &fixtures::ok(live_third));
    full(&mut h);
    contents(&mut h);
    h.answer(FORGE, &fixtures::ok(VIEWER));
    assert!(!contains(&h.request_bytes(FORGE), b"/app/installations/3/access_tokens"));
    h.run(Duration::ZERO);
    assert_eq!(tokens(&h).len(), 1);
    assert_eq!(h.start(FORGE), Start::Ready { cap: 100 });
}
#[test]
fn nontransient_read_refusals_report_their_stage_and_status() {
    skein_rows::check(
        [
            ("github_nontransient_start_refusals_name_the_read_or_installation", (Stage::Listing, StartRead::Installations, &[403, 422])),
            ("github_nontransient_viewer_refusals_name_the_read", (Stage::Identity, StartRead::Viewer, &[401, 422])),
        ],
        |(v0, v1, v2)| refused_read(v0, v1, v2),
    );
}

fn refused_read(stage: Stage, read: StartRead, statuses: &[u16]) {
    for status in statuses {
        let mut h = at(stage);
        h.answer(FORGE, &fixtures::status(*status));
        h.run(Duration::ZERO);
        assert_eq!(h.stopped(), Some((FORGE, Stopped::Refused { read, status: *status })), "{stage:?}/{status}");
    }
}

#[test]
fn github_nontransient_mint_refusals_name_the_installation() {
    for (stage, status) in [(Stage::Full, 400), (Stage::Contents, 422)] {
        let mut h = at(stage);
        h.answer(FORGE, &fixtures::status(status));
        h.run(Duration::ZERO);
        assert_eq!(h.stopped(), Some((FORGE, Stopped::TokenRefused { installation: 1, status })), "{stage:?}");
    }
}
#[test]
fn github_other_bot_user_stops_the_forge() {
    let mut h = at(Stage::Identity);
    h.answer(FORGE, &fixtures::ok(br#"{"data":{"viewer":{"databaseId":8,"login":"other"}}}"#));
    h.run(Duration::ZERO);
    assert_eq!(h.stopped(), Some((FORGE, Stopped::OtherUser { user: 8 })));
    assert_eq!(h.connected_count, 0);
}
const TRANSPORT_OR_RATE: &[(&str, u16, &[u8])] = &[("transport", 0, b""), ("429", 429, b"{}")];
const SERVER_OR_MALFORMED: &[(&str, u16, &[u8])] = &[("5xx", 503, b"{}"), ("malformed", 200, b"{")];
const MINT_SERVER_OR_MALFORMED: &[(&str, u16, &[u8])] = &[("5xx", 503, b"{}"), ("malformed", 201, b"{")];
fn transient(stage: Stage, cases: &[(&str, u16, &[u8])]) {
    for &(label, status, body) in cases {
        let mut h = at(stage);
        if status == 0 {
            h.reset(FORGE);
        } else {
            h.answer(FORGE, &fixtures::response(status, body));
        }
        waiting(&h, 1, 2);
        assert_eq!(h.stopped(), None, "{stage:?}/{label}");
        h.run(SECOND);
        assert_eq!(h.start(FORGE), Start::Listing { next: Duration::from_secs(2) }, "{stage:?}/{label}");
    }
}
#[test]
fn transient_start_failures_back_off_and_restart_listing() {
    skein_rows::check(
        [

            ("github_start_server_or_malformed_failures_back_off_to_listing", (Stage::Listing, SERVER_OR_MALFORMED)),

            ("github_full_mint_server_or_malformed_failures_back_off_to_listing", (Stage::Full, MINT_SERVER_OR_MALFORMED)),

            ("github_contents_mint_server_or_malformed_failures_back_off_to_listing", (Stage::Contents, MINT_SERVER_OR_MALFORMED)),

            ("github_identity_server_or_malformed_failures_back_off_to_listing", (Stage::Identity, SERVER_OR_MALFORMED)),
        ],
        |(stage, cases)| {
            for &(label, status, body) in cases {
                let mut h = at(stage);
                h.answer(FORGE, &fixtures::response(status, body));
                waiting(&h, 1, 2);
                assert_eq!(h.stopped(), None, "{stage:?}/{label}");
                h.run(SECOND);
                assert_eq!(h.start(FORGE), Start::Listing { next: Duration::from_secs(2) }, "{stage:?}/{label}");
            }
        },
    );
}







#[test]
fn github_wait_doubles_without_reset_at_a_successful_page() {
    let mut h = harness();
    for (until, next, wait) in [(1, 2, 1), (3, 4, 2), (7, 8, 4), (15, 8, 8)] {
        h.answer(FORGE, &fixtures::response_with_fields(200, LIVE, NEXT_PAGE));
        h.answer(FORGE, &fixtures::status(503));
        waiting(&h, until, next);
        h.run(Duration::from_secs(wait));
        h.connected(FORGE);
    }
}
#[test]
fn github_retry_after_exceeds_the_backoff_max_and_expires_waiting_calls() {
    let mut h = harness();
    h.answer(FORGE, &fixtures::response_with_fields(429, b"{}", b"Retry-After: 20\r\n"));
    waiting(&h, 20, 2);
    h.call(1, fixtures::REPO, skein_forge::api::Op::Read(skein_forge::api::Read::Settings));
    h.run(Duration::from_secs(10));
    assert_eq!(h.answered(1).expect("call deadline").result, Err(skein_forge::api::Error::Unavailable));
    assert_eq!(h.answered(1).expect("call deadline").cost, 0);
    assert_eq!(h.connections_opened(), 1);
    h.run(Duration::from_secs(10));
    assert_eq!(h.connections_opened(), 2);
}
#[test]
fn github_each_send_has_a_fresh_deadline() {
    let mut h = harness();
    for (response, deadline) in [
        (fixtures::response_with_fields(200, LIVE, NEXT_PAGE), 19),
        (fixtures::ok(b"[]"), 28),
        (mint(FULL, EXPIRES), 37),
        (mint(CONTENTS, EXPIRES), 46),
    ] {
        h.run(Duration::from_secs(9));
        h.answer(FORGE, &response);
        assert_eq!(h.component.next_deadline(), Some(Time::ZERO.saturating_add(Duration::from_secs(deadline))));
    }
    h.run(Duration::from_secs(10));
    waiting(&h, 47, 2);
}
#[test]
fn invalid_minted_tokens_publish_nothing_and_trigger_backoff() {
    skein_rows::check(
        [
            ("github_mint_tokens_must_be_safe_nonempty_bounded_and_future: empty", (b"".as_slice(), EXPIRES)),
            ("github_mint_tokens_must_be_safe_nonempty_bounded_and_future: unsafe", (b"bad\\nvalue".as_slice(), EXPIRES)),
            ("github_expired_mint_token_is_malformed", (FULL, b"1970-01-01T00:00:00Z".as_slice())),
        ],
        |(token, expires)| {
            let mut h = at(Stage::Full);
            h.answer(FORGE, &mint(token, expires));
            waiting(&h, 1, 2);
            h.run(Duration::ZERO);
            assert!(tokens(&h).is_empty());
        },
    );
}

#[test]
fn github_mint_token_at_the_bound_is_accepted() {
    let mut h = at(Stage::Full);
    h.answer(FORGE, &mint(&fixtures::filled(TOKEN_BOUND, b'x'), EXPIRES));
    assert_eq!(h.start(FORGE), Start::Minting { next: SECOND });
    assert!(contains(&h.request_bytes(FORGE), b"{\"permissions\":{\"contents\":\"write\"}}"));
}
#[test]
fn github_mint_small_token_at_the_bound_is_kept() {
    let mut h = Harness::with(
        skein_forge_connection::Limits { token_bytes: SMALL_TOKEN_BOUND, ..fixtures::limits() },
        fixtures::shared(),
        fixtures::github_settings(),
    );
    accepted(&mut h, &[1]);
    h.connected(FORGE);
    listed(&mut h);
    let token = fixtures::filled(usize::try_from(SMALL_TOKEN_BOUND).expect("bound"), b'x');
    let response = mint(&token, EXPIRES);
    h.answer(FORGE, &response);
    h.answer(FORGE, &response);
    h.answer(FORGE, &fixtures::ok(VIEWER));
    h.run(Duration::ZERO);
    assert_eq!(h.start(FORGE), Start::Ready { cap: 100 });
    assert_eq!(tokens(&h).len(), 1);
    assert!(contains(&h.request_bytes(FORGE), b"Authorization: Bearer xxxxxxxx\r\n"), "Full is kept for Viewer");
}
#[test]
fn github_name_bound_is_independent_of_shared_rows() {
    let mut h = Harness::with(
        fixtures::limits(),
        skein_forge_connection::Shared { rows: 1, ..fixtures::shared() },
        fixtures::github_settings(),
    );
    accepted(&mut h, &[]);
    h.connected(FORGE);
    h.answer(FORGE, &fixtures::response_with_fields(200, PAGE_ONE, NEXT_PAGE));
    h.answer(FORGE, &fixtures::ok(PAGE_TWO));
    assert_eq!(h.start(FORGE), Start::Ready { cap: 100 });
}
#[test]
fn github_accepted_set_changes_discard_an_inflight_mint() {
    for stage in [Stage::Full, Stage::Contents] {
        let mut h = at(stage);
        accepted(&mut h, &[]);
        h.answer(FORGE, &mint(CONTENTS, EXPIRES));
        h.run(Duration::ZERO);
        assert!(tokens(&h).is_empty(), "{stage:?}");
        assert_eq!(h.start(FORGE), Start::Ready { cap: 100 });
        assert_eq!(h.connected_count, 1);
    }
}
#[test]
fn github_accepted_set_changes_during_listing_select_the_new_set() {
    let mut h = harness();
    h.answer(FORGE, &fixtures::response_with_fields(200, LIVE, NEXT_PAGE));
    accepted(&mut h, &[2]);
    h.answer(FORGE, &fixtures::ok(PAGE_TWO));
    h.answer(FORGE, &fixtures::ok(PAGE_TWO));
    assert!(contains(&h.request_bytes(FORGE), b"POST /app/installations/2/access_tokens"));
    assert!(!contains(&h.request_bytes(FORGE), b"POST /app/installations/1/access_tokens"));
}
#[test]
fn github_removed_inflight_mint_refusal_is_discarded() {
    let mut h = at(Stage::Full);
    accepted(&mut h, &[]);
    h.answer(FORGE, &fixtures::status(422));
    h.run(Duration::ZERO);
    assert_eq!(h.start(FORGE), Start::Ready { cap: 100 });
    assert_eq!(h.stopped(), None);
    assert!(tokens(&h).is_empty());
}

#[test]
fn github_removed_pending_mint_is_never_written() {
    let mut h = harness();
    h.answer(FORGE, &fixtures::response_with_fields(200, LIVE, b"Connection: close\r\n"));
    // The mint is pending on its replacement connection, before any request is written.
    accepted(&mut h, &[]);
    h.connected(FORGE);
    h.run(Duration::ZERO);
    assert_eq!(h.start(FORGE), Start::Ready { cap: 100 });
    assert!(!contains(&h.request_bytes(FORGE), b"POST "));
    assert!(tokens(&h).is_empty());
}

#[test]
fn github_own_connection_is_reserved_while_forgejo_calls_run() {
    let mut settings = fixtures::two_forges();
    let github = settings.forges.get_mut(1).expect("second forge");
    github.kind = skein_forge_connection::settings::Kind::GitHub;
    github.access = fixtures::github_settings().forges.into_boxed().into_iter().next().expect("GitHub forge").access;
    let mut h = Harness::new(settings);
    h.connected(FORGE);
    h.answer(FORGE, &fixtures::user_ok());
    h.answer(FORGE, &fixtures::settings_ok());
    h.connected(fixtures::OTHER_FORGE);
    h.answer(fixtures::OTHER_FORGE, &fixtures::ok(b"[]"));
    h.call(1, fixtures::REPO, skein_forge::api::Op::Read(skein_forge::api::Read::Settings));
    h.call(2, fixtures::REPO, skein_forge::api::Op::Read(skein_forge::api::Read::Settings));
    h.connected(FORGE);
    assert_eq!(h.connections_for(FORGE), 2);
    assert_eq!(h.connections_for(fixtures::OTHER_FORGE), 1);
    assert!(h.answered(1).is_none());
    assert!(h.answered(2).is_none());
    assert!(!contains(&h.request_bytes(fixtures::OTHER_FORGE), b"/api/v1"));
}

#[test]
fn replacement_discards_old_startup_work_across_every_phase() {
    skein_rows::check(
        [
            ("github_replacement_discards_the_old_start_and_tokens", (Stage::Listing, 0)),
            ("github_replacement_discards_the_old_full_mint", (Stage::Full, 0)),
            ("github_replacement_discards_the_old_contents_mint", (Stage::Contents, 0)),
            ("github_replacement_discards_the_old_identity", (Stage::Identity, 1)),
        ],
        |(stage, count)| replacement(stage, count),
    );
}



fn replacement(stage: Stage, expected_tokens: usize) {
    let mut h = at(stage);
    let owner = *h.peers.first().expect("old peer").0;
    h.send(Request::Connect {
        forge: FORGE,
        access: fixtures::app_access(skein_fake_forge_github::keys::KEYS.first().expect("public key"), 8, 9),
    });
    h.component.up(
        &h.env,
        skein_io::Event::Stream { owner, up: skein_lib::stream::Up::Bytes(fixtures::ok(VIEWER)) },
        &mut h.up,
        &mut h.io,
    );
    h.run(Duration::ZERO);
    h.connected(FORGE);
    accepted(&mut h, &[]);
    h.answer(FORGE, &fixtures::ok(b"[]")); // The changed set discards the written page.
    listed(&mut h);
    h.run(Duration::ZERO);
    assert_eq!(h.connected_count, 1, "{stage:?}");
    assert_eq!(tokens(&h).len(), expected_tokens, "{stage:?}: no old mint published");
}
#[test]
fn closing_startup_phases_prevents_all_subsequent_private_work() {
    skein_rows::check(
        [
            ("github_close_ends_each_start_phase", Stage::Listing),
            ("github_close_ends_the_full_mint_phase", Stage::Full),
            ("github_close_ends_the_contents_mint_phase", Stage::Contents),
            ("github_close_ends_the_identity_phase", Stage::Identity),
        ],
        close_phase,
    );
}



fn close_phase(stage: Stage) {
    let mut h = at(stage);
    h.send(Request::Close);
    h.run(Duration::ZERO);
    assert!(h.closed, "{stage:?}");
    let count = tokens(&h).len();
    let connected = h.connected_count;
    h.send(Request::Installations { forge: FORGE, accepted: Box::from([1]) });
    h.send(Request::Connect {
        forge: FORGE,
        access: fixtures::app_access(skein_fake_forge_github::keys::KEYS.first().expect("public key"), 7, 9),
    });
    h.run(Duration::from_secs(20));
    assert_eq!((tokens(&h).len(), h.connected_count), (count, connected));
    assert!(h.up.is_empty());
}
#[test]
fn installation_secrets_leave_only_as_git_tokens() {
    let mut h = at(Stage::Identity);
    let debug = std::format!("{:?} {:?} {:?}", h.component, h.up, h.io);
    assert!(!contains(debug.as_bytes(), FULL));
    assert!(!contains(debug.as_bytes(), CONTENTS));
    h.answer(FORGE, &fixtures::ok(VIEWER));
    h.run(Duration::ZERO);
    for line in h.request_lines(FORGE).into_boxed() {
        assert!(!contains(&line, FULL));
        assert!(!contains(&line, CONTENTS));
    }
    let debug = std::format!("{:?} {:?} {:?}", h.component, h.up, h.git_tokens);
    assert!(!contains(debug.as_bytes(), FULL));
    assert!(!contains(debug.as_bytes(), CONTENTS));
    assert_eq!(h.component.diagnostics(), skein_forge_connection::Diagnostics { malformed: 0, long_bodies: 0 });
    assert!(h.answers.is_empty());
    assert_eq!(tokens(&h).len(), 1);
}

#[test]
fn github_bot_permission_waits_for_the_first_full_mint() {
    use skein_forge::api::{Answer, Op, Permission, Read};
    let mut h = at(Stage::Full);
    // Before the first full mint the kept levels are nothing; the bot's answer waits for them.
    h.call(1, fixtures::REPO, Op::Read(Read::Permission { user: 9 }));
    h.run(Duration::ZERO);
    assert!(h.answered(1).is_none());
    let full = br#"{"token":"artificial-full","expires_at":"1970-01-01T01:00:00Z","permissions":{"contents":"write","pull_requests":"write"}}"#;
    h.answer(FORGE, &fixtures::response(201, full));
    contents(&mut h);
    h.answer(FORGE, &fixtures::ok(VIEWER));
    h.run(Duration::ZERO);
    assert!(h.answered(1).is_none());
    h.connected(FORGE);
    h.run(Duration::ZERO);
    let lines = h.request_lines(FORGE);
    assert_eq!(lines.as_slice().last().expect("repository read").as_ref(), b"GET /repositories/1 HTTP/1.1");
    h.answer(FORGE, &fixtures::ok(fixtures::GITHUB_REPOSITORY));
    h.run(Duration::ZERO);
    let answer = h.answered(1).expect("call terminal");
    assert_eq!(
        answer.result,
        Ok(Answer::Permission(skein_forge::api::UserPermission {
            user: 9,
            login: Box::from(b"".as_slice()),
            permission: Permission::Write
        }))
    );
    for line in h.request_lines(FORGE).as_slice() {
        assert!(!contains(line, b"/collaborators/"));
    }
}

#[test]
fn github_transient_start_failures_back_off_to_listing() {
    transient(Stage::Listing, TRANSPORT_OR_RATE);
}

#[test]
fn github_transient_full_mint_failures_back_off_to_listing() {
    transient(Stage::Full, TRANSPORT_OR_RATE);
}

#[test]
fn github_transient_contents_mint_failures_back_off_to_listing() {
    transient(Stage::Contents, TRANSPORT_OR_RATE);
}

#[test]
fn github_transient_identity_failures_back_off_to_listing() {
    transient(Stage::Identity, TRANSPORT_OR_RATE);
}
