//! Values and edits of the forge's kinds of settings
//! (temper/docs/design/protocol/configuration.md, 13.2 and 13.3;
//! temper/docs/design/domain/forge.md, 8.9).
use alloc::boxed::Box;

use crate::{GateLast, GatePattern, JudgeFreshness, Limits, Requires, gates::covers};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    DeploymentRules,
    ProjectRules,
    Charters,
}

impl Kind {
    #[must_use]
    pub const fn number(self) -> u32 {
        match self {
            Self::DeploymentRules => 1,
            Self::ProjectRules => 2,
            Self::Charters => 3,
        }
    }

    #[must_use]
    pub const fn of(number: u32) -> Option<Self> {
        match number {
            1 => Some(Self::DeploymentRules),
            2 => Some(Self::ProjectRules),
            3 => Some(Self::Charters),
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Value {
    Rule(LandingRule),
    Charter(u32),
}

/// A value and the bytes of its encoding counted by jig, measured where it
/// was decoded or made (temper/docs/design/domain/root.md, 5.6).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SizedValue {
    pub value: Value,
    pub size: u32,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Edit {
    Set(SizedValue),
    Remove,
    Reset,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Asked {
    pub kind: u32,
    pub scope: u32,
    pub number: u32,
    pub edit: Edit,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Judged {
    Accept { size: u32, charter: Option<u32> },
    Same,
    Refuse(Refusal),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Stock {
    pub kind: u32,
    pub scope: u32,
    pub number: u32,
    pub value: SizedValue,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DefaultState {
    pub kind: u32,
    pub scope: u32,
    pub number: u32,
    pub size: u32,
    pub charter: Option<u32>,
    pub differs: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ForgeCharters {
    pub change: u32,
    pub review: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum SettingSlot {
    Default,
    Change,
    Proposal(u64),
}

/// The store's key order (temper/docs/design/protocol/store.md, sections 2 and 3).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SettingKey {
    pub kind: u32,
    pub scope: u32,
    pub number: u32,
    pub slot: SettingSlot,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SettingRow {
    pub key: SettingKey,
    pub edit: Edit,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bounds {
    pub landing_rules: u32,
    pub gates: u32,
    pub roles: u32,
    pub charters: u32,
    pub segments: u32,
    pub segment_bytes: u32,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Item {
    pub kind: u32,
    pub scope: u32,
    pub number: u32,
    pub value: Option<Value>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refusal {
    Kind,
    Scope,
    Number,
    Pattern,
    Gates,
    Approvals,
    Charter,
    Value,
}

impl Refusal {
    #[must_use]
    pub const fn words(self) -> &'static [u8] {
        match self {
            Self::Kind => b"not a kind of the forge's",
            Self::Scope => b"a scope this kind does not have",
            Self::Number => b"a number this kind does not have",
            Self::Pattern => b"a pattern that is not a landing branch's, or past the pattern bounds",
            Self::Gates => b"more gates than forge.change_policy.gates, or a gate twice",
            Self::Approvals => b"an approval of no people, a role twice, or one past authority.roles",
            Self::Charter => b"a charter outside 1 to tasks.charters, or the forge's charter removed",
            Self::Value => b"a value of another kind",
        }
    }
}

fn within(len: usize, bound: u32) -> bool {
    match u32::try_from(len) {
        Ok(len) => len <= bound,
        Err(_) => false,
    }
}

pub fn judge(item: &Item, bounds: &Bounds) -> Result<(), Refusal> {
    let kind = Kind::of(item.kind).ok_or(Refusal::Kind)?;
    match kind {
        Kind::DeploymentRules | Kind::Charters if item.scope != 0 => return Err(Refusal::Scope),
        Kind::ProjectRules if item.scope == 0 => return Err(Refusal::Scope),
        Kind::DeploymentRules | Kind::ProjectRules | Kind::Charters => {}
    }
    let numbers = match kind {
        Kind::DeploymentRules | Kind::ProjectRules => bounds.landing_rules,
        Kind::Charters => 2,
    };
    if item.number == 0 || item.number > numbers {
        return Err(Refusal::Number);
    }
    let rule = match kind {
        Kind::DeploymentRules | Kind::ProjectRules => match &item.value {
            Some(Value::Rule(rule)) => rule,
            Some(Value::Charter(_)) => return Err(Refusal::Value),
            None => return Ok(()),
        },
        Kind::Charters => {
            return match &item.value {
                Some(Value::Charter(charter)) if *charter > 0 && *charter <= bounds.charters => Ok(()),
                Some(Value::Charter(_)) | None => Err(Refusal::Charter),
                Some(Value::Rule(_)) => Err(Refusal::Value),
            };
        }
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

/// The retained defaults, changes and proposal items (temper/docs/design/protocol/store.md, 2).
#[must_use]
pub fn rows(l: &Limits) -> Option<u32> {
    let rules = l.judge_projects.checked_add(1)?.checked_mul(l.landing_rules)?;
    rules.checked_mul(2)?.checked_add(4)?.checked_add(l.setting_proposals.checked_mul(rules.checked_add(2)?)?)
}
