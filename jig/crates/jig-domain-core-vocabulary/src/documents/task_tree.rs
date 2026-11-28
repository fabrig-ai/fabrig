//! Task pages, live trees and goal lines (`client/domain/pages.md`, 5 and 6.2).
//! Closed at once (jig/docs/design/domain/people/acts.md, 2).
//! Copies bounded rows from their owners. Trees fold every unshown live row into
//! phase counts; held counts cover the whole subtree. Goal progress uses retained
//! goal projections, including ended rows. Nothing here mutates a task or policy.

pub use jig_domain_contract::parts::{
    BudgetLine, Dependency, Ending, EscalationState, HoldPhaseLine, PhaseCounts, PlacedLine, PriorPhase, ResourceLine,
    ResultValue,
};

use super::{Chip, Cursor, Missing, Proposer, Text};
use crate::Limits;
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{List, Wall};

pub use tasks::{Part, PartKey, PartKind};

/// Terminal coverage rule of a granted resource pattern.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum PatternEnd {
    /// One literal terminal segment.
    Exact(Box<[u8]>),
    /// A prefix and its descendants.
    Open(Box<[u8]>),
}

/// One connector permission on an unchanged resource pattern.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct GrantLine {
    pub connector: u16,
    pub kind: u16,
    pub segments: List<Box<[u8]>>,
    pub last: PatternEnd,
}

/// A connector resource scope where notes may be written.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct NoteScope {
    pub connector: u16,
    pub segments: List<Box<[u8]>>,
    pub last: PatternEnd,
}

/// An authority carrier in bounded document form.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AuthorityLine {
    pub models: tasks::Models,
    pub tools: tasks::Tools,
    pub grants: List<GrantLine>,
    pub executors: List<tasks::AuthorityExecutor>,
    pub tasks: u32,
    pub depth: u32,
    pub ceiling: tasks::Budget,
    pub notes: tasks::Scopes,
    pub note_resources: List<NoteScope>,
}

/// Result form and its admitted bounds, usable as a person task's form.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ContractLine {
    /// Maximum application result bytes (jig/docs/design/domain/tasks/lifecycle.md, 3.1).
    Job { bytes: u32 },
    /// Exact connector state required for approval; it is never shortened.
    Approval { state: Box<[u8]>, part: Part },
    /// A report in words.
    Report { words: u32 },
    /// A configured choice with an explanation and bounded follow-ups.
    Verdict { choices: List<tasks::Verdict>, cut: bool },
    /// A connector-owned change and its summary.
    Change { connector: u16, kind: u16, words: u32 },
}

/// One task reference, including a retained result still named by live work.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TaskLink {
    /// An ended task whose stored owner row supplied its identity and wall time.
    Retained { chip: Chip, at: Option<Wall> },
    /// The referenced task is live and visible.
    Live(Chip),
    /// Its title and complete result belong to a stored task row.
    Stored { number: u64, status: Option<tasks::Status> },
}

/// A pending proposal's immutable identity.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ProposalRef {
    pub proposer: Proposer,
    pub number: u64,
}

/// A bounded specification parameter; connector resources remain references.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Parameter {
    /// An opaque named numeric value.
    Number { name: u32, value: u64 },
    /// A named UTF-8 text value.
    Bytes { name: u32, value: Text },
    /// A connector-owned numbered resource.
    Resource { name: u32, part: Part },
}

/// The settings revisions an agent task's runs read their charter from
/// (client/domain/contract/documents.md, 1; domain/engine/settings.md, 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TaskSettings {
    /// The revision the task records.
    pub task: u32,
    /// Its live run's, only while that run was prepared under another.
    pub run: Option<u32>,
    /// The newest revision's number.
    pub newest: u32,
}

/// Maximum UTF-8 prefix of directing words in a task document
/// (client/domain/contract/documents.md, 1).
pub const UNREAD_HEAD_BYTES: u32 = 256;

/// Directing words no run has recorded (client/domain/contract/documents.md, 1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct UnreadWords {
    pub message: u64,
    pub by: tasks::Party,
    pub at: Wall,
    pub seen: bool,
    pub head: Text,
    pub bytes: u32,
}

