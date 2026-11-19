use github_conformance::suite_support::{
    Fixtures, Runner, Target, cases,
    fake::{self, World},
    journal::{Journal, Made},
};
use github_conformance::*;
use skein_forge::{
    api,
    github::{document as github_document, request::Method},
};
use skein_forge_conformance::{
    facts::Outcome,
    write::{self, SecretKind, Secrets},
};
use skein_scratch::Scratch;
use std::{
    collections::BTreeSet,
    fs,
    io::Write as _,
    os::unix::fs::{PermissionsExt, symlink},
    path::Path,
};
fn runner(out: &Path) -> Runner<'_> {
    let mut w = World::new().expect("hosted fixture");
    w.record_wire = true;
    let fixtures = World::fixtures().expect("invented fixtures");
    Runner::new(Target::Fake(w), fixtures, out).expect("runner").for_suite()
}
fn shutdown(r: &mut Runner<'_>) {
    if let Target::Fake(w) = &mut r.target {
        w.shutdown().expect("quiescent simulator");
    }
}
fn scan(dir: &Path, secrets: &Secrets) {
    for entry in fs::read_dir(dir).expect("output directory") {
        let path = entry.expect("entry").path();
        if path.is_dir() {
            scan(&path, secrets);
        } else {
            let bytes = fs::read(path).expect("output file");
            assert!(write::check("all output", &bytes, b"", secrets).is_ok(), "output contains a registered secret");
        }
    }
}
#[test]
fn github_conformance_holds_every_fact_on_the_fake() {
    // Live omits fourteen fake-only bootstrap, manifest, rename-log and
    // lifecycle requests from the full fake run.
    const FAKE_ONLY_REQUESTS: usize = 14;
    let out = Scratch::new("whole");
    let report = run_fake(out.path()).expect("whole hosted fake run");
    assert_eq!(report.facts.len(), cases::NAMES.len());
    for fact in report.facts {
        match fact.outcome {
            Outcome::Holds => {}
            Outcome::Unchecked(_) => assert_eq!(fact.name, "signatures-push-gh013"),
            Outcome::Observed(reason) | Outcome::Fails(reason) => panic!("{}: {reason}", fact.name),
        }
    }
    assert!(report.requests <= 400);
    // Fake commit creation uses the domain directly. Live creates each recorded
    // commit with four HTTP calls; four head advances also create a temporary
    // branch, patch the head and later delete that temporary branch (six each).
    let journal = fs::read_to_string(out.path().join("made.txt")).expect("invented journal");
    let commits = journal
        .lines()
        .filter(|line| {
            let fields: Vec<_> = line.split('\t').collect();
            matches!(fields.as_slice(), ["make", _, "branch", _, name] if name.starts_with("conformance-branch-"))
        })
        .count();
    // Extend both four-poll head waits to seven, the four-poll group wait
    // and merge wait to eleven, both listings to three tries, and the group
    // branch lookup from one read to thirteen.
    let extra_polls = 2 * 2 * 3 + 2 * 7 + 7 + 2 * 2 + 12;
    let projected_live_requests = report.requests + 4 * commits + 6 * 4 + extra_polls - FAKE_ONLY_REQUESTS;
    assert!(projected_live_requests <= 400, "live request upper bound: {projected_live_requests}");
    let facts = fs::read_to_string(out.path().join("facts.txt")).expect("recorded facts");
    assert!(facts.lines().any(|line| line == "# queue-dequeued-delivered group branch reads 1"));
    assert!(facts.lines().any(|line| line == "# issues-since-epoch-empty listing reads 1"));
    assert_eq!(report.leftover_mints.len(), 0);
}
#[test]
fn github_conformance_renews_the_app_jwt_at_half_life() {
    for (seconds, renewed) in [(299, false), (300, true), (601, true)] {
        let out = Scratch::new(&format!("jwt-age-{seconds}"));
        let mut r = runner(out.path());
        r.start().expect("start fake run");
        let old = r.jwt.clone();
        r.wait(seconds);
        let requests = r.requests;
        let response = r.app(Method::Get, "/app/hook/deliveries?per_page=100", b"").expect("App request");
        assert_eq!(response.status, 200);
        assert_eq!(r.jwt != old, renewed, "renewal after {seconds} seconds");
        assert_eq!(r.requests, requests + 1, "renewal adds no request");
        if seconds > 600 {
            let expired = r.raw(Method::Get, "/app/hook/deliveries?per_page=100", &old, b"").expect("old JWT request");
            assert!(expired.is(401, "Bad credentials"), "fake refuses the expired JWT");
        }
        r.finish().expect("cleanup");
        scan(out.path(), &r.secrets);
        shutdown(&mut r);
    }
}
const EMPTY_LISTING: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n[]";
const INVALID_LENGTH: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: invalid\r\n\r\n";
const EPOCH_LISTING: &str = "/repos/acme/widgets/issues?state=all&since=1970-01-01T00%3A00%3A00Z&sort=updated&direction=asc&per_page=100&page=1";
fn epoch_response(status: u16, body: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 {status} Invented\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    )
    .into_bytes()
}
#[test]
fn github_conformance_faults_fail_only_their_facts_and_the_run_continues() {
    let out = Scratch::new("faults");
    let mut r = runner(out.path());
    let Target::Fake(w) = &mut r.target else { panic!("fake target") };
    w.null_next_merged_pull_commit = true;
    r.start().expect("start fake run");
    r.response_overrides = vec![
        ("log-delivery-whole", "/app/hook/deliveries?per_page=1", EMPTY_LISTING.to_vec()),
        ("log-lists-failed-deliveries", "/app/hook/deliveries?per_page=100", INVALID_LENGTH.to_vec()),
        ("issues-since-epoch-empty", EPOCH_LISTING, epoch_response(503, "[]")),
    ];
    let run = cases::run(&mut r);
    r.complete(run).expect("faults permit later cases and cleanup");
    assert!(r.response_overrides.is_empty(), "every response fault exercised");
    let Target::Fake(w) = &r.target else { panic!("fake target") };
    assert!(!w.null_next_merged_pull_commit, "merged pull fault was exercised");
    assert_eq!(r.facts.len(), cases::NAMES.len());
    for fact in &r.facts {
        let reason = match fact.name {
            "log-delivery-whole" => Some("input absent: delivery id"),
            "log-lists-failed-deliveries" => Some("wire: invalid response length"),
            "issues-since-epoch-empty" => Some("wire observation differs from the fact"),
            "merge-repeat-idempotent" => Some("merged pull plan failed: InvalidAnswer"),
            "signatures-push-gh013" => {
                assert!(matches!(fact.outcome, Outcome::Unchecked(_)), "signature fact must be unchecked");
                continue;
            }
            _ => None,
        };
        let outcome = reason.map_or(Outcome::Holds, |reason| Outcome::Fails(reason.into()));
        assert_eq!(fact.outcome, outcome, "earlier and later fact {}", fact.name);
    }
    for name in ["merge-repeat-any-sha-200", "queue-merged-no-merge-commit-sha"] {
        let fact = r.facts.iter().find(|fact| fact.name == name).expect("later fact recorded");
        assert_eq!(fact.outcome, Outcome::Holds, "{name}");
    }
    let fact = r.facts.iter().find(|fact| fact.name == "merge-repeat-idempotent").expect("repeat merge fact");
    let facts = fs::read_to_string(out.path().join("facts.txt")).expect("recorded facts");
    assert!(facts.contains(&fact.line()));
    assert!(r.cleanup && r.journal.pending.is_empty());
    assert!(r.tokens.is_empty() && r.leftover_mints.is_empty());
    scan(out.path(), &r.secrets);
    shutdown(&mut r);
}
#[test]
fn github_conformance_response_framing_records_the_exchange_and_continues() {
    let rows = [
        (
            "invalid-length",
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: invalid\r\nSet-Cookie: invented-cookie\r\nX-GitHub-Request-Id: invented-request\r\n\r\n".as_slice(),
            Outcome::Fails("wire: invalid response length".into()),
        ),
        (
            "close-delimited",
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n[{\"status_code\":500}]".as_slice(),
            Outcome::Holds,
        ),
    ];
    for (name, response, outcome) in rows {
        let out = Scratch::new(name);
        let mut r = runner(out.path());
        r.start().expect("start fake run");
        r.response_overrides =
            vec![("log-lists-failed-deliveries", "/app/hook/deliveries?per_page=100", response.to_vec())];
        let run = cases::logs(&mut r);
        r.complete(run).expect("framing observation permits later cases and cleanup");
        assert!(r.response_overrides.is_empty(), "response fault exercised");
        assert_eq!(r.facts.len(), 6, "only log facts recorded");
        let fact = r.facts.iter().find(|fact| fact.name == "log-lists-failed-deliveries").expect("log fact");
        assert_eq!(fact.outcome, outcome);
        for fact in &r.facts {
            if fact.name != "log-lists-failed-deliveries" {
                assert_eq!(fact.outcome, Outcome::Holds, "later and earlier fact {}", fact.name);
            }
        }
        let dir = out.path().join("transcripts/log-lists-failed-deliveries");
        let request = fs::read_to_string(dir.join("request")).expect("failed request transcript");
        let response = fs::read_to_string(dir.join("response")).expect("failed response transcript");
        assert!(request.starts_with("GET /app/hook/deliveries?per_page=100 HTTP/1.1\r\n"));
        assert!(response.starts_with("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n"));
        if name == "invalid-length" {
            assert!(response.contains("Content-Length: invalid\r\n"));
            assert!(response.contains("X-GitHub-Request-Id: \"opaque-"));
            assert!(!request.contains("Authorization:"));
            assert!(!response.contains("Set-Cookie:"));
        } else {
            assert!(response.contains("[{\"status_code\":500}]"), "case read the close-delimited body");
            let facts = fs::read_to_string(out.path().join("facts.txt")).expect("recorded facts");
            assert!(facts.lines().any(|line| line == "# log-lists-failed-deliveries close-delimited response 200"));
        }
        assert!(r.cleanup && r.journal.pending.is_empty());
        assert!(r.tokens.is_empty() && r.leftover_mints.is_empty());
        scan(out.path(), &r.secrets);
        shutdown(&mut r);
    }
}
#[test]
fn github_conformance_epoch_refusals_and_non_json_fail_only_the_fact() {
    for (status, body) in [(503, "[]"), (503, "not json"), (200, "not json")] {
        let out = Scratch::new(&format!("epoch-{status}-{}", body.len()));
        let mut r = runner(out.path());
        r.start().expect("start fake run");
        r.response_overrides = vec![("issues-since-epoch-empty", EPOCH_LISTING, epoch_response(status, body))];
        let run = cases::issues_since_epoch(&mut r);
        r.complete(run).expect("epoch observation permits later facts and cleanup");
        assert!(r.response_overrides.is_empty(), "epoch response fault exercised");
        assert_eq!(r.facts.len(), 1, "only the epoch fact recorded");
        let fact = r.facts.iter().find(|fact| fact.name == "issues-since-epoch-empty").expect("epoch fact");
        assert_eq!(fact.outcome, Outcome::Fails("wire observation differs from the fact".into()));
        assert!(r.cleanup && r.journal.pending.is_empty());
        assert!(r.tokens.is_empty() && r.leftover_mints.is_empty());
        shutdown(&mut r);
    }
}
#[test]
fn github_conformance_queue_refusals_fail_one_fact() {
    // Pending approval and merged-pull refusals stay apart: they share a pull.
    let groups = [
        vec![
            ("enqueue-pending-check-protected", "approval", 0, "queue approval refused: Forbidden"),
            ("queue-dequeued-delivered", "approval", 0, "queue approval refused: Forbidden"),
        ],
        vec![
            ("queue-merged-no-merge-commit-sha", "approval", 0, "queue approval refused: Forbidden"),
            ("queue-dequeued-delivered", "checks", 0, "queue ci read failed: Forbidden"),
        ],
        vec![
            ("queue-merged-no-merge-commit-sha", "checks", 0, "queue ci read failed: Forbidden"),
            ("queue-dequeued-delivered", "raw enqueue", 0, "queue admission refused: invented queue admission refusal"),
        ],
        vec![
            ("queue-merged-no-merge-commit-sha", "enqueue", 0, "queue admission refused: Forbidden"),
            ("queue-dequeued-delivered", "branches", 0, "queue branches read failed: Forbidden"),
        ],
        vec![
            ("queue-merged-no-merge-commit-sha", "pull", 0, "merged pull read failed: Forbidden"),
            ("queue-dequeued-delivered", "group", 0, "queue group read failed: Forbidden"),
        ],
        // Four head polls cover 45 seconds; the next checks read is the group's.
        vec![("queue-dequeued-delivered", "checks", 4, "queue ci read failed: Forbidden")],
    ];
    let queue_facts = [
        "queue-direct-merge-405",
        "enqueue-stale-unprocessable",
        "enqueue-pending-check-protected",
        "queue-merged-no-merge-commit-sha",
        "queue-dequeued-delivered",
    ];
    for (index, rows) in groups.into_iter().enumerate() {
        let group =
            rows.iter().map(|(case, kind, skip, _)| format!("{case}/{kind}/{skip}")).collect::<Vec<_>>().join(", ");
        let out = Scratch::new(&format!("queue-refusals-{index}"));
        let mut r = runner(out.path());
        r.start().unwrap_or_else(|_| panic!("{group}: start fake run"));
        for &(case, kind, skip, _) in &rows {
            if kind == "raw enqueue" {
                r.enqueue_refusal = Some(case);
            } else {
                r.plan_refusals.push((case, kind, skip));
            }
        }
        let run = cases::queue(&mut r);
        r.complete(run).unwrap_or_else(|_| panic!("{group}: queue refusal permits later cases and cleanup"));
        for (case, kind, skip, reason) in rows {
            let row = format!("{case}/{kind}/{skip}");
            assert!(
                !r.plan_refusals
                    .iter()
                    .any(|(pending_case, pending_kind, _)| *pending_case == case && *pending_kind == kind)
                    && (kind != "raw enqueue" || r.enqueue_refusal != Some(case)),
                "{row}: refusal exercised"
            );
            assert_eq!(r.facts.len(), queue_facts.len(), "{row}: only queue facts recorded");
            for name in queue_facts {
                let fact = r.facts.iter().find(|fact| fact.name == name);
                assert!(fact.is_some(), "{row}: queue fact {name} recorded");
            }
            let fact = r.facts.iter().find(|fact| fact.name == case).unwrap_or_else(|| panic!("{row}: refusal fact"));
            assert_eq!(fact.outcome, Outcome::Fails(reason.into()), "{row}");
            if kind == "branches" || kind == "group" {
                assert!(
                    r.provenance.iter().any(|line| line == "# queue-dequeued-delivered group branch reads 1"),
                    "{row}: group branch read count recorded on refusal"
                );
            }
            assert!(r.cleanup, "{row}: cleanup ran");
            assert!(r.journal.pending.is_empty(), "{row}: all journal intentions cleaned up");
            assert!(r.tokens.is_empty(), "{row}: all minted tokens revoked");
            assert!(r.leftover_mints.is_empty(), "{row}: no leftover mints");
        }
        assert!(r.plan_refusals.is_empty() && r.enqueue_refusal.is_none(), "{group}: every refusal exercised");
        let Target::Fake(w) = &mut r.target else { panic!("{group}: fake target") };
        w.shutdown().unwrap_or_else(|_| panic!("{group}: quiescent simulator"));
    }
}
#[test]
fn github_conformance_names_every_fact_of_forge_github_13() {
    let root = skein_scratch::paths::tree().join("docs/design/forge/github.md");
    let doc = fs::read_to_string(root).expect("design contract");
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
    let out = Scratch::new("leak");
    for kind in
        [SecretKind::AppKey, SecretKind::PersonToken, SecretKind::Jwt, SecretKind::MintedToken, SecretKind::HookSecret]
    {
        let secrets = Secrets::of(&[(kind, "invented-secret-value")]);
        let mut m = mask::Mask::default();
        let unexpected = m.body(br#"{"unexpected":"invented-secret-value"}"#, false);
        let leak = write::case(out.path(), "unexpected-field", b"GET / HTTP/1.1\r\n", &unexpected, &secrets)
            .expect_err("leak refused");
        let error = leak.to_string();
        assert!(error.contains("unexpected-field"));
        assert!(error.contains(&format!("{kind:?}")));
        assert!(!error.contains("invented-secret-value"));
        assert!(!out.path().join("transcripts").exists());
    }
}
fn private_fixture(dir: &Path) {
    fs::set_permissions(dir, fs::Permissions::from_mode(0o700)).expect("private directory");
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
        fs::write(dir.join(name), bytes).expect("invented fixture file");
        fs::set_permissions(dir.join(name), fs::Permissions::from_mode(0o600)).expect("private file");
    }
}
#[test]
fn github_conformance_live_refuses_a_loose_directory_or_file() {
    let valid = Scratch::new("valid-fixture");
    private_fixture(valid.path());
    fs::write(valid.path().join("ignored"), b"unneeded").expect("ignored loose file");
    assert!(Fixtures::load(valid.path()).is_ok(), "valid invented fixtures refused");
    for name in ["app.json", "app.pem", "webhook_secret", "person-acme.token", "person-globex.token"] {
        for kind in ["file", "link", "non-file"] {
            let out = Scratch::new(&format!("{kind}-{name}"));
            private_fixture(out.path());
            let path = out.path().join(name);
            match kind {
                "file" => fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).expect("loose file"),
                "link" => {
                    let target = out.path().join("invented-target");
                    fs::rename(&path, &target).expect("move invented file");
                    symlink(&target, &path).expect("file link");
                }
                "non-file" => {
                    fs::remove_file(&path).expect("remove invented file");
                    fs::create_dir(&path).expect("directory in file slot");
                }
                _ => unreachable!(),
            }
            let attempted = Fixtures::load(out.path()).map(|_| -> () {
                panic!("connector reached after insecure fixture");
            });
            assert!(attempted.is_err());
        }
    }
    fs::set_permissions(valid.path(), fs::Permissions::from_mode(0o755)).expect("loose directory");
    assert!(Fixtures::load(valid.path()).is_err());
    fs::set_permissions(valid.path(), fs::Permissions::from_mode(0o700)).expect("private directory");
    let link = Scratch::new("directory-link");
    symlink(valid.path(), link.path().join("link")).expect("directory link");
    assert!(Fixtures::load(&link.path().join("link")).is_err());
}
#[test]
fn github_conformance_live_refuses_targets_outside_the_fixtures() {
    let dir = Scratch::new("guard-fixtures");
    private_fixture(dir.path());
    let fixtures = Fixtures::load(dir.path()).expect("invented private fixtures").guard_fixtures();
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
#[test]
fn github_conformance_fake_requests_replay_through_the_live_guard() {
    let out = Scratch::new("guard-replay");
    let mut r = runner(out.path());
    let fixtures = r.fixtures.guard_fixtures();
    r.start().expect("start fake run");
    let run = cases::run(&mut r);
    r.complete(run).expect("complete fake run and cleanup");
    assert_eq!(r.replay.len(), r.requests);
    let mut allowed = 0;
    let mut refused = 0;
    let mut blobs = 0;
    let mut documents = BTreeSet::new();
    for (phase, request) in &r.replay {
        if request.blob {
            blobs += 1;
            continue;
        }
        if *phase == "setup-lifecycle" {
            if request.method != Method::Get {
                assert!(
                    live::guarded(&fixtures, request.method, &request.target, &request.body, || {
                        panic!("connector called for fake-only setup mutation");
                    })
                    .is_err()
                );
                refused += 1;
            }
            continue;
        }
        live::guarded(&fixtures, request.method, &request.target, &request.body, || Ok(()))
            .expect("a case's request was refused by the live guard");
        allowed += 1;
        if request.target == "/graphql" {
            let body = skein_forge_conformance::json::Json::parse(&request.body).expect("GraphQL body");
            documents.insert(body.root().get("query").str().expect("query document").as_bytes().to_vec());
        }
    }
    assert!(allowed > 100);
    assert_eq!(refused, 5, "bootstrap manifest, conversion, hook change and two removals");
    assert!(blobs > 0, "blob redirects keep their separate boundary");
    assert_eq!(
        documents,
        [github_document::VIEWER, github_document::PULL, github_document::ENQUEUE].map(<[u8]>::to_vec).into()
    );
    shutdown(&mut r);
}
#[test]
fn github_conformance_names_a_token_expiring_within_30_days() {
    let out = Scratch::new("expiry");
    let mut r = runner(out.path());
    r.start().expect("start");
    assert!(r.provenance.iter().any(|p| p == "# person-acme.token expires within 30 days"));
    assert!(!r.provenance.iter().any(|p| p.contains("person-globex.token expires")));
    r.finish().expect("cleanup");
    r.report().expect("report");
    scan(out.path(), &r.secrets);
    shutdown(&mut r);
}
#[test]
fn github_conformance_stays_within_its_request_budget() {
    let out = Scratch::new("budget");
    let mut r = runner(out.path());
    r.start().expect("start");
    let token = r.token(0).expect("memory token");
    r.case_limit = r.requests;
    assert!(r.raw(Method::Get, "/installation/repositories?per_page=100&page=1", &token, b"").is_err());
    r.finish().expect("revocation has reserved budget");
    assert!(r.requests <= 400);
    assert!(r.journal.pending.is_empty());
    assert_eq!(
        r.raw(Method::Get, "/installation/repositories?per_page=100&page=1", &token, b"").expect("revoked read").status,
        401
    );
    shutdown(&mut r);
}
#[test]
fn github_conformance_failed_case_revokes_its_tokens() {
    for stop in ["case", "leak"] {
        let out = Scratch::new(stop);
        let mut r = runner(out.path());
        r.start().expect("start");
        let token = r.token(0).expect("memory token");
        let stopped: Result<()> = if stop == "leak" {
            write::check("failed-case", token.as_bytes(), b"", &r.secrets).map_err(Into::into)
        } else {
            Err("deliberately failed case".into())
        };
        assert!(r.complete(stopped).is_err(), "failed run must retain its failure after cleanup");
        assert!(r.journal.pending.is_empty());
        assert_eq!(r.leftover_mints.len(), 0);
        assert_eq!(
            r.raw(Method::Get, "/installation/repositories?per_page=100&page=1", &token, b"")
                .expect("revoked read")
                .status,
            401
        );
        scan(out.path(), &r.secrets);
        shutdown(&mut r);
    }
}
#[test]
fn github_conformance_clean_finishes_a_run_stopped_partway() {
    let out = Scratch::new("stopped");
    let mut r = runner(out.path());
    r.start().expect("start");
    let initial = r.base(0).expect("base");
    let base = r.copy_branch(0, &initial).expect("owned base");
    let (head, _) = r
        .branch(
            0,
            &initial,
            "probe",
            vec![skein_fake_forge_domain::api::File {
                path: fake::boxed(b"probe.txt"),
                content: fake::boxed(b"probe"),
            }],
        )
        .expect("branch");
    r.pull(0, &head, &base, false).expect("pull");
    r.issue(0).expect("issue");
    let repo = r.repo(0).expect("repo");
    let person = r.person(0).expect("person");
    r.before(Made::Rules { repo: repo.clone(), name: "stopped-rule".into() }).expect("write ahead");
    assert_eq!(r.raw(Method::Post, format!("/repos/{repo}/rulesets"), &person, br#"{"name":"stopped-rule","target":"branch","enforcement":"active","conditions":{"ref_name":{"include":["refs/heads/main"],"exclude":[]}},"bypass_actors":[],"rules":[{"type":"pull_request","parameters":{"required_approving_review_count":1,"dismiss_stale_reviews_on_push":false,"require_code_owner_review":false,"require_last_push_approval":false,"required_review_thread_resolution":false}}]}"#).expect("rule").status,201);
    r.before(Made::Access { installation: 1, repository: 6, owner: "acme".into(), add: false }).expect("write ahead");
    assert_eq!(r.raw(Method::Put, "/user/installations/1/repositories/6", &person, b"").expect("grant").status, 204);
    let minted = r.token(0).expect("memory token");
    // A request whose answer never arrived has the same durable mint record.
    r.before(Made::Mint { installation: 1, at: r.now().as_nanos() / 1_000_000_000 }).expect("lost mint reply");
    r.flush().expect("captured partial run");
    // A request-budget stop models the last controlled action before an external kill.
    r.case_limit = r.requests;
    assert!(r.raw(Method::Get, "/app", &person, b"").is_err());
    r.tokens.clear(); // Model a killed process: none of the token values reaches cleanup.
    let oracle = r.secrets;
    let mut r = Runner::resume(r.target, r.fixtures, out.path())
        .expect("fresh cleanup runner with retained hosted state")
        .for_suite();
    r.finish().expect("resumed cleanup");
    r.report().expect("report");
    assert!(r.journal.pending.is_empty());
    assert_eq!(r.leftover_mints.len(), 3);
    assert!(r.leftover_mints.iter().all(|p| p.contains("not revoked, expires by")));
    let journal_text = fs::read_to_string(out.path().join("made.txt")).expect("journal");
    assert!(!journal_text.contains("not revoked"));
    for entry in fs::read_dir(out.path().join("transcripts")).expect("transcripts") {
        let dir = entry.expect("case").path();
        for name in ["request", "response"] {
            assert!(!fs::read_to_string(dir.join(name)).expect("masked transcript").contains("not revoked"));
        }
    }
    assert_eq!(
        r.raw(Method::Get, "/installation/repositories?per_page=100&page=1", &minted, b"")
            .expect("unavailable values cannot revoke")
            .status,
        200
    );
    let branches = r
        .raw(Method::Get, format!("/repos/{repo}/branches?per_page=100&page=1"), &person, b"")
        .expect("branches")
        .json()
        .expect("branches document");
    assert_eq!(branches.root().items().len(), 1);
    assert!(branches.root().nth(0).get("name").is("main"));
    let issues = r
        .raw(
            Method::Get,
            format!("/repos/{repo}/issues?state=all&sort=updated&direction=asc&per_page=100&page=1"),
            &person,
            b"",
        )
        .expect("active objects")
        .json()
        .expect("document");
    assert!(issues.root().items().iter().all(|row| row.get("state").is("closed")));
    let protection = r
        .plan_as(0, api::Op::Read(api::Read::Protection { branch: fake::boxed(b"main") }), true)
        .expect("protection plan");
    assert!(matches!(protection, Ok(api::Answer::Protection(None))));
    assert_eq!(r.raw(Method::Get, "/repos/acme/spare", &minted, b"").expect("restored grant").status, 404);
    scan(out.path(), &oracle);
    shutdown(&mut r);
}

#[test]
fn github_conformance_clean_preserves_facts_and_ignores_an_incomplete_journal_tail() {
    let out = Scratch::new("clean-file");
    let secrets = Secrets::of(&[]);
    let mut journal = Journal::create(out.path(), false, &secrets).expect("journal");
    journal.before(Made::Mint { installation: 123, at: 100 }, &secrets).expect("mint intention");
    journal
        .before(Made::Branch { repo: "acme/widgets".into(), name: "conformance-owned".into() }, &secrets)
        .expect("branch intention");
    let mut file = fs::OpenOptions::new().append(true).open(out.path().join("made.txt")).expect("journal file");
    file.write_all(b"make\t3\tmint\t").expect("partial interrupted write");
    fs::write(out.path().join("facts.txt"), "# target fake\nholds retained-fact\n").expect("prior facts");
    clean(out.path(), None).expect("fresh fake cleanup");
    let facts = fs::read_to_string(out.path().join("facts.txt")).expect("facts");
    assert!(facts.contains("holds retained-fact"));
    assert!(facts.contains("minted token for installation 123: not revoked, expires by 3700"));
    assert!(!fs::read_to_string(out.path().join("made.txt")).expect("journal").contains("not revoked"));
    assert!(Journal::load(out.path()).expect("reloaded").0.pending.is_empty());
    clean(out.path(), None).expect("idempotent cleanup");
    assert_eq!(fs::read_to_string(out.path().join("facts.txt")).expect("facts"), facts);
}

#[test]
fn github_conformance_cleanup_finds_an_issue_after_more_than_one_page() {
    for (known_number, resume) in [(false, false), (false, true), (true, false), (true, true)] {
        let out = Scratch::new(&format!("issue-cleanup-number-{known_number}-resume-{resume}"));
        let mut r = runner(out.path());
        r.start().expect("start");
        let repo = r.repo(0).expect("repository");
        let Target::Fake(w) = &mut r.target else { unreachable!() };
        for i in 0..101 {
            w.call(
                &repo,
                fake::PERSON,
                skein_fake_forge_domain::api::Op::Write(skein_fake_forge_domain::api::Write::CreateIssue {
                    title: fake::boxed(format!("older issue {i}").as_bytes()),
                    body: fake::boxed(b"older fixture"),
                    labels: Box::new([]),
                }),
            )
            .expect("older issue");
        }
        let number = if known_number {
            let number = r.issue(0).expect("made issue");
            assert!(Journal::load(out.path()).expect("durable journal").0.numbers.values().any(|n| *n == number));
            number
        } else {
            let title = "conformance-killed-issue";
            r.before(Made::Issue { repo: repo.clone(), title: title.into() })
                .expect("intention before interrupted reply");
            let Target::Fake(w) = &mut r.target else { unreachable!() };
            let answer = w
                .call(
                    &repo,
                    fake::BOT,
                    skein_fake_forge_domain::api::Op::Write(skein_fake_forge_domain::api::Write::CreateIssue {
                        title: fake::boxed(title.as_bytes()),
                        body: fake::boxed(b"interrupted reply"),
                        labels: Box::new([]),
                    }),
                )
                .expect("issue created without received response");
            let skein_fake_forge_domain::api::Answer::Created(number) = answer else { panic!("created issue") };
            number
        };
        r.wait(4);
        let person = r.person(0).expect("person");
        let Target::Fake(w) = &mut r.target else { unreachable!() };
        w.refuse_issue_patch_for = Some(person.clone());
        let target = format!("/repos/{repo}/issues/{number}");
        assert_eq!(
            r.raw(Method::Patch, &target, &person, br#"{"state":"closed"}"#).expect("person close refusal").status,
            403
        );
        let mut r = if resume {
            r.flush().expect("partial run transcripts");
            Runner::resume(r.target, r.fixtures, out.path())
                .expect("resumed cleanup without token values or JWT")
                .for_suite()
        } else {
            r
        };
        let cleanup_start = r.replay.len();
        r.finish().expect("verified cleanup");
        assert!(r.journal.pending.is_empty());
        let methods: Vec<_> = r.replay[cleanup_start..]
            .iter()
            .filter(|(_, request)| request.target == target)
            .map(|(_, request)| request.method)
            .collect();
        assert_eq!(methods, [Method::Get, Method::Patch, Method::Get], "cleanup reads back its close");
        let read = r.raw(Method::Get, &target, &person, b"").expect("read closed issue");
        assert!(read.json().expect("issue").root().get("state").is("closed"));
        if !known_number {
            assert!(r.replay.iter().any(|(_, request)| request.target.ends_with("page=2")));
        }
        shutdown(&mut r);
    }
}

#[test]
fn github_conformance_cleanup_retains_an_unfound_issue() {
    let out = Scratch::new("missing-cleanup");
    let mut r = runner(out.path());
    r.start().expect("start");
    r.before(Made::Issue { repo: "acme/widgets".into(), title: "conformance-no-reply".into() }).expect("intention");
    assert!(r.finish().is_err());
    assert_eq!(r.journal.pending.len(), 1);
    assert_eq!(Journal::load(out.path()).expect("durable pending intention").0.pending.len(), 1);
    shutdown(&mut r);
}

#[test]
fn github_conformance_transcripts_put_every_status_and_headers_on_their_own_lines() {
    let out = Scratch::new("all-statuses");
    let mut r = runner(out.path());
    r.start().expect("start");
    r.next("many-responses").expect("case");
    let person = r.person(0).expect("person");
    for _ in 0..3 {
        r.raw(Method::Get, "/repos/acme/widgets", &person, b"").expect("response");
    }
    r.flush().expect("transcript");
    let dir = out.path().join("transcripts/many-responses");
    let response = fs::read_to_string(dir.join("response")).expect("responses");
    let request = fs::read_to_string(dir.join("request")).expect("requests");
    assert_eq!(response.lines().filter(|line| line.starts_with("HTTP/1.1 200")).count(), 3);
    assert_eq!(
        response.lines().filter(|line| line.to_ascii_lowercase().starts_with("x-ratelimit-remaining:")).count(),
        3
    );
    assert_eq!(request.lines().filter(|line| line.starts_with("GET ")).count(), 3);
    r.finish().expect("cleanup");
    shutdown(&mut r);
}

#[test]
fn github_conformance_raw_sends_the_supplied_narrowed_token() {
    let out = Scratch::new("raw-narrowed");
    let mut r = runner(out.path());
    r.start().expect("start");
    let full = r.token(0).expect("full token");
    let narrowed = r.mint(0, true).expect("narrowed token");
    assert!(full != narrowed, "minted credentials differ");
    let response = r.raw(Method::Get, "/repos/acme/widgets", &narrowed, b"").expect("raw read");
    assert_eq!(response.status, 200);
    let Target::Fake(w) = &r.target else { panic!("fake target") };
    let expected = format!("Authorization: Bearer {narrowed}\r\n");
    let other = format!("Authorization: Bearer {full}\r\n");
    assert!(
        w.last_wire.windows(expected.len()).any(|part| part == expected.as_bytes()),
        "supplied authorization on wire"
    );
    assert!(!w.last_wire.windows(other.len()).any(|part| part == other.as_bytes()), "full token is not substituted");
    r.finish().expect("cleanup");
    shutdown(&mut r);
}

#[test]
fn github_conformance_discovers_the_organization_before_minting() {
    let out = Scratch::new("organization");
    let mut r = runner(out.path());
    r.start().expect("organization discovered");
    assert_eq!(r.queue_installation, Some(1));
    assert_eq!(r.repo(r.queue_installation.expect("organization index")).expect("probe"), "globex/gadgets");
    assert!(r.replay.iter().take(2).all(|(_, request)| request.method == Method::Get));
    let mut in_queue = false;
    cases::queue(&mut r).expect("cases");
    for (_, request) in &r.replay {
        if request.target.contains("/rulesets") && request.body.windows(11).any(|w| w == b"merge_queue") {
            in_queue = true;
            assert!(request.target.starts_with("/repos/globex/gadgets/"));
        }
    }
    assert!(in_queue);
    r.complete(Ok(())).expect("cleanup");
    shutdown(&mut r);
}
#[test]
fn github_conformance_stops_without_an_organization_before_any_write() {
    let out = Scratch::new("no-organization");
    let world = World::with_organization(false).expect("user-owned repositories");
    let mut r = Runner::new(Target::Fake(world), World::fixtures().expect("fixtures"), out.path()).expect("runner");
    let error = r.start().expect_err("queue requires an organization");
    assert_eq!(error.to_string(), "no organization installation for the merge-queue cases");
    assert!(r.replay.iter().all(|(_, request)| request.method == Method::Get));
    assert!(r.journal.pending.is_empty());
    shutdown(&mut r);
}

#[test]
fn github_conformance_bot_permission_is_not_a_role() {
    let out = Scratch::new("bot-permission");
    let mut r = runner(out.path());
    r.start().expect("start");
    cases::bot_permission_is_not_a_role(&mut r).expect("bot permission fact");
    assert_eq!(r.facts.len(), 1);
    assert_eq!(r.facts[0].name, "bot-permission-is-not-a-role");
    assert_eq!(r.facts[0].outcome, Outcome::Holds);
    r.finish().expect("cleanup");
    shutdown(&mut r);
}
