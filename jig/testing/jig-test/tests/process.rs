//! The shipped service and agent, killed and restarted over one durable store.
use jig_client_protocol_kit::native::Script;
use jig_test_kit as support;
use smith_protocol_program::Status;
use std::{
    ffi::OsString,
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Stdio},
};

fn status_script(path: &Path, line: &str, code: u8) -> OsString {
    let script = format!(
        r#"#!/bin/sh
if [ "$#" -ne 2 ] || [ "$1" != auth ] || [ "$2" != status ]; then
    printf '%s\n' 'expected auth status' >&2
    exit 9
fi
printf '%s\n' '{line}'
exit {code}
"#
    );
    fs::write(path, script).expect("status script written and closed");
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).expect("executable status script");
    path.as_os_str().into()
}

#[test]
fn jig_test_serves_smiths_agent_modes() {
    let directory = support::scratch();
    let signed_in =
        status_script(&directory.path().join("signed-in"), r#"{"loggedIn":true,"authMethod":"claude.ai"}"#, 0);
    let not_signed_in = status_script(&directory.path().join("not-signed-in"), r#"{"loggedIn":false}"#, 1);
    let parent = OsString::from(std::process::id().to_string());
    let rows = [
        (
            "relay usage",
            vec![OsString::from("--mcp-relay")],
            1,
            b"".as_slice(),
            b"smith-agent: usage: smith-agent --mcp-relay 127.0.0.1:PORT, its secret in SMITH_MCP_SECRET\n".as_slice(),
            None,
        ),
        (
            "child refused",
            vec![OsString::from("--program-child")],
            1,
            b"".as_slice(),
            b"smith-agent: program child: missing parent\n".as_slice(),
            None,
        ),
        (
            "signed in",
            vec!["--program-child".into(), parent.clone(), signed_in, "auth".into(), "status".into()],
            0,
            b"{\"loggedIn\":true,\"authMethod\":\"claude.ai\"}\n".as_slice(),
            b"".as_slice(),
            Some(Status::Usable),
        ),
        (
            "not signed in",
            vec!["--program-child".into(), parent, not_signed_in, "auth".into(), "status".into()],
            1,
            b"{\"loggedIn\":false}\n".as_slice(),
            b"".as_slice(),
            Some(Status::NotSignedIn),
        ),
    ];
    let mut results = Vec::new();
    let mut expected = Vec::new();
    for (name, arguments, code, stdout, stderr, status) in rows {
        let mut child = Command::new(skein_scratch::paths::binary("jig-test"))
            .args(arguments)
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("agent mode starts");
        drop(child.stdin.take());
        let output = child.wait_with_output().expect("agent mode ends");
        let exit = output.status.code().expect("agent mode exits without a signal");
        let program_status = status
            .map(|_| smith_protocol_program::status(u8::try_from(exit).expect("program exit code"), &output.stdout));
        results.push((name, exit, output.stdout, output.stderr, program_status));
        expected.push((name, code, stdout.to_vec(), stderr.to_vec(), status));
    }
    assert_eq!(results, expected);
}

/// The fake provider checks the credential presented by the real run.
fn peer() -> (skein_fake_peers::llm::Peer, std::net::SocketAddr, skein_io::kernel::Fd) {
    let mut script = jig_channel_kit::fixtures::one_chat(b"Hello");
    script.result = br#"{"form":"report","text":"Process answer","fields":{}}"#.as_slice().into();
    support::ports::provider(
        Box::new([skein_fake_llm_domain::api::Script {
            cue: Box::default(),
            turns: jig_channel_kit::fixtures::provider_turns(&script),
        }]),
        skein_llm::Credential::bearer(b"token".as_slice().into()),
    )
}

#[test]
fn a_killed_jig_test_finds_its_work_after_restart() {
    let directory = support::scratch();
    support::configuration(directory.path());
    assert_eq!(support::run(directory.path(), Script::answered_chat(b"Hello", support::ANSWER), Some(peer())), (1, 1));
    assert_eq!(support::run(directory.path(), Script::found_chat(support::ANSWER), None), (1, 0));
}

#[path = "support/browser.rs"]
mod browser;
#[test]
fn jig_test_serves_its_bundle_s_page_under_the_reference_policy() {
    let directory = support::scratch();
    browser::configuration(directory.path(), &browser::fixture_bundle(directory.path()));
    let (status, headers, body) = browser::page(directory.path());
    assert_eq!(status, 200);
    assert_eq!(browser::field(&headers, "content-security-policy"), Some(browser::POLICY));
    assert_eq!(body, browser::PAGE);
}

#[test]
fn a_binary_told_to_stop_with_a_sign_in_under_way_ends_at_once() {
    accounts::sigterm();
}

#[path = "support/accounts.rs"]
mod accounts;
