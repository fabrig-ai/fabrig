//! Values and edits of the forge's kinds of settings
//! (temper/docs/design/protocol/configuration/settings.md, 2 and 3;
//! temper/docs/design/domain/forge/change/gates.md, 3).
use alloc::boxed::Box;

use crate::{GateLast, GatePattern, JudgeFreshness, Limits, Repository, Requires, Touched, gates::covers};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    DeploymentRules,
    ProjectRules,
    Charters,
    ProjectQuestions,
    ProjectWays,
    ProjectClasses,
    /// A charter's words (temper/docs/design/protocol/configuration/settings.md, 2).
    CharterWords,
    /// A project's words (temper/docs/design/protocol/configuration/settings.md, 2).
    ProjectWords,
    /// A goal template's work (temper/docs/design/protocol/configuration/settings.md, 2).
    TemplateGoal,
    /// A project's follow (temper/docs/design/domain/forge/follows.md, 2).
    Follow,
    /// A project dependency (temper/docs/design/domain/forge/dependencies.md, 2).
    Dependency,
    ProjectAreas,
    ProjectTriager,
}

impl Kind {
    #[must_use]
    pub const fn number(self) -> u32 {
        match self {
            Self::DeploymentRules => 1,
            Self::ProjectRules => 2,
            Self::Charters => 3,
            Self::ProjectQuestions => 6,
            Self::ProjectWays => 7,
            Self::ProjectClasses => 8,
            Self::CharterWords => 9,
            Self::ProjectWords => 10,
            Self::TemplateGoal => 11,
            Self::Follow => 12,
            Self::Dependency => 13,
            Self::ProjectAreas => 14,
            Self::ProjectTriager => 15,
        }
    }

