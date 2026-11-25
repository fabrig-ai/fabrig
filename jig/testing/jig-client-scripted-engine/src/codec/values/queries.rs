//! Wire projections of queries facts (client/domain/testing.md, 4).
use super::super::{c, facts, list, try_pieces, w};
use super::{wire_budget, wire_count, wire_cursor, wire_phase_counts, wire_requester, wire_task_flags};
pub(in crate::codec) fn wire_task_page(value: &c::TaskPage, limits: &w::Limits) -> Result<w::TaskPage, w::Problem> {
    Ok(w::TaskPage {
        ended_mark: 0,
        live: try_pieces(value.live.iter().map(|item| wire_task_line(item, limits)), limits)?,
        live_after: value.live_after.as_ref().map(wire_cursor),
        ended: wire_ended_group(&value.ended, limits)?,
    })
}

pub(in crate::codec) fn wire_task_line(value: &c::TaskLine, limits: &w::Limits) -> Result<w::TaskLine, w::Problem> {
    Ok(w::TaskLine {
        parts: skein_lib::List::with_capacity(0),
        chip: facts::chip(&value.chip, limits)?,
        project: value.project,
        requester: wire_requester(&value.requester),
        tracked: value.tracked,
        label: value.label,
        flags: wire_task_flags(&value.flags),
        changed: value.changed.as_nanos(),
        last_activity: value.last_activity.as_nanos(),
        ended_at: value.ended_at.as_ref().map(|item| (item).as_nanos()),
        budget: wire_budget(&value.budget),
        waits_on: wire_waits_on(&value.waits_on),
        wait: value.wait.as_ref().map(wire_wait),
        nearest: value.nearest.as_ref().map(wire_nearest),
        phases: wire_phase_counts(&value.phases),
        delegates_done: value.delegates_done,
        delegates_total: value.delegates_total,
        held_below: value.held_below,
        conversation: value.conversation.as_ref().map(wire_conversation_read),
    })
}

pub(in crate::codec) fn wire_waits_on(value: &c::WaitsOn) -> w::WaitsOn {
    match value {
        c::WaitsOn::Dependencies { pending, count } => w::WaitsOn::Dependencies(w::WaitsOnDependencies {
            pending: list((pending).iter().copied()),
            count: *(count),
        }),
        c::WaitsOn::Party { whom, entries, task, kind } => w::WaitsOn::Party(w::WaitsOnParty {
            whom: wire_whom(whom),
            entries: *(entries),
            task: *(task),
            kind: wire_waited_for(kind),
        }),
        c::WaitsOn::Delegates => w::WaitsOn::Delegates,
        c::WaitsOn::Question { number } => w::WaitsOn::Question(w::WaitsOnQuestion { number: *number }),
        c::WaitsOn::News => w::WaitsOn::News,
        c::WaitsOn::Nothing => w::WaitsOn::Nothing,
    }
}

pub(in crate::codec) fn wire_whom(value: &c::Whom) -> w::Whom {
    match value {
        c::Whom::Person(value) => w::Whom::Person(w::WhomPerson { value: *(value) }),
        c::Whom::Role { project, role } => w::Whom::Role(w::WhomRole { project: *(project), role: *(role) }),
    }
}

pub(in crate::codec) fn wire_waited_for(value: &c::WaitedFor) -> w::WaitedFor {
    match value {
        c::WaitedFor::Question => w::WaitedFor::Question,
        c::WaitedFor::Proposal => w::WaitedFor::Proposal,
        c::WaitedFor::Escalation => w::WaitedFor::Escalation,
        c::WaitedFor::PersonTask => w::WaitedFor::PersonTask,
    }
}

pub(in crate::codec) fn wire_conversation_read(value: &c::ConversationRead) -> w::ConversationRead {
    w::ConversationRead { seen: value.seen, unread: wire_count(&value.unread) }
}

pub(in crate::codec) fn wire_ended_group(
    value: &c::EndedGroup,
    limits: &w::Limits,
) -> Result<w::EndedGroup, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::EndedGroup::Rows { rows, after, cut } => w::EndedGroup::Rows(w::EndedRows {
            rows: try_pieces((rows).iter().map(|item| wire_task_line(item, limits)), limits)?,
            after: (after).as_ref().map(wire_cursor),
            cut: *(cut),
        }),
        c::EndedGroup::None => w::EndedGroup::None,
    })
}

