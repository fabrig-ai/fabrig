//! A long document built by bounded calls, then found by a fresh run's brief
//! (temper/docs/design/domain/agent.md, 8.3; temper/docs/design/protocol/setup.md, 8.4).
use super::{Act, Call, Walk, chat_document::written_turn, ctl};
use serde_json::{Value, json};
use std::os::unix::fs::PermissionsExt;

const CUE: &str = "Build the long document in bounded parts.";
const TITLE: &str = "Parts";
const BUILT: &str = "I built the document in parts and checked both refusals.";
const REVISED: &str = "I found the document in my brief and revised it by lines.";

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

fn call(name: &str, arguments: &Value) -> Value {
    let bytes = serde_json::to_vec(&arguments).expect("tool arguments").len();
    assert!(bytes < 10_000, "{name} arguments: {bytes} bytes");
    eprintln!("{name} arguments: {bytes} bytes");
    json!({"lines": [{"call": name, "arguments": arguments}], "finish": "tool_calls",
        "tokens": (name.len() + bytes).div_ceil(4)})
}

fn sentence(text: &str) -> Value {
    json!({"lines": [{"text": text}], "finish": "stop", "tokens": text.len().div_ceil(4)})
}

struct Script {
    turns: Vec<Value>,
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
        .map(|turn| serde_json::to_vec(&turn["lines"][0]["arguments"]).expect("arguments").len())
        .max()
        .expect("note calls");
    assert!(lines.iter().any(|line| line.len() > largest_call), "a line longer than any call");
    assert!(lines.join("\n").len() >= 30_000);
    Script { turns, results, lines }
}

