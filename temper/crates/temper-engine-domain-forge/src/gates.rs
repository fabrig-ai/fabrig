//! Forge landing patterns and gate selection (temper/docs/design/domain/forge.md, 8.7).
use crate::{Limits, Repository};
use alloc::boxed::Box;
use skein_lib::List;
use temper_engine_domain_forge_change as change;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum GateLast {
    Exact(Box<[u8]>),
    Open(Box<[u8]>),
}
#[derive(Clone, PartialEq, Eq, Debug)]
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
/// The procedure's waits from landing rules (temper/docs/design/domain/forge.md, 8.7).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Requires {
    pub ci: bool,
    pub up_to_date: bool,
}
pub(crate) fn covers(pattern: &GatePattern, name: &[Box<[u8]>]) -> bool {
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
