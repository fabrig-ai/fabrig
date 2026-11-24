//! Wire projections of owner escalation facts (client/domain/testing.md, 4).
use super::super::{c, try_pieces, w};
use super::{wire_offered, wire_task};
pub(in crate::codec) fn wire_escalation(
    value: &c::Escalation,
    limits: &w::Limits,
) -> Result<w::Escalation, w::Problem> {
    Ok(w::Escalation {
        acts: try_pieces(value.acts.iter().map(|item| wire_offered(item, limits)), limits)?,
        task: wire_task(&value.task, limits)?,
    })
}
