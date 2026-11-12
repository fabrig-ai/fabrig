//! Nonrecursive chip facts (`client/domain/objects.md`, 3 and 5). Durable
//! lifecycle clocks remain distinct from best-effort engine receipt clocks.

use super::{BudgetLine, HoldLine, Missing, Part, PartKey, PartKind, ResourceLine};
use crate::Limits;
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{List, Token, Wall};

/// Latest whole activity name, without transport or provisional content.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ChipWork {
    Preparing,
    Starting,
    Active,
    Writing,
    Tool { call: Box<[u8]>, tool: Box<[u8]> },
    Waiting,
    Ending,
}

/// Fenced host observation; all walls are engine receipt times.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ChipObservation {
    pub channel: Token,
    pub work: ChipWork,
    pub since: Wall,
    pub reported_at: Wall,
    pub started: Option<Wall>,
}

/// Facts needed beside every chip, without nesting other chips or histories.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ChipDetails {
    pub dependencies: List<super::task_tree::Dependency>,
    /// Owner context for a role address, including in a standalone retained chip.
    pub project: u32,
    /// Concrete owner identity; the chip's coarse executor kind remains separate.
    pub executor: tasks::Executor,
    pub phase_since: Wall,
    /// Placement/adoption, never inferred to be actual agent start.
    pub run_since: Option<Wall>,
    pub attempt: Option<u64>,
    pub retry: Option<tasks::RetryNotice>,
    pub wake: tasks::WakePolicy,
    pub person_wait: Option<tasks::PersonWait>,
    pub hold: Option<HoldLine>,
    pub budget: BudgetLine,
    /// Exact immutable activity tail for retained failure/result explanations.
    pub activity: u64,
    pub resources: List<ResourceLine>,
    pub resources_cut: bool,
    /// False for a row-only read: missing holder/writer is then unknown.
    pub resources_observed: bool,
    pub procedure: Option<Part>,
    pub live_delegates: u32,
    /// Connectors the task waits on (domain/engine/views.md, 3), unknown in an unjoined stored closing row.
    pub closing_connectors: Option<List<u16>>,
    pub ending: Option<super::Ending>,
    pub ended_at: Option<Wall>,
    pub observation: Option<ChipObservation>,
}

pub fn from_row(limits: &Limits, row: &tasks::TaskRecord) -> Result<ChipDetails, Missing> {
    let needs_resources = match row.phase {
        tasks::Phase::Waiting | tasks::Phase::Held { .. } => true,
        tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => false,
    };
    let mut resources = List::with_capacity(if needs_resources { limits.documents.lines } else { 0 });
    if needs_resources {
        for holding in row.holdings.iter().take(skein_lib::length::usize_of(limits.documents.lines)) {
            let (name, kind, part_kind) = match holding {
                tasks::Holding::Write { resource, kind } => (resource, *kind, PartKind::Resource),
                tasks::Holding::Slot { pool, kind } => (pool, *kind, PartKind::Pool),
            };
            resources
                .push(ResourceLine {
                    part: super::task_tree::part_path(limits, name.connector, part_kind, &name.path)?,
                    kind,
                    taken: row.holds_taken,
                    holder: None,
                    writer: None,
                    writer_lost: false,
                })
                .expect("bounded resource facts");
        }
    }
    let ending = match &row.phase {
        tasks::Phase::Closing(closing) => Some(super::task_tree::ending(limits, &closing.ending)?),
        tasks::Phase::Ended(ending) => Some(super::task_tree::ending(limits, ending)?),
        tasks::Phase::Held { was, .. } => match was {
            tasks::Was::Closing(closing) => Some(super::task_tree::ending(limits, &closing.ending)?),
            tasks::Was::Waiting | tasks::Was::Active(_) => None,
        },
        tasks::Phase::Waiting | tasks::Phase::Active(_) => None,
    };
    Ok(ChipDetails {
        dependencies: super::task_tree::dependency_rows(limits, row),
        project: row.project,
        executor: row.executor,
        phase_since: row.phase_since,
        run_since: row.run_since,
        attempt: if row.attempt == 0 { None } else { Some(row.attempt) },
        retry: row.retry_notice.clone(),
        wake: row.wake,
        person_wait: row.person_wait,
        hold: super::task_tree::hold(limits, row)?,
        budget: super::task_tree::budget(row),
        activity: row.activity,
        resources_cut: needs_resources && skein_lib::length::usize_of(resources.len()) < row.holdings.len(),
        resources,
        resources_observed: false,
        procedure: match row.executor {
            tasks::Executor::Procedure { connector, code } => {
                Some(Part { connector, kind: PartKind::Procedure, key: PartKey::Number(u64::from(code)) })
            }
            tasks::Executor::Agent { .. } | tasks::Executor::Person(_) | tasks::Executor::Job { .. } => None,
        },
        live_delegates: u32::try_from(row.delegates.len()).expect("bounded direct delegates"),
        closing_connectors: None,
        ending,
        ended_at: row.ended_at,
        observation: None,
    })
}

pub fn heap_bound(limits: &super::DocumentLimits) -> Option<u64> {
    let text = u64::from(limits.text_bytes);
    let lines = u64::from(limits.lines);
    let path = List::<Box<[u8]>>::price(limits.lines)?.checked_add(lines.checked_mul(text)?)?;
    u64::try_from(size_of::<ChipDetails>())
        .ok()?
        .checked_add(u64::from(tasks::HOLD_ACCOUNT_BYTES))?
        .checked_add(text.checked_mul(8)?)?
        .checked_add(List::<u64>::price(limits.lines)?)?
        .checked_add(List::<u16>::price(limits.lines)?)?
        .checked_add(List::<super::task_tree::Dependency>::price(limits.lines)?)?
        .checked_add(32 * 8)?
        .checked_add(List::<ResourceLine>::price(limits.lines)?)?
        .checked_add(lines.checked_mul(path)?)?
        .checked_add(path)
}
