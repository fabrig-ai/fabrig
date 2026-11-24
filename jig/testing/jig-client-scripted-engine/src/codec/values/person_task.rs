//! Wire projections of owner person task facts (client/domain/testing.md, 4).
use super::super::{c, facts, try_pieces, w};
use super::{
    wire_offered, wire_person_address, wire_requester, wire_result_document, wire_revision,
    wire_specification_document, wire_task_contract_document, wire_task_link,
};
pub(in crate::codec) fn wire_person_task(
    value: &c::PersonTask,
    limits: &w::Limits,
) -> Result<w::PersonTask, w::Problem> {
    Ok(w::PersonTask {
        task: facts::chip(&value.task, limits)?,
        revision: wire_revision(&value.revision),
        requester: wire_requester(&value.requester),
        spec: wire_specification_document(&value.spec),
        address: wire_person_address(&value.address),
        taken_by: value.taken_by,
        since: value.since.as_ref().map(|item| (item).as_nanos()),
        stall: value.stall,
        deadline: value.deadline.as_ref().map(|item| (item).as_nanos()),
        contract: wire_task_contract_document(&value.contract, limits)?,
        results: try_pieces(value.results.iter().map(|item| wire_shown_result(item, limits)), limits)?,
        cut: value.cut,
        acts: try_pieces(value.acts.iter().map(|item| wire_offered(item, limits)), limits)?,
    })
}

pub(in crate::codec) fn wire_shown_result(
    value: &c::ShownResult,
    limits: &w::Limits,
) -> Result<w::ShownResult, w::Problem> {
    Ok(w::ShownResult {
        task: wire_task_link(&value.task, limits)?,
        result: value.result.as_ref().map(|item| wire_result_document(item, limits)).transpose()?,
    })
}