/// A task page's header, overview, funding, authority and current held decision.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Task {
    pub concluded: Option<tasks::Concluded>,
    pub talk: Option<tasks::Talk>,
    pub session: Option<tasks::Session>,
    /// None for a procedure's or a person's task.
    pub settings: Option<TaskSettings>,
    pub waits_on: super::WaitsOn,
    /// The line's first wait, which may be a descendant's.
    pub wait: Option<crate::Wait>,
    /// The earliest bound among the first wait and the task's own waits.
    pub nearest: Option<crate::Nearest>,
    pub waits: List<crate::Wait>,
    pub wait_count: u32,
    pub label: u32,
    pub flags: tasks::Flags,
    pub last_activity: Wall,
    /// When the task was made: its record's `created_at` (jig/docs/design/client/domain/contract/documents.md, 1).
    pub made: Wall,
    pub chip: Chip,
    pub acts: List<crate::acts::Offered>,
    pub project: u32,
    pub goal: Option<u64>,
    pub priority: Option<u32>,
    pub executor: tasks::Executor,
    pub requester: tasks::Party,
    pub lineage: List<Chip>,
    /// Every live direct delegate; identities are never omitted from a close decision.
    pub delegates: List<super::LiveDelegate>,
    pub changed: Wall,
    pub spec_revision: u64,
    /// Present only for a person executor; independent of spec and hold.
    pub person_revision: Option<u64>,
    /// The party holding a role-addressed task, if any.
    pub taken_by: Option<u64>,
    /// Last committed immutable activity position.
    pub activity: u64,
    /// Current specification source facts; list loss is independent of the page's cut.
    pub spec: super::SpecLine,
    pub words: Text,
    pub parameters: List<Parameter>,
    pub instructions: Option<Text>,
    /// An agent task's LLM choice as kept, by the core's indices; `None` takes its charter's
    /// default (domain/tasks/README.md, 3).
    pub llm: Option<tasks::LlmChoice>,
    /// The last successfully claimed main account.
    pub kept_account: Option<u32>,
    pub run_wait: Option<super::TaskRunWait>,
    pub resources: List<ResourceLine>,
    pub inputs: List<TaskLink>,
    pub dependencies: List<TaskLink>,
    pub dependency_facts: List<Dependency>,
    pub unread: List<UnreadWords>,
    pub references: List<TaskLink>,
    pub subscriptions: List<tasks::Subscription>,
    pub wake: tasks::WakePolicy,
    pub contract: ContractLine,
    pub result: Option<Ending>,
    pub budget: BudgetLine,
    pub authority: AuthorityLine,
    /// Tasks left over the subtree's life, excluding itself (jig/docs/design/client/domain/contract/documents.md, 1).
    pub tasks_left: u32,
    pub hold: Option<HoldPhaseLine>,
    pub proposals: List<ProposalRef>,
    /// At least one list omitted rows; individual text values state their cuts.
    pub cut: bool,
}

/// A visible live descendant and its immediate requester relationship.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TreeRow {
    pub chip: Chip,
    pub parent: u64,
    pub depth: u32,
    pub proposals: List<ProposalRef>,
    pub spent: u64,
    pub dependencies: List<TaskLink>,
    pub cut: bool,
}

/// Unshown descendants folded under their nearest shown ancestor; overflow joins the root.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FoldedCount {
    pub parent: u64,
    pub phases: PhaseCounts,
    pub held_below: u32,
}

/// A window on a task's live subtree; counts include rows outside the window.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Tree {
    pub root: Chip,
    pub rows: List<TreeRow>,
    pub folded: List<FoldedCount>,
    pub held_below: u32,
    pub next: Option<Cursor>,
}

pub fn part_path(limits: &Limits, connector: u16, kind: PartKind, bytes: &[Box<[u8]>]) -> Result<Part, Missing> {
    Ok(Part { connector, kind, key: PartKey::Path(path(limits, bytes)?) })
}

