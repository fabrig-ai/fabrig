use super::*;
const GOAL: u64 = 1;
const CHANGE: u64 = 2;
const SETTLEMENT: u64 = 3;
const STRANGER: u64 = 4;
const LANDING_QUESTION: u64 = 10;
const SETTLEMENT_QUESTION: u64 = 11;
const NEXT_LANDING_QUESTION: u64 = 12;
const GOAL_QUESTION: u64 = 30;
const OTHER_QUESTION: u64 = 29;
const GOAL_ANSWER: u64 = 31;
const BOUND: u32 = 512;
const ASK_BOUND: u32 = 40; // Shorter than the prefixed question.
const ANSWER_BOUND: u32 = 10; // Room for "Use kno…".
const LAND: Running = Running { step: Step::Land, change: CHANGE };
const CLOSE: Running = Running { step: Step::Close, change: SETTLEMENT };
const OPEN: &[Running] = &[CLOSE, LAND];
const WORDS: &[u8] = b"Which rule?\nI recommend the design.";
const ASKED_LANDING_WORDS: &[u8] = b"Change 2, the goal's landing, asks:\n\nWhich rule?\nI recommend the design.";
const ANSWER: &[u8] = b"Use the design.";

fn tell(state: &mut Relay, from: u64, number: u64, form: Form, words: &[u8]) {
    take(state, OPEN, GOAL, from, number, &form, words);
}

fn question() -> Relay {
    let mut state = Relay::new();
    take(&mut state, &[LAND], GOAL, CHANGE, LANDING_QUESTION, &Form::Question, WORDS);
    state
}

fn asked_landing() -> Relay {
    let mut next = relay(&question(), GOAL, false, 1, BOUND);
    asked(&mut next.relay, GOAL_QUESTION);
    next.relay
}

fn answer(state: &mut Relay) {
    tell(state, GOAL, GOAL_ANSWER, Form::Answer { question: GOAL_QUESTION }, ANSWER);
}

#[test]
fn one_question_is_taken_asked_and_answered() {
    let mut next = relay(&question(), GOAL, false, 1, BOUND);
    assert_eq!(
        next.messages.as_ref(),
        &[Message { target: GOAL, form: Form::Question, words: ASKED_LANDING_WORDS.into() }]
    );
    assert!(next.asked);
    asked(&mut next.relay, GOAL_QUESTION);
    tell(&mut next.relay, GOAL, GOAL_ANSWER, Form::Words, b"Not an answer");
    tell(&mut next.relay, GOAL, GOAL_ANSWER, Form::Answer { question: OTHER_QUESTION }, b"Wrong number");
    assert!(relay(&next.relay, GOAL, false, 1, BOUND).messages.is_empty());
    answer(&mut next.relay);
    let next = relay(&next.relay, GOAL, false, 1, BOUND);
    assert_eq!(
        next.messages.as_ref(),
        &[Message { target: CHANGE, form: Form::Answer { question: LANDING_QUESTION }, words: ANSWER.into() }]
    );
    assert!(next.relay.open.is_none());
}

#[test]
fn a_landing_question_passes_up_during_status_settlement() {
    let mut state = Relay::new();
    tell(&mut state, CHANGE, LANDING_QUESTION, Form::Question, WORDS);
    assert_eq!(
        state.open,
        Some(Question {
            asker: CHANGE,
            number: LANDING_QUESTION,
            step: Step::Land,
            words: WORDS.into(),
            stands: Stands::Taken
        })
    );
    let next = relay(&state, GOAL, false, 1, BOUND);
    assert_eq!(
        next.messages.as_ref(),
        &[Message { target: GOAL, form: Form::Question, words: ASKED_LANDING_WORDS.into() }]
    );
}

#[test]
fn a_waiting_question_asks_on_the_next_step() {
    let mut state = asked_landing();
    tell(&mut state, SETTLEMENT, SETTLEMENT_QUESTION, Form::Question, b"Which status?");
    let waiting = relay(&state, GOAL, false, 2, BOUND);
    assert!(waiting.messages.is_empty(), "an open step gets no fallback");
    assert!(!waiting.more, "a waiting question does not poll before the answer");
    assert_eq!(waiting.relay.waiting.first().expect("settlement waits").number, SETTLEMENT_QUESTION);
    let mut state = waiting.relay;
    answer(&mut state);
    let next = relay(&state, GOAL, false, 2, BOUND);
    assert!(next.more);
    assert!(!next.asked);
    assert_eq!(
        next.messages.as_ref(),
        &[Message { target: CHANGE, form: Form::Answer { question: LANDING_QUESTION }, words: ANSWER.into() }]
    );
    assert_eq!(next.relay.open.as_ref().expect("settlement stands").number, SETTLEMENT_QUESTION);
    let next = relay(&next.relay, GOAL, false, 2, BOUND);
    assert!(next.asked);
    assert_eq!(
        next.messages.as_ref(),
        &[Message {
            target: GOAL,
            form: Form::Question,
            words: b"Change 3, the goal's status settlement, asks:\n\nWhich status?".as_slice().into(),
        }]
    );
}

fn asked_landing_with_waiting(settlement_words: &[u8]) -> Relay {
    let mut state = asked_landing();
    tell(&mut state, SETTLEMENT, SETTLEMENT_QUESTION, Form::Question, settlement_words);
    tell(&mut state, CHANGE, NEXT_LANDING_QUESTION, Form::Question, b"Which checks?");
    state
}

