//! Document translations; the byte-level read-plan tests follow in 13.3b.
use alloc::boxed::Box;
use skein_forge::api;
use skein_forge::forgejo::document as d;
use skein_forge::forgejo::plans::reading;
use skein_lib::bytes::concat;
use skein_lib::{Time, Wall};

#[test]
fn a_local_status_link_gets_its_repositorys_origin() {
    let rows: [(&str, &[u8], &[u8]); 3] = [
        ("local", b"/job", b"https://forge.test:443/job"),
        ("other origin", b"https://ci.test/job", b"https://ci.test/job"),
        ("empty", b"", b""),
    ];
    for (name, link, expected) in rows {
        assert_eq!(reading::absolute(link, b"https://forge.test:443/owner/repo"), Ok(Box::from(expected)), "{name}");
    }
}

#[test]
fn combined_statuses_map_their_state_and_zero_contexts_override_it() {
    let rows = [
        ("pending", d::StatusState::Pending, api::Ci::Pending),
        ("success", d::StatusState::Success, api::Ci::Passed),
        ("warning", d::StatusState::Warning, api::Ci::Passed),
        ("skipped", d::StatusState::Skipped, api::Ci::Passed),
        ("error", d::StatusState::Error, api::Ci::Failed),
        ("failure", d::StatusState::Failure, api::Ci::Failed),
    ];
    for (name, state, expected) in rows {
        assert_eq!(reading::ci(Some(state), 0), api::Ci::None, "{name}: no contexts");
        assert_eq!(reading::ci(Some(state), 1), expected, "{name}: contexts");
    }
    assert_eq!(reading::ci(None, 0), api::Ci::None, "empty state and no contexts");
}

#[test]
fn status_states_map_to_the_clients_check() {
    for (name, state, expected) in [
        ("pending", d::StatusState::Pending, api::Check::Pending),
        ("success", d::StatusState::Success, api::Check::Passed),
        ("warning", d::StatusState::Warning, api::Check::Passed),
        ("skipped", d::StatusState::Skipped, api::Check::Passed),
        ("error", d::StatusState::Error, api::Check::Failed),
        ("failure", d::StatusState::Failure, api::Check::Failed),
    ] {
        assert_eq!(reading::check(state), expected, "{name}");
    }
}

#[test]
fn pending_reviews_are_skipped_and_dismissed_reviews_are_comments() {
    for (name, state, dismissed, expected) in [
        ("approved", d::ReviewState::Approved, false, Some(api::Verdict::Approve)),
        ("changes", d::ReviewState::RequestChanges, false, Some(api::Verdict::RequestChanges)),
        ("comment", d::ReviewState::Comment, false, Some(api::Verdict::Comment)),
        ("pending", d::ReviewState::Pending, false, None),
        ("request", d::ReviewState::RequestReview, false, None),
        ("dismissed", d::ReviewState::Approved, true, Some(api::Verdict::Comment)),
    ] {
        assert_eq!(reading::verdict(state, dismissed), expected, "{name}");
    }
}

#[test]
fn an_owner_is_read_as_admin() {
    for (name, permission, expected) in [
        ("none", d::Permission::None, api::Permission::None),
        ("read", d::Permission::Read, api::Permission::Read),
        ("write", d::Permission::Write, api::Permission::Write),
        ("admin", d::Permission::Admin, api::Permission::Admin),
        ("owner", d::Permission::Owner, api::Permission::Admin),
    ] {
        assert_eq!(reading::permission(permission), expected, "{name}");
    }
}

#[test]
fn an_object_id_must_match_its_repositorys_format() {
    use skein_forge::forgejo::id::{ObjectFormat, ObjectId};
    for (name, from, repository, expected) in [
        ("sha1", ObjectFormat::Sha1, ObjectFormat::Sha1, Some([0; 32])),
        ("sha256", ObjectFormat::Sha256, ObjectFormat::Sha256, Some([0; 32])),
        ("another format", ObjectFormat::Sha1, ObjectFormat::Sha256, None),
    ] {
        assert_eq!(reading::commit(ObjectId { format: from, bytes: [0; 32] }, repository), expected, "{name}");
    }
}

#[test]
fn a_summary_keeps_its_marker_and_a_long_body_carries_no_key_or_bytes() {
    let rows = [
        (
            "kept",
            d::Body::Kept(Box::from(&b"<!-- kit:key 01 -->\ntext"[..])),
            Some(Box::from([1])),
            Box::from(&b"<!-- kit:key 01 -->\ntext"[..]),
        ),
        ("long", d::Body::Long(30), None, Box::from(&b""[..])),
    ];
    for (name, body, key, expected_body) in rows {
        let summary = reading::summary(
            fixtures::WORD,
            d::Issue {
                number: 7,
                pull: true,
                state: d::IssueState::Open,
                user: d::User { id: 2, login: Box::from(&b"writer"[..]), is_admin: false, full_name: Box::from([]) },
                labels: Box::from([]),
                updated_at: Wall::from_nanos(3),
                title: Box::from(&b"title"[..]),
                body,
            },
        );
        assert_eq!(
            summary,
            api::Summary {
                number: 7,
                kind: api::Kind::Pull,
                state: api::State::Open,
                author: 2,
                labels: Box::from([]),
                updated: Time::from_nanos(3),
                title: Box::from(&b"title"[..]),
                key,
                body: expected_body
            },
            "{name}"
        );
    }
}

