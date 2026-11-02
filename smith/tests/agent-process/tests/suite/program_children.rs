use serde_json::json;
use skein_lib::Reader;
use smith_agent_process_world::{
    program,
    program_world::{self, Case, Run},
};
use smith_events::{ConversationEnd, ConversationKind, Event};
use smith_fake_claude::{Decision, Script, Seen, Step, Story, Turn};

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

fn finish() -> Turn {
    Turn::new(vec![Step::Outcome {
        input: json!({"outcome":{"form":"report","text":"children surveyed","fields":{}}}),
    }])
}

fn read(path: &str) -> Step {
    Step::Act { tool: "Read".into(), input: json!({"file_path":path}) }
}

fn story_run(story: &Story) -> Run {
    program_world::run(
        Case::Events,
        story,
        Some(b"claude.ai"),
        &program_world::child_charter(),
        &program(b"source"),
        None,
    )
}

#[test]
fn two_program_children_run_side_by_side_and_each_answers() {
    let story = Story {
        turns: vec![
            Turn::new(vec![Step::Together(
                [("CUE-ALPHA", "alpha"), ("CUE-BETA", "beta")]
                    .into_iter()
                    .map(|(cue, word)| Step::Call {
                        tool: "sub_agent".into(),
                        input: json!({"brief":format!("{cue} Read the workspace and report {word}."),"tools":["inspect"],"llm":"fake"}),
                    })
                    .collect(),
            )]),
            finish(),
        ],
        cued: [("CUE-ALPHA", "alpha", "/work/repo/a"), ("CUE-BETA", "beta", "/library/a")]
            .into_iter()
            .map(|(cue, word, path)| Script {
                cue: cue.into(),
                turns: vec![Turn::new(vec![read(path), Step::Text(word.into())])],
            })
            .collect(),
        ..Default::default()
    };
    let run = story_run(&story);
    accepted(&run);
    let children: Vec<_> = run
        .invocations
        .iter()
        .filter(|invocation| invocation.arguments.windows(2).any(|pair| pair == ["--effort", "high"]))
        .collect();
    assert_eq!(children.len(), 2);
    for child in children {
        assert_eq!(child.arguments[0], "-p");
        assert!(child.arguments.windows(2).any(|pair| pair == ["--model", "fake"]));
        assert!(child.arguments.windows(2).any(|pair| pair == ["--tools", "Read,Glob,Grep"]));
    }
    let openings: Vec<_> = run
        .events
        .iter()
        .enumerate()
        .filter_map(|(index, record)| match &record.event {
            Event::ConversationOpened(opening) if opening.kind == ConversationKind::Child => {
                Some((index, opening.conversation))
            }
            _ => None,
        })
        .collect();
    assert_eq!(openings.len(), 2);
    for (opened, _) in &openings {
        for (_, conversation) in &openings {
            let closed = run.events.iter().position(|record| matches!(&record.event, Event::ConversationClosed(close) if close.conversation == *conversation)).expect("child closed");
            assert!(*opened < closed, "both children opened before either closed");
        }
    }
    assert_eq!(
        run.seen
            .iter()
            .filter(
                |seen| **seen == Seen::Prompted { tool: "Read".into(), decision: Decision::Allow { unchanged: true } }
            )
            .count(),
        2
    );
    for path in ["/work/repo/a", "/library/a"] {
        assert!(run.seen.contains(&Seen::Acted { tool: "Read".into(), path: path.into(), error: false }));
    }
    let results = results(&run);
    for word in [b"alpha".as_slice(), b"beta".as_slice()] {
        assert!(results.iter().any(|result| matches!(result.returned(), smith_transcript::Returned::Said(said) if !said.error() && skein_lib::bytes::find(said.text(), word).is_some())));
    }
}

