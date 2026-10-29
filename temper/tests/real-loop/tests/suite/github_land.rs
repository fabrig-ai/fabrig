//! A chat's change is checked, reviewed and landed by its connected GitHub App.
use temper_real_loop_world::{
    Ran,
    github::{
        self, check_chat_heard_the_landing, check_connection, check_landing, connect_adopt_and_ask_for_a_change,
        scripts,
    },
    landing, llm, run_github,
};

const ONE_PUSH: usize = 1;

#[test]
fn a_chats_change_lands_on_connected_github() {
    let mut fixture = github::fixture();
    fixture.connected = false;
    fixture.configure = |value, agent, at| (landing::fixture().configure)(value, agent, at);
    let ran = run_github(fixture, |at| llm::cued(scripts(at)), connect_adopt_and_ask_for_a_change);
    check_connection(&ran);
    let commit = check_landing(&ran);
    check_chat_heard_the_landing(&ran, &commit);
    check_authenticated_git(&ran);
    let files = files_outside_the_secrets_store(&ran);
    no_installation_token_pattern_or_app_key_outside_the_secrets_store(&ran, &files);
}

fn check_authenticated_git(ran: &Ran) {
    let seen = ran.github();
    assert_eq!(seen.installations, [(github::INSTALLATION, 10, b"acme".to_vec())]);
    assert!(seen.minted >= 1);
    assert_eq!(seen.git_calls.iter().filter(|call| call.writes).count(), ONE_PUSH);
    assert!(seen.git_calls.iter().any(|call| !call.writes && call.status == 200));
    for call in &seen.git_calls {
        let minted = call.minted.expect("installation token on every git call");
        assert_eq!(minted.installation, github::INSTALLATION);
        assert!(minted.mint > 0 && minted.mint <= seen.minted);
        assert!(call.status == 200 || (!call.writes && call.status == 404), "{call:?}");
    }
}

fn no_installation_token_pattern_or_app_key_outside_the_secrets_store(ran: &Ran, files: &[std::path::PathBuf]) {
    use skein_lib::{
        Writer,
        base64::{self, Alphabet},
    };
    let pem = ran.github().app.as_ref().expect("converted App").pem.expose();
    let text = std::str::from_utf8(pem).expect("PEM text");
    let compact: String = text.lines().filter(|line| !line.starts_with("-----")).collect();
    let key = base64::decode(Alphabet::Standard, compact.as_bytes(), 4096).expect("converted key DER");
    let size = base64::encoded_len(Alphabet::Standard, u32::try_from(key.len()).expect("test key length"))
        .expect("base64 length");
    let mut encoded = Writer::new(usize::try_from(size).expect("base64 size"));
    base64::encode(Alphabet::Standard, &key, &mut encoded).expect("base64 key");
    let encoded = encoded.finish();
    let clean = |bytes: &[u8]| {
        assert!(
            !bytes.windows(40).any(|part| part.starts_with(b"ghs_") && part[4..].iter().all(u8::is_ascii_alphanumeric)),
            "minted-token pattern outside jig's secrets store"
        );
        assert!(skein_lib::bytes::find(bytes, pem).is_none(), "App PEM outside jig's secrets store");
        assert!(skein_lib::bytes::find(bytes, &key).is_none(), "App DER outside jig's secrets store");
        assert!(skein_lib::bytes::find(bytes, &encoded).is_none(), "App base64 outside jig's secrets store");
    };
    for path in files {
        clean(&std::fs::read(path).expect("scratch file"));
    }
    for visit in ran.browsed() {
        clean(&visit.body);
    }
    for call in ran.calls() {
        clean(&call.stdout);
        clean(&call.stderr);
    }
}

// Walk once for both checks, never following a symlink into the secrets store.
fn files_outside_the_secrets_store(ran: &Ran) -> Vec<std::path::PathBuf> {
    use std::os::unix::fs::PermissionsExt;
    let secrets = ran.path().join("state/generations/1/secrets");
    assert!(secrets.is_dir(), "the excluded directory is jig's secrets store");
    // The person's original download is the sole deliberate plaintext input.
    // Exclude exactly this path; every other scratch file and all ctl output are scanned.
    let downloaded = ran.path().join("github-app.pem");
    assert_eq!(std::fs::metadata(&downloaded).expect("download").permissions().mode() & 0o777, 0o600);
    let mut files = Vec::new();
    let mut pending = vec![ran.path().to_path_buf()];
    while let Some(path) = pending.pop() {
        if path == secrets || path == downloaded {
            continue;
        }
        let metadata = std::fs::symlink_metadata(&path).expect("scratch metadata");
        if metadata.is_dir() {
            pending.extend(
                std::fs::read_dir(path).expect("scratch directory").map(|entry| entry.expect("scratch entry").path()),
            );
        } else if metadata.is_file() {
            files.push(path);
        }
    }
    files
}
