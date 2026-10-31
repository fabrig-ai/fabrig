//! Wire projections of result details facts (client/domain/testing.md, 4).
use super::super::{c, facts, pieces, w};
use super::{wire_owner_executor, wire_proposal_key, wire_task_contract_document};
pub(in crate::codec) fn wire_owner_requester(
    value: &c::OwnerRequester,
    limits: &w::Limits,
) -> Result<w::OwnerRequester, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerRequester::Task(value) => {
            w::OwnerRequester::Task(w::OwnerRequesterTask::new(limits, w::OwnerRequesterTaskParts { value: *(value) })?)
        }
        c::OwnerRequester::Person(value) => w::OwnerRequester::Person(w::OwnerRequesterPerson::new(
            limits,
            w::OwnerRequesterPersonParts { value: *(value) },
        )?),
        c::OwnerRequester::Deployment { project } => w::OwnerRequester::Deployment(w::OwnerRequesterDeployment::new(
            limits,
            w::OwnerRequesterDeploymentParts { project: *(project) },
        )?),
    })
}

pub(in crate::codec) fn wire_owner_task_status(
    value: &c::OwnerTaskStatus,
    _limits: &w::Limits,
) -> Result<w::OwnerTaskStatus, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerTaskStatus::Done => w::OwnerTaskStatus::Done,
        c::OwnerTaskStatus::Failed => w::OwnerTaskStatus::Failed,
        c::OwnerTaskStatus::Cancelled => w::OwnerTaskStatus::Cancelled,
    })
}

pub(in crate::codec) fn wire_owner_task_link(
    value: &c::OwnerTaskLink,
    limits: &w::Limits,
) -> Result<w::OwnerTaskLink, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerTaskLink::Live(value) => w::OwnerTaskLink::Live(w::OwnerTaskLinkLive::new(
            limits,
            w::OwnerTaskLinkLiveParts { value: facts::chip(value, limits)? },
        )?),
        c::OwnerTaskLink::Retained { chip, at } => w::OwnerTaskLink::Retained(w::OwnerTaskLinkRetained::new(
            limits,
            w::OwnerTaskLinkRetainedParts {
                chip: facts::chip(chip, limits)?,
                at: (at).as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            },
        )?),
        c::OwnerTaskLink::Stored { number, status } => w::OwnerTaskLink::Stored(w::OwnerTaskLinkStored::new(
            limits,
            w::OwnerTaskLinkStoredParts {
                number: *(number),
                status: (status).as_ref().map(|item| wire_owner_task_status(item, limits)).transpose()?,
            },
        )?),
    })
}

pub(in crate::codec) fn wire_result_details(
    value: &c::ResultDetails,
    limits: &w::Limits,
) -> Result<w::ResultDetails, w::Problem> {
    w::ResultDetails::new(
        limits,
        w::ResultDetailsParts {
            contract: wire_task_contract_document(&value.contract, limits)?,
            requester: wire_owner_requester(&value.requester, limits)?,
            executor: wire_owner_executor(&value.executor, limits)?,
            chosen_by: value.chosen_by.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            delegates: pieces(
                value.delegates.iter().map(|item| wire_owner_task_link(item, limits)),
                w::OwnerTaskLink::measure,
                w::OwnerTaskLink::encode,
            )?,
            followups: value
                .followups
                .as_ref()
                .map(|item| {
                    pieces(
                        (item).iter().map(|item| wire_owner_task_link(item, limits)),
                        w::OwnerTaskLink::measure,
                        w::OwnerTaskLink::encode,
                    )
                })
                .transpose()?,
            proposal: value.proposal.as_ref().map(|item| wire_proposal_key(item, limits)).transpose()?,
            cut: value.cut,
        },
    )
}
