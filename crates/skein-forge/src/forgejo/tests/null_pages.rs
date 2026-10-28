//! A null top-level list is Forgejo's empty page, with the normal shape checks around it.
use super::{Box, Document, Malformed, Request, client, fixtures};
use crate::forgejo::request::Page;

fn lists() -> [Request; 17] {
    let page = Page { page: Some(3), limit: Some(2) };
    [
        Request::PullFiles { at: fixtures::repository(), number: 1, page },
        Request::Issues { at: fixtures::repository(), state: None, kind: None, since: None, sort: None, page },
        Request::Comments { at: fixtures::repository(), number: 1 },
        Request::Pulls { at: fixtures::repository(), page },
        Request::Reviews { at: fixtures::repository(), number: 1, page },
        Request::ReviewComments { at: fixtures::repository(), number: 1, review: 1 },
        Request::Branches { at: fixtures::repository(), prefix: None, page },
        Request::Branches { at: fixtures::repository(), prefix: Some(Box::from(*b"work")), page },
        Request::Protections { at: fixtures::repository() },
        Request::Collaborators { at: fixtures::repository(), page },
        Request::UserRepos { page },
        Request::Jobs { at: fixtures::repository(), run: 1 },
        Request::Teams { org: Box::from(*b"fixture"), page },
        Request::TeamMembers { team: 1, page },
        Request::OrgHooks { org: Box::from(*b"fixture"), page },
        Request::RepoHooks { at: fixtures::repository(), page },
        Request::Tokens { login: Box::from(*b"fixture"), page },
    ]
}

#[test]
fn null_list_pages_decode_like_empty_arrays() {
    let limits = fixtures::limits();
    for request in lists() {
        for body in [b"null".as_slice(), b"[]"] {
            let decoded = fixtures::decode(client::decoder(&request, &limits).unwrap(), body, &limits);
            assert_eq!(decoded.document, Ok(Document::Listed { skipped: 0 }), "{request:?}/{body:?}");
            assert!(decoded.rows.is_empty(), "{request:?}/{body:?}");
        }
    }
}

#[test]
fn null_list_pages_still_refuse_wrong_shapes_rows_and_trailing_json() {
    let limits = fixtures::limits();
    for request in lists() {
        for body in [b"{}".as_slice(), b"false", b"[null]", b"null []"] {
            let decoded = fixtures::decode(client::decoder(&request, &limits).unwrap(), body, &limits);
            assert!(decoded.document.is_err(), "{request:?}/{body:?}");
        }
    }
}

#[test]
fn null_does_not_replace_an_object_document_or_required_nested_list() {
    let limits = fixtures::limits();
    for request in [Request::User, Request::RepoSearch { page: Page { page: Some(3), limit: Some(2) } }] {
        let decoded = fixtures::decode(client::decoder(&request, &limits).unwrap(), b"null", &limits);
        assert_eq!(decoded.document, Err(Malformed::Shape { field: b"" }), "{request:?}");
    }
    let request = Request::Compare {
        at: fixtures::repository(),
        before: crate::forgejo::id::read(b"0300000000000000000000000000000000000000").unwrap(),
        after: crate::forgejo::id::read(b"0100000000000000000000000000000000000000").unwrap(),
        files: true,
    };
    let decoded = fixtures::decode(
        client::decoder(&request, &limits).unwrap(),
        br#"{"total_commits":0,"commits":[],"files":null}"#,
        &limits,
    );
    assert_eq!(decoded.document, Err(Malformed::Missing { field: b"files" }));
}