pub fn path(limits: &Limits, bytes: &[Box<[u8]>]) -> Result<List<Box<[u8]>>, Missing> {
    if bytes.len() > skein_lib::length::usize_of(limits.documents.segments) {
        return Err(Missing::TooLarge);
    }
    let mut result = List::with_capacity(limits.documents.segments);
    for segment in bytes {
        if segment.len() > skein_lib::length::usize_of(limits.documents.text_bytes) {
            return Err(Missing::TooLarge);
        }
        result.push(skein_lib::bytes::copy_of(segment)).expect("validated path length");
    }
    Ok(result)
}

pub fn result_value(limits: &Limits, value: &tasks::TaskResult) -> Result<ResultValue, Missing> {
    Ok(match value {
        tasks::TaskResult::Job { .. } => return Err(Missing::Unknown),
        tasks::TaskResult::Approved { state, words } => {
            if state.len() > skein_lib::length::usize_of(limits.documents.text_bytes) {
                return Err(Missing::TooLarge);
            }
            ResultValue::Approved {
                state: state.clone(),
                words: match words {
                    Some(words) => Some(super::small::text(words, limits.documents.text_bytes)),
                    None => None,
                },
            }
        }
        tasks::TaskResult::Report { words } | tasks::TaskResult::NoChange { words } => {
            ResultValue::Report(super::small::text(words, limits.documents.text_bytes))
        }
        tasks::TaskResult::Verdict { code, words, fields } => {
            let mut total = 0_usize;
            if fields.len() > bound(limits) {
                return Err(Missing::TooLarge);
            }
            for field in fields {
                total = total
                    .checked_add(field.name.len())
                    .ok_or(Missing::TooLarge)?
                    .checked_add(field.value.len())
                    .ok_or(Missing::TooLarge)?;
            }
            if total > skein_lib::length::usize_of(limits.documents.text_bytes) {
                return Err(Missing::TooLarge);
            }
            ResultValue::Verdict {
                code: *code,
                fields: fields.clone(),
                words: super::small::text(words, limits.documents.text_bytes),
            }
        }
        tasks::TaskResult::Change { connector, kind, resource, words, .. } => ResultValue::Change {
            kind: *kind,
            part: Part { connector: *connector, kind: PartKind::Result, key: PartKey::Number(*resource) },
            words: super::small::text(words, limits.documents.text_bytes),
        },
        tasks::TaskResult::Failure { reason } => {
            ResultValue::Failure(super::small::text(reason, limits.documents.text_bytes))
        }
    })
}

pub fn ending(limits: &Limits, value: &tasks::Ending) -> Result<Ending, Missing> {
    Ok(match value {
        tasks::Ending::Done(result) => Ending::Done(result_value(limits, result)?),
        tasks::Ending::Failed { reason } => Ending::Failed(super::small::text(reason, limits.documents.text_bytes)),
        tasks::Ending::Cancelled { reason, result, by, at } => Ending::Cancelled {
            by: *by,
            at: *at,
            reason: super::small::text(reason, limits.documents.text_bytes),
            result: match result {
                Some(result) => Some(result_value(limits, result)?),
                None => None,
            },
        },
    })
}

/// Retention keeps cut result words but no fields, state or job payload
/// (protocol/store/retention.md, 3 and 7).
pub fn ending_from_summary(limits: &Limits, summary: &tasks::TaskSummary) -> Ending {
    let reason = super::small::kept_text(limits, &summary.reason, summary.reason_cut);
    let result = match &summary.result {
        Some(result) => Some(summary_result(limits, result)),
        None => None,
    };
    match summary.ending {
        tasks::SummaryEnding::Done => Ending::Done(result.expect("done summary keeps its result")),
        tasks::SummaryEnding::Failed => Ending::Failed(reason),
        tasks::SummaryEnding::Cancelled { by, at } => Ending::Cancelled { reason, result, by, at },
    }
}

