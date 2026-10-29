use super::{Listing, Shared, Unusable, check, largest_row, page_size};
use crate::tests::fixtures;
use alloc::boxed::Box;
use skein_forge::api;
use skein_lib::{List, Time};

#[test]
fn each_listing_uses_its_largest_answer_row() {
    let limits = fixtures::limits();
    for (listing, row) in [
        (Listing::Items, fixtures::ROW_ITEMS),
        (Listing::Reviews, fixtures::ROW_REVIEWS),
        (Listing::Statuses, fixtures::ROW_STATUSES),
        (Listing::Remarks, fixtures::ROW_REMARKS),
        (Listing::Branches, fixtures::ROW_BRANCHES),
        (Listing::Collaborators, fixtures::ROW_COLLABORATORS),
        (Listing::PullFiles, fixtures::ROW_PULLFILES),
        (Listing::Compare, fixtures::ROW_COMPARE),
        (Listing::Runs, fixtures::ROW_RUNS),
        (Listing::Tree, fixtures::ROW_TREE),
    ] {
        assert_eq!(
            skein_forge::measure::answer(&maximal_answer(listing)),
            Some(u64::from(row)),
            "{listing:?}: independent oracle"
        );
        assert_eq!(largest_row(listing, &limits), Some(u64::from(row)), "{listing:?}: production bound");
        for (answer_bytes, rows, cap, expected) in [
            (row - 1, fixtures::PAGE_ROWS, fixtures::PAGE_ROWS, fixtures::NO_PAGE_ROWS),
            (row, fixtures::PAGE_ROWS, fixtures::PAGE_ROWS, fixtures::ONE_PAGE_ROW),
            (fixtures::PAGE_ROWS * row, fixtures::PAGE_ROWS, fixtures::PAGE_ROWS, fixtures::PAGE_ROWS),
            (fixtures::PAGE_ROWS * row, fixtures::PAGE_ROWS, fixtures::ONE_PAGE_ROW, fixtures::ONE_PAGE_ROW),
            (fixtures::PAGE_ROWS * row, fixtures::ONE_PAGE_ROW, fixtures::PAGE_ROWS, fixtures::ONE_PAGE_ROW),
        ] {
            let shared = Shared { rows, answer_bytes, ..fixtures::shared() };
            assert_eq!(page_size(listing, cap, &limits, &shared), expected, "{listing:?}/{answer_bytes}/{rows}/{cap}");
        }
    }
}
#[test]
fn limits_check_admits_one_largest_row_of_every_listing() {
    let limits = fixtures::limits();
    let settings = fixtures::settings();
    for (answer_bytes, expected) in
        [(fixtures::MAX_ROW - 1, Err(Unusable::PageSize { listing: Listing::Items })), (fixtures::MAX_ROW, Ok(()))]
    {
        let shared = Shared { answer_bytes, ..fixtures::shared() };
        assert_eq!(check(&limits, &shared, &settings), expected);
    }
}
fn maximal_answer(listing: Listing) -> api::Answer {
    let limits = fixtures::limits();
    let string = usize::try_from(limits.string_bytes).expect("fixture string");
    let body = usize::try_from(limits.body_bytes).expect("fixture body");
    match listing {
        Listing::Runs => api::Answer::Runs { runs: Box::new([maximal_run(string)]), more: false },
        Listing::Tree => tree(string),
        Listing::Items => {
            let mut labels = List::with_capacity(limits.inner);
            for _ in 0..limits.inner {
                labels.push(fixtures::filled(string, b'l')).expect("labels");
            }
            api::Answer::Items {
                items: Box::new([api::Summary {
                    labels: labels.into_boxed(),
                    number: 1,
                    kind: api::Kind::Issue,
                    state: api::State::Open,
                    author: fixtures::WRITER,
                    key: Some(fixtures::filled(body / 2, b'k')),
                    title: fixtures::filled(string, b't'),
                    body: fixtures::filled(body, b'b'),
                    updated: Time::ZERO,
                }]),
                more: false,
                now: Time::ZERO,
            }
        }
        Listing::Reviews => api::Answer::Reviews {
            reviews: Box::new([api::Review {
                id: 1,
                provenance: api::Provenance::Original,
                revision: 0,
                author: fixtures::WRITER,
                verdict: api::Verdict::Approve,
                commit: fixtures::BRANCH_HEAD,
                key: Some(fixtures::filled(body / 2, b'k')),
                body: fixtures::filled(body, b'b'),
                at: Time::ZERO,
                official: false,
            }]),
            more: false,
        },
        Listing::Statuses => api::Answer::Statuses {
            ci: api::Ci::Passed,
            statuses: Box::new([api::Status {
                at: Time::ZERO,
                context: fixtures::filled(string, b'c'),
                check: api::Check::Passed,
                description: fixtures::filled(string, b'd'),
                url: fixtures::filled(string, b'u'),
                job: None,
            }]),
            more: false,
        },
        Listing::Remarks => api::Answer::Remarks {
            remarks: Box::new([api::Remark {
                id: 1,
                author: fixtures::WRITER,
                path: fixtures::filled(string, b'p'),
                line: 1,
                body: fixtures::filled(body, b'b'),
            }]),
            more: false,
        },
        Listing::Branches => api::Answer::Branches(Box::new([fixtures::filled(string, b'p')])),
        Listing::Collaborators => api::Answer::Collaborators {
            collaborators: Box::new([api::Collaborator { user: fixtures::WRITER, permission: api::Permission::Write }]),
            more: false,
        },
        Listing::Compare => api::Answer::Compare {
            before: [0; 32],
            after: fixtures::BRANCH_HEAD,
            contains_before: true,
            commits: Box::new([fixtures::BRANCH_HEAD]),
            files: Box::new([]),
        },
        Listing::PullFiles => api::Answer::PullFiles {
            head: fixtures::BRANCH_HEAD,
            files: Box::new([api::File {
                path: fixtures::filled(string, b'p'),
                before: Some(fixtures::filled(body, b'b')),
                after: Some(fixtures::filled(body, b'a')),
            }]),
            more: false,
        },
    }
}

