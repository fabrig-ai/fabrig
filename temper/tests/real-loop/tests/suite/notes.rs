//! Chat documents revised whole after a follow-up
//! (temper/docs/design/testing/real-loop.md, temper/docs/design/testing/process-tier.md; temper/docs/design/domain/agent/agents.md, 3).
use serde_json::{Value, json};
use skein_fake_llm_domain::api;
use std::net::SocketAddr;
use temper_real_loop_world::{
    self as world, argv,
    calls::{self, D1},
    fixtures, forge,
};

fn configure_notes(document: &mut Value) {
    fixtures::requester(document);
    for project in document["world_settings"]["policies"].as_array_mut().expect("project policies") {
        project["policy"]["ceiling"]["tools"] = jig_domain_core::ToolFamilies::JIG.note.0.into();
        project["policy"]["ceiling"]["notes"] = jig_domain_authority::Scopes::GOAL.0.into();
        project["policy"]["roles"][0]["authority"]["tools"] = jig_domain_core::ToolFamilies::JIG.note.0.into();
        project["policy"]["roles"][0]["authority"]["notes"] = jig_domain_authority::Scopes::GOAL.0.into();
    }
    // Chat words and authority, or an amended specification and title, share this record.
    document["engine"]["limits"]["overrides"]["people.amendment_bytes"] = 256.into();
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

const DOCUMENT_CUE: &str = "Please write this up. Write this up as a draft beside us.";
const FOLLOW_UP: &str = "Add what happens to a running goal.";
const DRAFT_REPLY: &str = "I wrote an Away mode draft with its purpose, return flow and open questions.";
const FOLLOW_UP_REPLY: &str = "I added how a running goal continues and waits for your decisions.";
const DRAFT_RESULT: &str = "Note 1 revision 1 written · 29 lines";
const FOLLOW_UP_RESULT: &str = "Note 1 revision 2 written · +7 −0 · line 14 · 36 lines";

// The two whole-note arguments from the template's cued Away mode scripts.
fn document_draft() -> Value {
    let body = r"# Away mode

Draft, 2026-11-05, with the user.

## Purpose

Away mode lets the person step away from ongoing work.
Agents keep working within the authority already granted.
They leave a clear account of progress for the person's return.

The person chooses when to leave and when to return.
Away mode does not grant extra authority.

## When the person returns

Show the work completed while the person was away.
Keep the decisions that need the person in one place.
Make each question name the goal and the choice it needs.

The person can answer questions before starting more work.
The conversation and its document remain available.

## Open questions

- Where should the person switch away mode on?
- Should it apply to one project or every project?
- How should urgent questions be shown on return?

These choices remain for the person to settle.
";
    json!({
        "scope": {"kind": "goal", "project": 1, "goal": 1},
        "description": "Away mode",
        "body": body
    })
}

fn document_revision() -> Value {
    let body = r"# Away mode

Draft, 2026-11-05, with the user.

## Purpose

Away mode lets the person step away from ongoing work.
Agents keep working within the authority already granted.
They leave a clear account of progress for the person's return.

The person chooses when to leave and when to return.
Away mode does not grant extra authority.

## A running goal

A running goal continues within its existing authority and budget.
When it needs the person's decision, it waits with a clear question.
Other work that can proceed independently keeps moving.
Away mode never supplies an answer on the person's behalf.

## When the person returns

Show the work completed while the person was away.
Keep the decisions that need the person in one place.
Make each question name the goal and the choice it needs.

The person can answer questions before starting more work.
The conversation and its document remain available.

## Open questions

- Where should the person switch away mode on?
- Should it apply to one project or every project?
- How should urgent questions be shown on return?

These choices remain for the person to settle.
";
    json!({
        "scope": {"kind": "goal", "project": 1, "goal": 1},
        "description": "Away mode",
        "body": body,
        "name": 1,
        "recalled": 1
    })
}

fn document_scripts(_: SocketAddr) -> Box<[api::Script]> {
    Box::new([api::Script {
        cue: DOCUMENT_CUE.as_bytes().into(),
        turns: Box::new([
            call("note", &document_draft()),
            sentence(DRAFT_REPLY),
            call("note", &document_revision()),
            sentence(FOLLOW_UP_REPLY),
        ]),
    }])
}

fn document_steps() -> Vec<calls::Step> {
    vec![
        calls::sign_in(D1),
        calls::call(D1, argv!["project", "show", "1", "--json"]),
        calls::call(D1, argv!["chat", "start", "--project", "1", "--key", "document-chat", DOCUMENT_CUE]),
        calls::call(D1, argv!["task", "show", "1", "--until", "replied", "--json"]),
        calls::call(D1, argv!["task", "turn", "1", "1.1"]),
        calls::call(D1, argv!["task", "activity", "1"]),
        calls::call(D1, argv!["task", "run", "1", "--until", "ended", "--json"]),
        calls::call(D1, argv!["task", "message", "1", FOLLOW_UP, "--key", "document-revise"]),
        calls::call(D1, argv!["task", "show", "1", "--until", "replied", "--json"]),
        calls::call(D1, argv!["task", "turn", "1", "2.1"]),
        calls::call(D1, argv!["task", "activity", "1"]),
    ]
}

#[test]
fn a_chats_document_is_revised_whole_after_a_follow_up() {
    let fixture = forge::Fixture { configure: |document, _, _| configure_notes(document), ..forge::fixture() };
    let ran = world::run_forge(fixture, document_scripts, document_steps);
    calls_succeed(&ran);
    let seen = ran.calls();
    let project: Value = serde_json::from_slice(&seen[1].stdout).expect("project JSON");
    assert_eq!(project["number"], 1, "document's literal project");
    let started = String::from_utf8_lossy(&seen[2].stdout);
    assert!(started.contains("done: task 1, chat"), "document's literal chat: {started}");
    let first = String::from_utf8_lossy(&seen[4].stdout);
    assert!(first.starts_with("turn 1.1\n"), "draft's committed turn: {first}");
    assert!(first.contains(DRAFT_RESULT), "complete draft result: {first}");
    assert!(String::from_utf8_lossy(&seen[5].stdout).contains(DRAFT_REPLY), "the draft reply is committed");
    let second = String::from_utf8_lossy(&seen[9].stdout);
    assert!(second.starts_with("turn 2.1\n"), "follow-up's committed turn: {second}");
    assert!(second.contains(FOLLOW_UP_RESULT), "complete follow-up result: {second}");
    assert!(String::from_utf8_lossy(&seen[10].stdout).contains(FOLLOW_UP_REPLY), "the follow-up reply is committed");
}

fn calls_succeed(ran: &world::Ran) {
    for (position, call) in ran.calls().iter().enumerate() {
        assert_eq!(
            call.exit,
            Some(world::Exit::Code(0)),
            "call {position}: {}\n{}",
            String::from_utf8_lossy(&call.stdout),
            String::from_utf8_lossy(&call.stderr)
        );
    }
}
