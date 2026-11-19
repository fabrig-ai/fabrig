//! A milestone's totals (temper/docs/design/client/forge/records.md).
use super::{forge, w, wire};
use alloc::boxed::Box;

pub(super) fn answer(value: Option<forge::MilestoneTotals>, limits: &w::Limits) -> Option<Box<[u8]>> {
    let result = match value {
        None => w::MilestoneResult::NotMilestone,
        Some(value) => w::MilestoneResult::Totals(forge::parts::totals(&value, limits)?),
    };
    let answer = w::MilestoneAnswer { result };
    Some(answer.encode(limits).ok()?)
}
