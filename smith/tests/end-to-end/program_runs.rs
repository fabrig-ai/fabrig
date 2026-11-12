//! Program-account paths through the shipped product (smith/docs/design/shell/commands.md and smith/docs/design/shell/README.md, section 5).
mod records;

use super::support;
use serde_json::json;
use smith_fake_claude::{End, OBSERVATIONS_FILE, STORY_FILE, Step, Story, Turn, observations};
use smith_real_world::Scratch;

const ANSWER: &str = "Program answer";
const TOLD_TEXT: &str = "Told program text";

fn signed_in(turn: Turn) -> Scratch {
    let scratch = Scratch::new();
    let executable = support::fake_claude();
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

fn signed_in_claude(turn: Turn) -> Scratch {
    let scratch = Scratch::new();
    let executable = support::fake_claude();
    let settings = format!(
        "model='anthropic/claude-haiku-5-5'\n[programs]\nclaude_code='{}'\n\
         [accounts.claude]\nprovider='anthropic'\nsource='program'\n\
         [endpoints.anthropic]\naccount='claude'\nprogram='claude_code'\n",
        executable.display()
    );
    let _file = support::user_layer(&scratch, settings.as_bytes());
    let login = support::run_product(&scratch, &["login", "claude"], b"", None);
    assert_eq!(login.code, 0, "fake Claude login succeeds");
    std::fs::write(
        scratch.path().join("state/smith/programs/claude").join(STORY_FILE),
        Story { turns: vec![turn], ..Story::default() }.encode(),
    )
    .expect("scratch story file");
    scratch
}

fn outcome() -> Step {
    Step::Outcome { input: json!({"outcome":{"form":"report","text":ANSWER,"fields":{}}}) }
}

fn answer() -> Turn {
    Turn::new(vec![Step::Text(TOLD_TEXT.into()), outcome()])
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
fn a_program_exec_traces_its_run_to_the_agent_file() {
    use smith_events::{Block, Event};
    let scratch = signed_in(Turn::new(vec![outcome()]));
    let trace = scratch.path().join("trace.jsonl");
    let trace_setting = format!("trace.path={}", toml::Value::String(trace.to_str().expect("scratch path").into()));
    let ran = support::run_product(
        &scratch,
        &["exec", "--json", "--chat", "chat", "-c", &trace_setting, "-c", "trace.capture='everything'", "Ask first"],
        b"",
        None,
    );
    assert_eq!(ran.code, 0, "program exec succeeds");
    let product = records::read(&ran.stdout);
    assert_eq!(records::answer(&product).status, smith_events::Status::Accepted);
    let agent = records::agent(&trace);
    assert!(
        agent.iter().any(|record| matches!(&record.event, Event::RunStarted(start) if !start.resumed)),
        "the agent records a fresh run"
    );
    assert!(
        agent.iter().any(|record| matches!(&record.event, Event::ResponseStarted(start)
            if start.prompt.as_ref().is_some_and(|prompt| prompt.messages.iter().any(|message|
                message.blocks.iter().any(|block| matches!(block, Block::Text { text }
                    if skein_lib::bytes::find(text, b"Ask first").is_some())))))),
        "the agent captures the task in its prompt"
    );
    assert!(
        agent
            .iter()
            .any(|record| matches!(&record.event, Event::ToolCompleted(tool) if tool.tool.as_ref() == b"finish")),
        "the agent records the finish completion"
    );
}

#[test]
fn a_program_exec_offers_the_live_claude_story_s_tools() {
    use smith_events::{Event, Family};
    let scratch = signed_in_claude(Turn::new(vec![outcome()]));
    let trace = scratch.path().join("trace.jsonl");
    let mut arguments = support::live::arguments("claude", scratch.path(), false, false);
    support::live::with_trace(&mut arguments, &trace);
    let arguments: Vec<_> = arguments.iter().map(|argument| argument.to_str().expect("scratch argument")).collect();
    let ran = support::run_product(&scratch, &arguments, b"Ask first", None);
    assert_eq!(ran.code, 0, "program exec succeeds");
    let agent = records::agent(&trace);
    let started = agent
        .iter()
        .find_map(|record| if let Event::RunStarted(start) = &record.event { Some(start) } else { None })
        .expect("program admission event");
    assert_eq!(
        started.tools.families.as_slice(),
        [Family::Inspect, Family::Modify, Family::Shell],
        "the program has exactly the live story's granted capabilities"
    );
    let tools = agent
        .iter()
        .find_map(|record| {
            if let Event::ResponseStarted(start) = &record.event {
                start.prompt.as_ref().and_then(|prompt| prompt.tools.as_ref()).filter(|tools| !tools.is_empty())
            } else {
                None
            }
        })
        .expect("the first prompt's offered tools");
    let tools: Vec<&[u8]> = tools.iter().map(AsRef::as_ref).collect();
    if let Some(fault) = support::program_contract::offered_tools_fault(&tools) {
        panic!("{fault}");
    }
}

#[test]
fn the_offered_tools_check_names_each_departure() {
    use support::program_contract::{OFFERED, offered_tools_fault};
    let offered: Vec<_> = OFFERED.iter().map(|name| name.as_bytes()).chain([b"remove".as_slice()]).collect();
    assert_eq!(offered_tools_fault(&offered), None, "the offered tools may include remove");
    for name in OFFERED {
        let missing: Vec<_> = offered.iter().copied().filter(|tool| *tool != name.as_bytes()).collect();
        assert_eq!(
            offered_tools_fault(&missing),
            Some(format!("the first prompt offers no {name}")),
            "a missing {name} is named"
        );
    }
    for (name, expected) in [
        ("finish", "the first prompt offers finish under a result contract"),
        ("sub_agent", "the first prompt offers sub_agent"),
    ] {
        let mut tools = offered.clone();
        tools.push(name.as_bytes());
        assert_eq!(offered_tools_fault(&tools).as_deref(), Some(expected), "an offered {name} is named");
    }
}

#[test]
fn a_program_turn_exit_keeps_the_run_failure() {
    let scratch = signed_in(Turn { end: End::Exit(7), ..Turn::new(vec![]) });
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
    let scratch = signed_in(Turn::new(vec![Step::Text(text.into()), Step::Hang]));
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
    let scratch = signed_in(Turn::new(vec![
        Step::Text(text.into()),
        Step::Builtin { tool: "Read".into(), input: json!({"file_path":"README.md"}), result: "file bytes".into() },
        outcome(),
    ]));
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