#[test]
fn connect_checks_name_the_refused_value() {
    use crate::{Access, Value};
    use skein_forge::credential::Credential;
    use skein_lib::Secret;
    let token = Credential::Token(Secret::new(Box::from(fixtures::TOKEN)));
    for (label, credential, user, listener, secret, expected) in [
        (
            "App",
            Credential::App { id: 1, key: Secret::new(Box::new([])) },
            fixtures::WRITER,
            true,
            None,
            Err(Value::Credential),
        ),
        ("empty token", Credential::Token(Secret::new(Box::new([]))), fixtures::WRITER, true, None, Err(Value::Token)),
        (
            "token bound",
            Credential::Token(Secret::new(fixtures::filled(fixtures::TOKEN_BOUND, b'a'))),
            fixtures::WRITER,
            true,
            None,
            Ok(()),
        ),
        (
            "token past bound",
            Credential::Token(Secret::new(fixtures::filled(fixtures::TOKEN_BOUND + 1, b'a'))),
            fixtures::WRITER,
            true,
            None,
            Err(Value::Token),
        ),
        (
            "token control",
            Credential::Token(Secret::new(Box::from(b"a\n".as_slice()))),
            fixtures::WRITER,
            true,
            None,
            Err(Value::Token),
        ),
        ("zero user", token.clone(), 0, true, None, Err(Value::User)),
        (
            "no listener",
            token.clone(),
            fixtures::WRITER,
            false,
            Some(Secret::new(Box::from(fixtures::HOOK_SECRET))),
            Err(Value::Secret),
        ),
        ("empty secret", token.clone(), fixtures::WRITER, true, Some(Secret::new(Box::new([]))), Err(Value::Secret)),
        (
            "secret bound",
            token.clone(),
            fixtures::WRITER,
            true,
            Some(Secret::new(fixtures::filled(fixtures::SECRET_BOUND, b'a'))),
            Ok(()),
        ),
        (
            "secret past bound",
            token,
            fixtures::WRITER,
            true,
            Some(Secret::new(fixtures::filled(fixtures::SECRET_BOUND + 1, b'a'))),
            Err(Value::Secret),
        ),
    ] {
        let access = Access { credential, user, webhook_secret: secret };
        assert_eq!(
            super::access(&fixtures::limits(), crate::settings::Kind::Forgejo, listener, &access),
            expected,
            "{label}"
        );
    }
}

