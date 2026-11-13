//! Forge landing patterns and gate selection (temper/docs/design/domain/forge/change/gates.md, 2).
use crate::{Limits, Repository};
use alloc::boxed::Box;
use skein_lib::List;
use temper_engine_domain_forge_change as change;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum GateLast {
    Exact(Box<[u8]>),
    Open(Box<[u8]>),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GatePattern {
    pub segments: Box<[Box<[u8]>]>,
    pub last: GateLast,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GateCandidate {
    pub number: u32,
    pub parameters: u32,
    pub blocking: bool,
    pub project_scope: bool,
    pub freshness: change::Freshness,
}
/// The procedure's waits from landing rules (temper/docs/design/domain/forge/change/gates.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Requires {
    pub ci: bool,
    pub up_to_date: bool,
}
pub(crate) fn covers_name(pattern: &GatePattern, name: &[Box<[u8]>]) -> bool {
    let name = match pattern.segments.first() {
        Some(first) if first.as_ref() == b"branch" => match name.get(4..) {
            Some(tail) => tail,
            None => return false,
        },
        Some(_) | None => name,
    };
    if name.get(..pattern.segments.len()) != Some(pattern.segments.as_ref()) {
        return false;
    }
    match &pattern.last {
        GateLast::Exact(last) => {
            name.len().checked_sub(1) == Some(pattern.segments.len()) && name.get(pattern.segments.len()) == Some(last)
        }
        GateLast::Open(last) => match name.get(pattern.segments.len()) {
            Some(segment) => segment.starts_with(last),
            None => false,
        },
    }
}
/// Whether `pattern` covers `branch` of `repository`: a pattern beginning at
/// `branch` is judged against the branch's resource path past its
/// repository's four segments (temper/docs/design/protocol/configuration/settings.md, 2).
#[must_use]
pub fn covers_branch(pattern: &GatePattern, repository: &Repository, branch: &[u8]) -> bool {
    match crate::domain::branch_resource_name(repository, branch) {
        Some(name) => covers_name(pattern, &name),
        None => false,
    }
}

/// Paths touched at one judged head (temper/docs/design/domain/forge/change/gates.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Touched {
    pub sets: u8,
    pub whole: bool,
}
impl Touched {
    pub const UNKNOWN: Self = Self { sets: 0b11_1111, whole: false };
}

#[must_use]
pub fn touched(classes: &temper_protocol_reaches::Classes, paths: &[Box<[u8]>], whole: bool) -> Touched {
    if !whole {
        return Touched::UNKNOWN;
    }
    let mut sets = 0;
    for number in 1_u8..=6 {
        let matched = match classes.set(number) {
            Some(set) => set.matched(paths) != 0,
            None => true,
        };
        if matched {
            sets |= set_bit(number);
        }
    }
    Touched { sets, whole }
}

pub(crate) fn set_bit(number: u8) -> u8 {
    match number.checked_sub(1) {
        Some(shift) => 1_u8.checked_shl(u32::from(shift)).unwrap_or(0),
        None => 0,
    }
}

/// One coverage predicate for every reader of landing rules.
#[must_use]
pub fn covers(rule: &crate::settings::LandingRule, repository: &Repository, branch: &[u8], touched: Touched) -> bool {
    if !covers_branch(&rule.pattern, repository, branch) {
        return false;
    }
    if rule.sets.is_empty() {
        return true;
    }
    for number in &rule.sets {
        if touched.sets & set_bit(*number) != 0 {
            return true;
        }
    }
    false
}

pub(crate) fn selected(
    limits: &Limits,
    repository: &Repository,
    candidates: &[GateCandidate],
    required: &[bool],
) -> Option<Box<[change::Gate]>> {
    if candidates.len() != required.len() {
        return None;
    }
    let mut gates: List<change::Gate> = List::with_capacity(limits.change_policy.gates);
    for (at, candidate) in candidates.iter().enumerate() {
        let is_check = !repository.ci && repository.checks.contains(&candidate.number);
        let required = candidate.blocking && *required.get(at)?;
        let mut present = false;
        for prior in gates.as_slice() {
            if prior.number == u64::from(candidate.number) {
                present = true;
            }
        }
        if present || (!is_check && candidate.blocking && !required) {
            continue;
        }
        gates
            .push(change::Gate {
                number: u64::from(candidate.number),
                kind: if is_check { change::GateKind::Check } else { change::GateKind::Agent },
                blocking: is_check || required,
                freshness: if is_check { change::Freshness::Exact } else { candidate.freshness },
                eager: false,
            })
            .ok()?;
    }
    if !repository.ci {
        for number in &repository.checks {
            let mut present = false;
            for prior in gates.as_slice() {
                if prior.number == u64::from(*number) {
                    present = true;
                }
            }
            if !present {
                gates
                    .push(change::Gate {
                        number: u64::from(*number),
                        kind: change::GateKind::Check,
                        blocking: true,
                        freshness: change::Freshness::Exact,
                        eager: false,
                    })
                    .ok()?;
            }
        }
    }
    Some(gates.into_boxed())
}
