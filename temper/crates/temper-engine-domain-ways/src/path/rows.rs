//! Rules by path (temper/docs/design/domain/forge/change/branch-path.md, Rules by path).
use super::{
    Inputs, approvals,
    cards::text,
    rules,
    scopes::{by_path, patterns},
};
use alloc::boxed::Box;
use skein_lib::{List, bytes};
use temper_engine_domain_forge::{JudgeFreshness, settings::Held};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PathRules {
    pub rows: Box<[PathRule]>,
    pub note: Box<[u8]>,
}
impl PathRules {
    #[must_use]
    pub fn empty() -> Self {
        Self { rows: Box::default(), note: Box::default() }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PathRule {
    pub paths: Box<[u8]>,
    pub review: Box<[u8]>,
    pub sign_off: Box<[u8]>,
    pub lands: Box<[u8]>,
}

#[must_use]
pub fn rules_by_path(inputs: &Inputs) -> PathRules {
    let held = rules::covering(inputs, &inputs.branch);
    let mut rows = List::uncapped();
    let mut every = List::uncapped();
    for held in &held {
        if !by_path(&held.rule, &inputs.sets) {
            every.push(held.clone()).expect("covering rules fit memory");
        }
    }
    for held in &held {
        if held.rule.enforce && by_path(&held.rule, &inputs.sets) && rows.len() < 8 {
            let covering = held_order(inputs, every.as_slice(), held);
            rows.push(row(inputs, covering.as_slice(), patterns(inputs, &held.rule.sets, 256)))
                .expect("eight path rows");
        }
    }
    if rows.is_empty() {
        return PathRules::empty();
    }
    let remaining = row(inputs, every.as_slice(), b"everything else".as_slice().into());
    if !remaining.review.is_empty() || !remaining.sign_off.is_empty() {
        rows.push(remaining).expect("last row");
    }
    let mut clean = false;
    let mut exact = false;
    for held in &held {
        if !held.rule.enforce {
            continue;
        }
        for approval in &held.rule.approvals {
            match approval.freshness {
                JudgeFreshness::Clean => clean = true,
                JudgeFreshness::Exact => exact = true,
            }
        }
    }
    let prefix = b"A sign-off is for one head.".as_slice();
    let note = match (clean, exact) {
        (true, true) => prefix.into(),
        (true, false) => bytes::concat(&[
            prefix,
            b" A change pushed after it needs signing again; taking ",
            &inputs.branch,
            b" cleanly keeps it.",
        ]),
        (false, true) => bytes::concat(&[
            prefix,
            b" A change pushed after it, or taking ",
            &inputs.branch,
            b", needs signing again.",
        ]),
        (false, false) => Box::default(),
    };
    PathRules { rows: rows.into_boxed(), note: crate::cut_on_word(&note, 160) }
}

fn held_order(inputs: &Inputs, every: &[Held], one: &Held) -> List<Held> {
    let mut result = List::uncapped();
    for held in &inputs.rules {
        if held == one || every.contains(held) {
            result.push(held.clone()).expect("row rules fit memory");
        }
    }
    result
}

fn row(inputs: &Inputs, held: &[Held], paths: Box<[u8]>) -> PathRule {
    let mut reviews = 0_u32;
    for gate in &rules::gates(inputs, held) {
        if !gate.check {
            reviews = reviews.saturating_add(1);
        }
    }
    let review = match reviews {
        0 => Box::default(),
        1 => b"an agent".as_slice().into(),
        2.. => bytes::concat(&[&approvals::count_words(reviews), b" agents"]),
    };
    // A row draws the greatest count of its applicable rules, not the line's every-change count.
    let approvals = approvals::collect(inputs, held, true);
    let mut sign_off = Box::from(b"".as_slice());
    for approval in &approvals {
        sign_off = bytes::concat(&[
            &sign_off,
            if sign_off.is_empty() { b"" } else { b" and " },
            &approvals::sign_words(&approval.role, approval.count, false, false),
        ]);
    }
    let lands = match approvals.as_slice() {
        [] => b"on its own".as_slice().into(),
        [approval] => {
            if approval.count == 1 {
                if approval.role.yours && approval.role.holders <= 1 {
                    b"once you sign off".as_slice().into()
                } else {
                    b"once signed".as_slice().into()
                }
            } else {
                bytes::concat(&[b"once ", &approvals::count_words(approval.count), b" have signed this head"])
            }
        }
        [_, _, ..] => b"once each has signed".as_slice().into(),
    };
    PathRule { paths, review: text(&review, 64), sign_off: crate::cut_on_word(&sign_off, 64), lands: text(&lands, 64) }
}