    #[must_use]
    pub const fn of(number: u32) -> Option<Self> {
        match number {
            1 => Some(Self::DeploymentRules),
            2 => Some(Self::ProjectRules),
            3 => Some(Self::Charters),
            6 => Some(Self::ProjectQuestions),
            7 => Some(Self::ProjectWays),
            8 => Some(Self::ProjectClasses),
            9 => Some(Self::CharterWords),
            10 => Some(Self::ProjectWords),
            11 => Some(Self::TemplateGoal),
            12 => Some(Self::Follow),
            13 => Some(Self::Dependency),
            14 => Some(Self::ProjectAreas),
            15 => Some(Self::ProjectTriager),
            _ => None,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LandingRule {
    pub pattern: GatePattern,
    pub enforce: bool,
    pub ci: bool,
    pub up_to_date: bool,
    pub gates: Box<[RuleGate]>,
    pub approvals: Box<[RuleApproval]>,
    pub sets: Box<[u8]>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct RuleGate {
    pub number: u32,
    pub blocking: bool,
    pub freshness: JudgeFreshness,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct RuleApproval {
    pub role: u32,
    pub people: u32,
    pub freshness: JudgeFreshness,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ProjectWays {
    pub drafts: bool,
    pub goals: bool,
    pub changes: bool,
    pub way: u32,
}

/// A charter's line's byte bound (temper/docs/design/protocol/configuration/settings.md, 2).
pub const LINE_BYTES: u32 = 120;
/// A project's headline's byte bound (temper/docs/design/protocol/configuration/settings.md, 2).
pub const HEADLINE_BYTES: u32 = 120;
/// A project's lede's byte bound (temper/docs/design/protocol/configuration/settings.md, 2).
pub const LEDE_BYTES: u32 = 480;

/// A project's words, kind 10 (temper/docs/design/protocol/configuration/settings.md, 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProjectWords {
    pub headline: Box<[u8]>,
    pub lede: Box<[u8]>,
}

/// Project ownership (temper/docs/design/domain/areas.md, 2 and 3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AreaOwner {
    Task(u64),
    Party(u64),
    Role(u32),
    Charter(u32),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Area {
    pub name: Box<[u8]>,
    pub words: Box<[u8]>,
    pub owner: AreaOwner,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProjectAreas {
    pub areas: Box<[Area]>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ProjectTriager {
    pub owner: AreaOwner,
}

/// Words saved with a setting fit `most`: 1 to `most` bytes of UTF-8 holding no
/// control character (temper/docs/design/protocol/configuration/settings.md, 2).
#[must_use]
#[expect(clippy::disallowed_methods, reason = "the forge judges whether setting words are UTF-8 at its boundary")]
pub fn words_fit(words: &[u8], most: u32) -> bool {
    if words.is_empty() || !within(words.len(), most) {
        return false;
    }
    let Ok(text) = core::str::from_utf8(words) else {
        return false;
    };
    for character in text.chars() {
        if character.is_control() {
            return false;
        }
    }
    true
}

/// A goal template's work, kind 11 (temper/docs/design/protocol/configuration/settings.md, 2):
/// whether its coordinator designs first and makes a close-out; neither with none.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct TemplateGoal {
    pub design: bool,
    pub close_out: bool,
}

/// A project's follow, kind 12 (temper/docs/design/domain/forge/follows.md, 2).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Follow {
    pub source: FollowSource,
    pub target: FollowTarget,
    pub owner: FollowOwner,
    pub start: FollowStart,
    pub producer: Option<Box<[u8]>>,
    pub check: FollowCheck,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FollowSource {
    pub repository: skein_forge::api::Repository,
    pub branch: Box<[u8]>,
    pub paths: Box<[Box<[u8]>]>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FollowTarget {
    pub repository: skein_forge::api::Repository,
    pub branch: Box<[u8]>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FollowOwner {
    Person(u64),
    Task(u64),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FollowStart {
    Owner,
    Code,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FollowCheck {
    pub workflow: Box<[u8]>,
    pub artifact: Box<[u8]>,
    pub format: temper_engine_domain_forge_tiers::Format,
}

#[path = "settings/follow.rs"]
mod follow;
pub use follow::{follow_from, follow_record, producer_charter};

/// A dependency on another project (temper/docs/design/domain/forge/dependencies.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Dependency {
    pub project: u32,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Value {
    Rule(LandingRule),
    Charter(u32),
    Questions(temper_engine_domain_forge_change::Route),
    Ways(ProjectWays),
    Classes(temper_protocol_reaches::Classes),
    /// A charter's line (temper/docs/design/protocol/configuration/settings.md, 2).
    CharterWords(Box<[u8]>),
    /// A project's headline and lede (temper/docs/design/protocol/configuration/settings.md, 2).
    ProjectWords(ProjectWords),
    /// A goal template's work (temper/docs/design/protocol/configuration/settings.md, 2).
    Goal(TemplateGoal),
    Follow(Follow),
    Dependency(Dependency),
    ProjectAreas(ProjectAreas),
    ProjectTriager(ProjectTriager),
}

/// A value and the bytes of its encoding counted by jig, measured where it
/// was decoded or made (temper/docs/design/domain/routing.md, 6).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SizedValue {
    pub value: Value,
    pub size: u32,
}

/// The value of a project's kind 6, 7 or 8 with none stored
/// (temper/docs/design/protocol/configuration/stock.md).
#[must_use]
pub fn project_stock(kind: u32) -> Option<SizedValue> {
    use temper_client_protocol_wire::v1;
    match kind {
        6 => {
            let record =
                v1::ProjectQuestions::new(&v1::CEILINGS, v1::ProjectQuestionsParts { route: v1::QuestionsRoute::None })
                    .expect("fixed route field");
            Some(SizedValue {
                value: Value::Questions(temper_engine_domain_forge_change::Route::None),
                size: record.measure(),
            })
        }
        7 => {
            let ways = ProjectWays { drafts: true, goals: false, changes: false, way: 0 };
            let record = v1::ProjectWays::new(
                &v1::CEILINGS,
                v1::ProjectWaysParts { drafts: ways.drafts, goals: ways.goals, changes: ways.changes, way: ways.way },
            )
            .expect("fixed ways fields");
            Some(SizedValue { value: Value::Ways(ways), size: record.measure() })
        }
        8 => {
            let classes = temper_protocol_reaches::Classes::stock(temper_protocol_reaches::Way::AgentsReview);
            let record = temper_protocol_reaches::classes_record(&classes).expect("stock table fits the record");
            Some(SizedValue { value: Value::Classes(classes), size: record.measure() })
        }
        _ => None,
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Judged {
    Accept { size: u32, charter: Option<u32> },
    Same,
    Refuse(Refusal),
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ForgeCharters {
    pub change: u32,
    pub review: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum SettingSlot {
    Default,
    Change,
    Proposal(u64),
    Replaced(u64),
}

/// The store's key order (temper/docs/design/protocol/store.md, sections 2 and 3).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SettingKey {
    pub kind: u32,
    pub scope: u32,
    pub number: u32,
    pub slot: SettingSlot,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
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
    /// Template numbers (temper/docs/design/protocol/configuration/settings.md, 2).
    pub templates: u32,
    pub follows: u32,
    pub dependencies: u32,
    pub follow_paths: u32,
    pub name_bytes: u32,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Item {
    pub kind: u32,
    pub scope: u32,
    pub number: u32,
    pub value: Option<Value>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Refusal {
    Kind,
    Scope,
    Number,
    Pattern,
    Gates,
    Approvals,
    Sets,
    SetNumber,
    SetTwice,
    DeploymentSets,
    Charter,
    Value,
    Replaced,
    Retired,
    Way,
    Classes(temper_protocol_reaches::Refusal),
    /// Invalid setting words (temper/docs/design/protocol/configuration/settings.md, 2).
    Words,
    FollowPaths,
    FollowPath,
    FollowProducer,
    FollowCheck,
    FollowBranch,
    FollowFixed,
    Owner {
        name: Box<[u8]>,
        owner: AreaOwner,
    },
    Area {
        name: Box<[u8]>,
        rule: &'static [u8],
    },
}

impl Refusal {
    #[must_use]
    pub fn words(&self) -> Box<[u8]> {
        if let Self::Owner { name, owner } = self {
            let (kind, number) = match owner {
                AreaOwner::Task(n) => (b"task".as_slice(), *n),
                AreaOwner::Party(n) => (b"party".as_slice(), *n),
                AreaOwner::Role(n) => (b"role".as_slice(), u64::from(*n)),
                AreaOwner::Charter(n) => (b"charter".as_slice(), u64::from(*n)),
            };
            let number = skein_lib::Decimal::of(number);
            let mut words = skein_lib::List::uncapped();
            for part in [name.as_ref(), b": owner ", kind, b" ", number.as_bytes(), b" is not of the project"] {
                for &byte in part {
                    words.push(byte).expect("bounded owner refusal");
                }
            }
            return words.into_boxed();
        }
        if let Self::Area { name, rule } = self {
            let mut words = skein_lib::List::uncapped();
            for part in [b"area ".as_slice(), name, b": ", rule] {
                for &byte in part {
                    words.push(byte).expect("bounded area refusal");
                }
            }
            return words.into_boxed();
        }
        let words: &[u8] = match self {
            Self::Kind => b"not a kind of the forge's",
            Self::Scope => b"a scope this kind does not have",
            Self::Number => b"a number this kind does not have",
            Self::Pattern => b"a pattern that is not a landing branch's, or past the pattern bounds",
            Self::Gates => b"more gates than forge.change_policy.gates, or a gate twice",
            Self::Approvals => b"an approval of no people, a role twice, or one past authority.roles",
            Self::Sets => b"more than 6 path sets",
            Self::SetNumber => b"a set outside 1 to 6",
            Self::SetTwice => b"a set named twice",
            Self::DeploymentSets => b"a deployment's rule names no path set",
            Self::Charter => b"a charter outside 1 to tasks.charters, or the forge's charter removed",
            Self::Value => b"a value of another kind",
            Self::Retired => b"retired stored setting",
            Self::Replaced => b"the replaced value is no longer kept",
            Self::FollowPaths => b"more paths than forge.follow_paths",
            Self::FollowPath => b"a malformed path or one past 256 bytes",
            Self::FollowProducer => b"its producer does not spell change parameter 11",
            Self::FollowCheck => b"a check with an empty member or one past 256 bytes",
            Self::FollowBranch => b"a branch past forge.name_bytes",
            Self::FollowFixed => b"its source and target are fixed; remove it and add another",
            Self::Words => b"words that are empty, past their bound, not UTF-8 or holding a control character",
            Self::Way => b"a way that is not a template the project offers",
            Self::Area { .. } | Self::Owner { .. } => unreachable!("area refusal rendered above"),
            Self::Classes(refusal) => match refusal {
                temper_protocol_reaches::Refusal::Items => b"more than 20 items in a classes table",
                temper_protocol_reaches::Refusal::Sets => b"more than 6 path sets in a classes table",
                temper_protocol_reaches::Refusal::Fixed { .. } => b"an item of a kind that always waits",
                temper_protocol_reaches::Refusal::Wait { .. } => b"an item whose class always waits",
                temper_protocol_reaches::Refusal::Words { .. } => b"an item's words past 120 bytes",
                temper_protocol_reaches::Refusal::NoSet { .. } => b"an item naming a path set the table lacks",
                temper_protocol_reaches::Refusal::Twice { .. } => b"a second item without words of its subject",
                temper_protocol_reaches::Refusal::Number { .. } => b"a path set numbered twice or outside 1 to 6",
                temper_protocol_reaches::Refusal::Name { .. } => b"a path set's name empty or past 40 bytes",
                temper_protocol_reaches::Refusal::Patterns { .. } => b"a path set with no pattern or more than 6",
                temper_protocol_reaches::Refusal::Pattern { .. } => {
                    b"a path set's pattern that is not well formed, empty or past 80 bytes"
                }
            },
        };
        words.into()
    }
}

fn within(len: usize, bound: u32) -> bool {
    match u32::try_from(len) {
        Ok(len) => len <= bound,
        Err(_) => false,
    }
}

#[expect(clippy::too_many_lines, reason = "one exhaustive judgement keeps each setting kind and its refusals together")]
pub fn judge(item: &Item, bounds: &Bounds) -> Result<(), Refusal> {
    let kind = Kind::of(item.kind).ok_or(Refusal::Kind)?;
    match kind {
        Kind::DeploymentRules | Kind::Charters | Kind::CharterWords | Kind::TemplateGoal if item.scope != 0 => {
            return Err(Refusal::Scope);
        }
        Kind::ProjectRules
        | Kind::ProjectQuestions
        | Kind::ProjectWays
        | Kind::ProjectClasses
        | Kind::ProjectAreas
        | Kind::ProjectTriager
        | Kind::ProjectWords
        | Kind::Dependency
        | Kind::Follow
            if item.scope == 0 =>
        {
            return Err(Refusal::Scope);
        }
        Kind::DeploymentRules
        | Kind::ProjectRules
        | Kind::Charters
        | Kind::ProjectQuestions
        | Kind::ProjectWays
        | Kind::ProjectClasses
        | Kind::CharterWords
        | Kind::ProjectWords
        | Kind::TemplateGoal
        | Kind::ProjectAreas
        | Kind::ProjectTriager
        | Kind::Dependency
        | Kind::Follow => {}
    }
    let numbers = match kind {
        Kind::DeploymentRules | Kind::ProjectRules => bounds.landing_rules,
        Kind::Charters => 2,
        Kind::CharterWords => bounds.charters,
        Kind::ProjectQuestions
        | Kind::ProjectWays
        | Kind::ProjectClasses
        | Kind::ProjectAreas
        | Kind::ProjectTriager
        | Kind::ProjectWords => 1,
        Kind::TemplateGoal => bounds.templates,
        Kind::Follow => bounds.follows,
        Kind::Dependency => bounds.dependencies,
    };
    if item.number == 0 || item.number > numbers {
        return Err(Refusal::Number);
    }
    let rule = match kind {
        Kind::DeploymentRules | Kind::ProjectRules => match &item.value {
            Some(Value::Rule(rule)) => rule,
            Some(
                Value::Dependency(_)
                | Value::Follow(_)
                | Value::Charter(_)
                | Value::Goal(_)
                | Value::Questions(_)
                | Value::Ways(_)
                | Value::CharterWords(_)
                | Value::Classes(_)
                | Value::ProjectAreas(_)
                | Value::ProjectTriager(_)
                | Value::ProjectWords(_),
            ) => return Err(Refusal::Value),
            None => return Ok(()),
        },
        Kind::Charters => {
            return match &item.value {
                Some(Value::Charter(charter)) if *charter > 0 && *charter <= bounds.charters => Ok(()),
                Some(Value::Charter(_)) | None => Err(Refusal::Charter),
                Some(
                    Value::Dependency(_)
                    | Value::Follow(_)
                    | Value::Rule(_)
                    | Value::Goal(_)
                    | Value::Questions(_)
                    | Value::Ways(_)
                    | Value::CharterWords(_)
                    | Value::Classes(_)
                    | Value::ProjectAreas(_)
                    | Value::ProjectTriager(_)
                    | Value::ProjectWords(_),
                ) => Err(Refusal::Value),
            };
        }
        Kind::ProjectQuestions => {
            return match &item.value {
                Some(Value::Questions(_)) | None => Ok(()),
                Some(
                    Value::Dependency(_)
                    | Value::Follow(_)
                    | Value::Rule(_)
                    | Value::Charter(_)
                    | Value::Goal(_)
                    | Value::Ways(_)
                    | Value::CharterWords(_)
                    | Value::Classes(_)
                    | Value::ProjectAreas(_)
                    | Value::ProjectTriager(_)
                    | Value::ProjectWords(_),
                ) => Err(Refusal::Value),
            };
        }
        Kind::ProjectClasses => {
            return match &item.value {
                Some(Value::Classes(classes)) => match classes.check() {
                    Ok(()) => Ok(()),
                    Err(refusal) => Err(Refusal::Classes(refusal)),
                },
                None => Ok(()),
                Some(
                    Value::Dependency(_)
                    | Value::Follow(_)
                    | Value::Rule(_)
                    | Value::Charter(_)
                    | Value::Questions(_)
                    | Value::Ways(_)
                    | Value::Goal(_)
                    | Value::CharterWords(_)
                    | Value::ProjectAreas(_)
                    | Value::ProjectTriager(_)
                    | Value::ProjectWords(_),
                ) => Err(Refusal::Value),
            };
        }
        Kind::ProjectWays => {
            return match &item.value {
                Some(Value::Ways(_)) | None => Ok(()),
                Some(
                    Value::Dependency(_)
                    | Value::Follow(_)
                    | Value::Rule(_)
                    | Value::Charter(_)
                    | Value::Goal(_)
                    | Value::Questions(_)
                    | Value::CharterWords(_)
                    | Value::Classes(_)
                    | Value::ProjectAreas(_)
                    | Value::ProjectTriager(_)
                    | Value::ProjectWords(_),
                ) => Err(Refusal::Value),
            };
        }
        Kind::CharterWords => {
            return match &item.value {
                Some(Value::CharterWords(line)) => {
                    if words_fit(line, LINE_BYTES) {
                        Ok(())
                    } else {
                        Err(Refusal::Words)
                    }
                }
                None => Ok(()),
                Some(
                    Value::Dependency(_)
                    | Value::Follow(_)
                    | Value::Rule(_)
                    | Value::Charter(_)
                    | Value::Goal(_)
                    | Value::Questions(_)
                    | Value::Ways(_)
                    | Value::Classes(_)
                    | Value::ProjectAreas(_)
                    | Value::ProjectTriager(_)
                    | Value::ProjectWords(_),
                ) => Err(Refusal::Value),
            };
        }
        Kind::ProjectWords => {
            return match &item.value {
                Some(Value::ProjectWords(words)) => {
                    if words_fit(&words.headline, HEADLINE_BYTES) && words_fit(&words.lede, LEDE_BYTES) {
                        Ok(())
                    } else {
                        Err(Refusal::Words)
                    }
                }
                None => Ok(()),
                Some(
                    Value::Dependency(_)
                    | Value::Follow(_)
                    | Value::Rule(_)
                    | Value::Charter(_)
                    | Value::Goal(_)
                    | Value::Questions(_)
                    | Value::Ways(_)
                    | Value::CharterWords(_)
                    | Value::Classes(_)
                    | Value::ProjectAreas(_)
                    | Value::ProjectTriager(_),
                ) => Err(Refusal::Value),
            };
        }
        Kind::ProjectAreas => {
            return match &item.value {
                Some(Value::ProjectAreas(value)) => judge_areas(value),
                None => Ok(()),
                Some(
                    Value::Follow(_)
                    | Value::Dependency(_)
                    | Value::Rule(_)
                    | Value::Charter(_)
                    | Value::Goal(_)
                    | Value::Questions(_)
                    | Value::Ways(_)
                    | Value::Classes(_)
                    | Value::CharterWords(_)
                    | Value::ProjectWords(_)
                    | Value::ProjectTriager(_),
                ) => Err(Refusal::Value),
            };
        }
        Kind::ProjectTriager => {
            return match &item.value {
                Some(Value::ProjectTriager(_)) | None => Ok(()),
                Some(
                    Value::Follow(_)
                    | Value::Dependency(_)
                    | Value::Rule(_)
                    | Value::Charter(_)
                    | Value::Goal(_)
                    | Value::Questions(_)
                    | Value::Ways(_)
                    | Value::Classes(_)
                    | Value::CharterWords(_)
                    | Value::ProjectWords(_)
                    | Value::ProjectAreas(_),
                ) => Err(Refusal::Value),
            };
        }
        Kind::Dependency => {
            return match &item.value {
                Some(Value::Dependency(value)) if value.project != 0 => Ok(()),
                None => Ok(()),
                Some(
                    Value::ProjectAreas(_)
                    | Value::ProjectTriager(_)
                    | Value::Dependency(_)
                    | Value::Follow(_)
                    | Value::Rule(_)
                    | Value::Charter(_)
                    | Value::Questions(_)
                    | Value::Ways(_)
                    | Value::Classes(_)
                    | Value::CharterWords(_)
                    | Value::ProjectWords(_)
                    | Value::Goal(_),
                ) => Err(Refusal::Value),
            };
        }
        Kind::Follow => {
            return match &item.value {
                Some(Value::Follow(value)) => follow::judge(value, bounds),
                None => Ok(()),
                Some(
                    Value::ProjectAreas(_)
                    | Value::ProjectTriager(_)
                    | Value::Dependency(_)
                    | Value::Rule(_)
                    | Value::Charter(_)
                    | Value::Questions(_)
                    | Value::Ways(_)
                    | Value::Classes(_)
                    | Value::CharterWords(_)
                    | Value::ProjectWords(_)
                    | Value::Goal(_),
                ) => Err(Refusal::Value),
            };
        }
        Kind::TemplateGoal => {
            return match &item.value {
                Some(Value::Goal(_)) | None => Ok(()),
                Some(
                    Value::Dependency(_)
                    | Value::Follow(_)
                    | Value::Rule(_)
                    | Value::Charter(_)
                    | Value::Questions(_)
                    | Value::Ways(_)
                    | Value::CharterWords(_)
                    | Value::Classes(_)
                    | Value::ProjectAreas(_)
                    | Value::ProjectTriager(_)
                    | Value::ProjectWords(_),
                ) => Err(Refusal::Value),
            };
        }
    };
    let Some(first) = rule.pattern.segments.first() else {
        return Err(Refusal::Pattern);
    };
    let first_ok = first.as_ref() == b"forge" || (kind == Kind::ProjectRules && first.as_ref() == b"branch");
    if !first_ok || !within(rule.pattern.segments.len(), bounds.segments) {
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
    if kind == Kind::DeploymentRules && !rule.sets.is_empty() {
        return Err(Refusal::DeploymentSets);
    }
    if !within(rule.sets.len(), 6) {
        return Err(Refusal::Sets);
    }
    for (at, set) in rule.sets.iter().enumerate() {
        if *set == 0 || *set > 6 {
            return Err(Refusal::SetNumber);
        }
        for prior in rule.sets.iter().take(at) {
            if set == prior {
                return Err(Refusal::SetTwice);
            }
        }
    }
    Ok(())
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Held {
    pub scope: u32,
    pub number: u32,
    pub rule: LandingRule,
}

/// Current rules by scope (temper/docs/design/domain/forge/change/gates.md, 3).
#[derive(Hash, Debug)]
pub struct HeldRules {
    pub deployment: Box<[Held]>,
    pub every_project: Box<[Held]>,
    pub projects: skein_lib::Map<u32, Box<[Held]>>,
}

/// The procedure's waits over the supplied scopes' rules, enforcing or not
/// (temper/docs/design/domain/forge/change/gates.md, 2).
#[must_use]
pub fn requires(held: &[Held], repository: &Repository, branch: &[u8], touched: Touched) -> Requires {
    let mut result = Requires { ci: false, up_to_date: false };
    let mut covered = false;
    for held in held {
        if covers(&held.rule, repository, branch, touched) {
            covered = true;
            result.ci |= held.rule.ci;
            result.up_to_date |= held.rule.up_to_date;
        }
    }
    if covered { result } else { Requires { ci: true, up_to_date: true } }
}

/// Defaults, changes, proposals and replaced values (jig/docs/design/domain/connectors.md, 17).
#[must_use]
pub fn rows(l: &Limits) -> Option<u32> {
    let rules = l.judge_projects.checked_add(1)?.checked_mul(l.landing_rules)?;
    let values = rules
        .checked_add(2)?
        .checked_add(l.judge_projects.checked_mul(2)?)?
        .checked_add(l.judge_projects.checked_mul(l.follows)?)?
        .checked_add(l.judge_projects.checked_mul(l.dependencies)?)?;
    values.checked_mul(2)?.checked_add(l.setting_proposals.checked_add(l.setting_history)?.checked_mul(values)?)
}

fn judge_areas(value: &ProjectAreas) -> Result<(), Refusal> {
    if !within(value.areas.len(), 16) {
        return Err(Refusal::Area {
            name: value.areas.get(16).expect("past 16 areas").name.clone(),
            rule: b"more than 16 areas",
        });
    }
    for (at, area) in value.areas.iter().enumerate() {
        let mut twice = false;
        for prior in value.areas.iter().take(at) {
            twice |= prior.name == area.name;
        }
        let rule: &'static [u8] = if !area_name_fits(&area.name) {
            b"name must be 1 to 32 bytes of lowercase ASCII letters, digits and -, starting with a letter"
        } else if !words_fit(&area.words, 480) {
            b"words must be 1 to 480 bytes of UTF-8 with no control character"
        } else if twice {
            b"name occurs twice"
        } else {
            continue;
        };
        return Err(Refusal::Area { name: area.name.clone(), rule });
    }
    Ok(())
}
fn area_name_fits(name: &[u8]) -> bool {
    let Some(first) = name.first() else { return false };
    if !first.is_ascii_lowercase() || !within(name.len(), 32) {
        return false;
    }
    for byte in name {
        if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() && *byte != b'-' {
            return false;
        }
    }
    true
}

/// Name a retired stored setting in ctl's terms (protocol/configuration/settings.md, 2).
#[must_use]
pub fn retired_words(key: SettingKey) -> Box<[u8]> {
    let (name, number) = if key.kind == 6 {
        (b"project-questions ".as_slice(), key.scope)
    } else {
        (b"forge-charter ".as_slice(), key.number)
    };
    let number = skein_lib::Decimal::of(u64::from(number));
    let reason = b": retired stored setting";
    let length = name
        .len()
        .checked_add(number.as_bytes().len())
        .expect("bounded setting identity")
        .checked_add(reason.len())
        .expect("bounded refusal");
    let mut words = skein_lib::Writer::new(length);
    words.put(name).expect("sized setting name");
    words.put(number.as_bytes()).expect("sized setting number");
    words.put(reason).expect("sized refusal");
    words.finish()
}
