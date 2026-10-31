//! Wire projections of history rows facts (client/domain/testing.md, 4).
use super::super::{c, w};
use super::{wire_history_line, wire_milestone_line};
pub(in crate::codec) fn wire_history_row(
    value: &c::HistoryRow,
    limits: &w::Limits,
) -> Result<w::HistoryRow, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HistoryRow::Change(value) => w::HistoryRow::Change(Box::new(w::HistoryRowChange::new(
            limits,
            w::HistoryRowChangeParts { value: wire_history_line(value, limits)? },
        )?)),
        c::HistoryRow::Milestone(value) => w::HistoryRow::Milestone(w::HistoryRowMilestone::new(
            limits,
            w::HistoryRowMilestoneParts { value: wire_milestone_line(value, limits)? },
        )?),
    })
}
