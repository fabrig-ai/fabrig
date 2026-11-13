//! One sign-off per role (temper/docs/design/domain/forge/change/branch-path.md, step 5 and An approval's card).
use super::{
    Card, Inputs, Role, VALUE, WORDS,
    cards::{card, fact, number, text},
    scopes::{Coverage, by_path, patterns, set_names},
};
use alloc::boxed::Box;
use skein_lib::{List, bytes};
use temper_engine_domain_forge_vocabulary::{JudgeFreshness, settings::Held};

pub(super) fn count_words(count: u32) -> Box<[u8]> {
    match count {
        1 => b"one".as_slice().into(),
        2 => b"two".as_slice().into(),
        3 => b"three".as_slice().into(),
        4 => b"four".as_slice().into(),
        5 => b"five".as_slice().into(),
        6 => b"six".as_slice().into(),
        7 => b"seven".as_slice().into(),
        8 => b"eight".as_slice().into(),
        9 => b"nine".as_slice().into(),
        10 => b"ten".as_slice().into(),
        0 | 11.. => number(count),
    }
}

/// A role's sign-off words, shared by the path and rule card (branch-path.md, step 5).
#[must_use]
pub fn approval_words(role: &Role, count: u32, conditional: bool) -> Box<[u8]> {
    sign_words(role, count, conditional, true)
}
pub(super) fn sign_words(role: &Role, count: u32, conditional: bool, approve: bool) -> Box<[u8]> {
    if role.yours && role.holders > 1 {
        return text(&bytes::concat(&[&count_words(count), b" of you"]), WORDS);
    }
    let mut words = if role.yours {
        if conditional || !approve { b"you".as_slice().into() } else { b"you approve".as_slice().into() }
    } else {
        let article = match role.name.first() {
            Some(b'a' | b'e' | b'i' | b'o' | b'u' | b'A' | b'E' | b'I' | b'O' | b'U') => b"an ".as_slice(),
            Some(_) | None => b"a ".as_slice(),
        };
        bytes::concat(&[article, &role.name, if approve { b" approves" } else { b"" }])
    };
    if count > 1 {
        words = bytes::concat(&[&words, " ×".as_bytes(), &number(count)]);
    }
    text(&words, WORDS)
}

pub(super) fn role(inputs: &Inputs, number_: u32) -> Role {
    for role in &inputs.roles {
        if role.number == number_ {
            return role.clone();
        }
    }
    Role { number: number_, name: bytes::concat(&[b"role ", &number(number_)]), yours: false, holders: 0 }
}

#[derive(Debug)]
pub(super) struct Approval {
    pub role: Role,
    pub count: u32,
    pub every: u32,
    pub coverage: Coverage,
    pub first: Held,
    pub clean: bool,
}
pub(super) fn collect(inputs: &Inputs, held: &[Held], row: bool) -> List<Approval> {
    let mut approvals: List<Approval> = List::uncapped();
    for held in held {
        if !held.rule.enforce {
            continue;
        }
        for ask in &held.rule.approvals {
            let mut present = false;
            for approval in approvals.iter_mut() {
                if approval.role.number == ask.role {
                    approval.count = approval.count.max(ask.people);
                    if !by_path(&held.rule, &inputs.sets) {
                        approval.every = approval.every.max(ask.people);
                    }
                    approval.coverage.add(inputs, &held.rule);
                    approval.clean &= ask.freshness == JudgeFreshness::Clean;
                    present = true;
                    break;
                }
            }
            if !present {
                let mut coverage = Coverage::new();
                coverage.add(inputs, &held.rule);
                approvals
                    .push(Approval {
                        role: role(inputs, ask.role),
                        count: ask.people,
                        every: if coverage.all { ask.people } else { 0 },
                        coverage,
                        first: held.clone(),
                        clean: ask.freshness == JudgeFreshness::Clean,
                    })
                    .expect("role approvals fit memory");
            }
        }
    }
    for approval in approvals.iter_mut() {
        if approval.coverage.all && !row {
            approval.count = approval.every;
        }
    }
    approvals
}

pub(super) fn approval_card(inputs: &Inputs, held: &[Held], approval: &Approval) -> Card {
    let mut facts = List::uncapped();
    if !approval.coverage.all {
        facts
            .push(fact(b"Only for", &patterns(inputs, approval.coverage.sets.as_slice(), VALUE)))
            .expect("approval facts");
    }
    let together = approval.role.yours && approval.role.holders > 1;
    let role = if together {
        bytes::concat(&[&approval.role.name, b": the ", &count_words(approval.role.holders), b" of you"])
    } else {
        approval.role.name.clone()
    };
    facts.push(fact(b"Role", &role)).expect("approval facts");
    let mut asks = bytes::concat(&[&number(approval.count), if together { b" of you" } else { b" of them" }]);
    for held in held {
        if held.rule.enforce && by_path(&held.rule, &inputs.sets) {
            for ask in &held.rule.approvals {
                if ask.role == approval.role.number && ask.people > approval.count {
                    asks = bytes::concat(&[
                        &asks,
                        b"; ",
                        &set_names(&held.rule, &inputs.sets),
                        b": ",
                        &number(ask.people),
                    ]);
                }
            }
        }
    }
    facts.push(fact(b"Asks", &asks)).expect("approval facts");
    let counts = if approval.clean {
        b"code counts the sign-offs at this head; a clean update keeps them".as_slice()
    } else {
        b"code counts the sign-offs at this head".as_slice()
    };
    facts.push(fact(b"Counts", counts)).expect("approval facts");
    card("a person · decides whether it lands".as_bytes(), &approval.role.name, facts.into_boxed())
}
