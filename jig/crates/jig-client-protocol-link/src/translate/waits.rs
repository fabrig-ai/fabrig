//! Read the engine's structured waits as supplied (jig/docs/design/client/domain/contract/documents.md, 1).
use super::{Limits, facts, task_wait, w};
use jig_client_domain::contract::{self as c, waits::Bound};
use skein_lib::{List, Wall};

pub(super) fn first(value: Option<&w::Wait>, limits: &Limits) -> Result<Option<c::Wait>, w::Problem> {
    match value {
        Some(value) => Ok(Some(wait(value, limits)?)),
        None => Ok(None),
    }
}

pub(super) fn nearest(value: Option<&w::Nearest>) -> Option<c::Nearest> {
    let value = value?;
    Some(c::Nearest { at: Wall::from_nanos(value.at), then: at_bound(&value.then), kind: kind(&value.kind) })
}

pub(super) fn list(value: &List<w::Wait>, limits: &Limits) -> Result<List<c::Wait>, w::Problem> {
    let mut waits = List::with_capacity(value.len());
    for row in value {
        waits.push(wait(row, limits)?).expect("supplied waits fit");
    }
    Ok(waits)
}

fn wait(value: &w::Wait, limits: &Limits) -> Result<c::Wait, w::Problem> {
    Ok(c::Wait {
        task: value.task,
        kind: kind(&value.kind),
        object: object(&value.object, limits)?,
        since: Wall::from_nanos(value.since),
        bound: bound(&value.bound),
        hears: match &value.hears {
            Some(holder) => Some(facts::escalation_holder(holder)),
            None => None,
        },
    })
}

fn kind(value: &w::WaitKind) -> c::WaitKind {
    match value {
        w::WaitKind::Party => c::WaitKind::Party,
        w::WaitKind::Question => c::WaitKind::Question,
        w::WaitKind::Dependency => c::WaitKind::Dependency,
        w::WaitKind::Resource => c::WaitKind::Resource,
        w::WaitKind::Account => c::WaitKind::Account,
        w::WaitKind::Host => c::WaitKind::Host,
        w::WaitKind::Delegate => c::WaitKind::Delegate,
        w::WaitKind::News => c::WaitKind::News,
        w::WaitKind::Step => c::WaitKind::Step,
        w::WaitKind::Room => c::WaitKind::Room,
        w::WaitKind::Turn => c::WaitKind::Turn,
        w::WaitKind::SignOff => c::WaitKind::SignOff,
    }
}

fn object(value: &w::WaitObject, limits: &Limits) -> Result<c::WaitObject, w::Problem> {
    Ok(match value {
        w::WaitObject::Entry(row) => c::WaitObject::Entry {
            kind: super::queries::waited_for(&row.kind),
            number: row.number,
            at: facts::escalation_holder(&row.at),
        },
        w::WaitObject::Question(row) => c::WaitObject::Question { number: row.number, answerer: row.answerer },
        w::WaitObject::Task(row) => c::WaitObject::Task(row.value),
        w::WaitObject::Resource(row) => {
            if !c::resource_path_fits(&row.name.path, limits.client().segments, limits.client().text) {
                return Err(super::too_large());
            }
            c::WaitObject::Resource {
                name: c::ResourceName { connector: row.name.connector, path: row.name.path.clone() },
                holder: row.holder,
            }
        }
        w::WaitObject::Run(row) => c::WaitObject::Run(task_wait::run(&row.value)),
        w::WaitObject::News(row) => c::WaitObject::News {
            subscriptions: row.subscriptions.clone(),
            timer: match row.timer {
                Some(at) => Some(Wall::from_nanos(at)),
                None => None,
            },
        },
        w::WaitObject::Procedure(row) => {
            let mut things = List::with_capacity(row.things.len());
            for thing in &row.things {
                things
                    .push(c::Thing { what: thing.what, number: thing.number, owner: thing.owner })
                    .expect("supplied things fit");
            }
            c::WaitObject::Procedure { step: row.step, things }
        }
    })
}

fn bound(value: &w::WaitBound) -> Bound {
    match value {
        w::WaitBound::At(row) => Bound::At { at: Wall::from_nanos(row.at), then: at_bound(&row.then) },
        w::WaitBound::None(row) => Bound::None(match &row.reason {
            w::Unbounded::NoDeadline => c::Unbounded::NoDeadline,
            w::Unbounded::Standing => c::Unbounded::Standing,
            w::Unbounded::Behind(row) => c::Unbounded::Behind(row.task),
            w::Unbounded::OperatorHold => c::Unbounded::OperatorHold,
        }),
    }
}

fn at_bound(value: &w::AtBound) -> c::AtBound {
    match value {
        w::AtBound::Hold => c::AtBound::Hold,
        w::AtBound::PassUp => c::AtBound::PassUp,
        w::AtBound::Overdue => c::AtBound::Overdue,
        w::AtBound::GiveUp => c::AtBound::GiveUp,
    }
}
