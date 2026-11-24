//! Program source startup and check through the shipped binary (smith/docs/design/shell/commands.md, 4, smith/docs/design/shell/settings.md, 1.3).
use super::support;
use smith_fake_claude::{OBSERVATIONS_FILE, Observation, STORY_FILE, Status, Story};
use smith_real_world::Scratch;
use std::os::unix::fs::DirBuilderExt;

pub(super) fn setup(method: Option<&str>, status: Status) -> (Scratch, std::path::PathBuf) {
    let scratch = Scratch::new();
    let executable = support::fake_claude();
    let settings = format!(
        "model='anthropic/claude-haiku-5-5'\nsub_agents=false\n[programs]\nclaude_code='{}'\n[accounts.work]\nprovider='anthropic'\nsource='program'\n[endpoints.anthropic]\npreset='anthropic'\naccount='work'\nprogram='claude_code'\n",
        executable.display()
    );
    let _file = support::user_layer(&scratch, settings.as_bytes());
    let configuration = scratch.path().join("state/smith/programs/work");
    std::fs::DirBuilder::new().recursive(true).mode(0o700).create(&configuration).expect("private configuration");
    if let Some(method) = method {
        std::fs::write(configuration.join("fake-claude.login"), method).expect("fake method");
    }
    std::fs::write(configuration.join(STORY_FILE), Story { status, ..Story::default() }.encode()).expect("fake story");
    (scratch, executable)
}

fn observations(scratch: &Scratch, account: &str) -> Vec<Observation> {
    let bytes = std::fs::read(scratch.path().join("state/smith/programs").join(account).join(OBSERVATIONS_FILE))
        .expect("observations");
    smith_fake_claude::observations(&bytes)
}

fn one_status(scratch: &Scratch, account: &str) {
    let arguments: Vec<_> = observations(scratch, account).into_iter().map(|record| record.arguments).collect();
    assert_eq!(arguments, vec![vec!["auth", "status"]], "the reported state matches the contract");
}

#[test]
fn check_reports_a_signed_in_program_account_as_usable() {
    let (scratch, executable) = setup(Some("claude.ai"), Status::Truthful);
    let ran = support::smith(&scratch, &["check", "--json"], b"");
    assert_eq!(ran.code, 0, "{}", String::from_utf8_lossy(&ran.stderr));
    let report: serde_json::Value = serde_json::from_slice(&ran.stdout).expect("check JSON");
    assert_eq!(report["version"], 2, "the reported state matches the contract");
    assert_eq!(
        report["credentials"]["work"],
        serde_json::json!({
            "required":true,"provider":"anthropic","source":"program","number":0x00e1_3ed7,
            "kept":{"kind":"directory","at":scratch.path().join("state/smith/programs/work")},
            "state":"usable","until":null
        }),
        "the reported state matches the contract"
    );
    assert_eq!(
        report["programs"]["claude_code"],
        executable.to_str().expect("path UTF-8"),
        "the reported state matches the contract"
    );
    one_status(&scratch, "work");
}

#[test]
fn exec_refuses_a_program_account_not_signed_in_before_its_run() {
    let (scratch, _) = setup(None, Status::Truthful);
    let ran = support::smith(&scratch, &["exec", "--json", "Do the work"], b"");
    assert_eq!(ran.code, 3, "{}", String::from_utf8_lossy(&ran.stderr));
    assert_eq!(
        String::from_utf8_lossy(&ran.stderr).lines().last(),
        Some("smith: account work: Claude Code is not signed in: run `smith login work`"),
        "the reported state matches the contract"
    );
    assert!(ran.stdout.is_empty(), "the reported state matches the contract");
    one_status(&scratch, "work");
}
