use super::*;
use crate::fake::World;
use skein_forge::github::{document as github_document, request::Method};
use skein_forge_conformance::write::{self, SecretKind, Secrets};

#[test]
fn github_conformance_close_delimited_waits_for_end_and_bounds_the_body() {
    let request = b"GET /app/hook/deliveries HTTP/1.1\r\n\r\n";
    let mut response = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n".to_vec();
    response.extend_from_slice(b"[]");
    assert!(crate::wire::read(request, &response, false).expect("wait for EOF").is_none());
    assert_eq!(crate::wire::read(request, &response, true).expect("EOF").expect("body").body, b"[]");
    response.truncate(response.len() - 2);
    response.resize(response.len() + 4_194_304, b' ');
    assert_eq!(
        crate::wire::read(request, &response, true).expect("body at bound").expect("body").body.len(),
        4_194_304
    );
    response.push(b' ');
    assert_eq!(
        crate::wire::read(request, &response, false).err().expect("over bound").to_string(),
        "response exceeds conformance bound"
    );
    for response in [b"HTTP/1.1 204 No Content\r\n\r\n".as_slice(), b"HTTP/1.1 304 Not Modified\r\n\r\n".as_slice()] {
        let response = crate::wire::read(request, response, false).expect("bodyless answer").expect("no EOF needed");
        assert!(response.body.is_empty() && !response.close_delimited());
    }
}
#[test]
fn github_conformance_masks_secrets_and_ids() {
    let mut m = mask::Mask::default();
    let head = m.headers(b"GET /repositories/987654321/pulls/735 HTTP/1.1\r\nAuthorization: Bearer invented-secret\r\nETag: W/\"invented-tag\"\r\nX-GitHub-Request-Id: invented-request\r\nLocation: https://invented.invalid/log?signature=invented-signature\r\n");
    for value in ["invented-secret", "invented-tag", "invented-request", "invented-signature", "987654321", "/735"] {
        assert!(!head.windows(value.len()).any(|w| w == value.as_bytes()), "header mask missed a category");
    }
    let body = m.body(br#"{"id":987654321,"number":735,"node_id":"PR_invented","guid":"aabbccdd-1234-5678-9012-0123456789ab","sha":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","token":"invented-minted","pem":"invented-key","webhook_secret":"invented-hook","expires_at":"2026-10-20T20:00:00Z"}"#, true);
    let text = std::str::from_utf8(&body).expect("masked JSON text");
    assert!(text.contains("\"id\":1,\"number\":2"));
    for value in
        ["PR_invented", "aabbccdd", "aaaaaaaa", "invented-minted", "invented-key", "invented-hook", "2026-10-20"]
    {
        assert!(!text.contains(value), "body mask missed a category");
    }
    assert_eq!(m.body(br#"{"id":735,"repository_id":987654321}"#, false), br#"{"id":2,"repository_id":1}"#);
    let query = m.body(
        br#"{"query":"mutation { enqueuePullRequest(input:{pullRequestId:\"PR_invented\"}) { clientMutationId } }"}"#,
        false,
    );
    assert!(!std::str::from_utf8(&query).expect("query").contains("PR_invented"));
    let date = m.headers(b"Date: Tue, 20 Oct 2026 20:00:00 GMT\r\n");
    assert!(std::str::from_utf8(&date).expect("header").contains("Wed, 01 Jan 2020"));
    assert_eq!(m.body(br#"{"databaseId":987654321}"#, false), br#"{"databaseId":1}"#);
    let signed = m.headers(b"GET /signed/blob?signature=invented-signature HTTP/1.1\r\n");
    assert!(!signed.windows(18).any(|v| v == b"invented-signature"));
}
#[test]
fn github_conformance_live_refuses_the_apps_changes_before_any_byte() {
    let fixtures = World::fixtures().expect("invented fixtures").guard_fixtures();
    for (method, path) in [
        (Method::Delete, "/app/installations/123"),
        (Method::Patch, "/app/hook/config"),
        (Method::Post, "/app-manifests/invented/conversions"),
        (Method::Post, "/settings/apps/new"),
        (Method::Patch, "/app"),
        (Method::Post, "/app/installations"),
        (Method::Put, "/app/hook/config"),
        (Method::Post, "/apps/example-app/installations/new"),
    ] {
        assert!(
            live::guarded(&fixtures, method, path, b"", || -> Result<()> {
                panic!("connector called for App mutation");
            })
            .is_err()
        );
    }
    for (method, path) in [
        (Method::Get, "/app"),
        (Method::Get, "/app/hook/config"),
        (Method::Post, "/app/installations/123/access_tokens"),
        (Method::Delete, "/installation/token"),
    ] {
        assert!(live::guarded(&fixtures, method, path, b"", || Ok(())).is_ok());
    }
}
#[test]
fn github_conformance_live_allows_fixture_requests() {
    let fixtures = World::fixtures().expect("invented fixtures").guard_fixtures();
    for (method, path) in [
        (Method::Get, "/repos/acme/widgets?per_page=1"),
        (Method::Get, "/repos/globex/gadgets"),
        (Method::Get, "/repos/acme/spare"),
        (Method::Post, "/repos/acme/widgets/pulls"),
        (Method::Patch, "/repos/globex/gadgets/issues/1"),
        (Method::Delete, "/repos/acme/spare/git/refs/heads/conformance-owned"),
        (Method::Get, "/repos/acme/widgets/branches/conformance%2Fslash"),
        (Method::Get, "/user"),
        (Method::Get, "/user/31"),
        (Method::Get, "/users/acme"),
        (Method::Get, "/repositories/2"),
        (Method::Put, "/user/installations/1/repositories/6"),
        (Method::Delete, "/user/installations/2/repositories/4?ignored=1"),
        (Method::Get, "/installation/repositories?per_page=100&page=1"),
    ] {
        assert!(live::guarded(&fixtures, method, path, b"", || Ok(())).is_ok(), "{method:?} {path}");
    }
    for document in [github_document::VIEWER, github_document::PULL, github_document::ENQUEUE] {
        let document = std::str::from_utf8(document).expect("constant GraphQL document");
        let body = skein_forge_conformance::json::object(&[("query", &document)]).expect("GraphQL body");
        assert!(live::guarded(&fixtures, Method::Post, "/graphql", &body, || Ok(())).is_ok());
    }
}
#[test]
fn github_conformance_checks_logged_payload_signatures() {
    let payload = br#"{"nested":[{"body":"a } brace and \" quote"}]}"#;
    let signature = skein_forge::github::delivery::sign(b"invented-hook-secret", payload);
    let mut body = format!(
        "{{\"request\":{{\"headers\":{{\"X-Hub-Signature-256\":\"{}\"}},\"payload\":",
        std::str::from_utf8(&signature).expect("signature text")
    )
    .into_bytes();
    body.extend_from_slice(payload);
    body.extend_from_slice(b"}}");
    let response = crate::wire::Response { status: 200, head: vec![], body };
    assert!(response.logged_signature(b"invented-hook-secret").expect("logged payload"));
    assert!(!response.logged_signature(b"different-invented-secret").expect("wrong secret"));
    let mut secrets = Secrets::of(&[]);
    live::register(&mut secrets, SecretKind::HookSecret, "invented\nsecret");
    assert!(write::check("escaped", br#"{"unexpected":"invented\nsecret"}"#, b"", &secrets).is_err());
}