fn pane(answer: &super::Ended, revision: u32, body: &str) {
    let page = ctl::json(answer);
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

fn older_activity(seen: &[super::Ended]) -> Call {
    let page = String::from_utf8_lossy(&seen.last().expect("activity page").stdout);
    let cursor = page
        .lines()
        .find_map(|line| line.strip_prefix("read on with: task activity 1 --before "))
        .unwrap_or_else(|| panic!("activity continuation: {page}"));
    Call::new(&["task", "activity", "1", "--before", cursor])
}

fn prepare(walk: &Walk, turns: &[Value], fresh: &Value) -> Vec<Act> {
    let initial = walk.run(vec![Act::Now(Call::new(&["settings", "show", "charter", "1", "--json"]))]);
    let mut charter = ctl::json(&initial.answers[0]);
    charter["value"]["resume"] = json!(false);
    let settings = walk.scratch.path().join("fresh-chat.json");
    super::super::write(
        &settings,
        &serde_json::to_vec(&json!({"items": [{
            "setting": charter["setting"], "set": charter["value"],
        }]}))
        .expect("fresh-run charter"),
    );
    let against = charter["revision"].as_u64().expect("charter revision").to_string();
    let config = walk.sandbox().join("config/temper.json");
    let mut document: Value = serde_json::from_slice(&std::fs::read(&config).expect("configuration")).expect("JSON");
    // These are this walk's scripts, within the combined eight-script bound.
    document["application"]["demo"]["llm"]["cued"] = json!([
        fresh, {"cue": CUE, "turns": turns},
    ]);
    std::fs::write(&config, serde_json::to_vec_pretty(&document).expect("script JSON")).expect("walk script");
    assert_eq!(std::fs::metadata(&config).expect("configuration mode").permissions().mode() & 0o777, 0o600);

    vec![
        Act::Now(Call::new(&["playground", "start", "--key", "parts-playground", "--json"])),
        Act::Now(Call::new(&["project", "show", "1", "--json"])),
        // Select fresh briefs rather than concrete transcript resumption;
        // every budget, timeout and size limit keeps its configured value.
        Act::Now(Call::new(&[
            "settings",
            "apply",
            super::path(&settings),
            "--against",
            &against,
            "--key",
            "parts-fresh",
        ])),
        Act::After(Box::new(|seen| {
            assert_eq!(ctl::json(&seen[1])["number"], 1, "script's literal project");
            // Call::new supplies empty stdin, the harness's </dev/null.
            Call::new(&["chat", "start", "--project", "1", "--model", "demo", "--key", "parts-chat", CUE])
        })),
        Act::Now(Call::new(&["task", "show", "1", "--until", "replied", "--json"])),
        Act::Now(Call::new(&["task", "notes", "1", "--project", "1", "--json"])),
    ]
}

fn read_results(acts: &mut Vec<Act>, results: &[String], body: String) -> Vec<usize> {
    acts.push(Act::After(Box::new(move |seen| {
        pane(&seen[5], 5, &body);
        Call::new(&["task", "activity", "1"])
    })));
    // The engine carries two activity rows per page; the eight committed
    // turns occupy four pages, newest first. Follow its opaque cursors.
    for _ in 0..3 {
        acts.push(Act::After(Box::new(older_activity)));
    }
    let mut read_results = Vec::new();
    for expected in results {
        let expected = expected.clone();
        read_results.push(acts.len());
        acts.push(Act::After(Box::new(move |seen| {
            let page = seen[6..10]
                .iter()
                .find(|answer| String::from_utf8_lossy(&answer.stdout).contains(&expected))
                .unwrap_or_else(|| panic!("activity missing {expected}"));
            written_turn(std::slice::from_ref(page), &expected)
        })));
    }
    read_results
}

fn start_fresh(acts: &mut Vec<Act>, walk: &Walk, index: String, result: String) -> usize {
    let follow_up = walk.scratch.path().join("follow-up.txt");
    super::super::write(&follow_up, b"Revise line 40 of the existing document.");
    acts.push(Act::Now(Call::new(&["task", "stop", "1", "--key", "parts-stop"])));
    acts.push(Act::Now(Call::new(&["task", "show", "1", "--until", "held", "--json"])));
    acts.push(Act::Now(Call::new(&["task", "run", "1", "--until", "ended", "--json"])));
    // Replace the original request's script cue. The follow-up supplies no
    // note name or revision: the fresh run must get those from its brief.
    acts.push(Act::Now(Call::new(&[
        "task",
        "amend",
        "1",
        "--rev",
        "0",
        "--spec",
        super::path(&follow_up),
        "Continue this chat from its document.",
        "--key",
        "parts-next",
    ])));
    acts.push(Act::After(Box::new(|seen| {
        let held = ctl::json(&seen[seen.len() - 3]);
        let revision = held["task"]["hold"]["revision"].as_u64().unwrap_or_else(|| panic!("hold revision: {held}"));
        Call::new(&["task", "release", "1", "--hold", &revision.to_string(), "--key", "parts-release"])
    })));
    acts.push(Act::Now(Call::new(&["task", "show", "1", "--until", "replied", "--json"])));
    acts.push(Act::Now(Call::new(&["task", "turn", "1", "2.1"])));
    acts.push(Act::After(Box::new(move |seen| {
        let turn = String::from_utf8_lossy(&seen.last().expect("fresh opening turn").stdout);
        assert!(turn.contains(&index), "the fresh brief indexes the existing note: {turn}");
        Call::new(&["task", "activity", "1"])
    })));
    let fresh_turn = acts.len();
    acts.push(Act::After(Box::new(move |seen| written_turn(seen, &result))));
    acts.push(Act::Now(Call::new(&["task", "notes", "1", "--project", "1", "--json"])));
    fresh_turn
}

#[test]
fn a_long_document_is_built_in_parts_on_the_fake_box() {
    let script = script();
    let body = script.lines.join("\n");
    let mut revised = script.lines.clone();
    revised[39] = "The fresh run changed this line after finding the note in its brief.".into();
    let revised_body = revised.join("\n");
    let fresh_result = format!("Note 1 revision 6 written · +1 −1 · line 40 · {} lines", revised.len());
    // Only a fresh brief's note index holds this cue, including its revision.
    let index = format!("1 r5: {TITLE}");
    let fresh = json!({"cue": index, "turns": [
        call("note", &range(&revised[39], 5, 40, 1)), sentence(REVISED),
    ]});
    let walk = Walk::new("walk-note-parts");
    let mut acts = prepare(&walk, &script.turns, &fresh);
    let read_results = read_results(&mut acts, &script.results, body.clone());
    let fresh_turn = start_fresh(&mut acts, &walk, index.clone(), fresh_result.clone());
    let seen = walk.run_then(acts, |_, _| ());
    let started = String::from_utf8_lossy(&seen.answers[3].stdout);
    assert!(started.contains("done: task 1, chat"), "script's literal chat: {started}");
    assert_eq!(ctl::json(&seen.answers[4])["met"], true);
    for (index, (at, expected)) in read_results.into_iter().zip(&script.results).enumerate() {
        let turn = String::from_utf8_lossy(&seen.answers[at].stdout);
        assert!(turn.starts_with(&format!("turn 1.{}\n", index + 1)), "committed turns in order: {turn}");
        assert!(turn.contains(expected), "complete tool result: {turn}");
    }
    let activity = String::from_utf8_lossy(&seen.answers[6].stdout);
    assert!(activity.contains(BUILT), "the scripted chat finished: {activity}");
    let turn = String::from_utf8_lossy(&seen.answers[fresh_turn].stdout);
    assert!(turn.starts_with("turn 2."), "a fresh attempt changed the note: {turn}");
    assert!(turn.contains(&fresh_result), "fresh run's complete result: {turn}");
    assert!(turn.contains(&index), "the fresh brief indexes the existing note: {turn}");
    let fresh_activity = String::from_utf8_lossy(&seen.answers[fresh_turn - 1].stdout);
    assert!(fresh_activity.contains(REVISED), "the fresh run replied: {fresh_activity}");
    pane(seen.answers.last().expect("pane after fresh run"), 6, &revised_body);
    eprintln!("note body: {} bytes; fresh revision: {} bytes", body.len(), revised_body.len());
    walk.scratch.remove();
}
