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
            Kind::Findings => Word::ReachFindings,
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
    if first == Offer::Decide(Subject::Kind(Kind::Findings)) {
        return w.say(Word::StopAskingFindingsRule, &[Hole::Name(findings_verb(w, classes, reach.under))]);
    }
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
    if offer == Offer::Decide(Subject::Kind(Kind::Findings)) {
        return w.say(Word::StopAskingFindingsDecide, &[Hole::Name(findings_verb(w, classes, reach.under))]);
    }
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
fn findings_verb(w: &Words, classes: &Classes, under: u8) -> Box<[u8]> {
    under_words(w, w.say(Word::ReachFindingsVerb, &[]), under, Some(classes))
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
    why_words(root, w, &temper_client_domain::pages::needs::why(root, need)?, false)
}
/// The same reason, said of one card or several (client/pages/needs-you.md, 1, In the plural).
#[must_use]
pub fn why_words(
    root: &temper_client_domain::Domain,
    w: &Words,
    why: &temper_client_domain::pages::needs::Why,
    plural: bool,
) -> Option<Box<[u8]>> {
    use temper_client_domain::pages::needs::{Holds, Why};
    let word = match why {
        Why::Direct => {
            if plural {
                Word::WhyPluralDirect
            } else {
                Word::WhyDirect
            }
        }
        Why::Proposal => {
            if plural {
                Word::WhyPluralProposal
            } else {
                Word::WhyProposal
            }
        }
        Why::Held => {
            if plural {
                Word::WhyPluralHeld
            } else {
                Word::WhyHeld
            }
        }
        Why::Funds => {
            if plural {
                Word::WhyPluralFunds
            } else {
                Word::WhyFunds
            }
        }
        Why::Finding { project, holds } => {
            let name = match holds {
                Holds::Area { name } => name.clone(),
                Holds::Triager | Holds::Owner => crate::common::project_name(root.pages().me(root.client()), *project)
                    .unwrap_or(w.say(Word::Project, &[Hole::Number(u64::from(*project))])),
            };
            let (one, many) = match holds {
                Holds::Area { .. } => (Word::WhyFindingArea, Word::WhyPluralFindingArea),
                Holds::Triager => (Word::WhyFindingTriager, Word::WhyPluralFindingTriager),
                Holds::Owner => (Word::WhyFindingOwner, Word::WhyPluralFindingOwner),
            };
            return Some(w.say(if plural { many } else { one }, &[Hole::Name(name)]));
        }
        Why::Reach { project, reach } => return why_reach(root, w, *project, *reach, plural),
    };
    Some(w.say(word, &[]))
}
fn why_reach(
    root: &temper_client_domain::Domain,
    w: &Words,
    project: u32,
    reach: Reach,
    plural: bool,
) -> Option<Box<[u8]>> {
    let classes = root.forge().classes(project);
    let clause = match reach.reason {
        Reason::Kind => w.say(why_kind(reach.kind, plural), &[]),
        Reason::Set(number) => {
            let name = match classes {
                Some(classes) => match classes.set(number) {
                    Some(set) => set.name.clone(),
                    None => path_set(root, w, project),
                },
                None => path_set(root, w, project),
            };
            w.say(if plural { Word::WhyPluralSet } else { Word::WhySet }, &[Hole::Name(name)])
        }
        Reason::Always => return Some(w.say(if plural { Word::WhyPluralAlways } else { Word::WhyAlways }, &[])),
    };
    let clause = under_words(w, clause, reach.under, classes);
    let class = match reach.class {
        Class::Ask => Word::WhyAsk,
        Class::AskAway => Word::WhyAskAway,
        Class::Wait => Word::WhyWait,
        Class::Decide => return None,
    };
    Some(w.say(
        if plural { Word::WhyPluralReach } else { Word::WhyReach },
        &[Hole::Name(clause), Hole::Name(w.say(class, &[]))],
    ))
}
fn why_kind(kind: Kind, plural: bool) -> Word {
    let (one, many) = match kind {
        Kind::Wording => (Word::WhyWording, Word::WhyPluralWording),
        Kind::Retry => (Word::WhyRetry, Word::WhyPluralRetry),
        Kind::Implementation => (Word::WhyImplementation, Word::WhyPluralImplementation),
        Kind::Tests => (Word::WhyTests, Word::WhyPluralTests),
        Kind::Dependency => (Word::WhyDependency, Word::WhyPluralDependency),
        Kind::Departure => (Word::WhyDeparture, Word::WhyPluralDeparture),
        Kind::Money => (Word::WhyMoney, Word::WhyPluralMoney),
        Kind::Irreversible => (Word::WhyIrreversible, Word::WhyPluralIrreversible),
        Kind::Rules => (Word::WhyRules, Word::WhyPluralRules),
        Kind::Other => (Word::WhyOther, Word::WhyPluralOther),
        Kind::Findings => (Word::WhyFindings, Word::WhyPluralFindings),
    };
    if plural { many } else { one }
}
fn path_set(root: &temper_client_domain::Domain, w: &Words, project: u32) -> Box<[u8]> {
    let name = crate::common::project_name(root.pages().me(root.client()), project)
        .unwrap_or(w.say(Word::Project, &[Hole::Number(u64::from(project))]));
    w.say(Word::WhyPathSet, &[Hole::Name(name)])
}
