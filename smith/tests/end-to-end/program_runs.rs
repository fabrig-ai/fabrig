//! Program-account paths through the shipped product (smith/docs/design/shell.md, sections 4 and 9).
mod records;

use super::support;
use serde_json::json;
use smith_fake_claude::{End, OBSERVATIONS_FILE, STORY_FILE, Step, Story, Turn, observations};
use smith_real_world::Scratch;

const ANSWER: &str = "Program answer";
const TOLD_TEXT: &str = "Told program text";

fn signed_in(turn: Turn) -> Scratch {
    let scratch = Scratch::new();
    let executable = skein_scratch::paths::binary("fake-claude");
    let settings = format!(
        "model='anthropic/claude-opus-5-5'\n[programs]\nclaude_code='{}'\n\
         [accounts.work]\nprovider='anthropic'\nsource='program'\n[endpoints.anthropic]\naccount='work'\nprogram='claude_code'\n",
        executable.display()
    );
    let _file = support::user_layer(&scratch, settings.as_bytes());
    let login = support::run_product(&scratch, &["login", "work"], b"", None);
    assert_eq!(login.code, 0, "{}", String::from_utf8_lossy(&login.stderr));
    std::fs::write(
        scratch.path().join("state/smith/programs/work").join(STORY_FILE),
        Story { turns: vec![turn], ..Story::default() }.encode(),
    )
    .expect("scratch story file");
    scratch
}

fn outcome() -> Step {
    Step::Outcome { input: json!({"outcome":{"form":"report","text":ANSWER,"fields":{}}}) }
}

fn answer() -> Turn {
    Turn { steps: vec![Step::Text(TOLD_TEXT.into()), outcome()], end: End::default() }
}

fn invocations(scratch: &Scratch) -> Vec<Vec<String>> {
    let bytes = std::fs::read(scratch.path().join("state/smith/programs/work").join(OBSERVATIONS_FILE))
        .expect("fake observations");
    observations(&bytes).into_iter().map(|record| record.arguments).collect()
}

#[test]
fn a_program_run_never_sees_the_person_s_credential_variables() {
    let scratch = signed_in(answer());
    let configuration = scratch.path().join("state/smith/programs/work");
    // Only this exec's invocations remain; the setup's login is separate.
    std::fs::remove_file(configuration.join(OBSERVATIONS_FILE)).expect("clear setup observations");
    let ran = super::program_commands::with_credentials(&scratch, &["exec", "Ask first"]);
    assert_eq!(ran.code, 0, "program run succeeds");
    assert_eq!(ran.stdout, ANSWER.as_bytes());
    // The fake opens its observation file relative to CLAUDE_CONFIG_DIR,
    // so finding every call here also proves the account directory was used.
    let bytes = std::fs::read(configuration.join(OBSERVATIONS_FILE)).expect("account observations");
    let calls = observations(&bytes);
    assert_eq!(calls.len(), 3, "startup status, agent status and print run");
    assert_eq!(calls[0].arguments, ["auth", "status"]);
    assert_eq!(calls[1].arguments, ["auth", "status"]);
    assert_eq!(calls[2].arguments[0], "-p");
    for call in calls {
        super::program_commands::credential_free(&call);
    }
    assert!(
        !smith_fake_claude::seen(&bytes)
            .iter()
            .any(|record| matches!(record, smith_fake_claude::Seen::Called { tool, .. } if tool == "finish")),
        "a contract outcome never calls finish over MCP"
    );
}

#[test]
fn exec_spawns_one_print_run_on_its_program() {
    let scratch = signed_in(answer());
    let ran = support::run_product(&scratch, &["exec", "Ask first"], b"", None);
    assert_eq!(ran.code, 0, "{}", String::from_utf8_lossy(&ran.stderr));
    assert_eq!(ran.stdout, ANSWER.as_bytes());
    let calls = invocations(&scratch);
    let print: Vec<_> = calls.iter().filter(|arguments| arguments[0] == "-p").collect();
    assert_eq!(print.len(), 1);
    assert_eq!(
        &print[0][..10],
        [
            "-p",
            "--input-format",
            "stream-json",
            "--output-format",
            "stream-json",
            "--verbose",
            "--include-partial-messages",
            "--setting-sources",
            "user",
            "--model"
        ]
    );
    assert!(print[0].windows(2).any(|pair| pair == ["--model", "claude-opus-5-5"]));
    assert_eq!(
        calls.iter().filter(|arguments| arguments[0] == "auth").map(Vec::as_slice).collect::<Vec<_>>(),
        [["auth", "login"], ["auth", "status"], ["auth", "status"], ["auth", "status"]],
        "login, its status, the startup status and the agent source check"
    );
}

