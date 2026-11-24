use super::*;
const CLOSER: u64 = 5;
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
const HELD_AT: Wall = Wall::from_nanos(951_827_696_999_999_999); // 2000-02-29T12:34:56Z

fn step(state: &Relay, ended: bool, left: u32, bound: u32, took_answer: bool) -> Relayed {
    relay(state, GOAL, ended, Room { left, batch: 2 }, bound, HELD_AT, took_answer)
}

fn tell(state: &mut Relay, from: u64, number: u64, form: Form, words: &[u8]) -> bool {
    take(state, OPEN, CLOSER, GOAL, from, number, &form, words)
}

fn question() -> Relay {
    let mut state = Relay::new();
    take(&mut state, &[LAND], CLOSER, GOAL, CHANGE, LANDING_QUESTION, &Form::Question, WORDS);
    state
}

fn asked_landing() -> Relay {
    let mut next = step(&question(), false, 1, BOUND, false);
    asked(&mut next.relay, GOAL_QUESTION);
    next.relay
}

fn answer(state: &mut Relay) {
    assert!(tell(state, GOAL, GOAL_ANSWER, Form::Answer { question: GOAL_QUESTION }, ANSWER));
}

#[test]
fn one_question_is_taken_asked_and_answered() {
    let mut next = step(&question(), false, 1, BOUND, false);
    assert_eq!(
        next.messages.as_ref(),
        &[Message { target: GOAL, form: Form::Question, words: ASKED_LANDING_WORDS.into() }]
    );
    assert!(next.asked);
    asked(&mut next.relay, GOAL_QUESTION);
    answer(&mut next.relay);
    let next = step(&next.relay, false, 1, BOUND, true);
    assert!(next.notice.is_none());
    assert_eq!(
        next.messages.as_ref(),
        &[Message { target: CHANGE, form: Form::Answer { question: LANDING_QUESTION }, words: ANSWER.into() }]
    );
    assert!(next.relay.open.is_none());
}

