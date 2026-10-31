//! Wire projections of queries facts (client/domain/testing.md, 4).
use super::super::{c, facts, list, pieces, w};
use super::{
    wire_count, wire_cursor, wire_owner_budget, wire_owner_phase_counts, wire_owner_requester, wire_task_flags,
};
pub(in crate::codec) fn wire_task_page(value: &c::TaskPage, limits: &w::Limits) -> Result<w::TaskPage, w::Problem> {
    w::TaskPage::new(
        limits,
        w::TaskPageParts {
            live: pieces(
                value.live.iter().map(|item| wire_task_line(item, limits)),
                w::TaskLine::measure,
                w::TaskLine::encode,
            )?,
            live_after: value.live_after.as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
            ended: wire_ended_group(&value.ended, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_task_line(value: &c::TaskLine, limits: &w::Limits) -> Result<w::TaskLine, w::Problem> {
    w::TaskLine::new(
        limits,
        w::TaskLineParts {
            chip: facts::chip(&value.chip, limits)?,
            project: value.project,
            requester: wire_owner_requester(&value.requester, limits)?,
            tracked: value.tracked.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            label: value.label,
            flags: wire_task_flags(&value.flags, limits)?,
            changed: value.changed.as_nanos(),
            last_activity: value.last_activity.as_nanos(),
            ended_at: value.ended_at.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            budget: wire_owner_budget(&value.budget, limits)?,
            waits_on: wire_waits_on(&value.waits_on, limits)?,
            phases: wire_owner_phase_counts(&value.phases, limits)?,
            held_below: value.held_below,
            conversation: value.conversation.as_ref().map(|item| wire_conversation_read(item, limits)).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_waits_on(value: &c::WaitsOn, limits: &w::Limits) -> Result<w::WaitsOn, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::WaitsOn::Dependencies { pending, count } => w::WaitsOn::Dependencies(w::WaitsOnDependencies::new(
            limits,
            w::WaitsOnDependenciesParts {
                pending: list((pending).iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
                count: *(count),
            },
        )?),
        c::WaitsOn::Party { whom, entries, task, kind } => w::WaitsOn::Party(w::WaitsOnParty::new(
            limits,
            w::WaitsOnPartyParts {
                whom: wire_whom(whom, limits)?,
                entries: *(entries),
                task: *(task),
                kind: wire_waited_for(kind, limits)?,
            },
        )?),
        c::WaitsOn::Delegates => w::WaitsOn::Delegates,
        c::WaitsOn::News => w::WaitsOn::News,
        c::WaitsOn::Nothing => w::WaitsOn::Nothing,
    })
}

pub(in crate::codec) fn wire_whom(value: &c::Whom, limits: &w::Limits) -> Result<w::Whom, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Whom::Person(value) => w::Whom::Person(w::WhomPerson::new(limits, w::WhomPersonParts { value: *(value) })?),
        c::Whom::Role { project, role } => {
            w::Whom::Role(w::WhomRole::new(limits, w::WhomRoleParts { project: *(project), role: *(role) })?)
        }
    })
}

pub(in crate::codec) fn wire_waited_for(value: &c::WaitedFor, _limits: &w::Limits) -> Result<w::WaitedFor, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::WaitedFor::Question => w::WaitedFor::Question,
        c::WaitedFor::Proposal => w::WaitedFor::Proposal,
        c::WaitedFor::Escalation => w::WaitedFor::Escalation,
        c::WaitedFor::PersonTask => w::WaitedFor::PersonTask,
    })
}

pub(in crate::codec) fn wire_conversation_read(
    value: &c::ConversationRead,
    limits: &w::Limits,
) -> Result<w::ConversationRead, w::Problem> {
    w::ConversationRead::new(
        limits,
        w::ConversationReadParts { seen: value.seen, unread: wire_count(&value.unread, limits)? },
    )
}

pub(in crate::codec) fn wire_ended_group(
    value: &c::EndedGroup,
    limits: &w::Limits,
) -> Result<w::EndedGroup, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::EndedGroup::Rows { rows, after, cut } => w::EndedGroup::Rows(w::EndedRows::new(
            limits,
            w::EndedRowsParts {
                rows: pieces(
                    (rows).iter().map(|item| wire_task_line(item, limits)),
                    w::TaskLine::measure,
                    w::TaskLine::encode,
                )?,
                after: (after).as_ref().map(|item| wire_cursor(item, limits)).transpose()?,
                cut: *(cut),
            },
        )?),
        c::EndedGroup::None => w::EndedGroup::None,
    })
}
