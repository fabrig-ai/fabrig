//! Comments since (skein/docs/design/forge/client.md, section 4).
use super::Live;
use skein_lib::Time;

pub(super) fn comments(row: &Live) -> Time {
    if !row.record.watch.participating {
        return Time::from_nanos(u64::MAX);
    }
    if row.record.comment_repair || row.record.comment_page > 1 {
        return Time::ZERO;
    }
    match &row.record.cached.item {
        Some(item) => item.updated,
        None => Time::ZERO,
    }
}
