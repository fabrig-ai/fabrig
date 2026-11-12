use super::fixtures::{self, ADMIN, reached, search, secret, user_repo, user_repos};
use crate::call::Auth;
use crate::{
    given::{Place, Refused},
    item::Item,
    reachable::{self, Reachable, Reached},
    reconcile::{Event, Need, Outcome, Reply, Stage, Why},
};
use skein_forge::forgejo::{
    document::{Document, Hook, Row},
    request::{Page, Request},
};
use skein_lib::Queue;

const READING: usize = 1;
const ADMIN_CHECK: usize = 2;
const SETTINGS: usize = 3;
const SEARCH: usize = 4;
fn step(machine: &mut Reachable, event: Event) -> Need {
    let mut out = Queue::uncapped();
    reachable::step(machine, event, &mut out);
    assert_eq!(out.len(), 1);
    out.pop().expect("one need")
}
fn at(phase: usize) -> Reachable {
    let mut machine = fixtures::reachable(1);
    for event in [
        Event::Begin,
        Event::Read(Some(secret(ADMIN))),
        fixtures::user_event(fixtures::admin()),
        fixtures::settings_event(50),
    ]
    .into_iter()
    .take(phase)
    {
        step(&mut machine, event);
    }
    machine
}
fn send(request: Request) -> Need {
    Need::Send { request: request.into(), auth: Auth::Token(secret(ADMIN)) }
}
fn page(page: u32, limit: u32) -> Need {
    send(Request::RepoSearch { page: Page { page: Some(page), limit: Some(limit) } })
}
fn failure(stage: Stage, why: Why) -> Need {
    fixtures::named_failure(stage, b"kit-admin", why)
}
fn rows(count: u32) -> Box<[Row]> {
    let mut rows = skein_lib::List::with_capacity(count);
    for id in 1..=count {
        rows.push(reached(u64::from(id), 7, b"ada", b"work", false)).expect("fixture rows");
    }
    rows.into_boxed()
}
fn outcome(count: u32, more: bool) -> Need {
    let mut rows = skein_lib::List::with_capacity(count);
    for id in 1..=count {
        rows.push(Reached {
            id: u64::from(id),
            owner_id: 7,
            owner: b"ada".as_slice().into(),
            name: b"work".as_slice().into(),
            private: false,
        })
        .expect("fixture rows");
    }
    Need::Done(Outcome::Reachable { rows: rows.into_boxed(), more })
}
#[test]
fn reachable_checks_the_admins_token_then_reads_the_cap_then_one_page() {
    let mut machine = fixtures::reachable(1);
    assert_eq!(step(&mut machine, Event::Begin), Need::Read(Item::AdminToken));
    assert_eq!(step(&mut machine, Event::Read(Some(secret(ADMIN)))), send(Request::User));
    assert_eq!(step(&mut machine, fixtures::user_event(fixtures::admin())), send(Request::ApiSettings));
    assert_eq!(step(&mut machine, fixtures::settings_event(50)), page(1, 4));
}
#[test]
fn listing_page_sizes_obey_configured_and_advertised_bounds() {
    skein_rows::check(
        [
            (
                "the_page_size_is_the_least_of_the_limit_and_the_cap: 1",
                (at(SETTINGS), fixtures::settings_event(1), page(1, 1)),
            ),
            (
                "the_page_size_is_the_least_of_the_limit_and_the_cap: 3",
                (at(SETTINGS), fixtures::settings_event(3), page(1, 3)),
            ),
            (
                "the_page_size_is_the_least_of_the_limit_and_the_cap: 4",
                (at(SETTINGS), fixtures::settings_event(4), page(1, 4)),
            ),
            (
                "the_page_size_is_the_least_of_the_limit_and_the_cap: 50",
                (at(SETTINGS), fixtures::settings_event(50), page(1, 4)),
            ),
            (
                "the_connected_page_size_is_the_least_of_the_limit_and_the_cap: 1",
                (connected_at(CONNECTED_SETTINGS), fixtures::settings_event(1), connected_page(1, 1)),
            ),
            (
                "the_connected_page_size_is_the_least_of_the_limit_and_the_cap: 3",
                (connected_at(CONNECTED_SETTINGS), fixtures::settings_event(3), connected_page(1, 3)),
            ),
            (
                "the_connected_page_size_is_the_least_of_the_limit_and_the_cap: 4",
                (connected_at(CONNECTED_SETTINGS), fixtures::settings_event(4), connected_page(1, 4)),
            ),
            (
                "the_connected_page_size_is_the_least_of_the_limit_and_the_cap: 50",
                (connected_at(CONNECTED_SETTINGS), fixtures::settings_event(50), connected_page(1, 4)),
            ),
        ],
        |(mut machine, event, expected)| {
            assert_eq!(step(&mut machine, event), expected);
        },
    );
}
#[test]
fn requested_pages_reach_the_listing_request() {
    skein_rows::check(
        [
            (
                "the_page_asked_for_is_the_page_sent: 3",
                (
                    {
                        let mut machine = fixtures::reachable(3);
                        step(&mut machine, Event::Begin);
                        step(&mut machine, Event::Read(Some(secret(ADMIN))));
                        step(&mut machine, fixtures::user_event(fixtures::admin()));
                        machine
                    },
                    fixtures::settings_event(50),
                    page(3, 4),
                ),
            ),
            (
                "the_connected_page_asked_for_is_the_page_sent: 3",
                (connected_at_page(CONNECTED_SETTINGS, 3), fixtures::settings_event(50), connected_page(3, 4)),
            ),
            (
                "the_page_asked_for_is_the_page_sent: u32::MAX",
                (
                    {
                        let mut machine = fixtures::reachable(u32::MAX);
                        step(&mut machine, Event::Begin);
                        step(&mut machine, Event::Read(Some(secret(ADMIN))));
                        step(&mut machine, fixtures::user_event(fixtures::admin()));
                        machine
                    },
                    fixtures::settings_event(50),
                    page(u32::MAX, 4),
                ),
            ),
            (
                "the_connected_page_asked_for_is_the_page_sent: u32::MAX",
                (
                    connected_at_page(CONNECTED_SETTINGS, u32::MAX),
                    fixtures::settings_event(50),
                    connected_page(u32::MAX, 4),
                ),
            ),
        ],
        |(mut machine, event, expected)| {
            assert_eq!(step(&mut machine, event), expected);
        },
    );
}
#[test]
fn a_full_page_says_there_is_more() {
    assert_eq!(step(&mut at(SEARCH), search(true, rows(4))), outcome(4, true));
    let mut capped = at(SETTINGS);
    step(&mut capped, fixtures::settings_event(3));
    assert_eq!(step(&mut capped, search(true, rows(3))), outcome(3, true));
}
#[test]
fn short_search_pages_report_no_more() {
    skein_rows::check(
        [
            ("a_short_last_page_says_there_is_no_more", (at(SEARCH), search(true, rows(3)), outcome(3, false))),
            (
                "an_empty_page_says_there_is_no_more_and_holds_no_row",
                (at(SEARCH), search(true, rows(0)), outcome(0, false)),
            ),
        ],
        |(mut machine, event, expected)| {
            assert_eq!(step(&mut machine, event), expected);
        },
    );
}
#[test]
fn listings_preserve_repository_identity_ownership_and_privacy() {
    skein_rows::check(
        [
            (
                "each_row_carries_its_id_owner_name_and_privacy",
                (
                    at(SEARCH),
                    search(
                        true,
                        Box::new([reached(9, 13, b"grace", b"private", true), reached(2, 7, b"ada", b"public", false)]),
                    ),
                    Need::Done(Outcome::Reachable {
                        rows: Box::new([
                            Reached {
                                id: 9,
                                owner_id: 13,
                                owner: b"grace".as_slice().into(),
                                name: b"private".as_slice().into(),
                                private: true,
                            },
                            Reached {
                                id: 2,
                                owner_id: 7,
                                owner: b"ada".as_slice().into(),
                                name: b"public".as_slice().into(),
                                private: false,
                            },
                        ]),
                        more: false,
                    }),
                ),
            ),
            (
                "each_connected_row_carries_its_id_owner_name_and_privacy",
                (
                    connected_at(CONNECTED_LISTING),
                    user_repos(Box::new([
                        user_repo(9, 13, b"grace", b"private", true, true),
                        user_repo(2, 7, b"ada", b"public", false, true),
                    ])),
                    Need::Done(Outcome::Reachable {
                        rows: Box::new([
                            Reached {
                                id: 9,
                                owner_id: 13,
                                owner: b"grace".as_slice().into(),
                                name: b"private".as_slice().into(),
                                private: true,
                            },
                            Reached {
                                id: 2,
                                owner_id: 7,
                                owner: b"ada".as_slice().into(),
                                name: b"public".as_slice().into(),
                                private: false,
                            },
                        ]),
                        more: false,
                    }),
                ),
            ),
        ],
        |(mut machine, event, expected)| {
            assert_eq!(step(&mut machine, event), expected);
        },
    );
}
#[test]
fn a_non_admin_is_refused_naming_the_admin() {
    assert_eq!(
        step(&mut at(ADMIN_CHECK), fixtures::user_event(fixtures::person(b"person", b"Person"))),
        failure(Stage::Admin, Why::NotAdmin)
    );
}
#[test]
fn neither_token_stops_missing_naming_api() {
    for admin in [None, Some(secret(b""))] {
        for api in [None, Some(secret(b""))] {
            let mut machine = at(READING);
            step(&mut machine, Event::Read(admin.clone()));
            assert_eq!(step(&mut machine, Event::Read(api)), connected_failure(Why::Missing));
        }
    }
}
#[test]
fn an_installation_given_to_a_forgejo_still_stops_before_either_token_is_read() {
    for (id, subject) in [(7, b"7".as_slice()), (0, b"0"), (u64::MAX, b"18446744073709551615")] {
        let mut machine = Reachable::new(&fixtures::given(), Some(id), 1).expect("given admitted");
        assert_eq!(
            step(&mut machine, Event::Begin),
            fixtures::named_failure(Stage::Installation, subject, Why::Unexpected)
        );
        let mut out = Queue::uncapped();
        reachable::step(&mut machine, Event::Read(Some(secret(ADMIN))), &mut out);
        assert!(out.is_empty());
    }
}
#[test]
fn malformed_search_replies_stop_unexpected_at_the_token() {
    skein_rows::check(
        [
            (
                "an_answer_whose_ok_is_not_true_is_unexpected",
                (at(SEARCH), search(false, rows(1)), failure(Stage::Token, Why::Unexpected)),
            ),
            (
                "a_row_that_is_not_a_found_row_is_unexpected",
                (
                    at(SEARCH),
                    search(
                        true,
                        Box::new([
                            reached(1, 7, b"ada", b"work", false),
                            Row::Hook(Hook { id: 1, url: Box::new([]), events: Box::new([]), active: true }),
                        ]),
                    ),
                    failure(Stage::Token, Why::Unexpected),
                ),
            ),
        ],
        |(mut machine, event, expected)| {
            assert_eq!(step(&mut machine, event), expected);
        },
    );
}
#[test]
fn zero_page_caps_name_the_active_credential() {
    skein_rows::check(
        [
            (
                "a_zero_page_cap_is_unexpected",
                (at(SETTINGS), fixtures::settings_event(0), failure(Stage::Token, Why::Unexpected)),
            ),
            (
                "a_zero_page_cap_is_unexpected_naming_api",
                (connected_at(CONNECTED_SETTINGS), fixtures::settings_event(0), connected_failure(Why::Unexpected)),
            ),
        ],
        |(mut machine, event, expected)| {
            assert_eq!(step(&mut machine, event), expected);
        },
    );
}
#[test]
fn oversized_listings_stop_before_completion() {
    skein_rows::check(
        [
            (
                "a_page_of_more_rows_than_asked_is_unexpected: 50",
                (at(SEARCH), search(true, rows(5)), failure(Stage::Token, Why::Unexpected)),
            ),
            (
                "a_page_of_more_rows_than_asked_is_unexpected: 3",
                (
                    {
                        let mut machine = at(SETTINGS);
                        step(&mut machine, fixtures::settings_event(3));
                        machine
                    },
                    search(true, rows(4)),
                    failure(Stage::Token, Why::Unexpected),
                ),
            ),
            (
                "a_connected_listing_of_more_rows_than_asked_is_unexpected: 50",
                (connected_at(CONNECTED_LISTING), user_repos(user_rows(5, false)), connected_failure(Why::Unexpected)),
            ),
            (
                "a_connected_listing_of_more_rows_than_asked_is_unexpected: 3",
                (
                    {
                        let mut machine = connected_at(CONNECTED_SETTINGS);
                        step(&mut machine, fixtures::settings_event(3));
                        machine
                    },
                    user_repos(user_rows(4, false)),
                    connected_failure(Why::Unexpected),
                ),
            ),
        ],
        |(mut machine, event, expected)| {
            assert_eq!(step(&mut machine, event), expected);
        },
    );
}
#[test]
fn refused_requests_name_their_credential_and_stage() {
    skein_rows::check(
        [
            (
                "a_refused_request_names_the_admin_at_each_of_its_three_requests: ADMIN_CHECK",
                (at(ADMIN_CHECK), Event::Reply(Reply::Status(403)), failure(Stage::Admin, Why::Refused(403))),
            ),
            (
                "a_refused_request_names_the_admin_at_each_of_its_three_requests: SETTINGS",
                (at(SETTINGS), Event::Reply(Reply::Status(403)), failure(Stage::Token, Why::Refused(403))),
            ),
            (
                "a_refused_request_names_the_admin_at_each_of_its_three_requests: SEARCH",
                (at(SEARCH), Event::Reply(Reply::Status(403)), failure(Stage::Token, Why::Refused(403))),
            ),
            (
                "a_refused_request_names_api_at_both_of_its_requests: CONNECTED_SETTINGS",
                (
                    connected_at(CONNECTED_SETTINGS),
                    Event::Reply(Reply::Status(401)),
                    connected_failure(Why::Refused(401)),
                ),
            ),
            (
                "a_refused_request_names_api_at_both_of_its_requests: CONNECTED_LISTING",
                (
                    connected_at(CONNECTED_LISTING),
                    Event::Reply(Reply::Status(401)),
                    connected_failure(Why::Refused(401)),
                ),
            ),
        ],
        |(mut machine, event, expected)| {
            assert_eq!(step(&mut machine, event), expected);
        },
    );
}
#[test]
fn unanswered_requests_name_their_active_credential() {
    skein_rows::check(
        [
            (
                "a_forge_that_stops_answering_stops_the_plan_at_each_request: ADMIN_CHECK",
                (at(ADMIN_CHECK), Event::Reply(Reply::Down), failure(Stage::Admin, Why::Down)),
            ),
            (
                "a_forge_that_stops_answering_stops_the_plan_at_each_request: SETTINGS",
                (at(SETTINGS), Event::Reply(Reply::Down), failure(Stage::Token, Why::Down)),
            ),
            (
                "a_forge_that_stops_answering_stops_the_plan_at_each_request: SEARCH",
                (at(SEARCH), Event::Reply(Reply::Down), failure(Stage::Token, Why::Down)),
            ),
            (
                "a_forge_that_stops_answering_stops_the_connected_plan_at_both: CONNECTED_SETTINGS",
                (connected_at(CONNECTED_SETTINGS), Event::Reply(Reply::Down), connected_failure(Why::Down)),
            ),
            (
                "a_forge_that_stops_answering_stops_the_connected_plan_at_both: CONNECTED_LISTING",
                (connected_at(CONNECTED_LISTING), Event::Reply(Reply::Down), connected_failure(Why::Down)),
            ),
        ],
        |(mut machine, event, expected)| {
            assert_eq!(step(&mut machine, event), expected);
        },
    );
}
#[test]
#[expect(clippy::too_many_lines, reason = "the labelled cases keep every input and expectation of one rule together")]
fn wrong_reply_kinds_stop_each_picker_request() {
    skein_rows::check(
        [
            (
                "a_reply_of_another_kind_is_unexpected_at_each_request: ADMIN_CHECK 0",
                (at(ADMIN_CHECK), Event::Reply(Reply::Empty), failure(Stage::Admin, Why::Unexpected)),
            ),
            (
                "a_reply_of_another_kind_is_unexpected_at_each_request: ADMIN_CHECK 1",
                (
                    at(ADMIN_CHECK),
                    Event::Reply(Reply::Document(Document::RepoSearch { ok: true })),
                    failure(Stage::Admin, Why::Unexpected),
                ),
            ),
            (
                "a_reply_of_another_kind_is_unexpected_at_each_request: ADMIN_CHECK 2",
                (
                    at(ADMIN_CHECK),
                    Event::Reply(Reply::Document(Document::RepoSearch { ok: false })),
                    failure(Stage::Admin, Why::Unexpected),
                ),
            ),
            (
                "a_reply_of_another_kind_is_unexpected_at_each_request: SETTINGS 0",
                (at(SETTINGS), Event::Reply(Reply::Empty), failure(Stage::Token, Why::Unexpected)),
            ),
            (
                "a_reply_of_another_kind_is_unexpected_at_each_request: SETTINGS 1",
                (
                    at(SETTINGS),
                    Event::Reply(Reply::Document(Document::RepoSearch { ok: true })),
                    failure(Stage::Token, Why::Unexpected),
                ),
            ),
            (
                "a_reply_of_another_kind_is_unexpected_at_each_request: SETTINGS 2",
                (
                    at(SETTINGS),
                    Event::Reply(Reply::Document(Document::RepoSearch { ok: false })),
                    failure(Stage::Token, Why::Unexpected),
                ),
            ),
            (
                "a_reply_of_another_kind_is_unexpected_at_each_request: SEARCH 0",
                (at(SEARCH), Event::Reply(Reply::Empty), failure(Stage::Token, Why::Unexpected)),
            ),
            (
                "a_reply_of_another_kind_is_unexpected_at_each_request: SEARCH 1",
                (
                    at(SEARCH),
                    Event::Reply(Reply::Document(Document::RepoSearch { ok: true })),
                    failure(Stage::Token, Why::Unexpected),
                ),
            ),
            (
                "a_reply_of_another_kind_is_unexpected_at_each_request: SEARCH 2",
                (
                    at(SEARCH),
                    Event::Reply(Reply::Document(Document::RepoSearch { ok: false })),
                    failure(Stage::Token, Why::Unexpected),
                ),
            ),
            (
                "a_reply_of_another_kind_is_unexpected_at_each_request: wrong document 0",
                (at(ADMIN_CHECK), fixtures::settings_event(50), failure(Stage::Admin, Why::Unexpected)),
            ),
            (
                "a_reply_of_another_kind_is_unexpected_at_each_request: wrong document 1",
                (at(ADMIN_CHECK), search(true, rows(0)), failure(Stage::Admin, Why::Unexpected)),
            ),
            (
                "a_reply_of_another_kind_is_unexpected_at_each_request: wrong document 2",
                (at(SETTINGS), fixtures::user_event(fixtures::admin()), failure(Stage::Token, Why::Unexpected)),
            ),
            (
                "a_reply_of_another_kind_is_unexpected_at_each_request: wrong document 3",
                (at(SETTINGS), search(true, rows(0)), failure(Stage::Token, Why::Unexpected)),
            ),
            (
                "a_reply_of_another_kind_is_unexpected_at_each_request: wrong document 4",
                (at(SEARCH), fixtures::settings_event(50), failure(Stage::Token, Why::Unexpected)),
            ),
            (
                "a_reply_of_another_kind_is_unexpected_at_each_request: wrong document 5",
                (at(SEARCH), fixtures::user_event(fixtures::admin()), failure(Stage::Token, Why::Unexpected)),
            ),
            (
                "a_reply_of_another_kind_is_unexpected_at_both: CONNECTED_SETTINGS 0",
                (connected_at(CONNECTED_SETTINGS), Event::Reply(Reply::Empty), connected_failure(Why::Unexpected)),
            ),
            (
                "a_reply_of_another_kind_is_unexpected_at_both: CONNECTED_SETTINGS 1",
                (
                    connected_at(CONNECTED_SETTINGS),
                    fixtures::user_event(fixtures::admin()),
                    connected_failure(Why::Unexpected),
                ),
            ),
            (
                "a_reply_of_another_kind_is_unexpected_at_both: CONNECTED_SETTINGS 2",
                (connected_at(CONNECTED_SETTINGS), search(true, rows(1)), connected_failure(Why::Unexpected)),
            ),
            (
                "a_reply_of_another_kind_is_unexpected_at_both: CONNECTED_SETTINGS 3",
                (
                    connected_at(CONNECTED_SETTINGS),
                    Event::Reply(Reply::Document(Document::RepoSearch { ok: false })),
                    connected_failure(Why::Unexpected),
                ),
            ),
            (
                "a_reply_of_another_kind_is_unexpected_at_both: CONNECTED_LISTING 0",
                (connected_at(CONNECTED_LISTING), Event::Reply(Reply::Empty), connected_failure(Why::Unexpected)),
            ),
            (
                "a_reply_of_another_kind_is_unexpected_at_both: CONNECTED_LISTING 1",
                (
                    connected_at(CONNECTED_LISTING),
                    fixtures::user_event(fixtures::admin()),
                    connected_failure(Why::Unexpected),
                ),
            ),
            (
                "a_reply_of_another_kind_is_unexpected_at_both: CONNECTED_LISTING 2",
                (connected_at(CONNECTED_LISTING), search(true, rows(1)), connected_failure(Why::Unexpected)),
            ),
            (
                "a_reply_of_another_kind_is_unexpected_at_both: CONNECTED_LISTING 3",
                (
                    connected_at(CONNECTED_LISTING),
                    Event::Reply(Reply::Document(Document::RepoSearch { ok: false })),
                    connected_failure(Why::Unexpected),
                ),
            ),
            (
                "a_reply_of_another_kind_is_unexpected_at_both: CONNECTED_SETTINGS wrong document",
                (connected_at(CONNECTED_SETTINGS), user_repos(user_rows(0, true)), connected_failure(Why::Unexpected)),
            ),
            (
                "a_reply_of_another_kind_is_unexpected_at_both: CONNECTED_LISTING wrong document",
                (connected_at(CONNECTED_LISTING), fixtures::settings_event(50), connected_failure(Why::Unexpected)),
            ),
        ],
        |(mut machine, event, expected)| {
            assert_eq!(step(&mut machine, event), expected);
        },
    );
}
/// Begin, the owner read, and the three requests, in transition order.
pub(super) fn phase_cases() -> [(Reachable, Stage, &'static [u8]); 5] {
    [
        (at(0), Stage::Admin, b"kit-admin"),
        (at(READING), Stage::Admin, b"kit-admin"),
        (at(ADMIN_CHECK), Stage::Admin, b"kit-admin"),
        (at(SETTINGS), Stage::Token, b"kit-admin"),
        (at(SEARCH), Stage::Token, b"kit-admin"),
    ]
}
#[test]
#[expect(clippy::too_many_lines, reason = "the labelled cases keep every input and expectation of one rule together")]
#[expect(
    clippy::disallowed_macros,
    reason = "the picker cases are allocated on the heap rather than in a large stack array"
)]
fn out_of_order_events_stop_each_picker_phase() {
    skein_rows::check(
        vec![
            (
                "an_out_of_order_event_stops_each_phase: begin / read",
                (at(0), Event::Read(None), failure(Stage::Admin, Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_phase: begin / written",
                (at(0), Event::Written, failure(Stage::Admin, Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_phase: begin / drawn",
                (at(0), fixtures::drawn(), failure(Stage::Admin, Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_phase: begin / reply",
                (at(0), Event::Reply(Reply::Empty), failure(Stage::Admin, Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_phase: admin read / begin",
                (at(READING), Event::Begin, failure(Stage::Admin, Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_phase: admin read / written",
                (at(READING), Event::Written, failure(Stage::Admin, Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_phase: admin read / drawn",
                (at(READING), fixtures::drawn(), failure(Stage::Admin, Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_phase: admin read / reply",
                (at(READING), Event::Reply(Reply::Empty), failure(Stage::Admin, Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_phase: admin check / begin",
                (at(ADMIN_CHECK), Event::Begin, failure(Stage::Admin, Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_phase: admin check / read",
                (at(ADMIN_CHECK), Event::Read(None), failure(Stage::Admin, Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_phase: admin check / written",
                (at(ADMIN_CHECK), Event::Written, failure(Stage::Admin, Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_phase: admin check / drawn",
                (at(ADMIN_CHECK), fixtures::drawn(), failure(Stage::Admin, Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_phase: settings / begin",
                (at(SETTINGS), Event::Begin, failure(Stage::Token, Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_phase: settings / read",
                (at(SETTINGS), Event::Read(None), failure(Stage::Token, Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_phase: settings / written",
                (at(SETTINGS), Event::Written, failure(Stage::Token, Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_phase: settings / drawn",
                (at(SETTINGS), fixtures::drawn(), failure(Stage::Token, Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_phase: search / begin",
                (at(SEARCH), Event::Begin, failure(Stage::Token, Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_phase: search / read",
                (at(SEARCH), Event::Read(None), failure(Stage::Token, Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_phase: search / written",
                (at(SEARCH), Event::Written, failure(Stage::Token, Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_phase: search / drawn",
                (at(SEARCH), fixtures::drawn(), failure(Stage::Token, Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_connected_phase: api read / begin",
                (connected_at(API_READ), Event::Begin, connected_failure(Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_connected_phase: api read / written",
                (connected_at(API_READ), Event::Written, connected_failure(Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_connected_phase: api read / drawn",
                (connected_at(API_READ), fixtures::drawn(), connected_failure(Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_connected_phase: api read / reply",
                (connected_at(API_READ), Event::Reply(Reply::Empty), connected_failure(Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_connected_phase: connected settings / begin",
                (connected_at(CONNECTED_SETTINGS), Event::Begin, connected_failure(Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_connected_phase: connected settings / written",
                (connected_at(CONNECTED_SETTINGS), Event::Written, connected_failure(Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_connected_phase: connected settings / drawn",
                (connected_at(CONNECTED_SETTINGS), fixtures::drawn(), connected_failure(Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_connected_phase: connected settings / read",
                (connected_at(CONNECTED_SETTINGS), Event::Read(None), connected_failure(Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_connected_phase: connected listing / begin",
                (connected_at(CONNECTED_LISTING), Event::Begin, connected_failure(Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_connected_phase: connected listing / written",
                (connected_at(CONNECTED_LISTING), Event::Written, connected_failure(Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_connected_phase: connected listing / drawn",
                (connected_at(CONNECTED_LISTING), fixtures::drawn(), connected_failure(Why::Unexpected)),
            ),
            (
                "an_out_of_order_event_stops_each_connected_phase: connected listing / read",
                (connected_at(CONNECTED_LISTING), Event::Read(None), connected_failure(Why::Unexpected)),
            ),
        ],
        |(mut machine, event, expected)| {
            assert_eq!(step(&mut machine, event), expected);
        },
    );
}
#[test]
fn ended_pickers_ignore_every_later_event() {
    skein_rows::check(
        [
            ("an_ended_reachable_ignores_every_event: done", {
                let mut machine = at(SEARCH);
                step(&mut machine, search(true, rows(0)));
                machine
            }),
            ("an_ended_reachable_ignores_every_event: stopped", {
                let mut machine = at(ADMIN_CHECK);
                step(&mut machine, Event::Reply(Reply::Down));
                machine
            }),
            ("an_ended_connected_reachable_ignores_every_event: done", {
                let mut machine = connected_at(CONNECTED_LISTING);
                step(&mut machine, user_repos(user_rows(0, false)));
                machine
            }),
            ("an_ended_connected_reachable_ignores_every_event: stopped", {
                let mut machine = connected_at(CONNECTED_SETTINGS);
                step(&mut machine, Event::Reply(Reply::Down));
                machine
            }),
        ],
        |mut machine| {
            for event in [
                Event::Begin,
                Event::Read(None),
                Event::Read(Some(secret(ADMIN))),
                Event::Reply(Reply::Down),
                Event::Written,
                fixtures::drawn(),
                Event::Read(Some(secret(API))),
            ] {
                let mut out = Queue::uncapped();
                reachable::step(&mut machine, event, &mut out);
                assert!(out.is_empty());
            }
        },
    );
}
#[test]
fn invalid_picker_inputs_name_the_first_refusal() {
    skein_rows::check(
        [
            (
                "reachable_refuses_page_zero_naming_its_place",
                (fixtures::given(), 0, Refused { place: Place::Page, other: None }),
            ),
            (
                "reachable_refuses_a_given_that_is_refused: 0",
                (
                    {
                        let mut given = fixtures::given();
                        given.admin_name = Box::new([]);
                        given
                    },
                    0,
                    Refused { place: Place::AdminName, other: None },
                ),
            ),
            (
                "reachable_refuses_a_given_that_is_refused: 1",
                (
                    {
                        let mut given = fixtures::given();
                        given.admin_name = Box::new([]);
                        given
                    },
                    1,
                    Refused { place: Place::AdminName, other: None },
                ),
            ),
        ],
        |(given, page, expected)| {
            assert_eq!(Reachable::new(&given, None, page).expect_err("refused input"), expected);
        },
    );
}
#[test]
#[expect(clippy::disallowed_macros, reason = "the redaction test inspects failure and machine Debug renderings")]
fn no_token_is_in_a_reachables_failure_debug() {
    for phase in 0..=SEARCH {
        let mut machine = at(phase);
        let event = if phase == 0 { Event::Written } else { Event::Begin };
        let failure = step(&mut machine, event);
        assert!(skein_lib::bytes::find(format!("{failure:?}").as_bytes(), ADMIN).is_none());
        let debug = format!("{machine:?}");
        assert!(skein_lib::bytes::find(debug.as_bytes(), ADMIN).is_none());
        assert!(debug.contains("token: None"));
    }
    let mut machine = at(SEARCH);
    step(&mut machine, search(true, rows(0)));
    assert!(format!("{machine:?}").contains("token: None"));
}

const API: &[u8] = b"picker-api-token";
const API_READ: usize = 2;
const CONNECTED_SETTINGS: usize = 3;
const CONNECTED_LISTING: usize = 4;
fn connected_at_page(phase: usize, page: u32) -> Reachable {
    let mut machine = fixtures::reachable(page);
    for event in [Event::Begin, Event::Read(None), Event::Read(Some(secret(API))), fixtures::settings_event(50)]
        .into_iter()
        .take(phase)
    {
        step(&mut machine, event);
    }
    machine
}
fn connected_at(phase: usize) -> Reachable {
    connected_at_page(phase, 1)
}
fn connected_send(request: Request) -> Need {
    Need::Send { request: request.into(), auth: Auth::Token(secret(API)) }
}
fn connected_page(page: u32, limit: u32) -> Need {
    connected_send(Request::UserRepos { page: Page { page: Some(page), limit: Some(limit) } })
}
fn connected_failure(why: Why) -> Need {
    fixtures::named_failure(Stage::Token, b"api", why)
}
fn user_rows(count: u32, push: bool) -> Box<[Row]> {
    let mut rows = skein_lib::List::with_capacity(count);
    for id in 1..=count {
        rows.push(user_repo(u64::from(id), 7, b"ada", b"work", false, push)).expect("fixture rows");
    }
    rows.into_boxed()
}
fn kept(ids: &[u64], more: bool) -> Need {
    let mut rows = skein_lib::List::with_capacity(u32::try_from(ids.len()).expect("fixture count"));
    for id in ids {
        rows.push(Reached {
            id: *id,
            owner_id: 7,
            owner: b"ada".as_slice().into(),
            name: b"work".as_slice().into(),
            private: false,
        })
        .expect("fixture rows");
    }
    Need::Done(Outcome::Reachable { rows: rows.into_boxed(), more })
}
#[test]
fn with_no_admin_token_the_api_token_reads_the_cap_then_one_page() {
    let mut machine = fixtures::reachable(1);
    assert_eq!(step(&mut machine, Event::Begin), Need::Read(Item::AdminToken));
    assert_eq!(step(&mut machine, Event::Read(None)), Need::Read(Item::ApiToken));
    assert_eq!(step(&mut machine, Event::Read(Some(secret(API)))), connected_send(Request::ApiSettings));
    assert_eq!(step(&mut machine, fixtures::settings_event(50)), connected_page(1, 4));
}
#[test]
fn a_kept_admin_token_is_used_and_the_api_token_is_never_read() {
    let mut machine = at(READING);
    assert_eq!(step(&mut machine, Event::Read(Some(secret(ADMIN)))), send(Request::User));
    assert_eq!(step(&mut machine, fixtures::user_event(fixtures::admin())), send(Request::ApiSettings));
    assert_eq!(step(&mut machine, fixtures::settings_event(50)), page(1, 4));
    assert_eq!(step(&mut machine, search(true, rows(1))), outcome(1, false));
}
#[test]
fn a_row_without_push_is_dropped_and_the_rest_keep_their_order() {
    assert_eq!(
        step(
            &mut connected_at(CONNECTED_LISTING),
            user_repos(Box::new([
                user_repo(9, 7, b"ada", b"work", false, true),
                user_repo(3, 7, b"ada", b"work", false, false),
                user_repo(2, 7, b"ada", b"work", false, true),
            ]))
        ),
        kept(&[9, 2], false)
    );
}
#[test]
fn more_is_read_before_the_rows_are_dropped() {
    assert_eq!(step(&mut connected_at(CONNECTED_LISTING), user_repos(user_rows(4, false))), kept(&[], true));
}
#[test]
fn a_short_page_has_no_more_though_rows_are_dropped() {
    assert_eq!(
        step(
            &mut connected_at(CONNECTED_LISTING),
            user_repos(Box::new([
                user_repo(1, 7, b"ada", b"work", false, true),
                user_repo(2, 7, b"ada", b"work", false, false),
                user_repo(3, 7, b"ada", b"work", false, true),
            ]))
        ),
        kept(&[1, 3], false)
    );
}
#[test]
#[expect(clippy::disallowed_macros, reason = "the redaction test inspects failure and machine Debug renderings")]
fn no_token_is_in_a_connected_failure_debug() {
    for phase in API_READ..=CONNECTED_LISTING {
        let mut machine = connected_at(phase);
        assert!(skein_lib::bytes::find(format!("{machine:?}").as_bytes(), API).is_none());
        let failure = step(&mut machine, Event::Begin);
        assert!(skein_lib::bytes::find(format!("{failure:?}").as_bytes(), API).is_none());
        assert!(format!("{machine:?}").contains("token: None"));
    }
    let mut machine = connected_at(CONNECTED_LISTING);
    step(&mut machine, user_repos(user_rows(0, true)));
    assert!(format!("{machine:?}").contains("token: None"));
}
