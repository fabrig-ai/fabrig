//! Wire projections of result details facts (client/domain/testing.md, 4).
use super::super::{c, facts, pieces, w};
use super::{wire_executor, wire_proposal_key, wire_task_contract_document};
pub(in crate::codec) fn wire_requester(value: &c::Requester, _limits: &w::Limits) -> Result<w::Requester, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::Requester::Task(value) => {
            w::Requester::Task(w::RequesterTask { value: *(value) })
        }
        c::Requester::Person(value) => {
            w::Requester::Person(w::RequesterPerson { value: *(value) })
        }
        c::Requester::Deployment { project } => w::Requester::Deployment(w::RequesterDeployment { project: *(project) }),
    })
}

pub(in crate::codec) fn wire_task_status(
    value: &c::TaskStatus,
    _limits: &w::Limits,
) -> Result<w::TaskStatus, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TaskStatus::Done => w::TaskStatus::Done,
        c::TaskStatus::Failed => w::TaskStatus::Failed,
        c::TaskStatus::Cancelled => w::TaskStatus::Cancelled,
    })
}

pub(in crate::codec) fn wire_task_link(value: &c::TaskLink, limits: &w::Limits) -> Result<w::TaskLink, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TaskLink::Live(value) => w::TaskLink::Live(w::TaskLinkLive { value: facts::chip(value, limits)? }),
        c::TaskLink::Retained { chip, at } => w::TaskLink::Retained(w::TaskLinkRetained {
                chip: facts::chip(chip, limits)?,
                at: (at).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            }),
        c::TaskLink::Stored { number, status } => w::TaskLink::Stored(w::TaskLinkStored {
                number: *(number),
                status: (status).as_ref().map(|item| wire_task_status(item, limits)).transpose()?,
            }),
    })
}

pub(in crate::codec) fn wire_result_details(
    value: &c::ResultDetails,
    limits: &w::Limits,
) -> Result<w::ResultDetails, w::Problem> {
    Ok(w::ResultDetails {
            contract: wire_task_contract_document(&value.contract, limits)?,
            requester: wire_requester(&value.requester, limits)?,
            executor: wire_executor(&value.executor, limits)?,
            chosen_by: value.chosen_by.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            delegates: pieces(value.delegates.iter().map(|item| wire_task_link(item, limits)), limits)?,
            followups: value
                .followups
                .as_ref()
                .map(|item| {
                    pieces((item).iter().map(|item| wire_task_link(item, limits)), limits)
                })
                .transpose()?,
            proposal: value.proposal.as_ref().map(|item| wire_proposal_key(item, limits)).transpose()?,
            cut: value.cut,
        })
}
