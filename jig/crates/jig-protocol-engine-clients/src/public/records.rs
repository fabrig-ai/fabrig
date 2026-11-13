//! Identifier-only records (jig/docs/design/protocol/client/public-reads.md).
use super::{Names, PublicKind};
use alloc::boxed::Box;
use jig_client_protocol_wire::{self as wire, public_v1 as w};
use jig_domain_core_vocabulary as c;
use skein_lib::{List, Writer};

pub(super) fn encode(
    document: &c::PublicDocument,
    names: &Names,
    limits: &w::Limits,
    max_age: u32,
    kind: PublicKind,
    answer_bytes: u32,
) -> Option<Box<[u8]>> {
    let bound = kind.bound.min(answer_bytes);
    match document {
        c::PublicDocument::Projects(value) => {
            if kind.owner != 0 || kind.kind != wire::PUBLIC_PROJECTS {
                return None;
            }
            let mut projects = List::with_capacity(count(value.projects.len(), limits.public_projects_projects)?);
            for item in &value.projects {
                projects
                    .push(
                        w::PublicProject::new(
                            limits,
                            w::PublicProjectParts {
                                project: names.project(item.project),
                                name: skein_lib::bytes::copy_of(item.name.as_bytes()),
                            },
                        )
                        .ok()?,
                    )
                    .ok()?;
            }
            let record = w::PublicProjects::new(limits, w::PublicProjectsParts { projects, max_age }).ok()?;
            let mut out = writer(record.measure(), bound)?;
            record.encode(&mut out).ok()?;
            Some(out.finish())
        }
        c::PublicDocument::Board(value) => {
            if kind.owner != 0 || kind.kind != wire::PUBLIC_BOARD {
                return None;
            }
            let record = w::PublicBoard::new(
                limits,
                w::PublicBoardParts {
                    project: names.project(value.project),
                    name: skein_lib::bytes::copy_of(value.name.as_bytes()),
                    live: goals(&value.live, names, limits, limits.public_board_live)?,
                    live_folded: value.live_folded,
                    ended: goals(&value.ended, names, limits, limits.public_board_ended)?,
                },
            )
            .ok()?;
            let mut out = writer(record.measure(), bound)?;
            record.encode(&mut out).ok()?;
            Some(out.finish())
        }
        c::PublicDocument::Card(value) => {
            if kind.owner != 0 || kind.kind != wire::PUBLIC_CARD {
                return None;
            }
            let mut lineage = List::with_capacity(count(value.lineage.len(), limits.public_card_lineage)?);
            for item in &value.lineage {
                lineage.push(names.task(*item)).ok()?;
            }
            let record = w::PublicCard::new(
                limits,
                w::PublicCardParts {
                    line: line(&value.line, names, limits)?,
                    lineage,
                    delegates: lines(&value.delegates, names, limits, limits.public_card_delegates)?,
                    folded: value.folded,
                },
            )
            .ok()?;
            let mut out = writer(record.measure(), bound)?;
            record.encode(&mut out).ok()?;
            Some(out.finish())
        }
        c::PublicDocument::Application(value) => {
            if kind.owner == 0 {
                return None;
            }
            let size = value.tasks.len().checked_mul(8)?.checked_add(8)?.checked_add(value.record.len())?;
            if size > skein_lib::length::usize_of(bound) {
                return None;
            }
            let mut tasks = List::with_capacity(count(value.tasks.len(), limits.public_application_tasks)?);
            for item in &value.tasks {
                tasks.push(names.task(*item)).ok()?;
            }
            let record =
                w::PublicApplication::new(limits, w::PublicApplicationParts { tasks, record: value.record.clone() })
                    .ok()?;
            let mut out = writer(record.measure(), bound)?;
            record.encode(&mut out).ok()?;
            Some(out.finish())
        }
    }
}
fn count(length: usize, bound: u32) -> Option<u32> {
    let count = u32::try_from(length).ok()?;
    if count > bound { None } else { Some(count) }
}
fn writer(measure: u32, bound: u32) -> Option<Writer> {
    if measure > bound { None } else { Some(Writer::new(skein_lib::length::usize_of(measure))) }
}
fn goals(values: &[c::PublicGoal], names: &Names, limits: &w::Limits, bound: u32) -> Option<List<w::PublicGoal>> {
    let mut goals = List::with_capacity(count(values.len(), bound)?);
    for value in values {
        goals
            .push(
                w::PublicGoal::new(
                    limits,
                    w::PublicGoalParts {
                        line: line(&value.line, names, limits)?,
                        tree: lines(&value.tree, names, limits, limits.public_goal_tree)?,
                        folded: value.folded,
                    },
                )
                .ok()?,
            )
            .ok()?;
    }
    Some(goals)
}
fn lines(values: &[c::PublicLine], names: &Names, limits: &w::Limits, bound: u32) -> Option<List<w::PublicLine>> {
    let mut lines = List::with_capacity(count(values.len(), bound)?);
    for value in values {
        lines.push(line(value, names, limits)?).ok()?;
    }
    Some(lines)
}
fn line(value: &c::PublicLine, names: &Names, limits: &w::Limits) -> Option<w::PublicLine> {
    if !skein_lib::bytes::is_utf8(&value.title.bytes) {
        return None;
    }
    let title =
        skein_lib::bytes::utf8_prefix(&value.title.bytes, skein_lib::length::usize_of(limits.public_line_title));
    let title_cut = value.title.cut || title.len() < value.title.bytes.len();
    let title = skein_lib::bytes::copy_of(title);
    w::PublicLine::new(
        limits,
        w::PublicLineParts {
            task: names.task(value.task),
            title,
            title_cut,
            parent: match value.parent {
                Some(n) => Some(names.task(n)),
                None => None,
            },
            goal: value.goal,
            priority: value.priority,
            executor: executor(value.executor),
            phase: phase(value.phase),
            hold: match value.hold {
                Some(hold) => Some(hold_kind(hold)),
                None => None,
            },
            made: value.made,
            changed: value.changed,
            active: value.active,
            ended: value.ended,
            spent: value.spent,
            delegates: w::PublicCounts::new(
                limits,
                w::PublicCountsParts {
                    working: value.delegates.working,
                    waiting: value.delegates.waiting,
                    held: value.delegates.held,
                    closing: value.delegates.closing,
                    ended: value.delegates.ended,
                },
            )
            .ok()?,
        },
    )
    .ok()
}
fn executor(value: c::PublicExecutor) -> w::PublicExecutor {
    match value {
        c::PublicExecutor::Agent => w::PublicExecutor::Agent,
        c::PublicExecutor::Procedure => w::PublicExecutor::Procedure,
        c::PublicExecutor::Person => w::PublicExecutor::Person,
        c::PublicExecutor::Job => w::PublicExecutor::Job,
    }
}
fn phase(value: c::PublicPhase) -> w::PublicPhase {
    match value {
        c::PublicPhase::Waiting => w::PublicPhase::Waiting,
        c::PublicPhase::Idle => w::PublicPhase::Idle,
        c::PublicPhase::Due => w::PublicPhase::Due,
        c::PublicPhase::Preparing => w::PublicPhase::Preparing,
        c::PublicPhase::Claimed => w::PublicPhase::Claimed,
        c::PublicPhase::Running => w::PublicPhase::Running,
        c::PublicPhase::BackingOff => w::PublicPhase::BackingOff,
        c::PublicPhase::Active => w::PublicPhase::Active,
        c::PublicPhase::Closing => w::PublicPhase::Closing,
        c::PublicPhase::Held => w::PublicPhase::Held,
        c::PublicPhase::Done => w::PublicPhase::Done,
        c::PublicPhase::Failed => w::PublicPhase::Failed,
        c::PublicPhase::Cancelled => w::PublicPhase::Cancelled,
    }
}
fn hold_kind(value: c::PublicHold) -> w::PublicHold {
    match value {
        c::PublicHold::Exceeded => w::PublicHold::Exceeded,
        c::PublicHold::Failures => w::PublicHold::Failures,
        c::PublicHold::Stalled => w::PublicHold::Stalled,
        c::PublicHold::Dependency => w::PublicHold::Dependency,
        c::PublicHold::Stopped => w::PublicHold::Stopped,
        c::PublicHold::Placed => w::PublicHold::Placed,
        c::PublicHold::Drift => w::PublicHold::Drift,
        c::PublicHold::Effects => w::PublicHold::Effects,
        c::PublicHold::EffectFailed => w::PublicHold::EffectFailed,
        c::PublicHold::Uncertain => w::PublicHold::Uncertain,
        c::PublicHold::Budget => w::PublicHold::Budget,
        c::PublicHold::Deadline => w::PublicHold::Deadline,
        c::PublicHold::HoldsWaited => w::PublicHold::HoldsWaited,
        c::PublicHold::Circle => w::PublicHold::Circle,
        c::PublicHold::ResourceUnavailable => w::PublicHold::ResourceUnavailable,
        c::PublicHold::ResourceAccess => w::PublicHold::ResourceAccess,
        c::PublicHold::PoolLost => w::PublicHold::PoolLost,
        c::PublicHold::Procedure => w::PublicHold::Procedure,
    }
}