#[test]
fn a_program_chat_answers_before_input_ends() {
    use smith_local_process_world::terminal::Action;
    let scratch = signed_in(answer());
    let ran = support::run_product_scripted(
        &scratch,
        &["--plain"],
        vec![Action::Send(b"Ask first\n".as_slice().into()), Action::Wait(ANSWER.as_bytes().into()), Action::Eof],
    );
    assert_eq!(ran.code, 0, "{}", String::from_utf8_lossy(&ran.stderr));
    let shown = String::from_utf8_lossy(&ran.stdout);
    assert!(shown.contains(ANSWER));
    assert_eq!(shown.matches(TOLD_TEXT).count(), 1);
    assert!(!shown.contains("through Claude Code"), "plain has no live status");
}

#[test]
fn a_program_turn_exit_keeps_the_run_failure() {
    let scratch = signed_in(Turn { steps: vec![], end: End::Exit(7) });
    let ran = support::run_product(
        &scratch,
        &["exec", "--json", "--trace", "failure.jsonl", "-c", "trace.capture='none'", "Ask first"],
        b"",
        None,
    );
    assert_eq!(ran.code, 1, "{}", String::from_utf8_lossy(&ran.stderr));
    assert_eq!(
        String::from_utf8_lossy(&ran.stderr).lines().last(),
        Some("smith: run failed: claude exited before its answer: ")
    );
    let product = records::read(&ran.stdout);
    let completed = records::answer(&product);
    assert_eq!(completed.status, smith_events::Status::Failed);
    let failure = completed.failure.as_ref().expect("program failure");
    assert_eq!(failure.class, smith_events::AnswerClass::Model);
    assert_eq!(failure.program.as_deref(), Some(b"claude_code".as_slice()));
    assert_eq!(failure.exit, Some(smith_events::AgentExit::Code(7)));
    assert_eq!(failure.detail, None);
}

#[test]
fn a_program_chat_shows_text_before_interrupting_an_uncommitted_turn() {
    use smith_local_process_world::terminal::Action;
    let text = "Text before the hang";
    let scratch = signed_in(Turn { steps: vec![Step::Text(text.into()), Step::Hang], end: End::default() });
    let ran = support::program_chat(
        &scratch,
        vec![
            Action::Wait(b"> ".as_slice().into()),
            Action::Send(b"Ask first\n".as_slice().into()),
            Action::Wait(text.as_bytes().into()),
            Action::Send(b"\x03".as_slice().into()),
            Action::Eof,
        ],
    );
    assert_eq!(ran.code, 0, "{}", String::from_utf8_lossy(&ran.stderr));
    let shown = String::from_utf8_lossy(&ran.stdout);
    assert_eq!(shown.matches(text).count(), 1);
    assert!(!shown.contains("through Claude Code"), "no turn committed before the interruption");
}

#[test]
fn a_program_chat_completes_its_live_view() {
    use smith_local_process_world::terminal::Action;
    let text = "Live settled text";
    let scratch = signed_in(Turn {
        steps: vec![
            Step::Text(text.into()),
            Step::Builtin { tool: "Read".into(), input: json!({"file_path":"README.md"}), result: "file bytes".into() },
            outcome(),
        ],
        end: End::default(),
    });
    let ran = support::program_chat(
        &scratch,
        vec![
            Action::Wait(b"> ".as_slice().into()),
            Action::Send(b"Ask first\n".as_slice().into()),
            Action::Wait(ANSWER.as_bytes().into()),
            Action::Eof,
        ],
    );
    assert_eq!(ran.code, 0, "{}", String::from_utf8_lossy(&ran.stderr));
    let shown = String::from_utf8_lossy(&ran.stdout);
    assert_eq!(shown.matches(text).count(), 1, "{shown}");
    assert!(shown.contains("tool Read: result"), "{shown}");
    assert!(shown.contains("anthropic/claude-opus-5-5; through Claude Code: in "), "{shown}");
    assert!(shown.contains(" turns, ") && shown.contains(" notional, "), "{shown}");
    assert!(!shown.contains(" ms)"), "no invented tool duration");
}
