//! Client history records (temper/docs/design/domain/landing-history.md, 2 and 5.1).
use super::{maps, w};
use alloc::boxed::Box;
use skein_codec::Reason;
use temper_client_protocol_wire::v1::Record;
use temper_engine_domain_forge::history as d;

// Priority is a signed value carried as its unsigned bits.
#[expect(clippy::trivially_copy_pass_by_ref, reason = "the converter signature borrows the domain value")]
pub(crate) fn priority_to_record(value: &i32) -> u32 {
    u32::from_be_bytes(value.to_be_bytes())
}

pub(crate) fn history_cursor_to_domain(value: [u8; 17]) -> Result<d::Cursor, Reason> {
    d::Cursor::decode(&value).ok_or(Reason::Malformed)
}

pub(crate) fn history_cursor_to_record(value: &d::Cursor) -> [u8; 17] {
    value.encode().as_ref().try_into().expect("17-byte cursor")
}

pub(super) fn query(value: w::HistoryRead) -> Option<d::Query> {
    maps::history_read_to_domain(value).ok()
}

pub(super) fn answer(value: d::Reply, limits: &w::Limits) -> Option<Box<[u8]>> {
    if let d::Reply::Page { events, .. } = &value
        && events.len() > skein_lib::usize_of(limits.history_page_events)
    {
        return None;
    }
    let result = maps::history_reply_to_record(&value).ok()?;
    (w::HistoryAnswer { result }).encode(limits).ok()
}