pub(in crate::codec) fn wire_wait(value: &c::Wait) -> w::Wait {
    w::Wait {
        task: value.task,
        kind: wire_wait_kind(value.kind),
        object: wire_wait_object(&value.object),
        since: value.since.as_nanos(),
        bound: match value.bound {
            c::waits::Bound::At { at, then } => {
                w::WaitBound::At(w::WaitAtBound { at: at.as_nanos(), then: wire_at_bound(then) })
            }
            c::waits::Bound::None(reason) => w::WaitBound::None(w::WaitNoBound {
                reason: match reason {
                    c::Unbounded::NoDeadline => w::Unbounded::NoDeadline,
                    c::Unbounded::Standing => w::Unbounded::Standing,
                    c::Unbounded::Behind(task) => w::Unbounded::Behind(w::UnboundedBehind { task }),
                    c::Unbounded::OperatorHold => w::Unbounded::OperatorHold,
                },
            }),
        },
        hears: value.hears.as_ref().map(super::wire_escalation_holder),
    }
}

fn wire_wait_object(value: &c::WaitObject) -> w::WaitObject {
    match value {
        c::WaitObject::Entry { kind, number, at } => w::WaitObject::Entry(w::WaitEntry {
            kind: wire_waited_for(kind),
            number: *number,
            at: super::wire_escalation_holder(at),
        }),
        c::WaitObject::Question { number, answerer } => {
            w::WaitObject::Question(w::WaitQuestion { number: *number, answerer: *answerer })
        }
        c::WaitObject::Task(value) => w::WaitObject::Task(w::WaitTask { value: *value }),
        c::WaitObject::Resource { name, holder } => w::WaitObject::Resource(w::WaitResource {
            name: w::WaitResourceName { connector: name.connector, path: name.path.clone() },
            holder: *holder,
        }),
        c::WaitObject::Run(value) => w::WaitObject::Run(w::WaitRun { value: super::wire_task_run_wait(value) }),
        c::WaitObject::News { subscriptions, timer } => w::WaitObject::News(w::WaitNews {
            subscriptions: subscriptions.clone(),
            timer: timer.map(skein_lib::Wall::as_nanos),
        }),
        c::WaitObject::Procedure { step, things } => w::WaitObject::Procedure(w::WaitProcedure {
            step: *step,
            things: list(things.iter().map(|row| w::Thing { what: row.what, number: row.number, owner: row.owner })),
        }),
    }
}

pub(in crate::codec) fn wire_nearest(value: &c::Nearest) -> w::Nearest {
    w::Nearest { at: value.at.as_nanos(), then: wire_at_bound(value.then), kind: wire_wait_kind(value.kind) }
}

fn wire_wait_kind(value: c::WaitKind) -> w::WaitKind {
    match value {
        c::WaitKind::Party => w::WaitKind::Party,
        c::WaitKind::Question => w::WaitKind::Question,
        c::WaitKind::Dependency => w::WaitKind::Dependency,
        c::WaitKind::Resource => w::WaitKind::Resource,
        c::WaitKind::Account => w::WaitKind::Account,
        c::WaitKind::Host => w::WaitKind::Host,
        c::WaitKind::Delegate => w::WaitKind::Delegate,
        c::WaitKind::News => w::WaitKind::News,
        c::WaitKind::Step => w::WaitKind::Step,
        c::WaitKind::Room => w::WaitKind::Room,
        c::WaitKind::Turn => w::WaitKind::Turn,
        c::WaitKind::SignOff => w::WaitKind::SignOff,
    }
}

fn wire_at_bound(value: c::AtBound) -> w::AtBound {
    match value {
        c::AtBound::Hold => w::AtBound::Hold,
        c::AtBound::PassUp => w::AtBound::PassUp,
        c::AtBound::Overdue => w::AtBound::Overdue,
        c::AtBound::GiveUp => w::AtBound::GiveUp,
    }
}
