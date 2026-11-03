//! One scratch keeps the chat while the product crosses Direct, program and Direct
//! (smith/docs/design/protocol/programs.md, 5.8; smith/docs/design/testing.md, 2.3).
//! Only product streams, each run's own trace and fake invocations are observed.

use super::support;
use smith_events::{Event, Record, Status};
use smith_fake_claude::{OBSERVATIONS_FILE, STORY_FILE, Step, Story, Turn, observations};
use smith_real_world::Scratch;

#[test]
fn an_exec_chat_crosses_from_direct_to_a_program_and_back() {
    let scratch = Scratch::new();
    let mut settings = support::fake_provider_settings(&scratch);
    settings.extend_from_slice(
        format!(
            "[programs]\nclaude_code='{}'\n\
             [accounts.claude]\nprovider='anthropic'\nsource='program'\n\
             [endpoints.anthropic]\naccount='claude'\nprogram='claude_code'\n",
            skein_scratch::paths::binary("fake-claude").display()
        )
        .as_bytes(),
    );
    let _file = support::user_layer(&scratch, &settings);
    let login = support::run_product(&scratch, &["login", "claude"], b"", None);
    assert_eq!(login.code, 0, "fake Claude login succeeds; stderr bytes={}", login.stderr.len());
    let configuration = scratch.path().join("credentials/programs/claude");
    std::fs::write(
        configuration.join(STORY_FILE),
        Story {
            summary: "portable summary".into(),
            turns: vec![Turn::new(vec![Step::Outcome {
                input: serde_json::json!({"outcome":{"form":"report","text":"Program answer","fields":{}}}),
            }])],
            ..Story::default()
        }
        .encode(),
    )
    .expect("scratch summary and window story");

    let first = direct(&scratch, "direct-first.jsonl", "Begin the chat");
    assert_resumed(&first, false, "Direct invocation 1");

    let program = support::run_product_scripted(
        &scratch,
        &[
            "exec",
            "--json",
            "--chat",
            "chat",
            "--trace",
            "program.jsonl",
            "-c",
            "endpoints.anthropic.account='claude'",
            "-m",
            "anthropic/claude-haiku-5-5",
            "Continue the chat",
        ],
        vec![smith_local_process_world::terminal::Action::Eof],
    );
    drop(assert_accepted(&program, "program invocation 2"));
    let agent = agent(&scratch.path().join("program.jsonl"));
    assert_resumed(&agent, true, "program invocation 2 agent");
    let calls = observations(&std::fs::read(configuration.join(OBSERVATIONS_FILE)).expect("fake invocations"));
    let print: Vec<_> =
        calls.iter().filter(|call| call.arguments.first().is_some_and(|argument| argument == "-p")).collect();
    assert_eq!(print.len(), 2, "program invocation 2 compacts then opens the window");
    let mut print = print.into_iter();
    let summary = print.next().expect("summary invocation");
    let window = print.next().expect("window invocation");
    assert!(
        !summary.arguments.iter().any(|argument| argument == "--mcp-config"),
        "program invocation 2 summary has no MCP tools"
    );
    assert!(
        window.arguments.iter().any(|argument| argument == "--mcp-config"),
        "program invocation 2 window has MCP tools"
    );

    let third = direct(&scratch, "direct-back.jsonl", "Return to Direct");
    assert_resumed(&third, true, "Direct invocation 3");
}

fn direct(scratch: &Scratch, trace: &str, prompt: &str) -> Vec<Record> {
    let ran = support::exec_with_provider(
        scratch,
        &[
            "exec",
            "--json",
            "--chat",
            "chat",
            "--trace",
            trace,
            "-c",
            "endpoints.openai.account='openai'",
            "-m",
            "openai/gpt-6-luna",
            prompt,
        ],
        b"",
        support::Reply::Accepted,
    );
    assert_accepted(&ran, trace)
}

fn assert_accepted(ran: &support::Ran, leg: &str) -> Vec<Record> {
    let records = read(&ran.stdout);
    let completed: Vec<_> = records
        .iter()
        .filter_map(|record| if let Event::RunCompleted(answer) = &record.event { Some(answer) } else { None })
        .collect();
    let ends: Vec<_> = records
        .iter()
        .filter_map(|record| if let Event::SessionEnded(end) = &record.event { Some(end) } else { None })
        .collect();
    assert_eq!(
        ran.code,
        0,
        "{leg}: exec succeeds; stderr bytes={}; run ends={:?}; session ends={:?}; agent failure={:?}",
        ran.stderr.len(),
        completed
            .iter()
            .map(|end| (
                &end.status,
                end.turns,
                end.failure.as_ref().map(|failure| (&failure.class, failure.exit, failure.ended))
            ))
            .collect::<Vec<_>>(),
        ends.iter().map(|end| (end.exit, end.answered, end.aborted)).collect::<Vec<_>>(),
        [
            "could not start",
            "ended without answering",
            "broke the channel rules",
            "exceeded its receiving bound",
            "made no progress",
            "reached its wall time bound"
        ]
        .into_iter()
        .filter(|reason| ran
            .stderr
            .split(|byte| *byte == b'\n')
            .any(|line| line.starts_with(format!("smith: run failed: the agent {reason}").as_bytes())))
        .collect::<Vec<_>>()
    );
    assert_eq!(completed.len(), 1, "{leg}: one run answer");
    let answer = completed.first().expect("one run answer");
    assert_eq!(answer.status, Status::Accepted, "{leg}: accepted answer");
    assert!(answer.result.is_some(), "{leg}: accepted result");
    assert_eq!(ends.len(), 1, "{leg}: one process terminal");
    let end = ends.first().expect("one process terminal");
    assert_eq!(end.exit, 0, "{leg}: successful process terminal");
    assert!(end.answered && !end.aborted, "{leg}: process answered and drained");
    records
}

fn assert_resumed(records: &[Record], expected: bool, leg: &str) {
    let starts: Vec<_> = records
        .iter()
        .filter_map(|record| if let Event::RunStarted(start) = &record.event { Some(start.resumed) } else { None })
        .collect();
    assert_eq!(starts, [expected], "{leg}: one run starts with the expected resume state");
}

#[expect(clippy::disallowed_types, reason = "bounded outside observation of the settled agent trace")]
fn agent(trace: &std::path::Path) -> Vec<Record> {
    use std::io::Read;
    let mut path = trace.as_os_str().to_os_string();
    path.push(".agent");
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .expect("program agent trace exists")
        .take(1_048_577)
        .read_to_end(&mut bytes)
        .expect("read program agent trace");
    assert!(bytes.len() <= 1_048_576, "agent trace stays within the process stream bound");
    read(&bytes)
}

fn read(bytes: &[u8]) -> Vec<Record> {
    const STREAM_BYTES: u32 = 1_048_576;
    let limits = {
        let mut limits = smith_events::fixture::SMALL;
        limits.string = STREAM_BYTES;
        limits.content = STREAM_BYTES;
        limits.items = STREAM_BYTES;
        limits
    };
    bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| {
            smith_events::read_with_ceiling(line, &limits, STREAM_BYTES)
                .expect("typed event record")
                .expect("known event kind")
        })
        .collect()
}
