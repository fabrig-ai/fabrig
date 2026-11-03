//! Landing rules as settings of the forge's kinds
//! (temper/docs/design/protocol/configuration.md, 13.2 and 13.3;
//! temper/docs/design/domain/forge.md, 8.9).
use alloc::boxed::Box;

use crate::{GateLast, GatePattern, JudgeFreshness, Requires, gates::covers};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    DeploymentRules,
    ProjectRules,
}

impl Kind {
    #[must_use]
    pub const fn number(self) -> u32 {
        match self {
            Self::DeploymentRules => 1,
            Self::ProjectRules => 2,
        }
    }

    #[must_use]
    pub const fn of(number: u32) -> Option<Self> {
        match number {
            1 => Some(Self::DeploymentRules),
            2 => Some(Self::ProjectRules),
            _ => None,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LandingRule {
    pub pattern: GatePattern,
    pub enforce: bool,
    pub ci: bool,
    pub up_to_date: bool,
    pub gates: Box<[RuleGate]>,
    pub approvals: Box<[RuleApproval]>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RuleGate {
    pub number: u32,
    pub blocking: bool,
    pub freshness: JudgeFreshness,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RuleApproval {
    pub role: u32,
    pub people: u32,
    pub freshness: JudgeFreshness,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bounds {
    pub landing_rules: u32,
    pub gates: u32,
    pub roles: u32,
    pub segments: u32,
    pub segment_bytes: u32,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Item {
    pub kind: u32,
    pub scope: u32,
    pub number: u32,
    pub rule: Option<LandingRule>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refusal {
    Kind,
    Scope,
    Number,
    Pattern,
    Gates,
    Approvals,
}

fn within(len: usize, bound: u32) -> bool {
    match u32::try_from(len) {
        Ok(len) => len <= bound,
        Err(_) => false,
    }
}

pub fn judge(item: &Item, bounds: &Bounds) -> Result<(), Refusal> {
    match Kind::of(item.kind).ok_or(Refusal::Kind)? {
        Kind::DeploymentRules if item.scope != 0 => return Err(Refusal::Scope),
        Kind::ProjectRules if item.scope == 0 => return Err(Refusal::Scope),
        Kind::DeploymentRules | Kind::ProjectRules => {}
    }
    if item.number == 0 || item.number > bounds.landing_rules {
        return Err(Refusal::Number);
    }
    let Some(rule) = &item.rule else {
        return Ok(());
    };
    let Some(first) = rule.pattern.segments.first() else {
        return Err(Refusal::Pattern);
    };
    if first.as_ref() != b"forge" || !within(rule.pattern.segments.len(), bounds.segments) {
        return Err(Refusal::Pattern);
    }
    for segment in &rule.pattern.segments {
        if !within(segment.len(), bounds.segment_bytes) {
            return Err(Refusal::Pattern);
        }
    }
    let last = match &rule.pattern.last {
        GateLast::Exact(last) | GateLast::Open(last) => last,
    };
    if !within(last.len(), bounds.segment_bytes) {
        return Err(Refusal::Pattern);
    }
    if !within(rule.gates.len(), bounds.gates) {
        return Err(Refusal::Gates);
    }
    for (at, gate) in rule.gates.iter().enumerate() {
        for prior in rule.gates.iter().take(at) {
            if gate.number == prior.number {
                return Err(Refusal::Gates);
            }
        }
    }
    if !within(rule.approvals.len(), bounds.roles) {
        return Err(Refusal::Approvals);
    }
    for (at, approval) in rule.approvals.iter().enumerate() {
        if approval.people == 0 || approval.role >= bounds.roles {
            return Err(Refusal::Approvals);
        }
        for prior in rule.approvals.iter().take(at) {
            if approval.role == prior.role {
                return Err(Refusal::Approvals);
            }
        }
    }
    Ok(())
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Held {
    pub scope: u32,
    pub number: u32,
    pub rule: LandingRule,
}

/// The procedure's waits over the supplied scopes' rules, enforcing or not
/// (temper/docs/design/domain/forge.md, 8.7).
#[must_use]
pub fn requires(held: &[Held], name: &[Box<[u8]>]) -> Requires {
    let mut result = Requires { ci: false, up_to_date: false };
    let mut covered = false;
    for held in held {
        if covers(&held.rule.pattern, name) {
            covered = true;
            result.ci |= held.rule.ci;
            result.up_to_date |= held.rule.up_to_date;
        }
    }
    if covered { result } else { Requires { ci: true, up_to_date: true } }
}