#[test]
fn the_closer_corrects_a_wrong_number_and_refuses_a_duplicate_answer() {
    let mut state = asked_landing();
    tell(&mut state, GOAL, GOAL_ANSWER, Form::Answer { question: OTHER_QUESTION }, b"Wrong number");
    let next = step(&state, false, 1, BOUND, false);
    assert_eq!(next.relay.open, state.open, "a wrong number leaves the question asked");
    let [reply] = next.messages.as_ref() else { panic!("one refusal: {:?}", next.messages) };
    assert_eq!((reply.target, &reply.form), (GOAL, &Form::Words));
    assert_eq!(reply.words.as_ref(), b"Question 29 is not open at task 5; its question to you is 30: Which rule?. Answer it: message target=5, form=answer, question=30.");
    let mut state = next.relay;
    answer(&mut state);
    tell(&mut state, GOAL, GOAL_ANSWER + 1, Form::Answer { question: GOAL_QUESTION }, b"Do not replace the answer");
    let next = step(&state, false, 2, BOUND, true);
    let [reply, relayed] = next.messages.as_ref() else { panic!("refusal and answer: {:?}", next.messages) };
    assert_eq!((reply.target, &reply.form), (GOAL, &Form::Words));
    assert_eq!(reply.words.as_ref(), b"Task 5 has no question open to you; read_task on task 5 shows its questions.");
    assert_eq!(
        relayed,
        &Message { target: CHANGE, form: Form::Answer { question: LANDING_QUESTION }, words: ANSWER.into() }
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
    let next = step(&state, false, 1, BOUND, false);
    assert_eq!(
        next.messages.as_ref(),
        &[Message { target: GOAL, form: Form::Question, words: ASKED_LANDING_WORDS.into() }]
    );
}

#[test]
fn a_waiting_question_asks_on_the_next_step() {
    let mut state = asked_landing();
    tell(&mut state, SETTLEMENT, SETTLEMENT_QUESTION, Form::Question, b"Which status?");
    let waiting = step(&state, false, 2, BOUND, false);
    assert!(waiting.messages.is_empty(), "an open step gets no fallback");
    assert!(!waiting.more, "a waiting question does not poll before the answer");
    assert_eq!(waiting.relay.waiting.first().expect("settlement waits").number, SETTLEMENT_QUESTION);
    let mut state = waiting.relay;
    answer(&mut state);
    let next = step(&state, false, 2, BOUND, true);
    assert!(next.more);
    assert!(!next.asked);
    assert_eq!(
        next.messages.as_ref(),
        &[Message { target: CHANGE, form: Form::Answer { question: LANDING_QUESTION }, words: ANSWER.into() }]
    );
    assert_eq!(next.relay.open.as_ref().expect("settlement stands").number, SETTLEMENT_QUESTION);
    let next = step(&next.relay, false, 2, BOUND, false);
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
    let next = step(&state, false, 2, BOUND, true);
    let next = step(&next.relay, true, 2, BOUND, false);
    assert_eq!(
        next.messages.as_ref(),
        &[Message {
            target: GOAL,
            form: Form::Words,
            words: b"Change 3 ended with its question open; it reached no one: Which status?".as_slice().into(),
        }]
    );
    assert!(next.more);
    let next = step(&next.relay, false, 2, BOUND, false);
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
    let next = step(&state, false, 1, BOUND, false);
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
        take(&mut state, open, CLOSER, GOAL, STRANGER, LANDING_QUESTION, &Form::Question, b"Will you decide?");
        let next = step(&state, false, 1, BOUND, false);
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
    let mut state = asked_landing();
    tell(&mut state, CHANGE, NEXT_LANDING_QUESTION, Form::Words, b"The asker ended.");
    tell(&mut state, GOAL, GOAL_ANSWER, Form::Words, b"Not an answer");
    let next = step(&state, false, 1, BOUND, false);
    assert_eq!(
        next.messages.as_ref(),
        &[Message { target: GOAL, form: Form::Words, words: b"Change 2: The asker ended.".as_slice().into() }]
    );
    assert!(next.more, "the goal's refusal finds no room at the first step");
    let next = step(&next.relay, false, 1, BOUND, false);
    let [reply] = next.messages.as_ref() else { panic!("the retained reply: {:?}", next.messages) };
    assert_eq!((reply.target, &reply.form), (GOAL, &Form::Words));
    assert_eq!(reply.words.as_ref(), b"This closer takes no words: it relays its open steps' questions and their answers alone. To answer, message form=answer with the question's number.");
    assert_eq!(next.relay.open, state.open, "words do not answer");
    assert!(next.relay.replies.is_empty());
    assert!(!next.more);
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
    let mut next = step(&state, false, 0, BOUND, false);
    assert_eq!(next.relay, state);
    assert!(next.more);
    assert!(!next.asked);
    next = step(&next.relay, false, 1, ASK_BOUND, false);
    assert!(next.asked);
    let words = &next.messages.first().expect("question sent").words;
    assert_eq!(words.len(), usize::try_from(ASK_BOUND).expect("bound fits usize"));
    assert!(words.ends_with("…".as_bytes()));
    asked(&mut next.relay, GOAL_QUESTION);
    assert!(tell(&mut next.relay, GOAL, GOAL_ANSWER, Form::Answer { question: GOAL_QUESTION }, b"Use known reds."));
    next = step(&next.relay, false, 0, BOUND, true);
    assert!(next.more);
    assert_eq!(
        next.relay.open.as_ref().expect("answer retained").stands,
        Stands::Asked {
            question: GOAL_QUESTION,
            answer: Some(b"Use known reds.".as_slice().into()),
            held: Some(Held { since: HELD_AT, notified: true }),
        }
    );
    next = step(&next.relay, false, 2, ANSWER_BOUND, false);
    assert_eq!(next.messages.first().expect("answer sent").words.as_ref(), "Use kno…".as_bytes());
    assert!(next.relay.open.is_none());
}

#[test]
fn a_held_answer_and_its_delivery_notice_wait_for_two_slots() {
    let mut state = asked_landing();
    tell(&mut state, CHANGE, NEXT_LANDING_QUESTION, Form::Words, b"An earlier message.");
    answer(&mut state);
    let held = step(&state, false, 1, BOUND, true);
    assert_eq!(held.messages.len(), 1, "the earlier message spends the only ordinary slot");
    assert_eq!(held.messages.first().expect("earlier message").words.as_ref(), b"Change 2: An earlier message.");
    let notice = held.notice.expect("the held notice uses the question's room");
    assert_eq!(notice.question, GOAL_QUESTION);
    assert_eq!(
        notice.words.as_ref(),
        b"Your answer to question 30 is held until this closer has message room; held since 2000-02-29T12:34:56Z."
    );
    assert!(notice.words.len() <= 256);
    assert!(held.more);

    let waiting = relay(&held.relay, GOAL, false, Room { left: 1, batch: 2 }, BOUND, Wall::EPOCH, false);
    assert!(waiting.messages.is_empty(), "one slot sends neither the answer nor its delivery notice");
    assert!(waiting.notice.is_none());
    assert_eq!(waiting.relay, held.relay);
    assert!(waiting.more);

    let delivered = relay(&waiting.relay, GOAL, false, Room { left: 2, batch: 2 }, BOUND, Wall::EPOCH, false);
    assert!(delivered.notice.is_none());
    assert_eq!(
        delivered.messages.as_ref(),
        &[
            Message { target: CHANGE, form: Form::Answer { question: LANDING_QUESTION }, words: ANSWER.into() },
            Message {
                target: GOAL,
                form: Form::Words,
                words: b"Your answer to question 30, held since 2000-02-29T12:34:56Z, is delivered to task 2."
                    .as_slice()
                    .into()
            },
        ]
    );
    assert!(!delivered.more);
    assert!(delivered.relay.open.is_none());
}

#[test]
fn ended_changes_drop_questions_with_one_notice() {
    let state = question();
    let blocked = step(&state, true, 0, BOUND, false);
    assert!(blocked.more);
    let next = step(&blocked.relay, true, 1, BOUND, false);
    assert_eq!(
        next.messages.first().expect("notice sent").words.as_ref(),
        b"Change 2 ended with its question open; it reached no one: Which rule?"
    );
    assert!(step(&next.relay, true, 1, BOUND, false).messages.is_empty());
    let mut state = asked_landing();
    assert!(step(&state, true, 1, BOUND, false).messages.is_empty());
    tell(&mut state, GOAL, GOAL_ANSWER, Form::Answer { question: GOAL_QUESTION }, b"Too late");
    let blocked = step(&state, false, 0, BOUND, true);
    assert!(blocked.more);
    let next = step(&blocked.relay, true, 1, BOUND, false);
    assert!(next.notice.is_none());
    assert_eq!(
        next.messages.first().expect("notice sent").words.as_ref(),
        b"Change 2 ended; your answer to question 30 goes no further: Which rule?"
    );
    assert!(next.relay.open.is_none());
    assert!(step(&next.relay, true, 1, BOUND, false).messages.is_empty());
}
