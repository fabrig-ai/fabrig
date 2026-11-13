//! Shared `all_home` items (temper/docs/design/client/view-crates.md, 3.3).
use super::home;
use crate::words::{TimeForm, Times, Word, Words};
use alloc::boxed::Box;
use skein_ui::phrases::Hole;
use temper_client_domain::pages::all_home;

pub(crate) fn headline(w: &Words, facts: &all_home::Facts, times: &mut Times) -> Box<[u8]> {
    let intro = all_home::intro(facts);
    let landed = landed(w, &intro.landed, times);
    let needs = home::needs_sentence(w, intro.needs, b"");
    headline_words(&landed, &needs)
}

pub(crate) fn headline_words(landed: &[u8], needs: &[u8]) -> Box<[u8]> {
    if landed.is_empty() {
        needs.into()
    } else if needs.is_empty() {
        landed.into()
    } else {
        skein_lib::bytes::concat(&[landed, b" ", needs])
    }
}

pub(crate) fn landed(w: &Words, landed: &all_home::Landed, times: &mut Times) -> Box<[u8]> {
    match landed {
        all_home::Landed::Nothing => Box::default(),
        all_home::Landed::Last { at } => {
            if times.now.as_nanos().saturating_sub(at.as_nanos()) < 60_000_000_000 {
                w.say(Word::HomeLandedNow, &[])
            } else {
                let span = times.say(w, TimeForm::Looked, *at);
                w.say(Word::HomeLandedLast, &[Hole::Name(span)])
            }
        }
        all_home::Landed::In { name, count: 1, .. } => w.say(Word::AllHomeOne, &[Hole::Name(name.clone())]),
        all_home::Landed::In { name, count, .. } => {
            w.say(Word::AllHomeIn, &[Hole::Number(*count), Hole::Name(name.clone())])
        }
        all_home::Landed::Across { count, projects } => {
            let number = match home::number_word(*projects) {
                Some(word) => {
                    let mut number = w.say(word, &[]);
                    number.make_ascii_lowercase();
                    number
                }
                None => skein_lib::Decimal::of(u64::from(*projects)).as_bytes().into(),
            };
            w.say(Word::AllHomeAcross, &[Hole::Number(*count), Hole::Name(number)])
        }
    }
}
