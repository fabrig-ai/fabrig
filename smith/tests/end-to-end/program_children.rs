//! Program-account sub-agents through the shipped product (smith/docs/design/protocol/programs.md,
//! section 5.10; smith/docs/design/shell/settings.md, section 1.2). Each scratch keeps the fake's whole
//! story and account observations; these tests use no person's login or real endpoint.

use super::support;
use serde_json::json;
use smith_fake_claude::{OBSERVATIONS_FILE, STORY_FILE, Script, Step, Story, Turn, observations};
use smith_real_world::Scratch;

const CHILD_MODEL: &str = "claude-haiku-5-5";
const ALPHA: &str = "ALPHA-REPORT";
const BETA: &str = "BETA-REPORT";

fn argument<'a>(arguments: &'a [String], flag: &str) -> Option<&'a str> {
    let mut arguments = arguments.iter();
    arguments.find(|argument| argument.as_str() == flag)?;
    arguments.next().map(String::as_str)
}

fn signed_in(story: &Story) -> Scratch {
    let scratch = Scratch::new();
    let executable = support::fake_claude();
    let settings = format!(
        "model='anthropic/claude-opus-5-5'\nsub_agents=true\n[programs]\nclaude_code='{}'\n\
         [accounts.work]\nprovider='anthropic'\nsource='program'\n[endpoints.anthropic]\naccount='work'\nprogram='claude_code'\n",
        executable.display()
    );
    let _file = support::user_layer(&scratch, settings.as_bytes());
    let login = support::run_product(&scratch, &["login", "work"], b"", None);
    assert_eq!(login.code, 0, "{}", String::from_utf8_lossy(&login.stderr));
    std::fs::write(scratch.path().join("state/smith/programs/work").join(STORY_FILE), story.encode())
        .expect("scratch story file");
    scratch
}

fn invocations(scratch: &Scratch) -> Vec<Vec<String>> {
    // The fake opens this file under CLAUDE_CONFIG_DIR, so finding every
    // print invocation here also checks that each used main's account directory.
    let bytes = std::fs::read(scratch.path().join("state/smith/programs/work").join(OBSERVATIONS_FILE))
        .expect("account observations");
    observations(&bytes).into_iter().map(|record| record.arguments).collect()
}

fn outcome(report: &str) -> Turn {
    Turn::new(vec![Step::Outcome { input: json!({"outcome":{"form":"report","text":report,"fields":{}}}) }])
}

#[test]
fn exec_on_claude_asks_two_sub_agents_side_by_side() {
    let report = format!("{ALPHA}; {BETA}");
    let scratch = signed_in(&Story {
        turns: vec![
            Turn::new(vec![Step::Together(vec![
                Step::CallChecked {
                    tool: "sub_agent".into(),
                    input: json!({"brief":"CUE-ALPHA Inspect exports and report.","tools":["inspect"],"llm":CHILD_MODEL}),
                    contains: ALPHA.into(),
                },
                Step::CallChecked {
                    tool: "sub_agent".into(),
                    input: json!({"brief":"CUE-BETA Inspect tests and report.","tools":["inspect"],"llm":CHILD_MODEL}),
                    contains: BETA.into(),
                },
            ])]),
            outcome(&report),
        ],
        cued: vec![
            Script { cue: "CUE-ALPHA".into(), turns: vec![Turn::new(vec![Step::Text(ALPHA.into())])] },
            Script { cue: "CUE-BETA".into(), turns: vec![Turn::new(vec![Step::Text(BETA.into())])] },
        ],
        ..Story::default()
    });
    let ran = support::run_product(&scratch, &["exec", "Ask two inspectors for reports"], b"", None);
    assert_eq!(ran.code, 0, "{}", String::from_utf8_lossy(&ran.stderr));
    assert_eq!(ran.stdout, report.as_bytes());
    let calls = invocations(&scratch);
    let print: Vec<_> = calls.iter().filter(|arguments| arguments.first().is_some_and(|arg| arg == "-p")).collect();
    assert_eq!(print.len(), 3, "main and two children use the account directory");
    let main: Vec<_> =
        print.iter().filter(|arguments| argument(arguments, "--model") == Some("claude-opus-5-5")).collect();
    assert_eq!(main.len(), 1, "main uses its selected model");
    let children: Vec<_> =
        print.iter().filter(|arguments| argument(arguments, "--model") == Some(CHILD_MODEL)).collect();
    assert_eq!(children.len(), 2, "each child uses the model its call names");
    for child in children {
        assert_eq!(argument(child, "--tools"), Some("Read,Glob,Grep"));
        assert!(!child.iter().any(|argument| argument == "--json-schema"));
    }
}

#[test]
fn exec_on_claude_refuses_a_writing_sub_agent() {
    let report = "Writing sub-agent refused";
    let scratch = signed_in(&Story {
        turns: vec![
            Turn::new(vec![Step::CallChecked {
                tool: "sub_agent".into(),
                input: json!({"brief":"CUE-ALPHA Inspect and run a command.","tools":["inspect","shell"],"llm":CHILD_MODEL}),
                contains: "tools".into(),
            }]),
            outcome(report),
        ],
        cued: vec![Script { cue: "CUE-ALPHA".into(), turns: vec![Turn::new(vec![Step::Text(ALPHA.into())])] }],
        ..Story::default()
    });
    let ran = support::run_product(&scratch, &["exec", "Ask a writing inspector for a report"], b"", None);
    assert_eq!(ran.code, 0, "{}", String::from_utf8_lossy(&ran.stderr));
    assert_eq!(ran.stdout, report.as_bytes());
    let calls = invocations(&scratch);
    let print: Vec<_> = calls.iter().filter(|arguments| arguments.first().is_some_and(|arg| arg == "-p")).collect();
    assert_eq!(print.len(), 1, "a writing request opens no child program");
    assert_eq!(argument(print[0], "--model"), Some("claude-opus-5-5"));
}
