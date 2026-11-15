//! Requester words, durable as the answer relay
//! (temper/docs/design/domain/forge/change/questions.md, 1, Its requester's words and amendments).
use alloc::boxed::Box;
use skein_lib::List;

use crate::{Questions, Stands, WORDS_WAITING, Waiting};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum WordAct {
    Pass { producer: u64, from: u64, words: Box<[u8]> },
    Tell { requester: u64, producer: u64, line: Option<Box<[u8]>> },
    Fallback { requester: u64 },
    NotPassed { requester: u64, words: Box<[u8]> },
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Passed {
    pub waiting: Box<[Waiting]>,
    /// Taken entries before any effect, for the failed-effect hold.
    pub retained: Box<[Waiting]>,
    pub acts: Box<[WordAct]>,
    pub more: bool,
}

/// Retain inputs before passing; every act uses one message of `room`.
#[must_use]
pub fn pass(questions: &Questions, words: &[Waiting], producer: Option<u64>, mut room: usize) -> Passed {
    let mut waiting = List::with_capacity(u32::try_from(WORDS_WAITING).expect("word bound fits u32"));
    let mut acts = List::uncapped();
    for word in &questions.waiting {
        waiting.push(word.clone()).expect("record respects word bound");
    }
    for word in words {
        if waiting.room() == 0 {
            if room > 0 {
                acts.push(WordAct::NotPassed {
                    requester: word.from,
                    words: b"Not passed: 16 words already wait to reach this change's producer; send yours again once they are passed.".as_slice().into(),
                }).expect("acts fit a list");
                room = room.saturating_sub(1);
            }
        } else {
            waiting.push(word.clone()).expect("word room checked");
        }
    }
    let retained = waiting.as_slice().into();
    let mut waiting = waiting.into_boxed();
    let mut done: usize = 0;
    for word in &mut waiting {
        if room == 0 {
            break;
        }
        if word.passed.is_none() {
            let Some(producer) = producer else {
                acts.push(WordAct::Fallback { requester: word.from }).expect("acts fit a list");
                room = room.saturating_sub(1);
                done = done.saturating_add(1);
                continue;
            };
            acts.push(WordAct::Pass {
                producer,
                from: word.from,
                words: core::mem::replace(&mut word.words, Box::new([])),
            })
            .expect("acts fit a list");
            word.passed = Some(producer);
            room = room.saturating_sub(1);
        }
        if room == 0 {
            break;
        }
        let producer = word.passed.expect("passed before telling");
        let mut line = None;
        for question in &questions.open {
            if question.asker == producer && (question.stands == Stands::Taken || question.stands == Stands::Asked) {
                line = Some(question.line.clone());
                break;
            }
        }
        acts.push(WordAct::Tell { requester: word.from, producer, line }).expect("acts fit a list");
        room = room.saturating_sub(1);
        done = done.saturating_add(1);
    }
    let remaining = waiting.get(done..).expect("only told heads leave");
    Passed { waiting: remaining.into(), retained, acts: acts.into_boxed(), more: !remaining.is_empty() }
}
