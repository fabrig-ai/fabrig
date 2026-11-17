use super::fixtures;
use crate::forgejo::{
    client,
    document::Document,
    id,
    request::{FileChange, Method, Request},
};
#[test]
fn change_files_encodes_one_person_commit_and_decodes_its_id() {
    let request = Request::ChangeFiles {
        at: fixtures::repository(),
        branch: Box::from(*b"main"),
        new_branch: None,
        message: Box::from(*b"msg"),
        files: Box::from([
            FileChange { path: Box::from(*b"a.txt"), content: Box::from(*b"x\n"), sha: None },
            FileChange { path: Box::from(*b"b.txt"), content: Box::from(*b"y"), sha: Some(Box::from(*b"blob")) },
        ]),
    };
    let limits = crate::forgejo::limits::Limits { inner: 2, ..fixtures::limits() };
    let encoded = client::encode(&request, &limits).unwrap();
    assert_eq!(encoded.method, Method::Post);
    assert_eq!(&*encoded.target, b"/api/v1/repos/fixture/specimen/contents");
    assert_eq!(encoded.body.as_deref(), Some(br#"{"branch":"main","message":"msg","files":[{"operation":"create","path":"a.txt","content":"eAo="},{"operation":"update","path":"b.txt","sha":"blob","content":"eQ=="}]}"#.as_slice()));
    assert_eq!(
        fixtures::request(encoded.method, &encoded.target, encoded.body.as_deref().unwrap(), &limits),
        Ok(request.clone())
    );
    let decoder = client::decoder(&request, &limits).unwrap();
    assert_eq!(
        fixtures::decode(decoder, br#"{"commit":{"sha":"1111111111111111111111111111111111111111"}}"#, &limits)
            .document,
        Ok(Document::CreatedFile { commit: id::read(b"1111111111111111111111111111111111111111").unwrap() })
    );
    let short =
        crate::forgejo::limits::Limits { writer: skein_json::writer::Limits { length: 1, ..limits.writer }, ..limits };
    assert_eq!(client::encode(&request, &short), Err(client::Refusal::TooLong));
}

#[test]
fn change_files_decoder_refuses_invalid_operations_shas_duplicates_and_bounds() {
    use crate::forgejo::decode::Malformed;
    let limits = fixtures::limits();
    let target = b"/api/v1/repos/fixture/specimen/contents";
    let body = r#"{"branch":"main","message":"msg","files":[{"operation":"create","path":"a","content":"eA=="}]}"#;
    fixtures::request(Method::Post, target, body.as_bytes(), &limits).unwrap();
    for (body, expected) in [
        (body.replace("create", "delete"), Malformed::Value { field: b"operation" }),
        (body.replace("create", "update"), Malformed::Value { field: b"sha" }),
        (body.replace("eA==", "!"), Malformed::Value { field: b"content" }),
        (body.replace(r#""path":"a""#, r#""path":"a","sha":"blob""#), Malformed::Value { field: b"sha" }),
        (
            body.replace(r#""files":[{"operation":"create","path":"a","content":"eA=="}]"#, r#""files":[]"#),
            Malformed::Value { field: b"files" },
        ),
    ] {
        assert_eq!(fixtures::request(Method::Post, target, body.as_bytes(), &limits), Err(expected));
    }
    let two = body.replace(r#""files":[{"operation":"create","path":"a","content":"eA=="}]"#, r#""files":[{"operation":"create","path":"a","content":"eA=="},{"operation":"create","path":"b","content":"eA=="}]"#);
    assert_eq!(
        fixtures::request(Method::Post, target, two.as_bytes(), &limits),
        Err(Malformed::Bound { field: b"files" })
    );
    let limits = crate::forgejo::limits::Limits { inner: 2, ..limits };
    fixtures::request(Method::Post, target, two.as_bytes(), &limits).unwrap();
    assert_eq!(
        fixtures::request(Method::Post, target, two.replace(r#""path":"b""#, r#""path":"a""#).as_bytes(), &limits),
        Err(Malformed::Value { field: b"path" })
    );
}
#[test]
fn server_routes_named_contents_and_status_refs_for_resolution() {
    use crate::forgejo::server;
    let limits = fixtures::limits();
    let commit = id::read(b"1111111111111111111111111111111111111111").unwrap();
    for (target, expected) in [
        (
            b"/api/v1/repos/fixture/specimen/contents/dir/file?ref=work%2Ftopic".as_slice(),
            Request::Contents { at: fixtures::repository(), path: Box::from(*b"dir/file"), reference: commit },
        ),
        (
            b"/api/v1/repos/fixture/specimen/commits/work%2Ftopic/status?page=2&limit=3".as_slice(),
            Request::CombinedStatus {
                at: fixtures::repository(),
                commit,
                page: crate::forgejo::request::Page { page: Some(2), limit: Some(3) },
            },
        ),
    ] {
        let server::Route::Reference(reference) = server::route(Method::Get, target, &limits).unwrap() else {
            panic!("named ref")
        };
        assert_eq!(reference.lookup(), Request::Branch { at: fixtures::repository(), name: Box::from(*b"work/topic") });
        assert_eq!(reference.resolve(commit), expected);
        assert_eq!(server::route(Method::Delete, target, &limits).unwrap_err(), server::Unrouted::Method);
    }
}

#[test]
fn change_files_round_trips_new_branch_in_the_contents_batch() {
    let request = Request::ChangeFiles {
        at: fixtures::repository(),
        branch: Box::from(*b"main"),
        new_branch: Some(Box::from(*b"work/topic")),
        message: Box::from(*b"push"),
        files: Box::from([FileChange { path: Box::from(*b"a"), content: Box::from(*b"x"), sha: None }]),
    };
    let limits = fixtures::limits();
    let encoded = client::encode(&request, &limits).unwrap();
    assert_eq!(encoded.body.as_deref().unwrap(), br#"{"branch":"main","new_branch":"work/topic","message":"push","files":[{"operation":"create","path":"a","content":"eA=="}]}"#);
    assert_eq!(
        fixtures::request(encoded.method, &encoded.target, encoded.body.as_deref().unwrap(), &limits),
        Ok(request)
    );
}
