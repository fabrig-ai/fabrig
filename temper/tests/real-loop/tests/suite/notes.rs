//! A long document built by bounded hosted-agent calls, then revised from a fresh brief
//! (temper/docs/design/domain/testing.md, 2.4, 2.6; temper/docs/design/domain/agent.md, 8.3).
use serde_json::{Value, json};
use skein_fake_llm_domain::api;
use std::{cell::RefCell, io::Write, net::SocketAddr, os::unix::fs::OpenOptionsExt};
use temper_real_loop_world::{
    self as world, argv,
    calls::{self, D1},
    fixtures, forge, llm, settings,
};

const CUE: &str = "Build the long document in bounded parts.";
const TITLE: &str = "Parts";
const BUILT: &str = "I built the document in parts and checked both refusals.";
const REVISED: &str = "I found the document in my brief and revised it by lines.";
const REPLACEMENT: &str = "The fresh run changed this line after finding the note in its brief.";
const INDEX: &str = "1 r5: Parts";
const FRESH_RESULT: &str = "Note 1 revision 6 written · +1 −1 · line 40 · 154 lines";
const HOLD_REVISION: &str = "1";

thread_local! {
    static CHARTER: RefCell<Option<Value>> = const { RefCell::new(None) };
}

fn configure(document: &mut Value, agent: &mut Value, _: SocketAddr) {
    fixtures::requester(document);
    for project in document["world_settings"]["policies"].as_array_mut().expect("project policies") {
        project["policy"]["ceiling"]["tools"] = jig_domain_core::ToolFamilies::JIG.note.0.into();
        project["policy"]["ceiling"]["notes"] = jig_domain_authority::Scopes::GOAL.0.into();
        project["policy"]["roles"][0]["authority"]["tools"] = jig_domain_core::ToolFamilies::JIG.note.0.into();
        project["policy"]["roles"][0]["authority"]["notes"] = jig_domain_authority::Scopes::GOAL.0.into();
    }
    // The shipped agent admits each 7.5 KiB part; the small note body bound stays.
    agent["profile"]["declared"]["tool_payload"] = 10_662.into();
    agent["endpoints"][0]["limits"]["output"] = 8192.into();
    document["engine"]["llm"]["models"][0]["output"] = 8192.into();
    document["application"]["limits"]["overrides"]["checkout.message_bytes"] = 10_662.into();
    // Follow that declared payload at both host-call admission boundaries.
    document["engine"]["limits"]["overrides"]["hub.outcome_bytes"] = 10_662.into();
    document["engine"]["limits"]["overrides"]
        .as_object_mut()
        .expect("engine overrides")
        .remove("core.call_answer_bytes");
    for worker in document["workers"].as_array_mut().expect("workers") {
        worker["limits"]["overrides"]["hub.event_bytes"] = 10_660.into();
        worker["limits"]["overrides"]["hub.outcome_bytes"] = 10_662.into();
        worker["limits"]["overrides"].as_object_mut().expect("worker overrides").remove("core.call_answer_bytes");
    }
    // A whole note record fits below the walk's standard-profile load allowance.
    document["engine"]["limits"]["overrides"]["core.load_bytes"] = 65_536.into();
    // The follow-up's specification and title share the amendment record.
    document["engine"]["limits"]["overrides"]["people.amendment_bytes"] = 256.into();
    let mut charter = settings::stock(document)["charter"].clone();
    charter["resume"] = false.into();
    CHARTER.with(|slot| *slot.borrow_mut() = Some(charter));
}

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
    turns.push(call("note", &range("This out-of-range edit must not be saved.", 5, lines.len() + 2, 0)));
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

fn scripts(_: SocketAddr) -> Box<[api::Script]> {
    Box::new([
        // Only the fresh brief's note index holds this cue, including its revision.
        api::Script {
            cue: INDEX.as_bytes().into(),
            turns: Box::new([call("note", &range(REPLACEMENT, 5, 40, 1)), sentence(REVISED)]),
        },
        api::Script { cue: CUE.as_bytes().into(), turns: script().turns.into() },
    ])
}

fn recipe(address: SocketAddr, _: &std::path::Path) -> llm::Recipe {
    // The peer owns both scripts and receives their growing conversation.
    llm::scripted_sized(scripts(address), 65_536)
}

fn pane(answer: &calls::Called, revision: u32, body: &str) {
    let page: Value = serde_json::from_slice(&answer.stdout).expect("pane JSON");
    assert!(page["after"].is_null(), "one whole page");
    let notes = page["notes"].as_array().expect("pane's notes read");
    let [note] = notes.as_slice() else { panic!("one chat document") };
    assert_eq!(note["number"], 1);
    assert_eq!(note["revision"], revision);
    assert_eq!(note["description"], TITLE);
    assert_eq!(note["cut"], false, "the pane reads the whole document");
    let read = note["body"].as_str().expect("body");
    assert!(
        read.as_bytes() == body.as_bytes(),
        "pane body, byte for byte: read {} bytes, expected {}",
        read.len(),
        body.len()
    );
    assert!(read.len() >= 30_000);
}

