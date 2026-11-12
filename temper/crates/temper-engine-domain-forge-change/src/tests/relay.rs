//! The relay's policy alone (temper/docs/design/domain/forge/change/questions.md, 1).
use crate::{Asked, Open, QUESTION_LINE, QUESTIONS_OPEN, Questions, RelayAct, RelayHeard, Route, Stands, relay};
use alloc::boxed::Box;
use skein_lib::{List, Writer, bytes};

const ASKER: u64 = 10;
const NEXT_ASKER: u64 = 11;
const NUMBER: u64 = 20;
const NEXT_NUMBER: u64 = 21;
// One question has already been taken in the head fixture.
const TAKEN: u32 = 1;
const ONE_MORE: u32 = 2;
// An answer down and the next question up can share this batch.
const ROOM: usize = 2;

fn heard() -> RelayHeard<'static> {
    RelayHeard { asked: &[], answer: None, ended: &[], kept: false, released: false }
}

fn open(asker: u64, number: u64, stands: Stands) -> Open {
    Open {
        asker,
        number,
        line: Box::from(b"Which path?".as_slice()),
        words: Box::from(b"Which path?\nOptions follow.".as_slice()),
        stands,
    }
}

fn head(stands: Stands) -> Questions {
    Questions { route: Route::Up, taken: TAKEN, open: Box::new([open(ASKER, NUMBER, stands)]) }
}

fn asked() -> Asked<'static> {
    Asked { asker: NEXT_ASKER, number: NEXT_NUMBER, words: b"Which path?\nOptions follow." }
}

fn answer(words: &[u8]) -> RelayAct {
    RelayAct::Answer { asker: ASKER, number: NUMBER, words: bytes::copy_of(words) }
}

fn ask(reasons: &[u8]) -> RelayAct {
    RelayAct::Ask { asker: ASKER, number: NUMBER, reasons: bytes::copy_of(reasons) }
}

#[test]
fn a_new_question_waits_its_turn_behind_the_head() {
    let q = head(Stands::Asked);
    let result = relay(&q, &RelayHeard { asked: &[asked()], ..heard() }, &super::LIMITS, ROOM);
    assert_eq!(
        result.questions.open.as_ref(),
        &[open(ASKER, NUMBER, Stands::Asked), open(NEXT_ASKER, NEXT_NUMBER, Stands::Taken),]
    );
    assert_eq!(result.questions.taken, TAKEN);
    assert!(result.acts.is_empty());
    assert!(!result.more);
}

#[test]
fn the_head_goes_up_under_up() {
    let q = Questions { route: Route::Up, ..head(Stands::Taken) };
    let result = relay(&q, &heard(), &super::LIMITS, ROOM);
    assert_eq!(result.acts.as_ref(), &[ask(b"")]);
    assert_eq!(result.questions, Questions { route: Route::Up, taken: ONE_MORE, ..head(Stands::Asked) });
    let waiting = relay(&result.questions, &heard(), &super::LIMITS, ROOM);
    assert_eq!(waiting.questions, result.questions);
    assert!(waiting.acts.is_empty());
}

#[test]
fn the_requesters_answer_comes_down() {
    let result = relay(
        &head(Stands::Asked),
        &RelayHeard { answer: Some(b"Use the path.\nKeep the rest."), ..heard() },
        &super::LIMITS,
        ROOM,
    );
    assert_eq!(result.acts.as_ref(), &[answer(b"Use the path.\nKeep the rest.")]);
    assert!(result.questions.open.is_empty());
    assert_eq!(result.questions.taken, TAKEN);
}

#[test]
fn an_answer_while_kept_is_held_then_relayed() {
    let result = relay(
        &head(Stands::Asked),
        &RelayHeard { answer: Some(b"Wait for the dependency."), kept: true, ..heard() },
        &super::LIMITS,
        0,
    );
    assert_eq!(result.questions, head(Stands::Held { answer: Box::from(b"Wait for the dependency.".as_slice()) }));
    assert!(result.acts.is_empty());
    assert!(!result.more);
    let waiting = relay(&result.questions, &RelayHeard { kept: true, ..heard() }, &super::LIMITS, ROOM);
    assert_eq!(waiting, result);
    let result = relay(&waiting.questions, &heard(), &super::LIMITS, ROOM);
    assert_eq!(result.acts.as_ref(), &[answer(b"Wait for the dependency.")]);
    assert!(result.questions.open.is_empty());
}

#[test]
fn past_the_bound_the_head_holds_once() {
    let q = Questions {
        taken: super::LIMITS.questions,
        open: Box::new([open(ASKER, NUMBER, Stands::Taken), open(NEXT_ASKER, NEXT_NUMBER, Stands::Taken)]),
        ..head(Stands::Taken)
    };
    let result = relay(&q, &heard(), &super::LIMITS, 0);
    assert_eq!(result.acts.as_ref(), &[RelayAct::Hold]);
    assert_eq!(result.questions, q);
    assert!(!result.more);
}

