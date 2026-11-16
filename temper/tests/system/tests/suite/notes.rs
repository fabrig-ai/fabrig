//! Bounded chat documents and a fresh brief across engine, worker and smith.
//! Contract: temper/docs/design/domain/agent/agents.md, 3;
//! temper/docs/design/testing/process-tier.md, “A long document built in parts”.
use jig_domain_core::{TurnRecord, documents, reads};
use jig_domain_people as people;
use serde_json::{Value, json};
use skein_fake_llm_domain::api;
use temper_system_world::composed::World;

const CUE: &str = "Build the long document in bounded parts.";
const TITLE: &str = "Parts";
const BUILT: &str = "I built the document in parts and checked both refusals.";
const REVISED: &str = "I found the document in my brief and revised it by lines.";
const REPLACEMENT: &str = "The fresh run changed this line after finding the note in its brief.";
const INDEX: &str = "1 r5: Parts";
const FRESH_RESULT: &str = "Note 1 revision 6 written · +1 −1 · line 40 · 154 lines";

fn line(number: usize, bytes: usize) -> String {
    let mut text = format!("Part line {number:03}: ");
    text.push_str(&"x".repeat(bytes - text.len()));
    assert_eq!(text.len(), bytes);
    text
}

fn note(body: &str) -> Value {
    json!({"scope": {"kind": "goal", "project": 1, "goal": 1}, "description": TITLE, "body": body})
}

fn range(body: &str, recalled: u32, first: usize, remove: usize) -> Value {
    let mut arguments = note(body);
    arguments["name"] = json!(1);
    arguments["recalled"] = json!(recalled);
    arguments["first"] = json!(first);
    arguments["remove"] = json!(remove);
    arguments
}

fn call(name: &str, arguments: &Value) -> api::Turn {
    let bytes = serde_json::to_vec(&arguments).expect("tool arguments").len();
    assert!(bytes < 10_000, "{name} arguments: {bytes} bytes");
    eprintln!("{name} arguments: {bytes} bytes");
    api::Turn {
        lines: Box::new([api::Line::Call {
            name: name.as_bytes().into(),
            arguments: serde_json::to_vec(arguments).expect("tool arguments").into(),
        }]),
        finish: api::Finish::ToolCalls,
        tokens: u64::try_from((name.len() + bytes).div_ceil(4)).expect("tool tokens"),
    }
}

fn sentence(text: &str) -> api::Turn {
    api::Turn {
        lines: Box::new([api::Line::Text { text: text.as_bytes().into() }]),
        finish: api::Finish::Stop,
        tokens: u64::try_from(text.len().div_ceil(4)).expect("reply tokens"),
    }
}

struct Script {
    turns: Vec<api::Turn>,
    results: Vec<String>,
    lines: Vec<String>,
}

fn script() -> Script {
    // The first two payloads form one 15 KB line. The remaining payloads
    // append ordinary lines to bodies without a final newline.
    let parts = [
        vec![line(1, 7_500)],
        vec![line(2, 7_500)],
        (3..79).map(|number| line(number, 100)).collect(),
        (79..155).map(|number| line(number, 100)).collect(),
    ];
    let mut lines = parts[0].clone();
    let mut turns = vec![call("note", &note(&lines.join("\n")))];
    let mut results = vec!["Note 1 revision 1 written · 1 lines".into()];
    for (index, part) in parts.iter().enumerate().skip(1) {
        let mut arguments = range(&part.join("\n"), u32::try_from(index).expect("revision"), lines.len() + 1, 0);
        let change = if index == 1 {
            arguments["join"] = json!(true);
            lines.last_mut().expect("first part").push_str(&part.join("\n"));
            "+1 −1 · line 1".to_owned()
        } else {
            let first = lines.len() + 1;
            lines.extend(part.iter().cloned());
            format!("+{} −0 · line {first}", part.len())
        };
        turns.push(call("note", &arguments));
        results.push(format!("Note 1 revision {} written · {change} · {} lines", index + 1, lines.len()));
    }
    let replacement: Vec<_> = (155..158).map(|number| line(number, 100)).collect();
    turns.push(call("note", &range(&replacement.join("\n"), 4, 40, 2)));
    // Work out the body from a list of lines, independently of notes::splice.
    lines = lines.iter().take(39).chain(replacement.iter()).chain(lines.iter().skip(41)).cloned().collect();
    results.push(format!("Note 1 revision 5 written · +3 −2 · line 40 · {} lines", lines.len()));
    turns.push(call("note", &range("This stale edit must not be saved.", 1, 2, 0)));
    results.push("Note refused: revision moved, now 5".into());
    turns.push(call("note", &range("This out-of-range removal must not be saved.", 5, lines.len() + 2, 1)));
    results.push(format!("Note refused: range past the end, {} lines", lines.len()));
    turns.push(sentence(BUILT));
    assert_eq!(turns.len(), 8);
    let largest_call = turns
        .iter()
        .take(7)
        .map(|turn| {
            let [api::Line::Call { arguments, .. }] = turn.lines.as_ref() else { panic!("one note call") };
            arguments.len()
        })
        .max()
        .expect("note calls");
    assert!(lines.iter().any(|line| line.len() > largest_call), "a line longer than any call");
    assert!(lines.join("\n").len() >= 30_000);
    Script { turns, results, lines }
}

