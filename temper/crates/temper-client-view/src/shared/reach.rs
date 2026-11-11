//! A reach's subject in shared words (temper/docs/design/client/pages/README.md, Needs you, Stop asking).
use crate::words::{Word, Words};
use alloc::boxed::Box;
use skein_ui::phrases::Hole;
use temper_protocol_reaches::{Class, Classes, Kind, Offer, Reach, Reason, Subject};

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
/// The first offer's narrower rule (client/pages/needs-you.md, 1, Stop asking).
#[must_use]
pub fn rule(w: &Words, classes: &Classes, reach: &Reach, first: Offer) -> Box<[u8]> {
    let object = match first {
        Offer::Decide(subject) => object(w, subject, reach.under, Some(classes)),
        Offer::Words(number) | Offer::Split(number) => {
            let Some(set) = classes.set(number) else { return Box::default() };
            w.say(Word::ReachSomeChanges, &[Hole::Name(set.name.clone())])
        }
    };
    w.say(Word::StopAskingRule, &[Hole::Name(object)])
}
/// An offer's words: the narrowed table's change (temper/docs/design/client/pages/README.md, Needs you, Stop asking).
#[must_use]
pub fn offer(w: &Words, classes: &Classes, reach: &Reach, offer: Offer) -> Box<[u8]> {
    match offer {
        Offer::Decide(subject) => {
            w.say(Word::StopAskingDecide, &[Hole::Name(object(w, subject, reach.under, Some(classes)))])
        }
        Offer::Words(number) => {
            let Some(set) = classes.set(number) else { return Box::default() };
            w.say(Word::StopAskingWords, &[Hole::Name(set.name.clone())])
        }
        Offer::Split(number) => {
            let Some(set) = classes.set(number) else { return Box::default() };
            let mut patterns: Box<[u8]> = Box::default();
            for (index, pattern) in set.patterns.iter().enumerate() {
                let bit = match u32::try_from(index) {
                    Ok(shift) => 1_u8.checked_shl(shift).unwrap_or(0),
                    Err(_error) => 0,
                };
                if reach.matched & bit == 0 {
                    continue;
                }
                if patterns.is_empty() {
                    patterns.clone_from(pattern);
                } else {
                    patterns = w.say(Word::StopAskingPatterns, &[Hole::Name(patterns), Hole::Name(pattern.clone())]);
                }
            }
            w.say(Word::StopAskingSplit, &[Hole::Name(patterns), Hole::Name(set.name.clone())])
        }
    }
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

/// A need's reason line, every shell's (temper/docs/design/client/pages/README.md, Needs you, Why it reached you).
#[must_use]
pub fn why(
    root: &temper_client_domain::Domain,
    w: &Words,
    need: temper_client_domain::pages::needs::Need<'_>,
) -> Option<Box<[u8]>> {
    use temper_client_domain::pages::needs::Why;
    let why = temper_client_domain::pages::needs::why(root, need)?;
    let word = match why {
        Why::Direct => Word::WhyDirect,
        Why::Proposal => Word::WhyProposal,
        Why::Held => Word::WhyHeld,
        Why::Funds => Word::WhyFunds,
        Why::Reach { project, reach } => {
            let classes = root.forge().classes(project);
            let clause = match reach.reason {
                Reason::Kind => w.say(
                    match reach.kind {
                        Kind::Wording => Word::WhyWording,
                        Kind::Retry => Word::WhyRetry,
                        Kind::Implementation => Word::WhyImplementation,
                        Kind::Tests => Word::WhyTests,
                        Kind::Dependency => Word::WhyDependency,
                        Kind::Departure => Word::WhyDeparture,
                        Kind::Money => Word::WhyMoney,
                        Kind::Irreversible => Word::WhyIrreversible,
                        Kind::Rules => Word::WhyRules,
                        Kind::Other => Word::WhyOther,
                    },
                    &[],
                ),
                Reason::Set(number) => {
                    let name = match classes {
                        Some(classes) => match classes.set(number) {
                            Some(set) => set.name.clone(),
                            None => path_set(root, w, project),
                        },
                        None => path_set(root, w, project),
                    };
                    w.say(Word::WhySet, &[Hole::Name(name)])
                }
                Reason::Always => return Some(w.say(Word::WhyAlways, &[])),
            };
            let clause = under_words(w, clause, reach.under, classes);
            let class = match reach.class {
                Class::Ask => Word::WhyAsk,
                Class::AskAway => Word::WhyAskAway,
                Class::Wait => Word::WhyWait,
                Class::Decide => return None,
            };
            return Some(w.say(Word::WhyReach, &[Hole::Name(clause), Hole::Name(w.say(class, &[]))]));
        }
    };
    Some(w.say(word, &[]))
}
fn path_set(root: &temper_client_domain::Domain, w: &Words, project: u32) -> Box<[u8]> {
    let name = crate::pages::project_name(root.pages().me(root.client()), project)
        .unwrap_or(w.say(Word::Project, &[Hole::Number(u64::from(project))]));
    w.say(Word::WhyPathSet, &[Hole::Name(name)])
}
