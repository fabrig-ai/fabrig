use super::*;
const GOAL: u64 = 1;
const CHANGE: u64 = 2;
fn question() -> Relay {
    let mut state = Relay::new();
    take(
        &mut state,
        Some(Running { step: Step::Land, change: CHANGE }),
        GOAL,
        CHANGE,
        10,
        &Form::Question,
        b"Which rule?\nI recommend the design.",
    );
    state
}
#[test]
fn one_question_is_taken_asked_and_answered() {
    let mut state = question();
    take(
        &mut state,
        Some(Running { step: Step::Land, change: CHANGE }),
        GOAL,
        CHANGE,
        11,
        &Form::Question,
        b"Another?",
    );
    assert_eq!(state.open.as_ref().unwrap().number, 10);
    let mut next = relay(&state, GOAL, false, 2, 512);
    assert_eq!(next.messages[0].form, Form::Answer { question: 11 });
    assert_eq!(next.messages[0].words.as_ref(), FALLBACK);
    assert_eq!(
        next.messages[1].words.as_ref(),
        b"Change 2, the goal's landing, asks:\n\nWhich rule?\nI recommend the design."
    );
    assert!(next.asked);
    asked(&mut next.relay, 30);
    take(&mut next.relay, None, GOAL, GOAL, 31, &Form::Words, b"Not an answer");
    take(&mut next.relay, None, GOAL, GOAL, 32, &Form::Answer { question: 29 }, b"Wrong number");
    assert!(relay(&next.relay, GOAL, false, 1, 512).messages.is_empty());
    take(&mut next.relay, None, GOAL, GOAL, 33, &Form::Answer { question: 30 }, b"Use the design.");
    let next = relay(&next.relay, GOAL, false, 1, 512);
    assert_eq!(
        next.messages[0],
        Message { target: CHANGE, form: Form::Answer { question: 10 }, words: b"Use the design.".as_slice().into() }
    );
    assert!(next.relay.open.is_none());
}
#[test]
fn no_message_room_keeps_inputs_and_marks_more() {
    let state = question();
    let mut next = relay(&state, GOAL, false, 0, 64);
    assert_eq!(next.relay, state);
    assert!(next.more);
    assert!(!next.asked);
    next = relay(&next.relay, GOAL, false, 1, 40);
    assert!(next.asked);
    assert_eq!(next.messages[0].words.len(), 40);
    assert!(next.messages[0].words.ends_with("…".as_bytes()));
    asked(&mut next.relay, 30);
    take(&mut next.relay, None, GOAL, GOAL, 31, &Form::Answer { question: 30 }, b"Use known reds.");
    next = relay(&next.relay, GOAL, false, 0, 64);
    assert!(next.more);
    assert_eq!(
        next.relay.open.as_ref().unwrap().stands,
        Stands::Asked { question: 30, answer: Some(b"Use known reds.".as_slice().into()) }
    );
    next = relay(&next.relay, GOAL, false, 1, 10);
    assert_eq!(next.messages[0].words.as_ref(), "Use kno…".as_bytes());
    assert!(next.relay.open.is_none());
}
#[test]
fn ended_changes_drop_questions_with_one_notice() {
    let state = question();
    let blocked = relay(&state, GOAL, true, 0, 512);
    assert!(blocked.more);
    let next = relay(&blocked.relay, GOAL, true, 1, 512);
    assert_eq!(
        next.messages[0].words.as_ref(),
        b"Change 2 ended with its question open; it reached no one: Which rule?"
    );
    assert!(relay(&next.relay, GOAL, true, 1, 512).messages.is_empty());
    let mut state = state;
    asked(&mut state, 30);
    assert!(relay(&state, GOAL, true, 1, 512).messages.is_empty());
    take(&mut state, None, GOAL, GOAL, 31, &Form::Answer { question: 30 }, b"Too late");
    let blocked = relay(&state, GOAL, true, 0, 512);
    assert!(blocked.more);
    let next = relay(&blocked.relay, GOAL, true, 1, 512);
    assert_eq!(
        next.messages[0].words.as_ref(),
        b"Change 2 ended; your answer to question 30 goes no further: Which rule?"
    );
    assert!(next.relay.open.is_none());
    assert!(relay(&next.relay, GOAL, true, 1, 512).messages.is_empty());
}
