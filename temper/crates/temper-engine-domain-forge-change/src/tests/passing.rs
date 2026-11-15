//! Words follow the record's order without touching questions (questions.md, 1).
use crate::{Open, Questions, Route, Stands, WORDS_WAITING, Waiting, WordAct, pass};
use alloc::boxed::Box;

const REQUESTER: u64 = 40;
const PRODUCER: u64 = 42;

fn word(number: u64) -> Waiting {
    Waiting { from: REQUESTER, number, words: b"Go on".as_slice().into(), passed: None }
}

#[test]
fn room_for_one_passes_then_tells_in_order_without_touching_questions() {
    for stands in [Stands::Taken, Stands::Asked, Stands::Held { answer: b"Wait".as_slice().into() }] {
        let mut questions = Questions {
            route: Route::Up,
            taken: 1,
            open: Box::new([Open {
                asker: PRODUCER,
                number: 3,
                words: b"Which?\nOptions".as_slice().into(),
                line: b"Which?".as_slice().into(),
                stands,
            }]),
            waiting: Box::new([]),
        };
        let before = questions.clone();
        let line = match &before.open[0].stands {
            Stands::Taken | Stands::Asked => Some(b"Which?".as_slice().into()),
            Stands::Held { .. } => None,
        };
        let held = pass(&questions, &[word(1), word(2)], Some(PRODUCER), 0);
        assert!(held.acts.is_empty());
        assert_eq!(held.waiting.as_ref(), &[word(1), word(2)]);
        assert!(held.more);
        questions.waiting = held.waiting;
        let first = pass(&questions, &[], Some(PRODUCER), 1);
        assert_eq!(first.acts.as_ref(), &[WordAct::Pass { producer: PRODUCER, from: REQUESTER, words: word(1).words }]);
        assert_eq!(first.retained.as_ref(), &[word(1), word(2)]);
        assert_eq!(first.waiting.len(), 2, "passed entries stay until told, before later words");
        assert_eq!(first.waiting[0], Waiting { passed: Some(PRODUCER), words: Box::new([]), ..word(1) });
        assert_eq!(first.waiting[1], word(2));
        assert!(first.more);
        questions.waiting = first.waiting;
        let next = pass(&questions, &[], Some(PRODUCER), 1);
        assert_eq!(
            next.acts.as_ref(),
            &[WordAct::Tell { requester: REQUESTER, producer: PRODUCER, line: line.clone() }]
        );
        assert_eq!(next.waiting.as_ref(), &[word(2)]);
        questions.waiting = next.waiting;
        let last = pass(&questions, &[], Some(PRODUCER), 2);
        assert_eq!(
            last.acts.as_ref(),
            &[
                WordAct::Pass { producer: PRODUCER, from: REQUESTER, words: word(2).words },
                WordAct::Tell { requester: REQUESTER, producer: PRODUCER, line: line.clone() },
            ]
        );
        assert!(last.waiting.is_empty());
        assert!(!last.more);
        assert_eq!(questions.open, before.open);
        assert_eq!(questions.taken, before.taken);
    }
}

#[test]
fn no_live_producer_falls_back_but_passed_words_still_tell() {
    let mut questions = Questions { waiting: Box::new([word(1)]), ..Questions::default() };
    let gone = pass(&questions, &[], None, 1);
    assert_eq!(gone.acts.as_ref(), &[WordAct::Fallback { requester: REQUESTER }]);
    assert!(gone.waiting.is_empty());
    let passed = pass(&questions, &[], Some(PRODUCER), 1);
    questions.waiting = passed.waiting;
    let told = pass(&questions, &[], None, 1);
    assert_eq!(told.acts.as_ref(), &[WordAct::Tell { requester: REQUESTER, producer: PRODUCER, line: None }]);
    assert!(told.waiting.is_empty());
}

#[test]
fn the_seventeenth_word_is_not_kept_or_passed() {
    let mut waiting = skein_lib::List::with_capacity(u32::try_from(WORDS_WAITING).unwrap());
    for number in 0..u64::try_from(WORDS_WAITING).unwrap() {
        waiting.push(word(number)).unwrap();
    }
    let waiting = waiting.into_boxed();
    let questions = Questions { waiting: waiting.clone(), ..Questions::default() };
    let extra = word(17);
    let result = pass(&questions, &[extra], Some(PRODUCER), 1);
    assert_eq!(result.acts.as_ref(), &[WordAct::NotPassed { requester: REQUESTER, words: b"Not passed: 16 words already wait to reach this change's producer; send yours again once they are passed.".as_slice().into() }]);
    assert_eq!(result.waiting, waiting);
    assert!(result.more);
}