fn summary_result(limits: &Limits, result: &tasks::SummaryResult) -> ResultValue {
    let words = super::small::kept_text(limits, &result.words, result.words_cut);
    match result.kind {
        tasks::SummaryResultKind::Job | tasks::SummaryResultKind::Report | tasks::SummaryResultKind::NoChange => {
            ResultValue::Report(words)
        }
        tasks::SummaryResultKind::Approved => ResultValue::Approved { state: Box::new([]), words: Some(words) },
        tasks::SummaryResultKind::Verdict { code } => ResultValue::Verdict { code, fields: Box::new([]), words },
        tasks::SummaryResultKind::Change { connector, kind, resource } => ResultValue::Change {
            kind,
            part: Part { connector, kind: PartKind::Result, key: PartKey::Number(resource) },
            words,
        },
        tasks::SummaryResultKind::Failed => ResultValue::Failure(words),
    }
}

pub fn hold_source(limits: &Limits, source: Option<&tasks::HoldSource>) -> Result<Option<Part>, Missing> {
    match source {
        Some(tasks::HoldSource::Account { connector, entry, .. }) => Ok(match entry {
            Some(entry) => Some(Part { connector: *connector, kind: PartKind::Outbox, key: PartKey::Number(*entry) }),
            None => None,
        }),
        None
        | Some(tasks::HoldSource::Preparation { .. } | tasks::HoldSource::Circle(_) | tasks::HoldSource::Wait { .. }) => {
            Ok(None)
        }
        Some(tasks::HoldSource::Resource(name)) => {
            Ok(Some(part_path(limits, name.connector, PartKind::Resource, &name.path)?))
        }
        Some(tasks::HoldSource::Effect { connector, entry }) => {
            Ok(Some(Part { connector: *connector, kind: PartKind::Outbox, key: PartKey::Number(*entry) }))
        }
    }
}

// client/domain/root.md, section 6: the core names the hold account from retained origin facts.
fn hold_account(task: u64, reason: tasks::Hold, source: Option<&tasks::HoldSource>) -> Option<Part> {
    if let Some(tasks::HoldSource::Account { connector, .. }) = source {
        return Some(Part { connector: *connector, kind: PartKind::Hold, key: PartKey::Number(task) });
    }
    match reason {
        tasks::Hold::Drift | tasks::Hold::Effects | tasks::Hold::EffectFailed => {
            let connector = match source {
                Some(tasks::HoldSource::Resource(name)) => name.connector,
                Some(tasks::HoldSource::Effect { connector, .. } | tasks::HoldSource::Account { connector, .. }) => {
                    *connector
                }
                None
                | Some(
                    tasks::HoldSource::Preparation { .. }
                    | tasks::HoldSource::Circle(_)
                    | tasks::HoldSource::Wait { .. },
                ) => return None,
            };
            Some(Part { connector, kind: PartKind::Hold, key: PartKey::Number(task) })
        }
        tasks::Hold::Exceeded(_)
        | tasks::Hold::Failures(_)
        | tasks::Hold::Stalled
        | tasks::Hold::Dependency(_)
        | tasks::Hold::Stopped
        | tasks::Hold::Placed { .. }
        | tasks::Hold::Uncertain { .. }
        | tasks::Hold::Budget
        | tasks::Hold::Deadline
        | tasks::Hold::HoldsWaited
        | tasks::Hold::ResourceUnavailable
        | tasks::Hold::ResourceAccess
        | tasks::Hold::PoolLost { .. }
        | tasks::Hold::Circle
        | tasks::Hold::Waited
        | tasks::Hold::Procedure => None,
    }
}
pub fn hold(
    limits: &Limits,
    row: &tasks::TaskRecord,
    domain: Option<&tasks::Domain>,
) -> Result<Option<HoldPhaseLine>, Missing> {
    match &row.phase {
        tasks::Phase::Held { was, why, source } => Ok(Some(HoldPhaseLine {
            placed: None,
            words: {
                let words = tasks::hold_words(domain, row).expect("held task has words");
                Some(Text { bytes: words.bytes, cut: words.cut })
            },
            circle: match source.as_deref() {
                Some(tasks::HoldSource::Circle(circle)) => {
                    let mut tasks = List::with_capacity(limits.documents.lines);
                    for number in circle.tasks.iter().take(bound(limits)) {
                        tasks.push(*number).expect("bounded circle chips");
                    }
                    Some(tasks::Circle {
                        cut: circle.cut || circle.tasks.len() > bound(limits),
                        tasks: tasks.into_boxed(),
                    })
                }
                None
                | Some(
                    tasks::HoldSource::Resource(_)
                    | tasks::HoldSource::Effect { .. }
                    | tasks::HoldSource::Account { .. }
                    | tasks::HoldSource::Preparation { .. }
                    | tasks::HoldSource::Wait { .. },
                ) => None,
            },
            account: hold_account(row.number, *why, source.as_deref()),
            source: hold_source(limits, source.as_deref())?,
            reason: *why,
            revision: row.revisions.hold,
            before: match was {
                tasks::Was::Waiting => PriorPhase::Waiting,
                tasks::Was::Active(active) => PriorPhase::Active(*active),
                tasks::Was::Closing(closing) => PriorPhase::Closing(closing.stage),
            },
            escalation: match &row.escalation {
                tasks::Escalation::Unheld { .. } => None,
                tasks::Escalation::Routing { .. } => Some(EscalationState::Routing),
                tasks::Escalation::Waiting { holder, entry, since, .. } => {
                    Some(EscalationState::Waiting { holder: *holder, entry: *entry, since: *since })
                }
                tasks::Escalation::Rejected { by, reason, .. } => Some(EscalationState::Rejected {
                    by: *by,
                    reason: super::small::text(reason, limits.documents.text_bytes),
                }),
            },
            tries: row.tries,
            hold_wait_since: row.hold_wait_since,
        })),
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => Ok(None),
    }
}

