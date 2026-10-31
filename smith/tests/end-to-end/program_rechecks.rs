//! Program account checks before runs (smith/docs/design/domain/host.md, section 8).
use super::{program_check, support};
use serde_json::json;
use smith_fake_claude::{End, OBSERVATIONS_FILE, STORY_FILE, Status, Step, Story, Turn, observations};
use smith_local_process_world::terminal::Action;
use smith_real_world::Scratch;

const ANSWER: &str = "Program answer";

fn setup(status: Status) -> Scratch {
    let (scratch, _) = program_check::setup(Some("claude.ai"), Status::Truthful);
    std::fs::write(
        scratch.path().join("state/smith/programs/work").join(STORY_FILE),
        Story {
            summary: "earlier answer".into(),
            status,
            turns: vec![Turn {
                steps: vec![Step::Outcome { input: json!({"outcome":{"form":"report","text":ANSWER,"fields":{}}}) }],
                end: End::default(),
            }],
            ..Story::default()
        }
        .encode(),
    )
    .expect("scratch story file");
    scratch
}

fn calls(scratch: &Scratch) -> Vec<Vec<String>> {
    let bytes =
        std::fs::read(scratch.path().join("state/smith/programs/work").join(OBSERVATIONS_FILE)).expect("observations");
    observations(&bytes).into_iter().map(|record| record.arguments).collect()
}

fn counts(scratch: &Scratch, statuses: usize, runs: usize) {
    let calls = calls(scratch);
    assert_eq!(calls.iter().filter(|arguments| arguments[0] == "-p").count(), runs, "print-mode runs");
    assert_eq!(
        calls.iter().filter(|arguments| **arguments == ["auth", "status"]).count(),
        statuses,
        "host and agent checks"
    );
}

fn two_lines() -> Vec<Action> {
    vec![
        Action::Send(b"Ask first\n".as_slice().into()),
        Action::Wait(ANSWER.as_bytes().into()),
        Action::Send(b"Ask second\n".as_slice().into()),
        Action::Eof,
    ]
}

#[test]
fn program_exec_keeps_the_agent_check() {
    let scratch = setup(Status::Truthful);
    let ran = support::run_product(&scratch, &["exec", "Ask first"], b"", None);
    assert_eq!(ran.code, 0, "{}", String::from_utf8_lossy(&ran.stderr));
    assert_eq!(ran.stdout, ANSWER.as_bytes());
    counts(&scratch, 2, 1);
}

#[test]
fn program_chat_refuses_a_signed_out_second_run() {
    let scratch = setup(Status::Sequence(vec![Status::Truthful, Status::Truthful, Status::NotSignedIn]));
    let ran = support::run_product_scripted(&scratch, &["--plain"], two_lines());
    assert_eq!(ran.code, 0, "{}", String::from_utf8_lossy(&ran.stderr));
    assert!(String::from_utf8_lossy(&ran.stdout).contains(ANSWER));
    assert_eq!(
        String::from_utf8_lossy(&ran.stderr).lines().last(),
        Some("smith: account work: Claude Code is not signed in: run `smith login work`")
    );
    counts(&scratch, 3, 1);
}

#[test]
fn program_chat_interrupts_a_silent_recheck() {
    let scratch = setup(Status::Sequence(vec![Status::Truthful, Status::Truthful, Status::Silent]));
    let ran = support::program_interrupt(&scratch, two_lines());
    assert_eq!(ran.code, 130, "{}", String::from_utf8_lossy(&ran.stderr));
    counts(&scratch, 3, 1);
}

#[test]
fn program_chat_runs_both_lines_after_input_ends() {
    let scratch = setup(Status::Truthful);
    let ran = support::run_product_scripted(&scratch, &["--plain"], two_lines());
    assert_eq!(ran.code, 0, "{}", String::from_utf8_lossy(&ran.stderr));
    assert_eq!(
        String::from_utf8_lossy(&ran.stdout).matches("Program answer").count(),
        2,
        "both answers: {} / {}",
        String::from_utf8_lossy(&ran.stdout),
        String::from_utf8_lossy(&ran.stderr)
    );
    let calls = calls(&scratch);
    let print: Vec<_> = calls.iter().filter(|arguments| arguments[0] == "-p").collect();
    assert_eq!(print.len(), 2, "{calls:?}");
    let first = print[0];
    let next = print[1];
    let id = &first.windows(2).find(|pair| pair[0] == "--session-id").expect("new session")[1];
    assert!(next.windows(2).any(|pair| pair[0] == "--resume" && &pair[1] == id));
    for ordinary in print {
        assert!(ordinary.iter().any(|argument| argument == "--mcp-config"));
    }
    counts(&scratch, 4, 2);
}