use super::fixtures;
use api::Answer;
const SHA: [u8; 32] = [0; 32];
fn summary() -> api::Summary {
    api::Summary {
        labels: Box::from([]),
        number: 1,
        kind: api::Kind::Issue,
        state: api::State::Open,
        author: 2,
        key: None,
        title: Box::from(&b"t"[..]),
        body: Box::from(&b"b"[..]),
        updated: Time::from_nanos(1_577_817_001_000_000_000),
    }
}
fn comment(id: u64) -> api::Comment {
    api::Comment {
        id,
        provenance: api::Provenance::Unknown,
        author: 2,
        created: Time::ZERO,
        revision: 0xba78_16bf_8f01_cfea,
        key: None,
        body: Box::from(&b"abc"[..]),
    }
}

fn expected_pull(number: u64, state: api::State) -> api::Pull {
    api::Pull {
        queued: None,
        dequeued: None,
        reviewers: Box::from([]),
        number,
        state,
        head: Box::from(&b"work"[..]),
        base: Box::from(&b"main"[..]),
        commit: SHA,
        base_commit: None,
        merged: None,
        mergeable: true,
        ci: api::Ci::None,
    }
}

fn measured_answers() -> [(&'static str, Answer); 10] {
    [
        ("items", Answer::Items { cut: false, items: Box::new([summary()]), more: false, now: Time::ZERO }),
        ("item", Answer::Item { item: summary(), comments: Box::new([comment(2), comment(3)]), more: true }),
        ("pull", Answer::Pull(expected_pull(13, api::State::Open))),
        ("reviews", Answer::Reviews { reviews: Box::from([]), more: true, cut: false }),
        ("statuses", Answer::Statuses { ci: api::Ci::None, statuses: Box::from([]), more: false }),
        (
            "Actions status",
            Answer::Statuses { ci: api::Ci::Failed, statuses: Box::new([actions_status()]), more: false },
        ),
        ("branches", Answer::Branches(Box::new([Box::from(&b"main"[..])]))),
        (
            "protection",
            Answer::Protection(Some(api::Protection {
                queue: false,
                signatures: false,
                linear: false,
                code_owners: false,
                bypass: api::Bypass::Never,
                branch: Box::from(&b"main"[..]),
                contexts: Box::new([Box::from(&b"build"[..])]),
                approvals: 2,
                dismiss_stale: true,
            })),
        ),
        (
            "settings",
            Answer::Settings(api::Settings {
                default_branch: Box::from(&b"main"[..]),
                merge: true,
                squash: false,
                rebase: false,
            }),
        ),
        (
            "collaborators",
            Answer::Collaborators {
                collaborators: Box::new([api::Collaborator { user: 2, permission: api::Permission::Admin }]),
                more: false,
            },
        ),
    ]
}
#[test]
fn the_running_answer_size_agrees_with_the_clients_measure() {
    for (name, answer) in measured_answers() {
        assert_eq!(
            skein_forge::forgejo::plans::reads::size::answer(&answer, 8).ok(),
            skein_forge::measure::answer(&answer),
            "{name}"
        );
    }
}
fn actions_status() -> api::Status {
    api::Status {
        at: Time::from_nanos(2_000_000_000),
        context: Box::from(&b"build"[..]),
        check: api::Check::Failed,
        description: Box::from(&b"failed"[..]),
        url: Box::from(&b"http://127.0.0.1:3000/fixture/specimen/actions/runs/1/jobs/0"[..]),
        job: None,
    }
}

#[test]
fn json_body_demands_are_batched_across_bounded_pumps() {
    use crate::connection::Phase;
    use skein_forge::forgejo::request;
    // Many structural bytes exceed one pump's 64 movements while the
    // whole body fits the local intake. Whitespace after the object must
    // be checked before the HTTP terminal is consumed.
    let body = concat(&[br#"{"id":2,"login":"writer","ignored":["#, &fixtures::filled(180, b' '), b"]}"]);
    let response = fixtures::ok(&body);
    let mut h = fixtures::ConnectionHarness::new();
    h.start_forgejo(request::Request::User, None).expect("user request");
    let heard = h.buffered_answer(&response, response.len(), false);
    user_decoded(&heard);
    assert_eq!(h.connection.phase(), Phase::Idle);
    assert!(!h.ready(), "no ready work after the terminal");
    assert!(h.local_pumps > 1, "buffered work survives a pump boundary");
    assert_eq!(h.body_demands, 1, "one buffered body batch");
}

fn user_decoded(heard: &skein_lib::List<skein_forge::forgejo::plans::Heard>) {
    use skein_forge::forgejo::plans;
    assert_eq!(heard.len(), 2, "one head and one terminal: {heard:?}");
    assert_eq!(
        heard.get(1),
        Some(&plans::Heard::Done(plans::Body::Document(d::Document::User(d::User {
            id: 2,
            login: Box::from(&b"writer"[..]),
            is_admin: false,
            full_name: Box::from([]),
        })))),
        "complete decoded body"
    );
}

fn user_request(h: &mut fixtures::ConnectionHarness) {
    h.start_forgejo(skein_forge::forgejo::request::Request::User, None).expect("user request");
}

#[test]
fn json_body_fragments_and_final_bytes_precede_end() {
    let response = fixtures::ok(br#"{"id":2,"login":"wri\u0074er","ignored":false}"#);
    for chunk in [1, 4, 1024] {
        for fragment in [1, 7, response.len()] {
            for eof in [false, true] {
                let mut h = fixtures::ConnectionHarness::with_chunk(chunk);
                user_request(&mut h);
                let heard = h.buffered_answer(&response, fragment, eof);
                user_decoded(&heard);
                assert!(!h.ready(), "chunk{chunk}/fragment{fragment}/eof{eof}: no residual work");
                assert!(h.body_demands > 0, "body read");
            }
        }
    }
}

#[test]
fn json_body_batches_cross_intake_boundaries_and_reuse_cleanly() {
    let body = concat(&[br#"{"id":2,"login":"writer","ignored":["#, &fixtures::filled(2500, b' '), b"]}"]);
    let response = fixtures::ok(&body);
    let mut h = fixtures::ConnectionHarness::new();
    user_request(&mut h);
    user_decoded(&h.buffered_answer(&response, response.len(), false));
    let capacity = usize::try_from(h.env.limits.chunk).expect("chunk fits");
    assert_eq!(usize::try_from(h.body_demands).expect("count fits"), body.len().div_ceil(capacity));
    assert!(h.local_pumps > 2, "local work survives repeated 64-movement caps");
    assert_eq!(h.connection.phase(), crate::connection::Phase::Idle);
    // A second exchange has no held bytes, end or demand from the first.
    user_request(&mut h);
    user_decoded(&h.buffered_answer(&fixtures::user_ok(), 7, false));
    assert_eq!(h.connection.phase(), crate::connection::Phase::Idle);
}

#[test]
fn closing_with_a_partial_json_batch_drops_local_work() {
    let response = b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\n{\"id\":2,\"login\":\"wri";
    let mut h = fixtures::ConnectionHarness::new();
    user_request(&mut h);
    let heard = h.buffered_answer(response, response.len(), false);
    assert_eq!(heard.len(), 1, "only the head before the incomplete string");
    assert!(!h.ready(), "waits for bytes, not a local spin");
    h.connection.close(&h.env, true, &mut h.io);
    h.connection.stream(&h.env, skein_lib::stream::Up::End);
    h.pump();
    assert!(!h.ready(), "close leaves no local work");
    assert!(h.forgejo.heard(&mut h.connection).is_none(), "no answer after close");
}

#[test]
#[expect(
    clippy::print_stdout,
    clippy::disallowed_macros,
    reason = "report the local intake peak and teardown for the memory ledger"
)]
fn json_body_intakes_stay_under_the_ceiling_and_release_on_teardown() {
    // All configured JSON intakes and their fixture peak, including teardown.
    const CEILING: u64 = 25_000;
    let span = skein_heap::Span::start();
    {
        let mut connections = skein_lib::List::with_capacity(fixtures::limits().connections);
        for _ in 0..fixtures::limits().connections {
            let mut h = fixtures::ConnectionHarness::new();
            user_request(&mut h);
            let body = concat(&[br#"{"id":2,"login":"writer","ignored":["#, &fixtures::filled(2500, b' '), b"]}"]);
            let response = fixtures::ok(&body);
            user_decoded(&h.buffered_answer(&response, response.len(), false));
            connections.push(h).expect("all configured connections");
        }
        for at in 0..connections.len() {
            let h = connections.get_mut(at).expect("configured connection");
            h.connection.close(&h.env, false, &mut h.io);
            assert!(!h.ready(), "closed connections do not spin");
        }
    }
    let grown = span.end();
    assert_eq!(grown.net, 0, "all fixture and local intake allocations released");
    assert!(u64::try_from(grown.peak).expect("positive peak") <= CEILING, "{} <= {CEILING}", grown.peak);
    println!("JSON intake fixture peak={} ceiling={CEILING} teardown_net={}", grown.peak, grown.net);
}