pub fn budget(row: &tasks::TaskRecord) -> BudgetLine {
    budget_figures(row.numbers, row.funder, row.authority.budget.deadline)
}

pub fn budget_figures(numbers: tasks::Numbers, funder: tasks::Funder, deadline: Option<Wall>) -> BudgetLine {
    let left = numbers
        .budget
        .checked_sub(numbers.spent)
        .expect("spend within allotment")
        .checked_sub(numbers.spent_below)
        .expect("settled spend within allotment")
        .checked_sub(numbers.reserved)
        .expect("reservations within remaining allotment");
    BudgetLine { numbers, left, funder, deadline }
}

pub fn bound(limits: &Limits) -> usize {
    size(limits.documents.lines)
}
pub fn size(value: u32) -> usize {
    skein_lib::length::usize_of(value)
}

pub fn overview_bound(limits: &super::DocumentLimits) -> Option<u64> {
    let text = u64::from(limits.text_bytes);
    let lines = u64::from(limits.lines);
    let rows = u64::from(limits.rows);
    let chip = super::chip_phase::heap_bound(limits)?;
    let path = List::<Box<[u8]>>::price(limits.segments)?.checked_add(u64::from(limits.segments).checked_mul(text)?)?;
    let authority = authority_bound(limits)?;
    let references = List::<TaskLink>::price(limits.lines)?.checked_add(lines.checked_mul(chip)?)?;
    let specification = text
        .checked_add(List::<Parameter>::price(limits.lines)?)?
        .checked_add(lines.checked_mul(text)?)?
        .checked_add(List::<u64>::price(limits.lines)?)?;
    let task =
        u64::try_from(size_of::<Task>())
            .ok()?
            .checked_add(specification)?
            .checked_add(u64::try_from(tasks::HOLD_WORDS).ok()?)?
            .checked_add(text.checked_mul(8)?)?
            .checked_add(path.checked_mul(2)?)?
            .checked_add(chip)?
            .checked_add(List::<Chip>::price(limits.lines)?.checked_add(lines.checked_mul(chip)?)?.checked_mul(2)?)?
            .checked_add(List::<Parameter>::price(limits.lines)?.checked_add(lines.checked_mul(text)?)?)?
            .checked_add(List::<ResourceLine>::price(limits.lines)?.checked_add(lines.checked_mul(path)?)?)?
            .checked_add(references.checked_mul(3)?)?
            .checked_add(List::<tasks::Subscription>::price(limits.lines)?)?
            .checked_add(List::<tasks::Verdict>::price(limits.lines)?)?
            .checked_add(lines.checked_mul(
                lines.checked_mul(u64::try_from(size_of::<tasks::FieldRule>()).ok()?)?.checked_add(text)?,
            )?)?
            .checked_add(lines.checked_mul(u64::try_from(size_of::<tasks::ResultField>()).ok()?)?.checked_add(text)?)?
            .checked_add(List::<Dependency>::price(limits.lines)?)?
            .checked_add(List::<u64>::price(limits.lines)?)?
            .checked_add(32 * 8)?
            .checked_add(List::<ProposalRef>::price(limits.lines)?)?
            .checked_add(
                u64::from(tasks::MODELS.checked_mul(2)?)
                    .checked_mul(u64::try_from(size_of::<tasks::ModelEffort>()).ok()?)?,
            )?
            .checked_add(authority)?;
    let tree = chip
        .checked_add(List::<TreeRow>::price(limits.rows)?)?
        .checked_add(rows.checked_mul(chip.checked_add(references)?.checked_add(List::<ProposalRef>::price(1)?)?)?)?
        .checked_add(List::<FoldedCount>::price(limits.rows)?)?;
    Some(task.max(tree))
}

