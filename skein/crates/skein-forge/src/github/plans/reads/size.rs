//! Checked branch-list admission and the running allocation measure.
use crate::api::Excess;
use super::{Bounds, api};
use crate::{github::pin, measure};
use skein_lib::len_u64;

pub(super) fn capacity(bounds: Bounds, width: usize) -> Result<u32, api::Error> {
    let Ok(width) = u32::try_from(width) else {
        return Err(api::Error::TooLarge(Excess::unnamed()));
    };
    let room = bounds.answer_bytes.checked_div(width).ok_or(api::Error::TooLarge(Excess::unnamed()))?;
    Ok(bounds.rows.min(room))
}
pub(super) fn page_size(bounds: Bounds, capacity: u32) -> Result<u32, api::Error> {
    let limit = bounds.page_size.min(capacity).min(pin::PAGE_CAP);
    if limit == 0 {
        return Err(api::Error::TooLarge(Excess::unnamed()));
    }
    Ok(limit)
}
pub(super) fn add(size: u64, name: &[u8], bounds: Bounds) -> Result<u64, api::Error> {
    let length = len_u64(name.len());
    let Ok(width) = u64::try_from(measure::STRING_BYTES) else {
        return Err(api::Error::TooLarge(Excess::unnamed()));
    };
    let total = size.checked_add(length).ok_or(api::Error::TooLarge(Excess::unnamed()))?.checked_add(width).ok_or(api::Error::TooLarge(Excess::unnamed()))?;
    if total > u64::from(bounds.answer_bytes) {
        return Err(api::Error::TooLarge(api::Excess { bound: api::Bound::AnswerBytes, limit: u64::from(bounds.answer_bytes), size: total, exact: false }));
    }
    Ok(total)
}
