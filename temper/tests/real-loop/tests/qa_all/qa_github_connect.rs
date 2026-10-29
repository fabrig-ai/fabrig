//! Adversarial public-client stories; client/README.md 8 and protocol/setup.md 17.3–4.
use std::net::SocketAddr;

use serde_json::Value;
use temper_real_loop_world::{
    Exit, Ran,
    calls::{self, Arg, Argv, D1, Step},
    github, run_github,
};

// Every story installs the App on one account; replays and races must not add a second.
const ONE_INSTALLATION: usize = 1;

fn fresh() -> github::Fixture {
    github::Fixture { connected: false, ..github::fixture() }
}

fn no_agents(_: SocketAddr) -> Box<[skein_fake_llm_domain::api::Script]> {
    Box::new([])
}

fn call(words: &[&'static str]) -> Step {
    calls::call(D1, Argv::new(words.iter().map(|word| Arg::Lit(word)).collect()))
}

fn connect(app: Arg, key: &'static str) -> Step {
    calls::call(
        D1,
        Argv::new(vec![
            Arg::Lit("forge"),
            Arg::Lit("connect"),
            Arg::Lit("1"),
            Arg::Lit("github-app"),
            app,
            calls::file("github-app.pem"),
            Arg::Lit("--key"),
            Arg::Lit(key),
        ]),
    )
}

fn begin() -> Vec<Step> {
    vec![
        calls::sign_in(D1),
        connect(calls::github_app(), "qa-connect"),
        call(&["forge", "list", "--until", "connected:1", "--json"]),
    ]
}

fn adopt(owner: &'static str, name: &'static str, key: &'static str) -> Step {
    call(&[
        "repository",
        "adopt",
        "--project",
        "1",
        "--forge",
        "1",
        "--owner",
        owner,
        "--name",
        name,
        "--landing",
        "main",
        "--key",
        key,
        "--json",
    ])
}

fn json(ran: &Ran, at: usize) -> Value {
    let called = &ran.calls()[at];
    let bytes = if called.stdout.is_empty() { &called.stderr } else { &called.stdout };
    serde_json::from_slice(bytes).unwrap_or_else(|error| {
        panic!(
            "call {at} JSON: {error}; stdout {}; stderr {}",
            String::from_utf8_lossy(&called.stdout),
            String::from_utf8_lossy(&called.stderr)
        )
    })
}

fn row(document: &Value) -> &Value {
    // `wait` wraps its current document with `met`; a direct read does not.
    let document = if document.get("met").is_some() { &document["forges"] } else { document };
    document["forges"]
        .as_array()
        .unwrap_or_else(|| panic!("forge rows: {document}"))
        .iter()
        .find(|row| row["number"] == 1)
        .expect("configured GitHub forge")
}

fn succeeded(ran: &Ran, at: usize) {
    let called = &ran.calls()[at];
    assert_eq!(
        called.exit,
        Some(Exit::Code(0)),
        "call {at}: {} {}",
        String::from_utf8_lossy(&called.stdout),
        String::from_utf8_lossy(&called.stderr)
    );
}

fn no_git_writes_or_key_leaks(ran: &Ran) {
    assert!(ran.github().git_calls.iter().all(|call| !call.writes), "no git mutation from connection/adoption");
    let pem = ran.github().app.as_ref().expect("fake-made App").pem.expose();
    let body_line = pem.split(|byte| *byte == b'\n').find(|line| line.len() == 64).expect("PEM body line");
    let check = |bytes: &[u8], context: &str| {
        assert!(
            !bytes.windows(body_line.len()).any(|part| part == body_line),
            "no PEM body outside secrets: {context}"
        );
        for der in skein_fake_forge_github::keys::KEYS {
            assert!(!bytes.windows(der.len()).any(|part| part == der), "no DER outside secrets: {context}");
        }
    };
    for called in ran.calls() {
        check(&called.stdout, "ctl stdout");
        check(&called.stderr, "ctl stderr");
    }
    for page in ran.browsed() {
        check(&page.body, "browser body");
    }
    // Only this test's nonsecret state and client trees; never open the secrets store.
    let mut pending = vec![ran.path().join("state"), ran.path().join("clients")];
    let mut files = 0;
    while let Some(path) = pending.pop() {
        if path == ran.path().join("state/generations/1/secrets") {
            continue;
        }
        if path.is_dir() {
            for entry in std::fs::read_dir(path).expect("test state directory") {
                pending.push(entry.expect("test state entry").path());
            }
        } else {
            check(&std::fs::read(&path).expect("test state file"), path.to_str().expect("test path"));
            files += 1;
        }
    }
    assert!(files > 0, "persisted state was checked");
}

#[test]
fn a_failed_paste_is_named_in_human_status_and_cleared_by_retry() {
    // client/forge.md 5.4: a stopped run stays on the row until the next connect.
    const HUMAN_FAILED: usize = 3;
    const INSTALL: usize = 4;
    const HUMAN_RETRIED: usize = 8;
    let ran = run_github(fresh(), no_agents, || {
        vec![
            calls::sign_in(D1),
            connect(calls::number(u64::MAX), "qa2-failed-paste"),
            call(&["forge", "list", "--until", "settled", "--json"]),
            call(&["forge", "list"]),
            call(&["forge", "install", "1", "--key", "qa2-install-too-soon", "--json"]),
            call(&["forge", "list", "--json"]),
            connect(calls::github_app(), "qa2-retry-paste"),
            call(&["forge", "list", "--until", "connected:1", "--json"]),
            call(&["forge", "list"]),
        ]
    });
    no_git_writes_or_key_leaks(&ran);
    for at in [0, 1, 2, HUMAN_FAILED, 5, 6, 7, HUMAN_RETRIED] {
        succeeded(&ran, at);
    }
    let failed = json(&ran, 2);
    let why = row(&failed)["ended"].as_str().unwrap_or_else(|| panic!("last connect reason: {failed}"));
    assert!(!why.is_empty() && why.len() <= 256);
    let text = String::from_utf8_lossy(&ran.calls()[HUMAN_FAILED].stdout);
    assert_eq!(ran.calls()[INSTALL].exit, Some(Exit::Code(2)));
    let refusal = json(&ran, INSTALL);
    assert!(refusal.to_string().contains("\"not_connected\""), "install refused as not connected: {refusal}");
    assert_eq!(row(&json(&ran, 5))["ended"], row(&failed)["ended"], "admission refusal keeps prior run's reason");
    let retried = json(&ran, 7);
    assert_eq!(row(&retried)["state"], "connected");
    assert!(row(&retried)["ended"].is_null());
    assert!(!String::from_utf8_lossy(&ran.calls()[HUMAN_RETRIED].stdout).contains("The last connect stopped:"));
    assert!(ran.github().installations.is_empty());
    assert_eq!(ran.github().minted, 0);
    assert!(
        text.contains("The last connect stopped:") && text.contains(why),
        "human last connect omits stored reason {why:?}: {text}"
    );
}

#[test]
fn refused_and_replayed_pastes_keep_keys_out_of_persisted_public_state() {
    // protocol/credentials.md 2.2: the carried PEM and DER never enter public records.
    const REFUSED: usize = 7;
    let ran = run_github(fresh(), no_agents, || {
        vec![
            calls::sign_in(D1),
            connect(calls::number(u64::MAX), "qa2-secrecy-failed"),
            call(&["forge", "list", "--until", "settled", "--json"]),
            connect(calls::number(u64::MAX), "qa2-secrecy-failed"),
            connect(calls::github_app(), "qa2-secrecy-ok"),
            call(&["forge", "list", "--until", "connected:1", "--json"]),
            connect(calls::github_app(), "qa2-secrecy-ok"),
            connect(calls::github_app(), "qa2-secrecy-refused"),
            call(&["forge", "install", "1", "--key", "qa2-secrecy-install"]),
            calls::browse(8),
            call(&["forge", "list", "--until", "settled", "--json"]),
        ]
    });
    no_git_writes_or_key_leaks(&ran);
    for at in [0, 1, 2, 3, 4, 5, 6, 8, 9] {
        succeeded(&ran, at);
    }
    assert_eq!(ran.calls()[REFUSED].exit, Some(Exit::Code(2)));
    let pem = ran.github().app.as_ref().expect("fake App").pem.expose();
    let body: Vec<_> = pem
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.starts_with(b"-----"))
        .flatten()
        .copied()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect();
    assert!(body.len() > 100, "nonempty fake key body");
    let mut pending = vec![ran.path().to_path_buf()];
    let mut checked = 0;
    while let Some(path) = pending.pop() {
        // Test input is intentionally private; the secrets store is never opened.
        if path.file_name().is_some_and(|name| name == "secrets" || name == "github-app.pem") {
            continue;
        }
        let metadata = std::fs::symlink_metadata(&path).expect("scratch metadata");
        if metadata.is_dir() {
            for entry in std::fs::read_dir(&path).expect("scratch directory") {
                pending.push(entry.expect("scratch entry").path());
            }
        } else if metadata.is_file() {
            let bytes = std::fs::read(&path).expect("public scratch artifact");
            assert!(!bytes.windows(body.len()).any(|part| part == body), "no unwrapped PEM body in {}", path.display());
            assert!(
                !bytes.windows(b"BEGIN RSA PRIVATE KEY".len()).any(|part| part == b"BEGIN RSA PRIVATE KEY"),
                "no PEM armour in {}",
                path.display()
            );
            for der in skein_fake_forge_github::keys::KEYS {
                assert!(!bytes.windows(der.len()).any(|part| part == der), "no DER in {}", path.display());
            }
            checked += 1;
        }
    }
    assert!(checked > 0, "nonsecret persisted files checked");
    assert_eq!(ran.github().installations.len(), ONE_INSTALLATION);
}

#[test]
#[ignore = "gap: Ran exposes successful mint count but no fake token fingerprint or redacted leakage observer"]
fn installation_tokens_stay_out_of_logs_records_and_saved_client_state() {
    // protocol/credentials.md 2.2; protocol/forge.md 2.4–5: no token outside secrets.
    let ran = run_github(fresh(), no_agents, || {
        let mut steps = begin();
        steps.extend([
            call(&["forge", "install", "1", "--key", "qa2-token-secrecy"]),
            calls::browse(3),
            call(&["forge", "list", "--until", "settled", "--json"]),
            adopt("acme", "app", "qa2-token-adopt"),
        ]);
        steps
    });
    no_git_writes_or_key_leaks(&ran);
    for at in 0..ran.calls().len() {
        succeeded(&ran, at);
    }
    assert!(ran.github().minted > 0, "a token was exercised");
    // An observer must compare emitted artifacts to the fake's minted tokens
    // without opening any credential record or jig's secrets store.
    panic!("gap: the real-loop result exposes no token leakage observation");
}