fn steps() -> Vec<calls::Step> {
    let charter = CHARTER.with(|slot| slot.borrow_mut().take().expect("fresh-run charter"));
    let mut steps = vec![calls::sign_in(D1)];
    steps.extend(fixtures::change_settings(D1, "parts-fresh", json!([settings::item("charter", 1, charter)])));
    steps.extend([
        calls::call(D1, argv!["project", "show", "1", "--json"]),
        calls::call(D1, argv!["chat", "start", "--project", "1", "--key", "parts-chat", CUE]).succeeds(),
        calls::call(D1, argv!["task", "show", "1", "--until", "replied", "--json"]),
        calls::call(D1, argv!["task", "notes", "1", "--project", "1", "--json"]),
        calls::call(D1, argv!["task", "activity", "1", "--json"]),
    ]);
    // Eight committed turns occupy four two-row activity pages, newest first.
    for previous in 6..9 {
        steps.push(calls::call(
            D1,
            argv!["task", "activity", "1", "--before", calls::activity_before(previous), "--json"],
        ));
    }
    for turn in ["1.1", "1.2", "1.3", "1.4", "1.5", "1.6", "1.7"] {
        steps.push(calls::call(D1, argv!["task", "turn", "1", turn]));
    }
    steps.extend([
        calls::call(D1, argv!["task", "stop", "1", "--key", "parts-stop"]),
        calls::call(D1, argv!["task", "show", "1", "--until", "held", "--json"]),
        calls::call(D1, argv!["task", "run", "1", "--until", "ended", "--json"]),
        // The follow-up gives no note number or revision; the brief supplies them.
        calls::call(
            D1,
            argv![
                "task",
                "amend",
                "1",
                "--rev",
                "0",
                "--spec",
                calls::file("follow-up.txt"),
                "Continue this chat from its document.",
                "--key",
                "parts-next"
            ],
        ),
        calls::call(D1, argv!["task", "release", "1", "--hold", HOLD_REVISION, "--key", "parts-release"]),
        calls::call(D1, argv!["task", "show", "1", "--until", "replied", "--json"]),
        calls::call(D1, argv!["task", "turn", "1", "2.1"]),
        calls::call(D1, argv!["task", "activity", "1"]),
        calls::call(D1, argv!["task", "notes", "1", "--project", "1", "--json"]),
        calls::call(D1, argv!["task", "stop", "1", "--key", "parts-finished"]),
        calls::call(D1, argv!["task", "show", "1", "--until", "held", "--json"]),
        calls::call(D1, argv!["task", "run", "1", "--until", "ended", "--json"]),
    ]);
    steps
}

#[test]
fn a_long_document_is_built_in_parts_and_revised_by_a_fresh_run() {
    let script = script();
    let body = script.lines.join("\n");
    let mut revised = script.lines.clone();
    revised[39] = REPLACEMENT.into();
    let revised_body = revised.join("\n");
    let scratch = world::Scratch::new("real-note-parts");
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(scratch.path().join("follow-up.txt"))
        .expect("follow-up file")
        .write_all(b"Revise line 40 of the existing document.")
        .expect("follow-up text");
    let peer = world::ForgeRecipe { source: scratch.path().into(), build: recipe };
    let ran = world::run_forge_with_on(scratch, forge::Fixture { configure, ..forge::fixture() }, peer, steps);
    let seen = ran.calls();
    for (position, call) in seen.iter().enumerate() {
        assert_eq!(
            call.exit,
            Some(world::Exit::Code(0)),
            "call {position}: {}\n{}",
            String::from_utf8_lossy(&call.stdout),
            String::from_utf8_lossy(&call.stderr)
        );
    }
    let project: Value = serde_json::from_slice(&seen[2].stdout).expect("project JSON");
    assert_eq!(project["number"], 1, "script's literal project");
    let started = String::from_utf8_lossy(&seen[3].stdout);
    assert!(started.contains("done: task 1, chat"), "script's literal chat: {started}");
    let replied: Value = serde_json::from_slice(&seen[4].stdout).expect("replied task JSON");
    assert_eq!(replied["met"], true);
    for (index, expected) in script.results.iter().enumerate() {
        let turn = String::from_utf8_lossy(&seen[10 + index].stdout);
        assert!(turn.starts_with(&format!("turn 1.{}\n", index + 1)), "committed turns in order: {turn}");
        assert!(turn.contains(expected), "complete tool result: {turn}");
        assert!(
            seen[6..10].iter().any(|page| String::from_utf8_lossy(&page.stdout).contains(expected)),
            "activity missing {expected}"
        );
    }
    pane(&seen[5], 5, &body);
    assert!(String::from_utf8_lossy(&seen[6].stdout).contains(BUILT), "the scripted chat finished");
    let held: Value = serde_json::from_slice(&seen[18].stdout).expect("held task JSON");
    assert_eq!(held["task"]["hold"]["revision"].as_u64(), Some(HOLD_REVISION.parse().expect("hold revision")));
    let turn = String::from_utf8_lossy(&seen[23].stdout);
    assert!(turn.starts_with("turn 2.1\n"), "a fresh attempt changed the note: {turn}");
    assert!(turn.contains(INDEX), "the fresh brief indexes the existing note: {turn}");
    assert!(turn.contains(FRESH_RESULT), "fresh run's complete result: {turn}");
    assert!(String::from_utf8_lossy(&seen[24].stdout).contains(REVISED), "the fresh run replied");
    pane(&seen[25], 6, &revised_body);
    eprintln!("note body: {} bytes; fresh revision: {} bytes", body.len(), revised_body.len());
}
