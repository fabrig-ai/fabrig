//! Carry the core's wait records without reordering or recomputing them (jig/docs/design/domain/engine/waits.md, 4).
use super::{chips, task_wait, w};
use jig_domain_core_vocabulary::waits as d;
use jig_domain_people::EntryKind;
use skein_lib::{List, bytes};

pub(super) fn first(value: Option<&d::Wait>) -> Result<Option<w::Wait>, w::Problem> {
    match value {
        Some(value) => Ok(Some(wait(value)?)),
        None => Ok(None),
    }
}

pub(super) fn nearest(value: Option<d::Nearest>) -> Option<w::Nearest> {
    let value = value?;
    Some(w::Nearest { at: value.at.as_nanos(), then: at_bound(value.then), kind: kind(value.kind) })
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
        kind: kind(value.kind),
        object: object(&value.object, value.task)?,
        since: value.since.as_nanos(),
        bound: bound(value.bound),
        hears: match value.hears {
            Some(holder) => Some(chips::holder(holder)),
            None => None,
        },
    })
}

fn kind(value: d::WaitKind) -> w::WaitKind {
    match value {
        d::WaitKind::Party => w::WaitKind::Party,
        d::WaitKind::Question => w::WaitKind::Question,
        d::WaitKind::Dependency => w::WaitKind::Dependency,
        d::WaitKind::Resource => w::WaitKind::Resource,
        d::WaitKind::Account => w::WaitKind::Account,
        d::WaitKind::Host => w::WaitKind::Host,
        d::WaitKind::Delegate => w::WaitKind::Delegate,
        d::WaitKind::News => w::WaitKind::News,
        d::WaitKind::Step => w::WaitKind::Step,
        d::WaitKind::Room => w::WaitKind::Room,
        d::WaitKind::Turn => w::WaitKind::Turn,
        d::WaitKind::SignOff => w::WaitKind::SignOff,
    }
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
            let mut path = List::with_capacity(super::count(name.path.len())?);
            for segment in &name.path {
                path.push(bytes::copy_of(segment)).expect("supplied path fits");
            }
            w::WaitObject::Resource(w::WaitResource {
                name: w::WaitResourceName { connector: name.connector, path },
                holder: *holder,
            })
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
                rows.push(w::Thing { what: row.what, number: row.number, owner: row.owner })
                    .expect("supplied things fit");
            }
            w::WaitObject::Procedure(w::WaitProcedure { step: *step, things: rows })
        }
    })
}

fn bound(value: d::Bound) -> w::WaitBound {
    match value {
        d::Bound::At { at, then } => w::WaitBound::At(w::WaitAtBound { at: at.as_nanos(), then: at_bound(then) }),
        d::Bound::None(why) => w::WaitBound::None(w::WaitNoBound {
            reason: match why {
                d::Unbounded::NoDeadline => w::Unbounded::NoDeadline,
                d::Unbounded::Standing => w::Unbounded::Standing,
                d::Unbounded::Behind(task) => w::Unbounded::Behind(w::UnboundedBehind { task }),
                d::Unbounded::OperatorHold => w::Unbounded::OperatorHold,
            },
        }),
    }
}

fn at_bound(value: d::AtBound) -> w::AtBound {
    match value {
        d::AtBound::Hold => w::AtBound::Hold,
        d::AtBound::PassUp => w::AtBound::PassUp,
        d::AtBound::Overdue => w::AtBound::Overdue,
        d::AtBound::GiveUp => w::AtBound::GiveUp,
    }
}
