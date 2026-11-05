//! A chat writes and revises its document through ctl and the real tools
//! (temper/docs/design/domain/agent.md, 8.3).
use super::{Act, Call, Ended, Walk, ctl};

const FIRST: &str = "Note 1 revision 1 written · 29 lines";
const SECOND: &str = "Note 1 revision 2 written · +7 −0 · line 14";

/// Find the committed turn through activity, then read its complete tool result.
fn written_turn(seen: &[Ended], expected: &str) -> Call {
    let activity = String::from_utf8_lossy(&seen.last().expect("task activity").stdout);
    let written = activity.find(expected).unwrap_or_else(|| panic!("missing {expected}: {activity}"));
    let turn = activity[..written]
        .lines()
        .rev()
        .find_map(|line| line.strip_prefix("turn "))
        .expect("the write belongs to a committed turn")
        .split_whitespace()
        .next()
        .expect("attempt.turn");
    Call::new(&["task", "turn", "1", turn])
}

#[test]
fn a_chat_document_is_written_and_revised_on_the_fake_box() {
    let walk = Walk::new("walk-chat-document");
    let seen = walk.run(vec![
        Act::Now(Call::new(&["playground", "start", "--key", "document-playground", "--json"])),
        Act::Now(Call::new(&["project", "show", "1", "--json"])),
        Act::After(Box::new(|seen| {
            // These literal bytes in the scripts name this fresh box's first
            // project P = 1, chat T = 1 and note name = 1. Another chat's scope
            // is refused; the scripted sentence still follows its tool result.
            assert_eq!(ctl::json(&seen[1])["number"], 1, "document's literal project");
            // Call::new supplies empty stdin, the harness's </dev/null.
            Call::new(&[
                "chat",
                "start",
                "--project",
                "1",
                "--model",
                "demo",
                "--key",
                "document-chat",
                "Please write this up. Write this up as a draft beside us.",
            ])
        })),
        Act::Now(Call::new(&["task", "show", "1", "--until", "replied", "--json"])),
        Act::Now(Call::new(&["task", "activity", "1"])),
        Act::After(Box::new(|seen| written_turn(seen, FIRST))),
        Act::Now(Call::new(&[
            "task",
            "message",
            "1",
            "Add what happens to a running goal.",
            "--key",
            "document-revise",
        ])),
        Act::Now(Call::new(&["task", "show", "1", "--until", "replied", "--json"])),
        Act::Now(Call::new(&["task", "activity", "1"])),
        Act::After(Box::new(|seen| written_turn(seen, SECOND))),
    ]);
    let started = String::from_utf8_lossy(&seen.answers[2].stdout);
    assert!(started.contains("done: task 1, chat"), "document's literal chat: {started}");
    for (index, expected) in [(5, FIRST), (9, SECOND)] {
        let turn = String::from_utf8_lossy(&seen.answers[index].stdout);
        assert!(turn.contains(expected), "complete note result: {turn}");
    }
    let first = String::from_utf8_lossy(&seen.answers[4].stdout);
    assert!(first.contains("I wrote an Away mode draft with its purpose, return flow and open questions."));
    let second = String::from_utf8_lossy(&seen.answers[8].stdout);
    assert!(second.contains("I added how a running goal continues and waits for your decisions."));
    walk.scratch.remove();
}
