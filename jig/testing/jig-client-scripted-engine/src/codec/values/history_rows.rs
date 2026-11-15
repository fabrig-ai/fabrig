//! Wire projections of history rows facts (client/domain/testing.md, 4).
use super::super::{c, w};
use super::{wire_attempt_line, wire_history_line, wire_hold_line};
pub(in crate::codec) fn wire_history_row(
    value: &c::HistoryRow,
    limits: &w::Limits,
) -> Result<w::HistoryRow, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HistoryRow::Change(value) => w::HistoryRow::Change(Box::new(w::HistoryRowChange::new(
            limits,
            w::HistoryRowChangeParts { value: wire_history_line(value, limits)? },
        )?)),
        c::HistoryRow::Attempt(value) => w::HistoryRow::Attempt(w::HistoryRowAttempt::new(
            limits,
            w::HistoryRowAttemptParts { value: wire_attempt_line(value, limits)? },
        )?),
        c::HistoryRow::Hold(value) => w::HistoryRow::Hold(w::HistoryRowHold::new(
            limits,
            w::HistoryRowHoldParts { value: wire_hold_line(value, limits)? },
        )?),
    })
}
