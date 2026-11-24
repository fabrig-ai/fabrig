//! Identifier-only records (jig/docs/design/protocol/client/public-reads.md).
use super::{Names, PublicKind};
use crate::generated::public as map;
use alloc::boxed::Box;
use jig_client_protocol_wire::Record as _;
use jig_client_protocol_wire::{self as wire, public_v1 as w};
use jig_domain_core_vocabulary as c;
use skein_lib::{List, Writer};

pub(crate) fn encode(
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
                    .push(w::PublicProject {
                        project: names.project(item.project),
                        name: skein_lib::bytes::copy_of(item.name.as_bytes()),
                    })
                    .ok()?;
            }
            let record = w::PublicProjects { projects, max_age };
            let mut out = writer(record.measure(limits).ok()?, bound)?;
            record.write(limits, &mut out).ok()?;
            Some(out.finish())
        }
        c::PublicDocument::Board(value) => {
            if kind.owner != 0 || kind.kind != wire::PUBLIC_BOARD {
                return None;
            }
            let record = w::PublicBoard {
                project: names.project(value.project),
                name: skein_lib::bytes::copy_of(value.name.as_bytes()),
                live: goals(&value.live, names, limits, limits.public_board_live)?,
                live_folded: value.live_folded,
                ended: goals(&value.ended, names, limits, limits.public_board_ended)?,
            };
            let mut out = writer(record.measure(limits).ok()?, bound)?;
            record.write(limits, &mut out).ok()?;
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
            let record = w::PublicCard {
                line: public_line_to_record(&value.line, &(names, limits.public_line_title)).ok()?,
                lineage,
                delegates: lines(&value.delegates, names, limits, limits.public_card_delegates)?,
                folded: value.folded,
            };
            let mut out = writer(record.measure(limits).ok()?, bound)?;
            record.write(limits, &mut out).ok()?;
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
            let record = w::PublicApplication { tasks, record: value.record.clone() };
            let mut out = writer(record.measure(limits).ok()?, bound)?;
            record.write(limits, &mut out).ok()?;
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
    let context = (names, limits.public_line_title);
    for value in values {
        // Check the configured count before translation allocates the tree.
        count(value.tree.len(), limits.public_goal_tree)?;
        goals.push(map::public_goal_to_record(value, &context).ok()?).ok()?;
    }
    Some(goals)
}
fn lines(values: &[c::PublicLine], names: &Names, limits: &w::Limits, bound: u32) -> Option<List<w::PublicLine>> {
    let mut lines = List::with_capacity(count(values.len(), bound)?);
    let context = (names, limits.public_line_title);
    for value in values {
        lines.push(public_line_to_record(value, &context).ok()?).ok()?;
    }
    Some(lines)
}
pub(crate) fn public_line_to_record(
    value: &c::PublicLine,
    context: &(&Names, u32),
) -> Result<w::PublicLine, w::Problem> {
    let &(names, title_limit) = context;
    if !skein_lib::bytes::is_utf8(&value.title.bytes) {
        return Err(w::paths::PUBLIC_LINE_TITLE.problem(skein_codec::Reason::Utf8));
    }
    let title = skein_lib::bytes::utf8_prefix(&value.title.bytes, skein_lib::length::usize_of(title_limit));
    let title_cut = value.title.cut || title.len() < value.title.bytes.len();
    let title = skein_lib::bytes::copy_of(title);
    Ok(w::PublicLine {
        task: names.task(value.task),
        title,
        title_cut,
        parent: match value.parent {
            Some(n) => Some(names.task(n)),
            None => None,
        },
        goal: value.goal,
        priority: value.priority,
        executor: map::public_executor_to_record(&value.executor),
        phase: map::public_phase_to_record(&value.phase),
        hold: match value.hold {
            Some(hold) => Some(map::public_hold_to_record(&hold)),
            None => None,
        },
        made: value.made,
        changed: value.changed,
        active: value.active,
        ended: value.ended,
        spent: value.spent,
        delegates: map::public_counts_to_record(&value.delegates),
    })
}
