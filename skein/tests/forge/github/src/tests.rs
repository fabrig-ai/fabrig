use super::*;
use crate::{
    directory::{Directory, transcript},
    fake::World,
    journal::{Journal, Made},
    live::Fixtures,
};
use skein_fake_machine::{How, Item, Machine};
use skein_forge::github::{document as github_document, request::Method};
use skein_forge_conformance::write::{self, SecretKind, Secrets};
use std::collections::BTreeSet;

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

#[test]
fn github_conformance_names_every_fact_of_forge_github_13() {
    let doc = include_str!("../../../../docs/design/forge/github.md");
    let section = doc.split("## 13.").nth(1).expect("section 13").split("## 14.").next().expect("section end");
    let mut names: BTreeSet<_> = section.split('`').enumerate().filter(|(i, _)| i % 2 == 1).map(|(_, s)| s).collect();
    names.remove("unchecked");
    names.retain(|s| s.contains('-') && !s.contains('/'));
    names.insert("installation-repository-removal-refusals");
    assert_eq!(cases::NAMES.iter().copied().collect::<BTreeSet<_>>(), names);
    assert_eq!(cases::NAMES.len(), names.len());
}
#[test]
fn github_conformance_stops_on_a_leaked_secret() {
    let mut machine = Machine::new();
    let root = machine.lay(&[]);
    let mut out = Directory::fake(machine, root, "");
    for kind in
        [SecretKind::AppKey, SecretKind::PersonToken, SecretKind::Jwt, SecretKind::MintedToken, SecretKind::HookSecret]
    {
        let secrets = Secrets::of(&[(kind, "invented-secret-value")]);
        let mut m = mask::Mask::default();
        let unexpected = m.body(br#"{"unexpected":"invented-secret-value"}"#, false);
        let leak = transcript(&mut out, "unexpected-field", b"GET / HTTP/1.1\r\n", &unexpected, &secrets)
            .expect_err("leak refused");
        let error = leak.to_string();
        assert!(error.contains("unexpected-field"));
        assert!(error.contains(&format!("{kind:?}")));
        assert!(!error.contains("invented-secret-value"));
        assert!(!out.exists("transcripts"));
    }
}
#[test]
fn github_conformance_journal_keeps_synced_records_after_a_crash() {
    const THIRD: &str = "make\t3\tmint\t1\t100\n";
    let secrets = Secrets::of(&[]);
    let mut saw_torn_tail = false;
    for seed in 0..=7 {
        let mut machine = Machine::new();
        let root = machine.lay(&[]);
        let mut out = Directory::fake(machine, root, "");
        let mut journal = Journal::create(&mut out, false, &secrets).expect("new journal");
        for installation in [1, 2] {
            journal.before(&mut out, Made::Mint { installation, at: 100 }, &secrets).expect("synced intention");
        }
        let durable = out.read_text("made.txt").expect("synced bytes");
        let Directory::Fake(tree) = &mut out else { panic!("fake directory") };
        let file = tree.machine.open(tree.root, b"made.txt", How::Write).expect("open journal");
        tree.machine.append(file, THIRD.as_bytes()).expect("unsynced third intention");
        tree.machine.close(file);
        tree.machine.crash(seed);
        tree.root = tree.machine.reopen_root(0);

        let survived = out.read_text("made.txt").expect("crashed journal");
        let tail = survived.strip_prefix(&durable).unwrap_or_else(|| panic!("seed {seed}: lost a synced intention"));
        let complete = tail == THIRD;
        assert!(THIRD.starts_with(tail), "seed {seed}: only a prefix of the unsynced record can survive");
        saw_torn_tail |= !complete && !tail.is_empty();
        let (reloaded, live) = Journal::load(&mut out).expect("a torn tail is cut, not an error");
        assert!(!live);
        let ids: Vec<_> = reloaded.pending.iter().map(|(id, _)| *id).collect();
        assert_eq!(ids, if complete { &[1, 2, 3][..] } else { &[1, 2][..] }, "seed {seed}");
        assert_eq!(out.read_text("made.txt").expect("recovered journal"), if complete { survived } else { durable });
    }
    assert!(saw_torn_tail, "the seeds exercise a torn third record");
}

fn private_fixture(mode: u32, replaced: Option<(&str, &str)>, path: &str) -> Directory {
    let mut items = vec![Item::directory(b"fixtures").mode(mode), Item::file(b"fixtures/ignored", b"unneeded")];
    let fixtures = World::fixtures().expect("invented fixtures");
    let app = br#"{"id":8,"slug":"example-app","name":"Example App","client_id":"invented-client","node_id":"invented-node","html_url":"https://example-app.invalid/","permissions":{"contents":"write"},"events":["push"],"created_at":"2026-01-01T00:00:00Z","owner":"acme","installations":[{"id":1,"account":"acme","repositories":["acme/widgets"]},{"id":2,"account":"globex","repositories":["globex/gadgets"]}],"spare_repository":"acme/spare","hook_url":"https://example-app.invalid/","api_version":"2026-03-10"}"#;
    // Pin the invented document to the same dialect constant as the runner.
    let app =
        std::str::from_utf8(app).expect("metadata text").replace("2026-03-10", skein_forge::github::pin::API_VERSION);
    for (name, bytes) in [
        ("app.json", app.as_bytes()),
        ("app.pem", fixtures.pem.as_bytes()),
        ("webhook_secret", fixtures.webhook.as_bytes()),
        ("person-acme.token", fixtures.people["acme"].as_bytes()),
        ("person-globex.token", fixtures.people["globex"].as_bytes()),
    ] {
        let path = format!("fixtures/{name}");
        let path = path.as_bytes();
        match replaced {
            Some((entry, "file")) if entry == name => items.push(Item::file(path, bytes).mode(0o644)),
            Some((entry, "link")) if entry == name => {
                items.push(Item::file(b"fixtures/invented-target", bytes).mode(0o600));
                items.push(Item::link(path, b"invented-target"));
            }
            Some((entry, "non-file")) if entry == name => items.push(Item::directory(path)),
            _ => items.push(Item::file(path, bytes).mode(0o600)),
        }
    }
    items.push(Item::link(b"link", b"fixtures"));
    let mut machine = Machine::new();
    let root = machine.lay(&items);
    Directory::fake(machine, root, path)
}
#[test]
fn github_conformance_live_refuses_a_loose_directory_or_file() {
    let mut valid = private_fixture(0o700, None, "fixtures");
    assert!(Fixtures::load(&mut valid).is_ok(), "valid invented fixtures refused");
    for name in ["app.json", "app.pem", "webhook_secret", "person-acme.token", "person-globex.token"] {
        for kind in ["file", "link", "non-file"] {
            let mut out = private_fixture(0o700, Some((name, kind)), "fixtures");
            let attempted = Fixtures::load(&mut out).map(|_| -> () {
                panic!("connector reached after insecure fixture");
            });
            assert!(attempted.is_err());
        }
    }
    assert!(Fixtures::load(&mut private_fixture(0o755, None, "fixtures")).is_err());
    assert!(Fixtures::load(&mut private_fixture(0o700, None, "link")).is_err());
}
#[test]
fn github_conformance_live_refuses_targets_outside_the_fixtures() {
    let fixtures = Fixtures::load(&mut private_fixture(0o700, None, "fixtures"))
        .expect("invented private fixtures")
        .guard_fixtures();
    let delete =
        br#"{"query":"mutation { deleteRepository(input: {repositoryId: \"invented\"}) { clientMutationId } }"}"#
            .as_slice();
    let mixed = br#"{"query":"mutation { enqueuePullRequest(input: {pullRequestId: \"invented\"}) { clientMutationId } deleteRepository(input: {repositoryId: \"invented\"}) { clientMutationId } }"}"#.as_slice();
    for (method, verb, target, body) in [
        (Method::Post, "POST", "/repos/acme/other/pulls", b"".as_slice()),
        (Method::Get, "GET", "/repos/acme/other", b""),
        (Method::Get, "GET", "/repos/other/widgets/pulls", b""),
        (Method::Get, "GET", "/repos/Acme/widgets", b""),
        (Method::Get, "GET", "/repos/acme/Widgets", b""),
        (Method::Delete, "DELETE", "/repos/acme/widgets", b""),
        (Method::Patch, "PATCH", "/repos/acme/widgets?ignored=1", b""),
        (Method::Post, "POST", "/repos/acme/spare", b""),
        (Method::Put, "PUT", "/repos/globex/gadgets", b""),
        (Method::Delete, "DELETE", "/repos/acme/widgets/", b""),
        (Method::Post, "POST", "/user/repos", b""),
        (Method::Patch, "PATCH", "/user", b""),
        (Method::Put, "PUT", "/user/installations/999/repositories/1", b""),
        (Method::Delete, "DELETE", "/user/installations/999/repositories/1", b""),
        (Method::Post, "POST", "/user/installations/1/repositories/1", b""),
        (Method::Put, "PUT", "/user/installations/1/repositories/not-digits", b""),
        (Method::Put, "PUT", "/user/installations/1/repositories/1/extra", b""),
        (Method::Put, "PUT", "/user/installations/1/repositories/", b""),
        (Method::Put, "PUT", "/user/installations//repositories/1", b""),
        (Method::Put, "PUT", "/user/installations/18446744073709551616/repositories/1", b""),
        (Method::Delete, "DELETE", "/users/acme", b""),
        (Method::Patch, "PATCH", "/repositories/2", b""),
        (Method::Post, "POST", "/graphql", delete),
        (Method::Post, "POST", "/graphql", mixed),
        (Method::Get, "GET", "/graphql", br#"{"query":"query { viewer { login databaseId } }"}"#),
        (Method::Post, "POST", "/graphql", br#"{"query":"subscription { viewer { login } }"}"#),
        (Method::Post, "POST", "/graphql", br#"{"query":"query { viewer { login } }"}"#),
        (Method::Post, "POST", "/graphql", br#"{"query":"query { viewer { login databaseId } } "}"#),
        (Method::Post, "POST", "/graphql", b"{}"),
        (Method::Post, "POST", "/graphql", br#"{"query":42}"#),
        (Method::Post, "POST", "/graphql", b"invalid JSON"),
        (Method::Get, "GET", "/repos/acme/widgets%2F..", b""),
        (Method::Get, "GET", "/repos/ac%6de/widgets", b""),
        (Method::Get, "GET", "/repos/acme%2Fother/widgets", b""),
        (Method::Get, "GET", "/repos//widgets", b""),
        (Method::Get, "GET", "/repos/acme/", b""),
        (Method::Get, "GET", "/repos/./widgets", b""),
        (Method::Get, "GET", "/repos/../widgets", b""),
        (Method::Get, "GET", "/repos/acme/.", b""),
        (Method::Get, "GET", "/repos/acme/..", b""),
        (Method::Get, "GET", "/repos/acme/widgets/../other", b""),
        (Method::Get, "GET", "/repos/acme/widgets#ignored", b""),
        (Method::Get, "GET", "/repos/acme/widgets\\other", b""),
        (Method::Get, "GET", "/repos/acme", b""),
        (Method::Get, "GET", "/user-other", b""),
        (Method::Get, "GET", "/installation/token", b""),
        (Method::Post, "POST", "/installation/repositories", b""),
        (Method::Get, "GET", "/installation/repositories-other", b""),
    ] {
        let error = live::guarded(&fixtures, method, target, body, || {
            panic!("connector called for a refused target");
        })
        .expect_err("target must be refused before the connector");
        let path = target.split('?').next().expect("path");
        assert_eq!(error.to_string(), format!("live target refuses {verb} {path} before the connector"));
    }
}
