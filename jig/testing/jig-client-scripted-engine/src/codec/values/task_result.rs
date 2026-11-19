//! Wire projections of task result facts (client/domain/testing.md, 4).
use super::super::{c, facts, w};
use super::wire_result_document;
pub(in crate::codec) fn wire_task_result(
    value: &c::TaskResult,
    limits: &w::Limits,
) -> Result<w::TaskResult, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TaskResult::Document(value) => w::TaskResult::Document(Box::new(w::TaskResultDocument { value: wire_result_document(value, limits)? })),
        c::TaskResult::Ending(value) => w::TaskResult::Ending(w::TaskResultEnding { value: facts::chip_ending(value, limits)? }),
    })
}
