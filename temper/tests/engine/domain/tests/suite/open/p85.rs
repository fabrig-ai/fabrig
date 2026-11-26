//! P85's symptom: a held relay answer accounts to its requester, then reaches its asker.
//! temper/docs/design/domain/forge/change/questions.md, 1.
use crate::forge_routes::{live_task, questions};
use jig_domain_tasks::{MessageKind, Party};

#[test]
fn a_held_answer_tells_its_requester() {
    // The shared fixture runs 200 ticks after the answer, with a sibling still pending.
    let mut story = questions::existing_sibling_question();
    let dependency = story.dependency.to_string();
    let inbox = &live_task(&story.world, story.requester.task).inbox;
    let told = inbox.iter().any(|message| {
        let words = std::str::from_utf8(&message.words).expect("relay words are UTF-8").to_lowercase();
        message.from == Party::Task(story.change)
            && message.kind == MessageKind::Words
            && words.split(|c: char| !c.is_ascii_alphanumeric()).any(|word| word == dependency)
            && words.contains("answer")
            && (words.contains("held") || words.contains("wait"))
    });
    let heard = inbox.clone();
    // Run the release even when the notice is missing: the shared helper checks the
    // resumed producer's answer, question number and landed base, then both landings.
    questions::land_sibling_and_resume(&mut story);
    assert!(told, "requester was not told that sibling {} holds its answer: {heard:?}", story.dependency);
}