#[test]
fn released_past_the_bound_it_goes_up() {
    let q = Questions { taken: super::LIMITS.questions, ..head(Stands::Taken) };
    let result = relay(&q, &RelayHeard { released: true, ..heard() }, &super::LIMITS, ROOM);
    assert_eq!(result.acts.as_ref(), &[ask(b"")]);
    assert_eq!(result.questions, head(Stands::Asked));
}

fn left(answered: bool) -> RelayAct {
    RelayAct::Left { asker: ASKER, number: NUMBER, line: Box::from(b"Which path?".as_slice()), answered }
}

#[test]
fn an_ended_asker_reports_each_question() {
    for (stands, answered) in [(Stands::Taken, false), (Stands::Held { answer: Box::from(b"Unused".as_slice()) }, true)]
    {
        let result = relay(&head(stands), &RelayHeard { ended: &[ASKER], kept: true, ..heard() }, &super::LIMITS, 1);
        let expected = vec![left(answered)];
        assert_eq!(result.acts.as_ref(), expected);
        assert!(result.questions.open.is_empty());
        assert!(!result.more);
        let next = relay(&result.questions, &RelayHeard { ended: &[ASKER], ..heard() }, &super::LIMITS, ROOM);
        assert!(next.acts.is_empty(), "the ended question is reported once");
    }
}

#[test]
fn an_asked_question_of_an_ended_asker_reports_at_its_answer() {
    let q = head(Stands::Asked);
    let waiting = relay(&q, &RelayHeard { ended: &[ASKER], ..heard() }, &super::LIMITS, ROOM);
    assert_eq!(waiting.questions, q);
    assert!(waiting.acts.is_empty());
    for kept in [false, true] {
        let result = relay(
            &waiting.questions,
            &RelayHeard { ended: &[ASKER], answer: Some(b"Unused"), kept, ..heard() },
            &super::LIMITS,
            1,
        );
        assert_eq!(result.acts.as_ref(), &[left(true)]);
        assert!(result.questions.open.is_empty());
        assert!(!result.more);
    }
}

#[test]
fn no_room_keeps_an_ended_question_until_it_can_be_reported_once() {
    for stands in [Stands::Taken, Stands::Asked, Stands::Held { answer: Box::from(b"Unused".as_slice()) }] {
        let q = head(stands.clone());
        let result = relay(&q, &RelayHeard { ended: &[ASKER], answer: Some(b"Unused"), ..heard() }, &super::LIMITS, 0);
        let (expected, answered) = match stands {
            Stands::Asked | Stands::Held { .. } => {
                (head(Stands::Held { answer: Box::from(b"Unused".as_slice()) }), true)
            }
            Stands::Taken => (q, false),
        };
        assert_eq!(result.questions, expected);
        assert!(result.acts.is_empty(), "a question stays whole while the notice waits");
        assert!(result.more);
        let next = relay(&result.questions, &RelayHeard { ended: &[ASKER], ..heard() }, &super::LIMITS, 1);
        assert_eq!(next.acts[0], left(answered));
        assert!(next.questions.open.is_empty());
        assert!(!next.more);
        let again = relay(&next.questions, &RelayHeard { ended: &[ASKER], ..heard() }, &super::LIMITS, ROOM);
        assert!(again.acts.is_empty());
    }
}

#[test]
fn an_ended_question_shares_message_room_with_the_next_question() {
    let q = Questions {
        route: Route::Up,
        open: Box::new([open(ASKER, NUMBER, Stands::Taken), open(NEXT_ASKER, NEXT_NUMBER, Stands::Taken)]),
        ..head(Stands::Taken)
    };
    let result = relay(&q, &RelayHeard { ended: &[ASKER], ..heard() }, &super::LIMITS, 1);
    assert_eq!(result.acts.as_ref(), &[left(false)]);
    assert_eq!(result.questions.open.as_ref(), &[open(NEXT_ASKER, NEXT_NUMBER, Stands::Taken)]);
    assert!(result.more);
}

#[test]
fn a_live_askers_question_is_never_reported_as_ended() {
    for stands in [Stands::Taken, Stands::Asked, Stands::Held { answer: Box::from(b"Use the path.".as_slice()) }] {
        let result = relay(&head(stands), &RelayHeard { ended: &[NEXT_ASKER], ..heard() }, &super::LIMITS, ROOM);
        let mut reported = false;
        for act in &result.acts {
            match act {
                RelayAct::Left { .. } => reported = true,
                RelayAct::Answer { .. } | RelayAct::Ask { .. } | RelayAct::Hold => {}
            }
        }
        assert!(!reported);
    }
}