fn pane(world: &mut World, revision: u32, body: &str) {
    let answer = world.notes_read(reads::Read::ScopedNotes {
        scope: jig_domain_notes::Scope::Goal { project: 1, goal: 1 },
        description: None,
        from: None,
    });
    let reads::Answer::Page(page) = answer else { panic!("pane's notes page: {answer:?}") };
    assert!(page.next.is_none(), "one whole page");
    assert!(!page.cut);
    let [reads::PageRow::Note(note)] = page.rows.as_slice() else { panic!("one chat document") };
    assert_eq!(note.number, 1);
    assert_eq!(note.revision, revision);
    assert_eq!(note.description.bytes.as_ref(), TITLE.as_bytes());
    assert!(!note.cut, "the pane reads the whole document");
    assert!(!note.body.cut);
    assert_eq!(note.body.bytes.as_ref(), body.as_bytes(), "pane body, byte for byte");
    assert!(note.body.bytes.len() >= 30_000);
}

fn task(world: &mut World) -> Box<documents::Task> {
    let answer = world.notes_read(reads::Read::Task { task: 1 });
    let reads::Answer::Document(document) = answer else { panic!("chat document: {answer:?}") };
    let documents::Document::Task(task) = *document else { panic!("chat overview") };
    assert_eq!(task.chip.number, 1, "script's literal chat");
    assert_eq!(task.project, 1, "script's literal project");
    task
}

fn turn(world: &mut World, attempt: u64, turn: u32) -> TurnRecord {
    let answer = world.notes_read(reads::Read::Turn { task: 1, attempt, turn });
    let reads::Answer::Ended(ended) = answer else { panic!("committed turn: {answer:?}") };
    let reads::EndedValue::Turn(record) = ended.value else { panic!("whole turn") };
    assert_eq!((record.task, record.attempt, record.turn), (1, attempt, turn), "committed turns in order");
    record
}

fn activity_turns(world: &mut World) -> Vec<TurnRecord> {
    let mut before = None;
    let mut turns = Vec::new();
    loop {
        let answer = world.notes_read(reads::Read::Activity { task: 1, before });
        let reads::Answer::Page(page) = answer else { panic!("task activity: {answer:?}") };
        assert!(!page.cut);
        for row in &page.rows {
            if let reads::PageRow::Activity { line, turn } = row
                && matches!(line.line, jig_domain_tasks::Line::Turn { .. })
            {
                let Some(reads::PageTurn::Carried(turn)) = turn else { panic!("activity's whole turn") };
                assert_eq!(turn.task, 1);
                turns.push(turn.clone());
            }
        }
        let Some(next) = page.next else { break };
        before = Some(next);
    }
    // Activity pages are newest first; the story reads committed order.
    turns.reverse();
    turns
}

fn decoded(record: &TurnRecord) -> smith_transcript::Turn {
    smith_transcript::decode_archived(&smith_transcript::CEILINGS, &record.transcript).expect("whole archived turn")
}

fn result(record: &TurnRecord, expected: &str) {
    let turn = decoded(record);
    let results: Vec<_> = turn
        .messages()
        .iter()
        .flat_map(smith_transcript::Message::blocks)
        .filter_map(|block| if let smith_transcript::Block::Result(result) = block { Some(result) } else { None })
        .collect();
    let [result] = results.as_slice() else { panic!("one tool result: {turn:?}") };
    let smith_transcript::Returned::Said(said) = result.returned() else { panic!("host's complete tool result") };
    assert_eq!(said.text(), expected.as_bytes(), "complete tool result");
    assert_eq!(said.error(), expected.starts_with("Note refused:"), "successful or refused call");
}

fn reply(record: &TurnRecord, expected: &str) {
    let turn = decoded(record);
    assert!(
        turn.messages()
            .iter()
            .filter(|message| *message.role() == smith_transcript::Role::Assistant)
            .flat_map(smith_transcript::Message::blocks)
            .any(|block| {
                matches!(block, smith_transcript::Block::Text(text) if text.text() == expected.as_bytes())
            }),
        "the scripted reply is committed"
    );
}

