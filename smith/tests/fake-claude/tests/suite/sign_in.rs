//! Terminal sign-in against the executable fake (smith/docs/design/protocol/programs.md, 4.1).

use skein_scratch::Scratch;
use smith_agent_shell::program::{prepare, sign_in};
use smith_fake_claude::{LOGIN_FILE, OBSERVATIONS_FILE, observations};
use std::{fs, process::Stdio};

#[test]
fn a_sign_in_through_the_fake_writes_its_login() {
    let scratch = Scratch::new("shell-fake-login");
    let configuration = prepare(&scratch.path().join("programs"), "source").expect("source directories");
    let mut command = sign_in(
        &skein_scratch::paths::binary("fake-claude"),
        &configuration,
        [("HOME".into(), scratch.path().as_os_str().into()), ("ANTHROPIC_API_KEY".into(), "x".into())],
    );
    let output = command.stdin(Stdio::null()).stdout(Stdio::piped()).output().expect("fake sign-in");
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(fs::read(configuration.join(LOGIN_FILE)).expect("saved login"), b"claude.ai");
    let records = observations(&fs::read(configuration.join(OBSERVATIONS_FILE)).expect("observations"));
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].arguments, ["auth", "login"]);
    assert_eq!(records[0].environment, ["CLAUDE_CONFIG_DIR", "HOME"]);
    assert!(!scratch.path().join(".claude").exists());
}
