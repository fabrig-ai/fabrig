//! Typed map walks (jig/docs/design/protocol/store.md, 7.1 and 14).
use crate::{Cursor, KeptMap, Store, pages, restore};
use alloc::boxed::Box;
use restore::{Arithmetic, Cause, Failure, RangeFailure, WalkFailure};
use skein_kv::Range;
use skein_lib::{List, Queue, Token};

fn cursor(store: &Store, map: KeptMap, intervals: &[Range], after: Option<&Cursor>) -> Result<(), Failure> {
    if let Some(after) = after {
        let index = usize::try_from(after.interval).ok().ok_or(Failure::rowless(Cause::Range(RangeFailure::Cursor)))?;
        let Some(range) = intervals.get(index) else {
            return Err(Failure::rowless(Cause::Range(RangeFailure::Cursor)));
        };
        let limit = usize::try_from(pages::map_limits(store, map).key).expect("map key allowance fits usize");
        if after.after.len() > limit
            || (after.after < range.start
                || match &range.end {
                    Some(end) => after.after >= *end,
                    None => false,
                })
        {
            return Err(Failure::rowless(Cause::Range(RangeFailure::Cursor)));
        }
    }
    Ok(())
}

pub(crate) fn past(key: &[u8]) -> Result<Box<[u8]>, Failure> {
    pages::past(key).ok_or(Failure::rowless(Cause::Arithmetic(Arithmetic::CursorLength)))
}

pub(crate) fn row_bytes(store: &Store, row: &skein_kv::Row) -> Result<u64, Failure> {
    let failure = Failure::at(Cause::Arithmetic(Arithmetic::RowBytes), restore::site(store, &row.key));
    let key = u64::try_from(row.key.len()).ok().ok_or(failure.clone())?;
    let value = u64::try_from(row.value.len()).ok().ok_or(failure.clone())?;
    let size = key.checked_add(value);
    size.ok_or(Failure::at(Cause::Arithmetic(Arithmetic::RowBytes), restore::site(store, &row.key)))
}

pub(crate) fn walk(
    store: &mut Store,
    intervals: &[Range],
    after: Option<&Cursor>,
    rows: u32,
) -> Result<pages::Walked, Failure> {
    walk_map(store, KeptMap::Store, intervals, after, rows)
}

pub(crate) fn walk_map(
    store: &mut Store,
    map: KeptMap,
    intervals: &[Range],
    after: Option<&Cursor>,
    rows: u32,
) -> Result<pages::Walked, Failure> {
    cursor(store, map, intervals, after)?;
    let capacity = rows.min(pages::map_limits(store, map).page.bytes);
    let mut found = List::with_capacity(capacity);
    let mut encoded = 0_u64;
    let mut last = after.cloned().unwrap_or(Cursor { interval: 0, after: Box::new([]) });
    let first = usize::try_from(last.interval).expect("validated interval fits usize");
    for (index, interval) in intervals.iter().enumerate().skip(first) {
        let index = u32::try_from(index).ok().ok_or(Failure::rowless(Cause::Arithmetic(Arithmetic::RangeCount)))?;
        let mut range = interval.clone();
        if index == last.interval && after.is_some() {
            range.start = range.start.max(past(&last.after)?);
        }
        for _ in 0..=capacity {
            if let Some(end) = &range.end
                && range.start >= *end
            {
                break;
            }
            let page = load_map(store, map, range.clone())?;
            let mut rows = page.rows.into_iter();
            let Some(row) = rows.next() else { break };
            let size = row_bytes(store, &row)?;
            let total = encoded
                .checked_add(size)
                .ok_or(Failure::at(Cause::Arithmetic(Arithmetic::PageBytes), restore::site(store, &row.key)))?;
            if found.len() == capacity || total > u64::from(pages::map_limits(store, map).page.bytes) {
                return Ok(pages::Walked { rows: found.into_boxed(), next: Some(last) });
            }
            encoded = total;
            last = Cursor { interval: index, after: row.key.clone() };
            range.start = match past(&row.key) {
                Ok(next) => next,
                Err(failure) => return Err(Failure::at(failure.cause, restore::site(store, &row.key))),
            };
            found.push(pages::WalkRow { row, cursor: last.clone() }).expect("encoded and row bounds checked");
            if page.next.is_none() {
                break;
            }
        }
    }
    Ok(pages::Walked { rows: found.into_boxed(), next: None })
}

fn load_map(store: &mut Store, map: KeptMap, range: Range) -> Result<pages::Page, Failure> {
    let mut limits = pages::map_limits(store, map);
    limits.key = limits.key.checked_add(1).ok_or(Failure::rowless(Cause::Arithmetic(Arithmetic::CursorLength)))?;
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    let target = match map {
        KeptMap::Store => &mut store.stores.store,
        KeptMap::Secrets => &mut store.stores.secrets,
    };
    skein_kv::down(
        target,
        &crate::memory_env(limits),
        skein_kv::Request::Load {
            owner: Token::new(0),
            range,
            max: skein_kv::Page { rows: 1, bytes: limits.page.bytes },
        },
        &mut above,
        &mut below,
    );
    assert!(below.is_empty(), "memory pages issue no files");
    terminal(above.pop())
}

pub(crate) fn terminal(event: Option<skein_kv::Event>) -> Result<pages::Page, Failure> {
    let failure = match event {
        Some(skein_kv::Event::Loaded { rows, next, .. }) => return Ok(pages::Page { rows, next }),
        None => WalkFailure::NoTerminal,
        Some(skein_kv::Event::Refused { refusal, .. }) => WalkFailure::Refused(refusal),
        Some(skein_kv::Event::OpenFailed { failure, .. } | skein_kv::Event::RecoveryFailed { failure }) => {
            WalkFailure::Failed(failure)
        }
        Some(
            skein_kv::Event::Opened { .. }
            | skein_kv::Event::Committed { .. }
            | skein_kv::Event::Failed { .. }
            | skein_kv::Event::Got { .. }
            | skein_kv::Event::Closed { .. }
            | skein_kv::Event::Recovered { .. },
        ) => WalkFailure::UnexpectedTerminal,
    };
    Err(Failure::rowless(Cause::Walk(failure)))
}
