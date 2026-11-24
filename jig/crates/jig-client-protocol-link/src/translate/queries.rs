//! The fixed task query and the task owner's whole lines.
use super::{documents, facts, w};
use crate::Limits;
use crate::generated::client;
use jig_client_domain::contract as c;
use skein_lib::Wall;

pub(super) fn query(value: &c::TaskQuery) -> w::TaskQuery {
    client::task_query_to_record(value)
}
pub(super) fn flags(value: &w::TaskFlags) -> c::TaskFlags {
    client::task_flags_to_domain(value.clone())
}
pub(super) fn waits(value: &w::WaitsOn) -> c::WaitsOn {
    match value {
        w::WaitsOn::Party(v) => c::WaitsOn::Party {
            whom: match &v.whom {
                w::Whom::Person(v) => c::Whom::Person(v.value),
                w::Whom::Role(v) => c::Whom::Role { project: v.project, role: v.role },
            },
            entries: v.entries,
            task: v.task,
            kind: waited_for(&v.kind),
        },
        w::WaitsOn::Dependencies(value) => {
            c::WaitsOn::Dependencies { pending: value.pending.clone(), count: value.count }
        }
        w::WaitsOn::Delegates => c::WaitsOn::Delegates,
        w::WaitsOn::Question(value) => c::WaitsOn::Question { number: value.number },
        w::WaitsOn::News => c::WaitsOn::News,
        w::WaitsOn::Nothing => c::WaitsOn::Nothing,
    }
}
pub(super) fn waited_for(kind: &w::WaitedFor) -> c::WaitedFor {
    match kind {
        w::WaitedFor::Question => c::WaitedFor::Question,
        w::WaitedFor::Proposal => c::WaitedFor::Proposal,
        w::WaitedFor::Escalation => c::WaitedFor::Escalation,
        w::WaitedFor::PersonTask => c::WaitedFor::PersonTask,
    }
}

pub fn line(value: &w::TaskLine, limits: &Limits) -> Result<c::TaskLine, w::Problem> {
    Ok(c::TaskLine {
        parts: row_parts(&value.parts)?,
        chip: documents::chip_fields(&value.chip, limits)?,
        project: value.project,
        requester: facts::requester(&value.requester),
        tracked: value.tracked,
        label: value.label,
        flags: flags(&value.flags),
        changed: Wall::from_nanos(value.changed),
        last_activity: Wall::from_nanos(value.last_activity),
        ended_at: match value.ended_at {
            None => None,
            Some(v) => Some(Wall::from_nanos(v)),
        },
        budget: facts::budget(&value.budget),
        waits_on: waits(&value.waits_on),
        wait: super::waits::first(value.wait.as_ref(), limits)?,
        nearest: super::waits::nearest(value.nearest.as_ref()),
        phases: documents::phase_counts_fields(&value.phases),
        delegates_done: value.delegates_done,
        delegates_total: value.delegates_total,
        held_below: value.held_below,
        conversation: match &value.conversation {
            None => None,
            Some(v) => Some(c::ConversationRead { seen: v.seen, unread: documents::count_fields(&v.unread) }),
        },
    })
}

fn row_parts(value: &skein_lib::List<w::RowPart>) -> Result<skein_lib::List<c::RowPart>, w::Problem> {
    let mut parts = skein_lib::List::with_capacity(value.len());
    for part in value {
        let part = match part {
            w::RowPart::Value(value) => {
                let w::Owner::Family(owner) = &value.owner else {
                    return Err(super::piece_problem(skein_codec::Reason::Tag));
                };
                c::RowPart::Value(jig_client_domain::Family {
                    family: owner.number,
                    version: value.version,
                    bytes: skein_lib::bytes::copy_of(&value.record),
                })
            }
            w::RowPart::NotServed(why) => c::RowPart::NotServed(not_served(why)),
            w::RowPart::Unknown => c::RowPart::Unknown,
            w::RowPart::Failed(why) => c::RowPart::Failed(match &why.why {
                w::PartFailure::Busy => c::PartFailure::Busy,
                w::PartFailure::TooLarge => c::PartFailure::TooLarge,
                w::PartFailure::Unreadable => c::PartFailure::Unreadable,
            }),
        };
        parts.push(part).expect("same part count");
    }
    Ok(parts)
}

/// Contextual checks happen before a decoded answer or delivery leaves the link.
pub(crate) fn check_rows(
    rows: &skein_lib::List<c::TaskLine>,
    asks: &skein_lib::List<c::PartAsk>,
) -> Result<(), super::Undecodable> {
    for row in rows {
        if row.parts.len() != asks.len() {
            return Err(super::Undecodable::Malformed);
        }
        for (part, ask) in row.parts.iter().zip(asks.iter()) {
            match part {
                c::RowPart::Value(value) => {
                    if value.family != ask.owner || value.version != ask.version {
                        return Err(super::Undecodable::Malformed);
                    }
                }
                c::RowPart::Unknown | c::RowPart::Failed(_) | c::RowPart::NotServed(_) => (),
            }
        }
    }
    Ok(())
}

fn not_served(value: &w::NotServed) -> c::NotServed {
    client::not_served_to_domain(value.clone())
}
