//! Wire projections of owner person task facts (client/domain/testing.md, 4).
use super::super::{c, facts, pieces, w};
use super::{
    wire_offered, wire_person_address, wire_requester, wire_result_document, wire_revision,
    wire_specification_document, wire_task_contract_document, wire_task_link,
};
pub(in crate::codec) fn wire_person_task(
    value: &c::PersonTask,
    limits: &w::Limits,
) -> Result<w::PersonTask, w::Problem> {
    w::PersonTask::new(
        limits,
        w::PersonTaskParts {
            task: facts::chip(&value.task, limits)?,
            revision: wire_revision(&value.revision, limits)?,
            requester: wire_requester(&value.requester, limits)?,
            spec: wire_specification_document(&value.spec, limits)?,
            address: wire_person_address(&value.address, limits)?,
            taken_by: value.taken_by.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            since: value.since.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            stall: value.stall.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            deadline: value.deadline.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            contract: wire_task_contract_document(&value.contract, limits)?,
            results: pieces(
                value.results.iter().map(|item| wire_shown_result(item, limits)),
                w::ShownResult::measure,
                w::ShownResult::encode,
            )?,
            cut: value.cut,
            acts: pieces(
                value.acts.iter().map(|item| wire_offered(item, limits)),
                w::Offered::measure,
                w::Offered::encode,
            )?,
        },
    )
}

pub(in crate::codec) fn wire_shown_result(
    value: &c::ShownResult,
    limits: &w::Limits,
) -> Result<w::ShownResult, w::Problem> {
    w::ShownResult::new(
        limits,
        w::ShownResultParts {
            task: wire_task_link(&value.task, limits)?,
            result: value.result.as_ref().map(|item| wire_result_document(item, limits)).transpose()?,
        },
    )
}
