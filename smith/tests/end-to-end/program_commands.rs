//! The product hands sign-in to the executable fake, then uses the host half.
use super::support;
use smith_fake_claude::{OBSERVATIONS_FILE, Observation, observations};
use smith_real_world::Scratch;

pub(super) const HOST_VARIABLES: [(&str, &str); 7] = [
    ("ANTHROPIC_API_KEY", "key-for-work"),
    ("ANTHROPIC_AUTH_TOKEN", "token-for-work"),
    ("CLAUDE_CODE_OAUTH_TOKEN", "token-for-work"),
    ("CLAUDE_CODE_USE_BEDROCK", "1"),
    ("AWS_BEARER_TOKEN_BEDROCK", "token-for-work"),
    ("CLAUDE_CONFIG_DIR", "/elsewhere"),
    ("TERM", "dumb"),
];

pub(super) fn with_credentials(scratch: &Scratch, arguments: &[&str]) -> support::Ran {
    let ran = support::smith_with_environment(
        scratch,
        arguments,
        b"",
        HOST_VARIABLES.map(|(name, value)| (name.into(), value.into())).into(),
    );
    for bytes in [&ran.stdout, &ran.stderr] {
        let words = String::from_utf8_lossy(bytes);
        // The cloud selector is a switch, not a distinctive credential value.
        for value in ["key-for-work", "token-for-work", "/elsewhere"] {
            assert!(!words.contains(value), "host credential value absent from output");
        }
    }
    ran
}

pub(super) fn credential_free(invocation: &Observation) {
    let names = &invocation.environment;
    for (name, _) in &HOST_VARIABLES[..5] {
        assert!(!names.iter().any(|kept| kept == name), "program kept {name}");
    }
    assert_eq!(names.iter().filter(|name| *name == "CLAUDE_CONFIG_DIR").count(), 1, "one account directory setting");
    assert_eq!(
        names.iter().filter(|name| name.starts_with("ANTHROPIC_")).map(String::as_str).collect::<Vec<_>>(),
        ["ANTHROPIC_CONFIG_DIR"],
        "only the source's Anthropic profiles setting reaches the program"
    );
}

fn configured_program() -> (Scratch, std::path::PathBuf) {
    let scratch = Scratch::new();
    let executable = support::fake_claude();
    let settings = format!(
        "[programs]\nclaude_code='{}'\n[accounts.work]\nprovider='anthropic'\nsource='program'\n",
        executable.display()
    );
    let _file = support::user_layer(&scratch, settings.as_bytes());
    let configuration = scratch.path().join("state/smith/programs/work");
    (scratch, configuration)
}

fn signed_in_program() -> (Scratch, std::path::PathBuf) {
    use std::os::unix::fs::DirBuilderExt;

    let (scratch, configuration) = configured_program();
    // Private whatever the umask, as smith makes it at a login.
    std::fs::DirBuilder::new().recursive(true).mode(0o700).create(&configuration).expect("program directory");
    std::fs::write(configuration.join("fake-claude.login"), b"claude.ai").expect("signed-in fixture");
    (scratch, configuration)
}

#[test]
fn program_login_checks_status_without_inheriting_host_credentials() {
    let (scratch, configuration) = configured_program();
    let login = with_credentials(&scratch, &["login", "work"]);
    assert_eq!(login.code, 0);
    assert!(configuration.join("fake-claude.login").exists());
    let login_observations = std::fs::read(configuration.join(OBSERVATIONS_FILE)).expect("login observations");
    let calls = observations(&login_observations);
    assert_eq!(calls.len(), 2, "login and its status check");
    assert_eq!(calls[0].arguments, ["auth", "login"]);
    assert_eq!(calls[1].arguments, ["auth", "status"]);
    for call in calls {
        credential_free(&call);
    }
}

#[test]
fn program_logout_removes_the_login_and_checks_status() {
    let (scratch, configuration) = signed_in_program();
    let logout = support::run_product(&scratch, &["logout", "work"], b"", None);
    assert_eq!(logout.code, 0);
    assert!(!configuration.join("fake-claude.login").exists());
    let bytes = std::fs::read(configuration.join(OBSERVATIONS_FILE)).expect("logout observations");
    let arguments: Vec<_> = observations(&bytes).into_iter().map(|record| record.arguments).collect();
    assert_eq!(arguments, [["auth", "logout"], ["auth", "status"]]);
}
