//! A delivered head moves on despite an answered question and a sibling's landing
//! (temper/docs/design/domain/forge/change/README.md, 2).
use super::goal_questions::{ask_up, finish_story, independent, land, next_producer, resumed};

#[test]
fn an_answered_producer_opens_its_head_after_a_sibling_moves_the_goal_branch() {
    let mut story = independent();
    let (question, up) = ask_up(&mut story.world, &story.producer, story.a);
    story.world.goal_answer(&story.goal, 105, story.a, up, b"Keep the design.");
    let producer = resumed(&mut story.world, &story.producer, story.a, question, b"Keep the design.");
    let sibling = next_producer(&mut story.world, story.b);
    let tip = land(&mut story.world, story.goal.task, story.b, &sibling, story.main, b"B", b"Done: B.");
    assert_ne!(tip, story.main);
    // A's pushed head starts from the old base, not B's landed head.
    land(&mut story.world, story.goal.task, story.a, &producer, story.main, b"A", b"Done: A.");
    assert_eq!(story.world.merges, 2);
    finish_story(&mut story);
}