#[test]
fn a_program_childs_final_turn_answers_for_its_budget() {
    let story = Story {
        turns: vec![
            Turn::new(vec![Step::CallChecked {
                tool: "sub_agent".into(),
                input: json!({"brief":"CUE-ALPHA Survey the workspace.","tools":["inspect"],"llm":"fake","max_turns":2}),
                contains: "final survey".into(),
            }]),
            finish(),
        ],
        cued: vec![Script {
            cue: "CUE-ALPHA".into(),
            turns: vec![
                Turn::new(vec![read("/work/repo/a"), read("/library/a"), Step::Hang]),
                Turn::new(vec![Step::Text("final survey".into())]),
            ],
        }],
        ..Default::default()
    };
    let run = story_run(&story);
    accepted(&run);
    assert_eq!(
        run.seen.iter().filter(|seen| **seen == Seen::Interrupted).count(),
        1,
        "seen={:?}, received={:?}",
        run.seen,
        run.received
    );
    let result = results(&run).into_iter().next().expect("child result");
    let smith_transcript::Returned::Said(said) = result.returned() else { panic!("child answer") };
    assert!(!said.error());
    for text in [b"final survey".as_slice(), b"stopped for budget: yes".as_slice()] {
        assert!(skein_lib::bytes::find(said.text(), text).is_some(), "child answer and budget usage are told");
    }
    let child = run
        .events
        .iter()
        .find_map(|record| match &record.event {
            Event::ConversationOpened(opening) if opening.kind == ConversationKind::Child => Some(opening.conversation),
            _ => None,
        })
        .expect("child opened");
    assert!(run.events.iter().any(|record| matches!(&record.event, Event::ConversationClosed(close) if close.conversation == child && close.end == ConversationEnd::Budget)));
    assert!(run.received.iter().any(|message| {
        message["message"]["content"].as_str().is_some_and(|text| {
            text.starts_with("(The agent paused your last turn to send the finishing note.)\n")
                && text.contains("Your share is ending.")
        })
    }));
    assert!(!run.seen.iter().any(|seen| matches!(seen, Seen::Acted { .. })), "the interrupt ends the pending read");
}

fn cancelled_child(step: Step) -> Run {
    let story = Story {
        turns: vec![Turn::new(vec![
            Step::Call {
                tool: "sub_agent".into(),
                input: json!({"brief":"CUE-ALPHA Survey the workspace.","tools":["inspect"],"llm":"fake"}),
            },
            Step::Hang,
        ])],
        cued: vec![Script { cue: "CUE-ALPHA".into(), turns: vec![Turn::new(vec![step])] }],
        ..Default::default()
    };
    let run = program_world::cancel_with_child(&story, &program_world::child_charter());
    assert!(
        matches!(run.answer().expect("answer").result(), smith_channel::RunResult::Failed(failed) if matches!(failed.reason(), smith_channel::RunFailure::Cancelled))
    );
    assert_eq!(
        run.invocations.iter().filter(|invocation| invocation.arguments.first().is_some_and(|arg| arg == "-p")).count(),
        2
    );
    let results = results(&run);
    assert_eq!(results.len(), 1, "the cancelled child call settles once");
    assert!(matches!(results[0].returned(), smith_transcript::Returned::Withdrawn));
    assert!(run.seen.contains(&Seen::Called { tool: "sub_agent".into(), error: true }));
    assert!(run.run_directories.is_empty());
    run
}

#[test]
fn a_cancelled_program_run_stops_its_children() {
    let run = cancelled_child(Step::Hang);
    let signals: Vec<_> = run.seen.iter().filter(|seen| matches!(seen, Seen::Interrupted | Seen::Terminated)).collect();
    assert_eq!(signals, [&Seen::Interrupted, &Seen::Interrupted]);
}

#[test]
fn a_cancelled_program_run_terminates_a_deaf_child() {
    let run = cancelled_child(Step::Deaf);
    let signals: Vec<_> = run.seen.iter().filter(|seen| matches!(seen, Seen::Interrupted | Seen::Terminated)).collect();
    assert_eq!(signals, [&Seen::Interrupted, &Seen::Interrupted, &Seen::Terminated]);
}
