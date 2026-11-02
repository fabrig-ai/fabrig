use serde_json::json;
use skein_lib::Reader;
use smith_agent_process_world::{
    program,
    program_world::{self, Case, Run},
};
use smith_events::{ConversationKind, Event};
use smith_fake_claude::{Script, Step, Story, Turn};

fn run(modify: bool) -> Run {
    let tools = if modify { vec!["inspect", "modify"] } else { vec!["inspect"] };
    let story = Story {
        turns: vec![
            Turn::new(vec![Step::CallChecked {
                tool: "sub_agent".into(),
                input: json!({"brief":"CUE-ALPHA List what lib.rs exports.","tools":tools,"llm":"fake"}),
                contains: if modify { "tools" } else { "exports run" }.into(),
            }]),
            Turn::new(vec![Step::Call {
                tool: "finish".into(),
                input: json!({"outcome":{"form":"report","text":"exports run","fields":{}}}),
            }]),
        ],
        cued: vec![Script { cue: "CUE-ALPHA".into(), turns: vec![Turn::new(vec![Step::Text("exports run".into())])] }],
        observe_tools: true,
        ..Default::default()
    };
    program_world::run(
        Case::Events,
        &story,
        Some(b"claude.ai"),
        &program_world::child_charter(),
        &program(b"source"),
        None,
    )
}

fn accepted(run: &Run) {
    assert!(
        matches!(run.answer().expect("answer").result(), smith_channel::RunResult::Accepted(_)),
        "answer={:?}, errors={}",
        run.answer(),
        String::from_utf8_lossy(&run.errors)
    );
}

fn results(run: &Run) -> Vec<smith_transcript::ToolResult> {
    let mut results = Vec::new();
    for frame in &run.frames {
        if frame.kind != 0x0109 {
            continue;
        }
        let turn = smith_channel::Turn::decode(&smith_channel::CEILINGS, &mut Reader::new(&frame.body)).expect("turn");
        let turn = smith_transcript::Turn::decode(&smith_transcript::CEILINGS, &mut Reader::new(turn.body()))
            .expect("transcript");
        for message in turn.messages().as_slice() {
            for block in message.blocks().as_slice() {
                if let smith_transcript::Block::Result(result) = block {
                    results.push(result.clone());
                }
            }
        }
    }
    results
}

#[test]
fn a_program_main_reads_its_sub_agents_answer() {
    let run = run(false);
    accepted(&run);
    assert!(run.seen.contains(&smith_fake_claude::Seen::Called { tool: "sub_agent".into(), error: false }));
    let child = run
        .invocations
        .iter()
        .find(|invocation| invocation.arguments.windows(2).any(|pair| pair == ["--effort", "high"]))
        .expect("child invocation at its own effort");
    let args = &child.arguments;
    assert_eq!(args[0], "-p");
    assert!(args.windows(2).any(|pair| pair == ["--model", "fake"]));
    assert!(args.windows(2).any(|pair| pair == ["--tools", "Read,Glob,Grep"]));
    assert!(args.iter().any(|arg| arg == "--permission-prompt-tool"));
    assert!(args.iter().any(|arg| arg == "--mcp-config"));
    assert!(!args.iter().any(|arg| arg == "--json-schema"));
    assert!(run.seen.contains(&smith_fake_claude::Seen::Tools { names: vec!["permission_prompt".into()] }));
    let child = run
        .events
        .iter()
        .find_map(|record| match &record.event {
            Event::ConversationOpened(opening) if opening.kind == ConversationKind::Child => Some(opening),
            _ => None,
        })
        .expect("child conversation opened");
    assert!(child.call.is_some());
    assert_eq!(child.model.model.as_ref(), b"fake");
    assert_eq!(child.model.effort.as_deref(), Some(b"high".as_slice()));
    assert!(run.events.iter().any(
        |record| matches!(&record.event, Event::ConversationClosed(close) if close.conversation == child.conversation)
    ));
    let result = results(&run).into_iter().next().expect("sub_agent result is told");
    let smith_transcript::Returned::Said(said) = result.returned() else { panic!("child answer") };
    assert!(!said.error());
    assert!(skein_lib::bytes::find(said.text(), b"exports run").is_some());
    assert!(skein_lib::bytes::find(said.text(), b"Child usage:").is_some());
}

#[test]
fn a_program_childs_program_ends_with_its_conversation() {
    let run = run(false);
    accepted(&run);
    assert_eq!(
        run.invocations.iter().filter(|invocation| invocation.arguments.first().is_some_and(|arg| arg == "-p")).count(),
        2
    );
    assert!(run.run_directories.is_empty(), "main and child private directories are removed");
}

#[test]
fn a_program_child_asking_to_write_is_refused() {
    let run = run(true);
    accepted(&run);
    assert_eq!(
        run.invocations.iter().filter(|invocation| invocation.arguments.first().is_some_and(|arg| arg == "-p")).count(),
        1
    );
    let result = results(&run).into_iter().next().expect("sub_agent result is told");
    let smith_transcript::Returned::Invalid(invalid) = result.returned() else { panic!("invalid tool result") };
    assert!(matches!(invalid.problem(), smith_transcript::CallProblem::BadValue(field) if field.field() == b"tools"));
}

#[test]
fn a_program_main_reads_two_children_from_one_message() {
    let story = Story {
        turns: vec![
            Turn::new(vec![Step::Together(vec![
                Step::Call {
                    tool: "sub_agent".into(),
                    input: json!({"brief":"CUE-ALPHA List exports.","tools":["inspect"],"llm":"fake"}),
                },
                Step::Call {
                    tool: "sub_agent".into(),
                    input: json!({"brief":"CUE-BETA List tests.","tools":["inspect"],"llm":"fake"}),
                },
            ])]),
            Turn::new(vec![Step::Call {
                tool: "finish".into(),
                input: json!({"outcome":{"form":"report","text":"exports run; tests run","fields":{}}}),
            }]),
        ],
        cued: vec![
            Script { cue: "CUE-ALPHA".into(), turns: vec![Turn::new(vec![Step::Text("exports run".into())])] },
            Script { cue: "CUE-BETA".into(), turns: vec![Turn::new(vec![Step::Text("tests run".into())])] },
        ],
        ..Default::default()
    };
    let run = program_world::run(
        Case::Events,
        &story,
        Some(b"claude.ai"),
        &program_world::child_charter(),
        &program(b"source"),
        None,
    );
    accepted(&run);
    assert_eq!(
        run.invocations
            .iter()
            .filter(|invocation| invocation.arguments.windows(2).any(|pair| pair == ["--effort", "high"]))
            .count(),
        2
    );
    let children: Vec<_> = run
        .events
        .iter()
        .filter_map(|record| match &record.event {
            Event::ConversationOpened(opening) if opening.kind == ConversationKind::Child => Some(opening),
            _ => None,
        })
        .collect();
    assert_eq!(children.len(), 2);
    assert_ne!(children[0].call, children[1].call);
    assert_ne!(children[0].thread, children[1].thread);
    let results = results(&run);
    for answer in [b"exports run".as_slice(), b"tests run".as_slice()] {
        assert!(results.iter().any(|result| matches!(result.returned(), smith_transcript::Returned::Said(said) if !said.error() && skein_lib::bytes::find(said.text(), answer).is_some())));
    }
    for child in children {
        assert!(run.events.iter().any(|record| matches!(&record.event, Event::ConversationClosed(close) if close.conversation == child.conversation)));
    }
    assert!(run.run_directories.is_empty());
}
