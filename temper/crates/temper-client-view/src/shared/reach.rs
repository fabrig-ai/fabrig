//! A reach's subject in shared words (temper/docs/design/client/README.md, 3.5, Needs you, Stop asking).
use crate::words::{Word, Words};
use alloc::boxed::Box;
use skein_ui::phrases::Hole;
use temper_protocol_reaches::{Class, Classes, Kind, Reach, Reason, Subject};

/// A subject's plural, as Stop asking and "Allowed:" name it (README 3.5, Needs you, Stop asking).
#[must_use]
pub fn object(w: &Words, subject: Subject, under: u8, classes: Option<&Classes>) -> Box<[u8]> {
    let word = match subject {
        Subject::Kind(kind) => match kind {
            Kind::Wording => Word::ReachWording,
            Kind::Retry => Word::ReachRetry,
            Kind::Implementation => Word::ReachImplementation,
            Kind::Tests => Word::ReachTests,
            Kind::Dependency => Word::ReachDependency,
            Kind::Departure => Word::ReachDeparture,
            Kind::Money => Word::ReachMoney,
            Kind::Irreversible => Word::ReachIrreversible,
            Kind::Rules => Word::ReachRules,
            Kind::Other => Word::ReachOther,
        },
        Subject::Set(number) => {
            if let Some(classes) = classes
                && let Some(set) = classes.set(number)
            {
                return under_words(w, w.say(Word::ReachSet, &[Hole::Name(set.name.clone())]), under, Some(classes));
            }
            Word::ReachPathSet
        }
    };
    under_words(w, w.say(word, &[]), under, classes)
}
fn under_words(w: &Words, object: Box<[u8]>, under: u8, classes: Option<&Classes>) -> Box<[u8]> {
    if let Some(classes) = classes
        && let Some(index) = under.checked_sub(1)
        && let Some(item) = classes.items.get(usize::from(index))
        && !item.words.is_empty()
    {
        return w.say(Word::ReachUnder, &[Hole::Name(object), Hole::Name(item.words.clone())]);
    }
    object
}
/// What allowed a decided reach (README 3.5, Decided for you); empty for a reach that asked.
#[must_use]
pub fn allowed(w: &Words, reach: &Reach, classes: Option<&Classes>) -> Box<[u8]> {
    let word = match reach.class {
        Class::Decide => Word::AllowedDecide,
        Class::Ask if reach.away => Word::AllowedAway,
        Class::Ask | Class::AskAway | Class::Wait => return Box::default(),
    };
    let subject = match reach.reason {
        Reason::Kind => Subject::Kind(reach.kind),
        Reason::Set(number) => Subject::Set(number),
        Reason::Always => return Box::default(),
    };
    w.say(word, &[Hole::Name(object(w, subject, reach.under, classes))])
}
