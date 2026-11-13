use serde_json::json;
use skein_lib::Reader;
use smith_agent_process_world::{
    program,
    program_world::{self, Case, Run},
};
use smith_events::{ConversationEnd, ConversationKind, Event};
use smith_fake_claude::{Decision, Script, Seen, Step, Story, Turn};

fn argument<'a>(arguments: &'a [String], flag: &str) -> Option<&'a str> {
    let mut arguments = arguments.iter();
    arguments.find(|argument| argument.as_str() == flag)?;
    arguments.next().map(String::as_str)
}

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
        .find(|invocation| argument(&invocation.arguments, "--effort") == Some("high"))
        .expect("child invocation at its own effort");
    let args = &child.arguments;
    assert_eq!(args[0], "-p");
    assert_eq!(argument(args, "--model"), Some("fake"));
    assert_eq!(argument(args, "--tools"), Some("Read,Glob,Grep"));
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

// The declared room includes main, leaving four child conversations (domain/run.md, 5.3).
const BURST_ROOM: u32 = 5;
const CHILD_ANSWERS: [&str; 6] =
    ["exports run", "tests run", "third answer", "fourth answer", "fifth answer", "sixth answer"];

#[test]
fn a_program_main_reads_a_burst_past_its_room() {
    for (count, direct, opened, refused, programs) in [
        (2, false, 2, 0, 2),
        (2, true, 2, 0, 0),
        (3, false, 3, 0, 3),
        (3, true, 3, 0, 0),
        (4, false, 4, 0, 4),
        (4, true, 4, 0, 0),
        (5, false, 4, 1, 4),
        (5, true, 4, 1, 0),
        (6, false, 4, 2, 4),
        (6, true, 4, 2, 0),
    ] {
        eprintln!("burst count={count} direct={direct}");
        let story = Story {
            turns: vec![
                Turn::new(vec![Step::AllAtOnce((0..count).map(|index| Step::Call {
                    tool: "sub_agent".into(),
                    input: json!({"brief":format!("CUE-{index} Survey the workspace."),"tools":["inspect"],"llm":"fake"}),
                }).collect())]),
                Turn::new(vec![Step::Call {
                    tool: "finish".into(),
                    input: json!({"outcome":{"form":"report","text":"exports run; tests run","fields":{}}}),
                }]),
            ],
            cued: CHILD_ANSWERS.iter().enumerate().take(count).map(|(index, answer)| Script {
                cue: format!("CUE-{index}"), turns: if index == 0 {
                    vec![Turn::new(vec![Step::Text((*answer).into())])]
                } else {
                    vec![Turn::new(vec![read("/work/repo/a")]), Turn::new(vec![Step::Text((*answer).into())])]
                },
            }).collect(),
            ..Default::default()
        };
        let run = program_world::run(
            Case::Burst { conversations: BURST_ROOM },
            &story,
            Some(b"claude.ai"),
            &program_world::room_child_charter(direct),
            &program(b"source"),
            None,
        );
        accepted(&run);
        assert_eq!(
            run.invocations
                .iter()
                .filter(|invocation| argument(&invocation.arguments, "--effort") == Some("high"))
                .count(),
            programs
        );
        let children: Vec<_> = run
            .events
            .iter()
            .filter_map(|record| match &record.event {
                Event::ConversationOpened(opening) if opening.kind == ConversationKind::Child => Some(opening),
                _ => None,
            })
            .collect();
        assert_eq!(children.len(), opened);
        for (index, child) in children.iter().enumerate() {
            assert!(child.call.is_some());
            for other in children.iter().skip(index + 1) {
                assert_ne!(child.call, other.call);
                assert_ne!(child.thread, other.thread);
            }
            assert!(run.events.iter().any(|record| matches!(&record.event, Event::ConversationClosed(close) if close.conversation == child.conversation && close.end == ConversationEnd::Closed)));
        }
        let results = results(&run);
        assert_eq!(results.len(), count + 1, "each child call and finish are answered");
        for answer in CHILD_ANSWERS.iter().take(opened) {
            assert!(results.iter().any(|result| matches!(result.returned(), smith_transcript::Returned::Said(said) if !said.error() && skein_lib::bytes::find(said.text(), answer.as_bytes()).is_some())), "missing {answer}: {results:?}");
        }
        assert_eq!(results.iter().filter(|result| matches!(result.returned(), smith_transcript::Returned::Said(said) if said.error() && skein_lib::bytes::find(said.text(), b"refused reason=too-many").is_some())).count(), refused);
        assert_eq!(run.run_directories.len(), 0);
    }
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

fn survey_story(count: usize) -> Story {
    Story {
        turns: vec![
            Turn::new(vec![Step::Together(
                (0..count).map(|index| inspect_child(&format!("SURVEY-{index}:"), true)).collect(),
            )]),
            finish(),
        ],
        cued: (0..count)
            .map(|index| Script {
                cue: format!("SURVEY-{index}:"),
                turns: vec![Turn::new(vec![Step::Text(format!("survey answer {index}"))])],
            })
            .collect(),
        ..Default::default()
    }
}

fn survey_charter(direct: bool) -> Box<[u8]> {
    smith_agent_process_world::wire::edit_charter(&program_world::child_charter(), |parts| {
        smith_agent_process_world::wire::budget(parts, 32, 80);
        if direct {
            let mut child = parts.models.as_slice()[0].clone().into_parts();
            child.endpoint = b"direct".as_slice().into();
            child.account = Some(1);
            parts.models = skein_lib::List::with_capacity(1);
            parts
                .models
                .push(smith_charter::Llm::new(&smith_charter::CEILINGS, child).expect("direct child"))
                .expect("model");
        }
    })
}

fn survey_answers(run: &Run, count: usize, room: usize) {
    accepted(run);
    let mut results = results(run);
    assert_eq!(results.len(), count + 1, "survey calls and finish each answer: count={count}");
    results.pop().expect("finish result");
    let mut names = std::collections::BTreeSet::new();
    let mut answers = std::collections::BTreeSet::new();
    for result in &results {
        assert!(names.insert(result.id()), "each call answers once");
        let smith_transcript::Returned::Said(said) = result.returned() else {
            panic!("unexpected survey result: {:?}", result.returned());
        };
        if said.error() {
            assert!(said.text() == b"busy" || said.text() == b"refused reason=too-many", "{said:?}");
        } else {
            assert!(skein_lib::bytes::find(said.text(), b"survey answer").is_some(), "{said:?}");
            assert!(answers.insert(said.text()), "each child returns its own survey answer");
        }
    }
    assert_eq!(answers.len(), count.min(room), "admitted surveys answer");
    let mut live = std::collections::BTreeSet::new();
    let mut peak = 0;
    for record in &run.events {
        match &record.event {
            Event::ConversationOpened(opening) if opening.kind == ConversationKind::Child => {
                assert!(live.insert(opening.conversation));
                peak = peak.max(live.len());
            }
            Event::ConversationClosed(close) if close.conversation != 0 => {
                assert!(live.remove(&close.conversation), "each opened child closes once");
            }
            _ => {}
        }
    }
    assert_eq!(peak, count.min(room), "survey children run side by side");
    assert!(live.is_empty(), "every child conversation settles");
    assert!(run.run_directories.is_empty(), "private program directories settle");
}

#[test]
fn a_program_main_survives_a_sub_agent_burst() {
    let room = usize::try_from(smith_agent_process_world::limits().domain.run.run_conversations - 1).expect("room");
    let mut counts = vec![2, 5, 6, room, room + 1];
    counts.sort_unstable();
    counts.dedup();
    for (direct, count) in [false, true].into_iter().flat_map(|direct| counts.iter().map(move |count| (direct, *count)))
    {
        let run = program_world::run(
            Case::SurveyBurst { direct },
            &survey_story(count),
            Some(b"claude.ai"),
            &survey_charter(direct),
            &program(b"source"),
            None,
        );
        survey_answers(&run, count, room);
        let programs = run
            .invocations
            .iter()
            .filter(|invocation| argument(&invocation.arguments, "--effort") == Some("high"))
            .count();
        assert_eq!(programs, if direct { 0 } else { count.min(room) }, "the selected child backend runs");
    }
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
        .filter(|invocation| argument(&invocation.arguments, "--effort") == Some("high"))
        .collect();
    assert_eq!(children.len(), 2);
    for child in children {
        assert_eq!(child.arguments[0], "-p");
        assert_eq!(argument(&child.arguments, "--model"), Some("fake"));
        assert_eq!(argument(&child.arguments, "--tools"), Some("Read,Glob,Grep"));
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
    assert_eq!(run.run_directories.len(), 0);
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

fn chat_charter() -> Box<[u8]> {
    smith_agent_process_world::wire::edit_charter(&program_world::child_charter(), |parts| {
        parts.mode = smith_charter::Mode::Open;
        parts.resume = true;
        let mut tools = parts.tools.clone().into_parts();
        tools.wait = true;
        tools.deliver = None;
        parts.tools = smith_charter::Tools::new(&smith_charter::CEILINGS, tools).expect("chat tools");
        parts.waiting = skein_lib::Duration::from_millis(10);
        smith_agent_process_world::wire::budget(parts, 24, 80);
        let mut main = parts.main.clone().into_parts();
        main.effort = Some(b"medium".as_slice().into());
        parts.main = smith_charter::Llm::new(&smith_charter::CEILINGS, main).expect("main effort");
        parts.brief = skein_lib::List::with_capacity(1);
        parts
            .brief
            .push(
                smith_charter::Section::new(
                    &smith_charter::CEILINGS,
                    smith_charter::SectionParts {
                        title: b"Request".as_slice().into(),
                        body: b"Have two sub-agents read src/main.rs and README.md side by side.".as_slice().into(),
                    },
                )
                .expect("chat brief"),
            )
            .expect("one brief");
    })
}

fn inspect_child(cue: &str, named: bool) -> Step {
    let mut input = json!({"brief":format!("{cue} Read the file and report."),"tools":["inspect"],"max_turns":3});
    if named {
        input["llm"] = json!("fake");
    }
    Step::Call { tool: "sub_agent".into(), input }
}

fn chat_story() -> Story {
    Story {
        turns: vec![
            Turn::new(vec![Step::Together(vec![inspect_child("CUE-MAIN", true), inspect_child("CUE-README", true)])]),
            Turn::new(vec![Step::Text("Both files read.".into())]),
        ],
        asked: vec![Script {
            cue: "FOLLOWUP".into(),
            turns: vec![
                Turn::new(vec![Step::Together(vec![
                    inspect_child("CUE-README", true),
                    inspect_child("CUE-MAIN", false),
                ])]),
                Turn::new(vec![Step::Text("Read the README again.".into())]),
            ],
        }],
        cued: [
            ("CUE-MAIN", "/work/repo/src/main.rs", "main contains an empty entry point"),
            ("CUE-README", "/work/repo/README.md", "README describes a small example"),
        ]
        .into_iter()
        .map(|(cue, path, answer)| Script {
            cue: cue.into(),
            turns: vec![Turn::new(vec![read(path)]), Turn::new(vec![Step::Text(answer.into())])],
        })
        .collect(),
        ..Default::default()
    }
}

fn chat_parked(run: &Run, children: usize) {
    assert!(
        matches!(run.answer().expect("answer").result(), smith_channel::RunResult::Parked),
        "answer={:?}, errors={}",
        run.answer(),
        String::from_utf8_lossy(&run.errors)
    );
    let results = results(run);
    assert_eq!(results.len(), children, "seen={:?}", run.seen);
    for result in results {
        assert!(
            matches!(result.returned(), smith_transcript::Returned::Said(said)
            if !said.error() && skein_lib::bytes::find(said.text(), b"unanswered").is_none()
                && (skein_lib::bytes::find(said.text(), b"README describes a small example").is_some()
                    || skein_lib::bytes::find(said.text(), b"main contains an empty entry point").is_some())),
            "child must return its scripted answer: {:?}; faults={:?}",
            result.returned(),
            program_faults(run)
        );
    }
    let ends: Vec<_> = run
        .events
        .iter()
        .filter_map(|record| match &record.event {
            Event::ConversationClosed(close) if close.conversation != 0 => Some(close.end.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(ends.len(), children);
    assert!(ends.iter().all(|end| *end == ConversationEnd::Closed));
    assert_eq!(run.run_directories.len(), 0);
}

fn print_calls(run: &Run) -> Vec<&smith_fake_claude::Observation> {
    run.invocations.iter().filter(|invocation| invocation.arguments[0] == "-p").collect()
}

#[test]
fn a_resumed_program_main_can_start_fresh_children() {
    let story = chat_story();
    let charter = chat_charter();
    let control = program_world::HostMessage {
        when: program_world::When::Waiting(1),
        text: "FOLLOWUP Have two sub-agents read README.md and src/main.rs once more.".into(),
        finish_requested: false,
    };
    let mut first = program_world::children_chat(&story, &charter, &[control]);
    chat_parked(&first, 4);
    let before: Vec<_> = print_calls(&first)
        .iter()
        .map(|call| argument(&call.arguments, "--session-id").expect("fresh session").to_owned())
        .collect();
    assert_eq!(before.len(), 5, "same-activation follow-up keeps main alive");
    assert!(first.seen.contains(&Seen::Acted {
        tool: "Read".into(),
        path: "/work/repo/src/main.rs".into(),
        error: false
    }));
    let later = program_world::resume_children(&mut first, &story, &charter, 2);
    chat_parked(&later, 2);
    let calls = print_calls(&later);
    assert_eq!(calls.len(), 3);
    assert_eq!(argument(&calls[0].arguments, "--resume"), Some(before[0].as_str()));
    let sessions: Vec<_> = calls.iter().filter_map(|call| argument(&call.arguments, "--session-id")).collect();
    assert_eq!(sessions.len(), 2, "both children start fresh sessions");
    assert_ne!(sessions[0], sessions[1], "children have distinct sessions");
    let mut efforts = Vec::new();
    for session in sessions {
        let call = calls
            .iter()
            .find(|call| argument(&call.arguments, "--session-id") == Some(session))
            .expect("fresh child's call");
        assert!(argument(&call.arguments, "--resume").is_none());
        assert!(!before.iter().any(|prior| prior == session), "each activation needs fresh children");
        efforts.push(argument(&call.arguments, "--effort"));
    }
    efforts.sort_unstable();
    assert_eq!(efforts, [Some("high"), Some("medium")], "named and inherited efforts both reach fresh children");
    assert!(later.seen.contains(&Seen::Acted {
        tool: "Read".into(),
        path: "/work/repo/README.md".into(),
        error: false
    }));
}

fn program_faults(run: &Run) -> Vec<String> {
    run.events
        .iter()
        .filter_map(|record| match &record.event {
            Event::ConversationClosed(close) => {
                close.failure.as_ref()?.detail.as_ref().map(|text| String::from_utf8_lossy(text).into_owned())
            }
            Event::ResponseCompleted(response) => {
                response.failure.as_ref()?.detail.as_ref().map(|text| String::from_utf8_lossy(text).into_owned())
            }
            _ => None,
        })
        .collect()
}
