//! Adversarial GitHub real-loop QA from domain/testing.md, 2.4, story 6.
use skein_fake_llm_domain::api::{Part, Script};
use std::{net::SocketAddr, path::Path};
use temper_real_loop_world::{
    Exit, Ran, argv,
    calls::{self, D1, Step, call, delegate, file, number, sign_in, task, until},
    github, landing, run_github,
};

fn adopt(owner: &'static str, name: &'static str, key: &'static str) -> Step {
    call(
        D1,
        argv![
            "repository",
            "adopt",
            "--project",
            "1",
            "--owner",
            owner,
            "--name",
            name,
            "--landing",
            "main",
            "--key",
            key,
            "--json"
        ],
    )
}

fn account() -> Vec<Step> {
    vec![
        sign_in(D1),
        call(D1, argv!["account", "add-key", "1", file("fixture.key"), "--deployment", "--key", "qa-llm"]),
        call(D1, argv!["account", "list", "--until", "usable"]),
    ]
}

fn delivered(at: SocketAddr) -> Box<[Script]> {
    landing::scripts(at, u64::from(github::REPOSITORY), landing::README)
}

fn successful(ran: &Ran, indices: &[usize]) {
    for &index in indices {
        assert_eq!(
            ran.calls()[index].exit,
            Some(Exit::Code(0)),
            "call {index}: stdout {}; stderr {}",
            String::from_utf8_lossy(&ran.calls()[index].stdout),
            String::from_utf8_lossy(&ran.calls()[index].stderr)
        );
    }
}

fn owner_authenticated(ran: &Ran) {
    assert!(!ran.github().git_calls.is_empty(), "the scenario must reach the authenticated git route");
    for (index, request) in ran.github().git_calls.iter().enumerate() {
        // Git probes a not-yet-created change branch before its first push.
        // That read may be 404; authentication failures may not be hidden by it.
        assert!(request.status == 200 || (!request.writes && request.status == 404), "git call {index}: {request:?}");
        let minted = request.minted.expect("every git call uses an installation token, never an App JWT");
        assert_eq!(minted.installation, github::INSTALLATION, "git call {index}: {request:?}");
        assert!(minted.mint > 0 && minted.mint <= ran.github().minted, "git call {index}: {request:?}");
    }
}

// Fresh deployment: chat T1, change T2, coder T3. The push run's chat is
// its call 4, and the tree read naming the coder its call 6.
const PUSH_CHAT: usize = 4;
const PROCEDURE: u64 = 2;
const CODER: u64 = 3;
const CODER_SEEN: usize = 6;

fn push_run() -> Ran {
    run_github(github::fixture(), delivered, || {
        let mut steps = account();
        steps.extend([
            adopt("acme", "app", "installed"),
            call(D1, argv!["chat", "start", "--project", "1", "--key", "push", "@forge_chat Make one small edit"]),
            call(D1, argv!["task", "show", task(PUSH_CHAT), "--until", "replied"]),
            // Fresh deployment: chat T1, change T2, coder T3. The coder's
            // terminal follows its delivery, without waiting for review/landing.
            // The procedure creates its coder after the chat can already park;
            // discover it before opening a watch on that task.
            until(D1, argv!["task", "show", number(PROCEDURE), "--tree", "--json"], b"\"task\":3", b"\"task\":2"),
            call(D1, argv!["task", "show", delegate(CODER_SEEN), "--until", "ended", "--json"]),
            call(D1, argv!["me", "--json"]),
            call(D1, argv!["task", "show", number(CODER), "--json"]),
            calls::ends_leaving(&[temper_real_loop_world::ending::Leftover {
                task: PROCEDURE,
                may_be: &[
                    temper_real_loop_world::ending::Left::Due,
                    temper_real_loop_world::ending::Left::Preparing,
                    temper_real_loop_world::ending::Left::Claimed,
                    temper_real_loop_world::ending::Left::Running,
                ],
                why: "token and secrecy probes end on the coder's push, before review and landing",
            }]),
        ]);
        steps
    })
}

fn no_git_secret(bytes: &[u8], place: &str) {
    // The fake's public mint contract is ghs_ plus 36 alphanumeric bytes.
    // The Basic prefix also catches an environment header accidentally persisted
    // after base64 encoding, where the raw token is no longer a substring.
    let key = skein_fake_forge_github::keys::KEYS[temper_github_app::APP.key];
    let key_line = key.split(|byte| *byte == b'\n').nth(1).expect("public fixture PEM body");
    for needle in [b"ghs_".as_slice(), b"eC1hY2Nlc3MtdG9rZW46Z2hz".as_slice(), key_line] {
        assert_eq!(calls::occurrences(bytes, needle), 0, "git secret exposed in {place}");
    }
}

fn scan_files(path: &Path, secrets: &Path, count: &mut usize) {
    let kind = std::fs::symlink_metadata(path).expect("scratch entry").file_type();
    if kind.is_dir() {
        for entry in std::fs::read_dir(path).expect("scratch directory") {
            scan_files(&entry.expect("scratch entry").path(), secrets, count);
        }
    } else if kind.is_file() {
        *count += 1;
        let bytes = std::fs::read(path).expect("scratch file");
        if path.starts_with(secrets) {
            // protocol/credentials.md, 4: installation tokens are memory-only, even
            // though the App's key is allowed in jig's secrets store.
            assert_eq!(calls::occurrences(&bytes, b"ghs_"), 0, "installation token persisted in jig's secrets");
        } else {
            no_git_secret(&bytes, path.to_str().expect("scratch path"));
        }
    }
}

/// protocol/credentials.md, 7; domain/testing.md, 6: no git token or App key in observable state outside jig's secrets.
#[test]
fn git_secrets_stay_out_of_files_client_answers_and_agent_context() {
    let ran = push_run();
    successful(&ran, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    owner_authenticated(&ran);
    assert!(ran.github().git_calls.iter().any(|request| request.writes), "scan must follow a real delivery");
    for result in ran.calls() {
        no_git_secret(&result.stdout, "ctl stdout");
        no_git_secret(&result.stderr, "ctl stderr");
    }
    for seen in ran.llm() {
        let skein_fake_peers::llm::Observation::Query { query, .. } = seen else { continue };
        no_git_secret(&query.system, "agent system prompt");
        for part in query.messages.iter().flat_map(|message| message.parts.iter()) {
            match part {
                Part::Opaque { bytes } => no_git_secret(bytes, "agent opaque context"),
                Part::Text { text } => no_git_secret(text, "agent text"),
                Part::ToolCall { id, name, arguments } => {
                    no_git_secret(id, "agent tool call id");
                    no_git_secret(name, "agent tool call name");
                    no_git_secret(arguments, "agent tool call arguments");
                }
                Part::ToolOutput { id, output, .. } => {
                    no_git_secret(id, "agent tool output id");
                    no_git_secret(output, "agent tool output");
                }
            }
        }
    }
    let mut files = 0;
    assert!(ran.path().join("state/generations/1/store").is_dir(), "main store is part of the scan");
    assert!(ran.path().join("state/transcripts").is_dir(), "agent transcripts are part of the scan");
    scan_files(ran.path(), &ran.path().join("state/generations/1/secrets"), &mut files);
    assert!(files > 0, "the scan must inspect persisted files");
}