#[test]
fn a_github_app_access_is_accepted() {
    let access = fixtures::app_access(skein_fake_forge_github::keys::KEYS.first().expect("test key"), 7, 9);
    assert_eq!(super::access(&fixtures::limits(), crate::settings::Kind::GitHub, false, &access), Ok(()));
}

#[test]
fn a_github_access_names_the_refused_value() {
    use crate::{Access, Value, settings::Kind};
    use skein_forge::credential::Credential;
    use skein_lib::Secret;
    let l = fixtures::limits();
    let key = *skein_fake_forge_github::keys::KEYS.first().expect("test key");
    let overlong =
        fixtures::filled(usize::try_from(l.key_bytes).expect("key bound").checked_add(1).expect("one past"), 0);
    let mut secret = fixtures::app_access(key, 7, 9);
    secret.webhook_secret = Some(Secret::new(Box::from(fixtures::HOOK_SECRET)));
    for (label, access, listener, expected) in [
        ("key past bound", fixtures::app_access(&overlong, 7, 9), false, Value::Key),
        ("key will not sign", fixtures::app_access(&[0; 64], 7, 9), false, Value::Key),
        (
            "token",
            Access {
                credential: Credential::Token(Secret::new(Box::from(fixtures::TOKEN))),
                user: 9,
                webhook_secret: None,
            },
            false,
            Value::Credential,
        ),
        ("zero user", fixtures::app_access(key, 7, 0), false, Value::User),
        ("secret with listener", secret, true, Value::Secret),
        ("key before user", fixtures::app_access(&[0; 64], 7, 0), false, Value::Key),
    ] {
        assert_eq!(super::access(&l, Kind::GitHub, listener, &access), Err(expected), "{label}");
    }
}

#[test]
fn an_app_credential_on_a_forgejo_forge_is_the_other_dialects() {
    for key in [*skein_fake_forge_github::keys::KEYS.first().expect("test key"), &[0; 64]] {
        let access = fixtures::app_access(key, 7, 9);
        assert_eq!(
            super::access(&fixtures::limits(), crate::settings::Kind::Forgejo, false, &access),
            Err(crate::Value::Credential)
        );
    }
}

#[test]
fn github_document_depth_needs_ten_only_for_github() {
    for (depth, expected) in
        [(9, Err(Unusable::GitHubDocuments(skein_forge::github::limits::Problem::Depth))), (10, Ok(()))]
    {
        let limits = super::Limits { depth, ..fixtures::limits() };
        assert_eq!(check(&limits, &fixtures::shared(), &fixtures::github_settings()), expected);
        assert_eq!(check(&limits, &fixtures::shared(), &fixtures::settings()), Ok(()));
    }
}

#[test]
fn installation_pages_use_their_own_full_page_budget() {
    let l = super::Limits { installations: 1, ..fixtures::limits() };
    let s = Shared { rows: 1, answer_bytes: fixtures::MAX_ROW, name_bytes: 4, ..fixtures::shared() };
    assert_eq!(
        super::installation_bounds(&l, &s),
        Some(skein_forge::github::plans::plan::Bounds {
            inner: l.inner,
            page_size: 100,
            rows: 100,
            answer_bytes: 2000,
            body_bytes: fixtures::BODY_ROOM,
        })
    );
    assert_eq!(check(&l, &s, &fixtures::github_settings()), Ok(()));
}
#[test]
fn installation_page_budget_refuses_checked_overflow() {
    const CAP: u32 = skein_forge::github::pin::PAGE_CAP;
    const OVERHEAD: u32 = 16; // measure::STRING_BYTES, the dialect's per-login overhead.
    const NAME_BOUND: u32 = u32::MAX / CAP - OVERHEAD;
    let l = fixtures::limits();
    let at = Shared { name_bytes: NAME_BOUND, ..fixtures::shared() };
    let past = Shared { name_bytes: NAME_BOUND + 1, ..at };
    assert_eq!(super::installation_bounds(&l, &at).expect("at derived bound").answer_bytes, 4_294_967_200);
    assert_eq!(super::installation_bounds(&l, &past), None);
    assert_eq!(super::installation_bounds(&l, &Shared { name_bytes: u32::MAX, ..at }), None);
    assert_eq!(check(&l, &past, &fixtures::github_settings()), Err(Unusable::Zero));
    assert_eq!(
        check(&l, &past, &fixtures::settings()),
        Err(Unusable::StringBytes { string_bytes: l.string_bytes, name_bytes: past.name_bytes })
    );
}
fn maximal_run(string: usize) -> api::Run {
    api::Run {
        id: 1,
        number: 1,
        workflow: fixtures::filled(string, b'w'),
        commit: fixtures::BRANCH_HEAD,
        event: api::Event::Dispatch,
        actor: fixtures::WRITER,
        state: api::RunState::Waiting,
        conclusion: None,
        url: fixtures::filled(string, b'u'),
        created: Time::ZERO,
        jobs: Box::new([]),
    }
}
fn tree(string: usize) -> api::Answer {
    api::Answer::Tree {
        entries: Box::new([api::TreeEntry {
            name: fixtures::filled(string, b'n'),
            mode: 0o100_644,
            kind: api::TreeEntryKind::Blob,
            object: fixtures::BRANCH_HEAD,
        }]),
        more: false,
    }
}