fn follow_up(world: &mut World) {
    let stopped = world.notes_act(people::Ask::Stop { project: 1, task: 1 }, [70; 16]);
    assert_eq!(stopped, people::Reply::Outcome(people::Outcome::Stopped { task: 1 }));
    world.run();
    let held = task(world);
    assert_eq!(held.hold.expect("the person's hold").revision, 1);
    assert_eq!(held.spec_revision, 0);
    let amended = world.notes_act(
        people::Ask::Amend {
            project: 1,
            task: 1,
            revision: 0,
            release: None,
            amendment: people::Amendment {
                tracked: None,
                addressed: None,
                instructions: None,
                spec: Some(people::Spec {
                    words: b"Revise line 40 of the existing document.".as_slice().into(),
                    parameters: Box::new([]),
                    inputs: Box::new([]),
                    asks: false,
                }),
                wake: None,
                dependencies: None,
                authority: None,
                llm: None,
                reason: b"Continue this chat from its document.".as_slice().into(),
            },
        },
        [71; 16],
    );
    assert_eq!(amended, people::Reply::Outcome(people::Outcome::Amended { task: 1, released: false }));
    world.notes_follow_up(api::Script {
        // Only the fresh brief's note index holds this cue and its revision.
        cue: INDEX.as_bytes().into(),
        turns: Box::new([call("note", &range(REPLACEMENT, 5, 40, 1)), sentence(REVISED)]),
    });
    let released = world.notes_act(people::Ask::Release { project: 1, task: 1, revision: 1, again: None }, [72; 16]);
    assert_eq!(released, people::Reply::Outcome(people::Outcome::Released { task: 1 }));
    world.run();
}

#[test]
fn a_long_document_is_built_in_parts_and_revised_by_a_fresh_run() {
    let script = script();
    let body = script.lines.join("\n");
    assert_eq!(script.lines.len(), 154);
    assert_eq!(body.len(), 30_453);
    let mut revised = script.lines.clone();
    revised[39] = REPLACEMENT.into();
    let revised_body = revised.join("\n");
    let mut world =
        World::notes_chat(941, CUE.as_bytes(), api::Script { cue: CUE.as_bytes().into(), turns: script.turns.into() });
    world.run();
    let answer = world.notes_read(reads::Read::Project { project: 1 });
    let reads::Answer::Document(document) = answer else { panic!("project read") };
    let documents::Document::Project(project) = *document else { panic!("project document") };
    assert_eq!(project.number, 1);
    task(&mut world);
    let activity = activity_turns(&mut world);
    assert_eq!(activity.len(), 8);
    for (index, expected) in script.results.iter().enumerate() {
        let turn = turn(&mut world, 1, u32::try_from(index + 1).expect("turn number"));
        result(&turn, expected);
        assert_eq!(activity[index], turn, "activity carries the same whole turn");
        result(&activity[index], expected);
    }
    pane(&mut world, 5, &body);
    assert_eq!(activity[7], turn(&mut world, 1, 8));
    reply(&activity[7], BUILT);
    follow_up(&mut world);
    assert_eq!(world.assignment().attempt, 2);
    assert!(!world.assignment().run.policy.resume);
    assert!(world.assignment().transcript.is_empty(), "a fresh run, not a resumed conversation");
    let fresh = turn(&mut world, 2, 1);
    // An Open chat's task brief is user context beside the host's system text.
    let decoded = decoded(&fresh);
    assert!(
        decoded.messages().iter().flat_map(smith_transcript::Message::blocks).any(|block| {
            matches!(block, smith_transcript::Block::Text(text)
            if text.text().windows(INDEX.len()).any(|bytes| bytes == INDEX.as_bytes()))
        }),
        "the fresh brief indexes revision 5"
    );
    result(&fresh, FRESH_RESULT);
    let activity = activity_turns(&mut world);
    assert_eq!(activity.len(), 10);
    assert_eq!(activity[8], fresh, "activity's fresh turn 2.1");
    result(&activity[8], FRESH_RESULT);
    assert_eq!(activity[9], turn(&mut world, 2, 2));
    reply(&activity[9], REVISED);
    pane(&mut world, 6, &revised_body);
    eprintln!("note body: {} bytes; fresh revision: {} bytes", body.len(), revised_body.len());
    let stopped = world.notes_act(people::Ask::Stop { project: 1, task: 1 }, [73; 16]);
    assert_eq!(stopped, people::Reply::Outcome(people::Outcome::Stopped { task: 1 }));
    world.run();
}
