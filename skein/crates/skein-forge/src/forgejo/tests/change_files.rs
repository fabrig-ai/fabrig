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
        message: Box::from(*b"msg"),
        files: Box::from([
            FileChange { path: Box::from(*b"a.txt"), content: Box::from(*b"x\n"), sha: None },
            FileChange { path: Box::from(*b"b.txt"), content: Box::from(*b"y"), sha: Some(Box::from(*b"blob")) },
        ]),
    };
    let limits = fixtures::limits();
    let encoded = client::encode(&request, &limits).unwrap();
    assert_eq!(encoded.method, Method::Post);
    assert_eq!(&*encoded.target, b"/api/v1/repos/fixture/specimen/contents");
    assert_eq!(encoded.body.as_deref(), Some(br#"{"branch":"main","message":"msg","files":[{"operation":"create","path":"a.txt","content":"eAo="},{"operation":"update","path":"b.txt","sha":"blob","content":"eQ=="}]}"#.as_slice()));
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