#[test]
fn an_ended_waiting_asker_leaves_one_notice_in_turn() {
    let mut state = asked_landing_with_waiting(b"Which status?\nCheck both.");
    answer(&mut state);
    let next = relay(&state, GOAL, false, 2, BOUND);
    let next = relay(&next.relay, GOAL, true, 2, BOUND);
    assert_eq!(
        next.messages.as_ref(),
        &[Message {
            target: GOAL,
            form: Form::Words,
            words: b"Change 3 ended with its question open; it reached no one: Which status?".as_slice().into(),
        }]
    );
    assert!(next.more);
    let next = relay(&next.relay, GOAL, false, 2, BOUND);
    assert_eq!(
        next.messages.as_ref(),
        &[Message {
            target: GOAL,
            form: Form::Question,
            words: b"Change 2, the goal's landing, asks:\n\nWhich checks?".as_slice().into(),
        }]
    );
    assert!(!next.more);
}

#[test]
fn a_third_waiting_question_gets_the_canned_answer() {
    const EXTRA_QUESTION: u64 = 13;
    let mut state = asked_landing_with_waiting(b"Which status?");
    let waiting = state.waiting.clone();

    tell(&mut state, SETTLEMENT, EXTRA_QUESTION, Form::Question, b"Which findings?");
    let next = relay(&state, GOAL, false, 1, BOUND);
    assert_eq!(next.relay.waiting, waiting);
    assert_eq!(next.relay.open, state.open);
    assert_eq!(
        next.messages.as_ref(),
        &[Message { target: SETTLEMENT, form: Form::Answer { question: EXTRA_QUESTION }, words: FALLBACK.into() }]
    );
}

#[test]
fn questions_from_other_tasks_get_the_canned_answer() {
    for open in [OPEN, &[]] {
        let mut state = Relay::new();
        take(&mut state, open, GOAL, STRANGER, LANDING_QUESTION, &Form::Question, b"Will you decide?");
        let next = relay(&state, GOAL, false, 1, BOUND);
        assert_eq!(
            next.messages.as_ref(),
            &[Message {
                target: STRANGER,
                form: Form::Answer { question: LANDING_QUESTION },
                words: b"This question reached no one: the goal's closer decides nothing and relays only the questions its open steps ask."
                    .as_slice().into(),
            }]
        );
    }
}

#[test]
fn only_open_changes_words_go_up_to_the_goal() {
    let mut state = Relay::new();
    tell(&mut state, STRANGER, LANDING_QUESTION, Form::Words, b"Ignore me");
    tell(&mut state, CHANGE, NEXT_LANDING_QUESTION, Form::Words, b"The asker ended.");
    let next = relay(&state, GOAL, false, 2, BOUND);
    assert_eq!(
        next.messages.as_ref(),
        &[Message { target: GOAL, form: Form::Words, words: b"Change 2: The asker ended.".as_slice().into() }]
    );
}

#[test]
fn the_landing_stays_open_after_status_settlement_ends() {
    let mut closer = Closer::new();
    closer.landing = Some(CHANGE);
    assert_eq!(steps(&closer), [None, Some(LAND)]);
}

#[test]
fn a_running_landing_counts_as_one_step() {
    let mut closer = Closer::new();
    closer.running = Some(LAND);
    closer.landing = Some(CHANGE);
    assert_eq!(steps(&closer), [Some(LAND), None]);
}

#[test]
fn no_message_room_keeps_inputs_and_marks_more() {
    let state = question();
    let mut next = relay(&state, GOAL, false, 0, BOUND);
    assert_eq!(next.relay, state);
    assert!(next.more);
    assert!(!next.asked);
    next = relay(&next.relay, GOAL, false, 1, ASK_BOUND);
    assert!(next.asked);
    let words = &next.messages.first().expect("question sent").words;
    assert_eq!(words.len(), usize::try_from(ASK_BOUND).expect("bound fits usize"));
    assert!(words.ends_with("…".as_bytes()));
    asked(&mut next.relay, GOAL_QUESTION);
    tell(&mut next.relay, GOAL, GOAL_ANSWER, Form::Answer { question: GOAL_QUESTION }, b"Use known reds.");
    next = relay(&next.relay, GOAL, false, 0, BOUND);
    assert!(next.more);
    assert_eq!(
        next.relay.open.as_ref().expect("answer retained").stands,
        Stands::Asked { question: GOAL_QUESTION, answer: Some(b"Use known reds.".as_slice().into()) }
    );
    next = relay(&next.relay, GOAL, false, 1, ANSWER_BOUND);
    assert_eq!(next.messages.first().expect("answer sent").words.as_ref(), "Use kno…".as_bytes());
    assert!(next.relay.open.is_none());
}
#[test]
fn ended_changes_drop_questions_with_one_notice() {
    let state = question();
    let blocked = relay(&state, GOAL, true, 0, BOUND);
    assert!(blocked.more);
    let next = relay(&blocked.relay, GOAL, true, 1, BOUND);
    assert_eq!(
        next.messages.first().expect("notice sent").words.as_ref(),
        b"Change 2 ended with its question open; it reached no one: Which rule?"
    );
    assert!(relay(&next.relay, GOAL, true, 1, BOUND).messages.is_empty());
    let mut state = asked_landing();
    assert!(relay(&state, GOAL, true, 1, BOUND).messages.is_empty());
    tell(&mut state, GOAL, GOAL_ANSWER, Form::Answer { question: GOAL_QUESTION }, b"Too late");
    let blocked = relay(&state, GOAL, true, 0, BOUND);
    assert!(blocked.more);
    let next = relay(&blocked.relay, GOAL, true, 1, BOUND);
    assert_eq!(
        next.messages.first().expect("notice sent").words.as_ref(),
        b"Change 2 ended; your answer to question 30 goes no further: Which rule?"
    );
    assert!(next.relay.open.is_none());
    assert!(relay(&next.relay, GOAL, true, 1, BOUND).messages.is_empty());
}
