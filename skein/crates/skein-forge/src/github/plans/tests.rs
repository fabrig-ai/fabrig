use super::tables::{self, Method, Tables};
use crate::{api, github::document};
use alloc::boxed::Box;

fn key(repository: u32) -> api::Repository {
    api::Repository { forge: 3, repository }
}
fn document() -> document::Repository {
    document::Repository {
        id: 7,
        owner: Box::from(*b"owner"),
        name: Box::from(*b"name"),
        owner_id: 9_007_199_254_740_993,
        organization: true,
        default_branch: Box::from(*b"main"),
        html_url: Box::from(*b"https://example.invalid/owner/name"),
        merge: true,
        squash: true,
        rebase: true,
    }
}

#[test]
fn github_plans_tables_evict_least_recently_used() {
    let row = tables::repository(&document());
    let mut tables = Tables::new(2, 2);
    tables.keep_repository(key(1), row.clone());
    tables.keep_repository(key(2), row.clone());
    tables.keep_repository(key(3), row.clone());
    assert_eq!(tables.repository(key(1)), None);
    assert_eq!(tables.repository(key(2)), Some(&row));
    tables.keep_repository(key(4), row.clone());
    assert_eq!(tables.repository(key(3)), None);
    assert_eq!(tables.repository(key(2)), Some(&row));
    assert_eq!(tables.repository(key(4)), Some(&row));

    let mut replacement = row.clone();
    replacement.name = Box::from(*b"renamed");
    tables.keep_repository(key(2), replacement.clone());
    assert_eq!(tables.repository(key(2)), Some(&replacement));
    assert_eq!(tables.repository(key(4)), Some(&row));
    tables.drop_repository(key(2));
    tables.drop_repository(key(2));
    assert_eq!(tables.repository(key(2)), None);

    tables.keep_user(3, 1, Box::from(*b"first"));
    tables.keep_user(3, 2, Box::from(*b"second"));
    tables.keep_user(3, 3, Box::from(*b"third"));
    assert_eq!(tables.user(3, 1), None);
    assert_eq!(tables.user(3, 2), Some(b"second".as_slice()));
    tables.keep_user(4, 2, Box::from(*b"other-forge"));
    assert_eq!(tables.user(3, 3), None);
    assert_eq!(tables.user(3, 2), Some(b"second".as_slice()));
    assert_eq!(tables.user(4, 2), Some(b"other-forge".as_slice()));
    tables.keep_user(3, 2, Box::from(*b"renamed"));
    assert_eq!(tables.user(3, 2), Some(b"renamed".as_slice()));
    assert_eq!(tables.user(4, 2), Some(b"other-forge".as_slice()));

    let mut zero = Tables::new(0, 0);
    zero.keep_repository(key(1), row);
    zero.keep_user(3, 1, Box::from(*b"first"));
    assert_eq!(zero.repository(key(1)), None);
    assert_eq!(zero.user(3, 1), None);
    zero.drop_repository(key(1));
}

#[test]
fn github_plans_row_from_document() {
    let mut document = document();
    for (merge, squash, rebase, method) in [
        (true, true, true, Some(Method::Merge)),
        (false, true, true, Some(Method::Squash)),
        (false, false, true, Some(Method::Rebase)),
        (false, false, false, None),
    ] {
        document.merge = merge;
        document.squash = squash;
        document.rebase = rebase;
        assert_eq!(
            tables::repository(&document),
            tables::Repository {
                owner: document.owner.clone(),
                name: document.name.clone(),
                owner_id: document.owner_id,
                organization: document.organization,
                merge: method,
                html_url: document.html_url.clone(),
                settings: api::Settings { default_branch: document.default_branch.clone(), merge, squash, rebase },
            }
        );
    }
}

#[test]
fn github_pull_ci_fold_flags() {
    for (failed, pending, present, expected) in [
        (false, false, false, api::Ci::None),
        (false, false, true, api::Ci::Passed),
        (false, true, false, api::Ci::Pending),
        (false, true, true, api::Ci::Pending),
        (true, false, false, api::Ci::Failed),
        (true, false, true, api::Ci::Failed),
        (true, true, false, api::Ci::Failed),
        (true, true, true, api::Ci::Failed),
    ] {
        assert_eq!(super::reading::ci(failed, pending, present), expected);
    }
}

#[test]
fn github_settings_from_resolved_located_or_settings_row_sends_nothing() {
    use super::{
        Body, Heard,
        plan::{Bounds, Plan, Step},
    };
    use crate::github::{header::Head, request::Request};
    let bounds = Bounds { inner: 8, page_size: 8, rows: 16, answer_bytes: 4096, body_bytes: 1024 };
    for read in [
        api::Read::Branch { branch: Box::from(*b"main") },
        api::Read::Locate { owner: Box::from(*b"owner"), name: Box::from(*b"name") },
        api::Read::Settings,
    ] {
        let mut tables = Tables::new(2, 2);
        let mut plan = Plan::new(key(7), api::Op::Read(read.clone()));
        let request = if let api::Read::Locate { owner, name } = &read {
            Request::Locate { owner: owner.clone(), name: name.clone() }
        } else {
            Request::Repository { id: 7 }
        };
        assert_eq!(plan.begin(bounds, b"app", &mut tables), Ok(Step::send(request)));
        assert_eq!(plan.heard(Heard::Head { status: 200, head: Head::new() }, bounds, b"app", &mut tables), Ok(None));
        let mut repository = document();
        repository.default_branch = Box::from(*b"trunk");
        repository.merge = false;
        repository.rebase = false;
        let step = plan
            .heard(Heard::Done(Body::Document(document::Document::Repository(repository))), bounds, b"app", &mut tables)
            .unwrap();
        if let api::Read::Branch { .. } = read {
            let Some(Step::Send(_)) = step else { panic!("branch after resolution") };
            assert_eq!(
                plan.heard(Heard::Head { status: 200, head: Head::new() }, bounds, b"app", &mut tables),
                Ok(None)
            );
            assert_eq!(
                plan.heard(
                    Heard::Done(Body::Document(document::Document::Branch(document::Branch {
                        name: Box::from(*b"main"),
                        commit: [1; 32],
                    }))),
                    bounds,
                    b"app",
                    &mut tables
                ),
                Ok(Some(Step::Done(Ok(api::Answer::Commit([1; 32])))))
            );
        } else {
            let Some(Step::Done(Ok(_))) = step else { panic!("locate or settings answer") };
        }
        for _ in 0_u8..2 {
            let mut settings = Plan::new(key(7), api::Op::Read(api::Read::Settings));
            assert_eq!(
                settings.begin(bounds, b"app", &mut tables),
                Ok(Step::Done(Ok(api::Answer::Settings(api::Settings {
                    default_branch: Box::from(*b"trunk"),
                    merge: false,
                    squash: true,
                    rebase: false,
                }))))
            );
        }
    }
}
