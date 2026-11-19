//! A milestone's totals (temper/docs/design/client/forge/records.md).
use super::{forge, w};
use alloc::boxed::Box;
use temper_client_protocol_wire::v1::Record;

pub(super) fn answer(value: Option<forge::MilestoneTotals>, limits: &w::Limits) -> Option<Box<[u8]>> {
    let result = match value {
        None => w::MilestoneResult::NotMilestone,
        Some(value) => w::MilestoneResult::Totals(forge::parts::totals(&value, limits)?),
    };
    let answer = w::MilestoneAnswer { result };
    answer.encode(limits).ok()
}
