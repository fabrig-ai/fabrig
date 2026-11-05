//! Answer row limits beside the allocation measure (skein/docs/design/forge.md, 3.7).
use crate::api;

fn rows(count: usize, limit: u32) -> Result<(), api::Error> {
    if skein_lib::len_u64(count) > u64::from(limit) {
        return Err(api::Error::TooLarge);
    }
    Ok(())
}

pub(super) fn summary(item: &api::Summary, limit: u32) -> Result<u64, api::Error> {
    rows(item.labels.len(), limit)?;
    crate::measure::summary(item).ok_or(api::Error::TooLarge)
}

pub fn answer(answer: &api::Answer, limit: u32) -> Result<u64, api::Error> {
    if !crate::measure::answer_rows(answer, limit) {
        return Err(api::Error::TooLarge);
    }
    crate::measure::answer(answer).ok_or(api::Error::TooLarge)
}
