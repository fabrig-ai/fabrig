//! Client history records (temper/docs/design/domain/landing-history.md, 2 and 5.1).
use super::{maps, w};
use alloc::boxed::Box;
use skein_lib::{List, Wall};
use temper_client_protocol_wire::v1::Record;
use temper_engine_domain_forge::history as d;

// Priority is a signed value carried as its unsigned bits.
#[expect(clippy::trivially_copy_pass_by_ref, reason = "the converter signature borrows the domain value")]
pub(crate) fn priority_to_record(value: &i32) -> u32 {
    u32::from_be_bytes(value.to_be_bytes())
}

pub(super) fn query(value: w::HistoryRead) -> Option<d::Query> {
    let w::HistoryRead { project, of, since, until, after } = value;
    Some(d::Query {
        project,
        of: maps::history_of_to_domain(of),
        since: match since {
            Some(n) => Some(Wall::from_nanos(n)),
            None => None,
        },
        until: match until {
            Some(n) => Some(Wall::from_nanos(n)),
            None => None,
        },
        after: match after {
            Some(bytes) => Some(d::Cursor::decode(&bytes)?),
            None => None,
        },
    })
}
pub(super) fn answer(value: d::Reply, limits: &w::Limits) -> Option<Box<[u8]>> {
    let result = match value {
        d::Reply::Busy => w::HistoryReply::Busy,
        d::Reply::Gone { from, first } => w::HistoryReply::Gone(w::HistoryGone { from, first }),
        d::Reply::Page { events, after, upper, whole_since } => {
            let mut rows = List::with_capacity(limits.history_page_events);
            for event in &events {
                rows.push(maps::history_event_to_record(event).ok()?).ok()?;
            }
            let after = match after {
                Some(cursor) => Some(cursor.encode().as_ref().try_into().expect("17-byte cursor")),
                None => None,
            };
            w::HistoryReply::Page(w::HistoryPage {
                events: rows,
                after,
                upper,
                whole_since: match whole_since {
                    Some(at) => Some(at.as_nanos()),
                    None => None,
                },
            })
        }
    };
    (w::HistoryAnswer { result }).encode(limits).ok()
}