#[test]
fn no_room_leaves_more_for_the_next_step() {
    let q = Questions { route: Route::Up, ..head(Stands::Asked) };
    let result =
        relay(&q, &RelayHeard { answer: Some(b"Use the path."), asked: &[asked()], ..heard() }, &super::LIMITS, 1);
    assert_eq!(result.acts.as_ref(), &[answer(b"Use the path.")]);
    assert_eq!(result.questions.open.as_ref(), &[open(NEXT_ASKER, NEXT_NUMBER, Stands::Taken)]);
    assert_eq!(result.questions.taken, TAKEN);
    assert!(result.more);
    let next = relay(&result.questions, &heard(), &super::LIMITS, 1);
    assert_eq!(next.acts.as_ref(), &[RelayAct::Ask { asker: NEXT_ASKER, number: NEXT_NUMBER, reasons: Box::new([]) }]);
    assert_eq!(next.questions.open.as_ref(), &[open(NEXT_ASKER, NEXT_NUMBER, Stands::Asked)]);
    assert_eq!(next.questions.taken, ONE_MORE);
    assert!(!next.more);
}

#[test]
fn a_long_first_line_is_cut_and_marked() {
    // Two three-byte characters put the line one byte past the bound;
    // the first straddles the cut after the ellipsis is reserved.
    const PREFIX_BYTES: usize = QUESTION_LINE.saturating_sub(5);
    const LINE_BYTES: usize = QUESTION_LINE.saturating_add(1);
    let mut words = Writer::new(LINE_BYTES);
    for _ in 0..PREFIX_BYTES {
        words.put(b"a").unwrap();
    }
    words.put("€€".as_bytes()).unwrap();
    let words = words.finish();
    let expected = joined_test(words.get(..PREFIX_BYTES).unwrap(), "…".as_bytes());
    let result = relay(
        &head(Stands::Asked),
        &RelayHeard { asked: &[Asked { words: &words, ..asked() }], ..heard() },
        &super::LIMITS,
        ROOM,
    );
    assert_eq!(result.questions.open[1].line, expected);
    assert!(result.questions.open[1].line.len() <= QUESTION_LINE);
}

fn joined_test(prefix: &[u8], suffix: &[u8]) -> Box<[u8]> {
    let mut out = Writer::new(prefix.len().saturating_add(suffix.len()));
    out.put(prefix).unwrap();
    out.put(suffix).unwrap();
    out.finish()
}

#[test]
fn route_none_relays_nothing() {
    let q = Questions { route: Route::None, ..head(Stands::Asked) };
    let result = relay(
        &q,
        &RelayHeard { asked: &[asked()], answer: Some(b"Unused"), ended: &[ASKER], released: true, ..heard() },
        &super::LIMITS,
        ROOM,
    );
    assert_eq!(result.questions, q);
    assert!(result.acts.is_empty());
    assert!(!result.more);
}

#[test]
fn an_open_queue_takes_only_the_questions_that_fit() {
    let mut open_questions = List::with_capacity(u32::try_from(QUESTIONS_OPEN).unwrap());
    for _ in 0..QUESTIONS_OPEN.saturating_sub(1) {
        open_questions.push(open(ASKER, NUMBER, Stands::Taken)).unwrap();
    }
    let q = Questions { taken: super::LIMITS.questions, open: open_questions.into_boxed(), ..head(Stands::Taken) };
    let result = relay(
        &q,
        &RelayHeard { asked: &[asked(), Asked { number: NUMBER, ..asked() }], ..heard() },
        &super::LIMITS,
        ROOM,
    );
    assert_eq!(result.questions.open.len(), QUESTIONS_OPEN);
    assert_eq!(result.questions.open.last(), Some(&open(NEXT_ASKER, NEXT_NUMBER, Stands::Taken)));
    let full = relay(&result.questions, &RelayHeard { asked: &[asked()], ..heard() }, &super::LIMITS, ROOM);
    assert_eq!(full.questions, result.questions);
}

#[test]
fn no_room_preserves_each_message_before_it_is_sent() {
    let heard_answer = RelayHeard { answer: Some(b"Use the path."), ..heard() };
    for (q, h) in [
        (Questions { route: Route::Up, ..head(Stands::Taken) }, heard()),
        (head(Stands::Asked), heard_answer),
        (head(Stands::Held { answer: Box::from(b"Use the path.".as_slice()) }), heard()),
    ] {
        let result = relay(&q, &h, &super::LIMITS, 0);
        assert_eq!(result.questions, q);
        assert!(result.acts.is_empty());
        assert!(result.more);
    }
}

#[test]
fn a_first_line_at_the_byte_bound_is_kept_whole() {
    let mut words = Writer::new(QUESTION_LINE);
    for _ in 0..QUESTION_LINE {
        words.put(b"a").unwrap();
    }
    let words = words.finish();
    let input = joined_test(&words, b"\nSecond line");
    let result = relay(
        &head(Stands::Asked),
        &RelayHeard { asked: &[Asked { words: &input, ..asked() }], ..heard() },
        &super::LIMITS,
        ROOM,
    );
    assert_eq!(result.questions.open[1].line, words);
}
