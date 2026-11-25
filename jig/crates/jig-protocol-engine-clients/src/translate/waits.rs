//! Carry the core's wait records without reordering or recomputing them (jig/docs/design/domain/engine/waits.md, 4).
use super::{chips, task_wait, w};
use crate::generated::client as mapped;
use jig_domain_core_vocabulary::waits as d;
use jig_domain_people::EntryKind;
use skein_lib::List;

pub(super) fn first(value: Option<&d::Wait>) -> Result<Option<w::Wait>, w::Problem> {
    match value {
        Some(value) => Ok(Some(wait(value)?)),
        None => Ok(None),
    }
}

pub(super) fn nearest(value: Option<d::Nearest>) -> Option<w::Nearest> {
    Some(mapped::nearest_to_record(&value?))
}

pub(super) fn list(value: &List<d::Wait>) -> Result<List<w::Wait>, w::Problem> {
    let mut waits = List::with_capacity(value.len());
    for row in value {
        waits.push(wait(row)?).expect("supplied waits fit");
    }
    Ok(waits)
}

fn wait(value: &d::Wait) -> Result<w::Wait, w::Problem> {
    Ok(w::Wait {
        task: value.task,
        kind: mapped::wait_kind_to_record(&value.kind),
        object: object(&value.object, value.task)?,
        since: value.since.as_nanos(),
        bound: mapped::wait_bound_to_record(&value.bound),
        hears: match value.hears {
            Some(holder) => Some(chips::holder(holder)),
            None => None,
        },
    })
}

fn object(value: &d::WaitObject, task: u64) -> Result<w::WaitObject, w::Problem> {
    Ok(match value {
        d::WaitObject::Entry { kind, at } => {
            let (kind, number) = match *kind {
                EntryKind::Question { message } => (w::WaitedFor::Question, message),
                EntryKind::Proposal { number, .. } => (w::WaitedFor::Proposal, number),
                EntryKind::Escalation { revision } => (w::WaitedFor::Escalation, revision),
                EntryKind::PersonTask => (w::WaitedFor::PersonTask, task),
                EntryKind::Reply { .. } | EntryKind::ChatReply { .. } => {
                    return Err(w::Problem { path: w::paths::WAIT_ENTRY_KIND, reason: skein_codec::Reason::Tag });
                }
            };
            w::WaitObject::Entry(w::WaitEntry { kind, number, at: chips::holder(*at) })
        }
        d::WaitObject::Question { number, answerer } => {
            w::WaitObject::Question(w::WaitQuestion { number: *number, answerer: *answerer })
        }
        d::WaitObject::Task(value) => w::WaitObject::Task(w::WaitTask { value: *value }),
        d::WaitObject::Resource { name, holder } => {
            // A path count past u32 is refused at the enclosing record.
            let name = match mapped::wait_resource_name_to_record(name) {
                Ok(name) => name,
                Err(problem) => return Err(super::problem(problem.reason)),
            };
            w::WaitObject::Resource(w::WaitResource { name, holder: *holder })
        }
        d::WaitObject::Run(value) => w::WaitObject::Run(w::WaitRun { value: task_wait::run(*value) }),
        d::WaitObject::News { subscriptions, timer } => w::WaitObject::News(w::WaitNews {
            subscriptions: subscriptions.clone(),
            timer: match timer {
                Some(at) => Some(at.as_nanos()),
                None => None,
            },
        }),
        d::WaitObject::Procedure { step, things } => {
            let mut rows = List::with_capacity(things.len());
            for row in things {
                rows.push(mapped::thing_to_record(row)).expect("supplied things fit");
            }
            w::WaitObject::Procedure(w::WaitProcedure { step: *step, things: rows })
        }
    })
}