#[test]
fn valid_accepts_test_limits() {
    assert_eq!(check(&fixtures::limits(), &fixtures::shared(), &fixtures::settings()), Ok(()));
}

fn refuses_limits(limits: super::Limits, shared: Shared, settings: crate::Settings) {
    let error = check(&limits, &shared, &settings).expect_err("unrepresentable limits");
    assert_eq!(crate::Component::new(settings, &limits, &shared).err(), Some(error));
}

#[test]
fn valid_refuses_an_unrepresentable_http_head() {
    refuses_limits(super::Limits { head_bytes: 1, ..fixtures::limits() }, fixtures::shared(), fixtures::settings());
}

#[test]
fn valid_refuses_unrepresentable_tls_intake() {
    let mut limits = fixtures::limits();
    limits.tls.read = u32::MAX;
    refuses_limits(limits, fixtures::shared(), fixtures::settings());
}

#[test]
fn valid_refuses_a_json_wire_length_past_u32() {
    refuses_limits(
        super::Limits { body_bytes_in: u32::MAX, ..fixtures::limits() },
        fixtures::shared(),
        fixtures::settings(),
    );
}

#[test]
fn valid_refuses_a_hook_wire_length_past_u32() {
    refuses_limits(
        super::Limits { hook_body_bytes: u32::MAX, ..fixtures::limits() },
        fixtures::shared(),
        fixtures::settings(),
    );
}

#[test]
fn valid_refuses_socket_slots_past_u32() {
    let limits = super::Limits { connections: u32::MAX, ..fixtures::limits() };
    let shared = Shared { calls: u32::MAX, ..fixtures::shared() };
    assert!(super::capacity_bound(&limits, &shared, super::Forges { forgejo: 0, github: 1 }).is_none());
    refuses_limits(limits, shared, fixtures::github_settings());
}

#[test]
fn valid_refuses_staging_slots_past_u32() {
    let limits = super::Limits { inner: u32::MAX, ..fixtures::limits() };
    assert!(super::capacity_bound(&limits, &fixtures::shared(), super::Forges { forgejo: 1, github: 0 }).is_none());
    refuses_limits(limits, fixtures::shared(), fixtures::settings());
}

#[test]
fn valid_refuses_a_log_page_past_u32() {
    let limits = super::Limits { string_bytes: u32::MAX, ..fixtures::limits() };
    assert!(super::capacity_bound(&limits, &fixtures::shared(), super::Forges { forgejo: 0, github: 1 }).is_none());
    refuses_limits(limits, fixtures::shared(), fixtures::github_settings());
}

#[test]
fn valid_ignores_heap_call_tables() {
    let limits = super::Limits { connections: u32::MAX, ..fixtures::limits() };
    let shared = Shared { calls: u32::MAX, rows: u32::MAX, answer_bytes: u32::MAX, ..fixtures::shared() };
    assert_eq!(check(&limits, &shared, &fixtures::settings()), Ok(()));
}