pub fn grant_bound(limits: &super::DocumentLimits) -> Option<u64> {
    List::<Box<[u8]>>::price(limits.segments)?
        .checked_add(u64::from(limits.segments).checked_add(1)?.checked_mul(u64::from(limits.text_bytes))?)
}
pub fn authority_bound(limits: &super::DocumentLimits) -> Option<u64> {
    List::<GrantLine>::price(limits.grants)?
        .checked_add(List::<NoteScope>::price(limits.grants)?)?
        .checked_add(u64::from(limits.grants).checked_mul(grant_bound(limits)?)?.checked_mul(2)?)?
        .checked_add(List::<tasks::AuthorityExecutor>::price(limits.executors)?)
}

pub fn document_bound(limits: &super::DocumentLimits) -> Option<u64> {
    overview_bound(limits)?.checked_add(super::offers::heap_bound(limits)?)
}

pub fn dependency_rows(limits: &Limits, row: &tasks::TaskRecord) -> List<Dependency> {
    let mut facts = List::with_capacity(limits.documents.lines);
    let keeps = match row.phase {
        tasks::Phase::Active(_) => true,
        tasks::Phase::Waiting | tasks::Phase::Held { .. } | tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => false,
    } && !row.waiting_on.is_empty();
    for number in row.dependencies.iter().take(bound(limits)) {
        let mut added = None;
        for value in &row.added {
            if value.task == *number {
                added = Some(value.clone());
                break;
            }
        }
        facts
            .push(Dependency {
                number: *number,
                pending: row.waiting_on.contains(number),
                added,
                keeps: keeps && row.waiting_on.contains(number),
                status: None,
            })
            .expect("bounded dependency facts");
    }
    facts
}

pub fn count_phase(counts: &mut PhaseCounts, phase: &tasks::Phase) {
    let count = match phase {
        tasks::Phase::Waiting => &mut counts.waiting,
        tasks::Phase::Active(active) => match active {
            tasks::Active::Idle => &mut counts.idle,
            tasks::Active::Due | tasks::Active::Preparing | tasks::Active::Claimed { .. } => &mut counts.starting,
            tasks::Active::Running { .. } => &mut counts.running,
            tasks::Active::BackingOff { .. } => &mut counts.backing_off,
        },
        tasks::Phase::Closing(_) => &mut counts.closing,
        tasks::Phase::Held { .. } => &mut counts.held,
        tasks::Phase::Ended(ending) => match ending {
            tasks::Ending::Done(_) => &mut counts.done,
            tasks::Ending::Failed { .. } => &mut counts.failed,
            tasks::Ending::Cancelled { .. } => &mut counts.cancelled,
        },
    };
    *count = count.checked_add(1).expect("census bounded by task limits");
}
