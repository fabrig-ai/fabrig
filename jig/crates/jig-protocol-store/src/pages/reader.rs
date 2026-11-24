//! Typed map walks (jig/docs/design/protocol/store/loads.md, 1, and jig/docs/design/protocol/store/testing.md, 1).
use crate::{Cursor, KeptMap, Store, pages, restore};
use alloc::boxed::Box;
use restore::{Arithmetic, Cause, Failure, RangeFailure, WalkFailure};
use skein_kv::Range;
use skein_lib::{List, Queue, Token, length::len_u64};

fn cursor(intervals: &[Range], after: Option<&Cursor>, key_limit: u32) -> Result<(), Failure> {
    if let Some(after) = after {
        let index = usize::try_from(after.interval).ok().ok_or(Failure::rowless(Cause::Range(RangeFailure::Cursor)))?;
        let Some(range) = intervals.get(index) else {
            return Err(Failure::rowless(Cause::Range(RangeFailure::Cursor)));
        };
        let limit = skein_lib::length::usize_of(key_limit);
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
    let key = len_u64(row.key.len());
    let value = len_u64(row.value.len());
    let Some(size) = key.checked_add(value) else {
        return Err(Failure::at(Cause::Arithmetic(Arithmetic::RowBytes), restore::site(store, &row.key)));
    };
    Ok(size)
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
    walk_values(store, map, intervals, after, rows, true)
}

/// Written jobs walk layout ids and bytes as stored, not serving values.
pub(crate) fn walk_stored(
    store: &mut Store,
    intervals: &[Range],
    after: Option<&Cursor>,
    rows: u32,
) -> Result<pages::Walked, Failure> {
    walk_stored_map(store, KeptMap::Store, intervals, after, rows)
}

/// Rehearsals decode each stored layout, so neither map crosses the serving read edge.
pub(crate) fn walk_stored_map(
    store: &mut Store,
    map: KeptMap,
    intervals: &[Range],
    after: Option<&Cursor>,
    rows: u32,
) -> Result<pages::Walked, Failure> {
    walk_values(store, map, intervals, after, rows, false)
}

fn walk_values(
    store: &mut Store,
    map: KeptMap,
    intervals: &[Range],
    after: Option<&Cursor>,
    rows: u32,
    mapped: bool,
) -> Result<pages::Walked, Failure> {
    let limits = pages::map_limits(store, map);
    cursor(intervals, after, limits.key)?;
    let capacity = rows.min(limits.page.bytes);
    let mut found: List<pages::WalkRow> = List::with_capacity(capacity);
    let mut encoded = 0_u64;
    let first = match after {
        Some(after) => skein_lib::length::usize_of(after.interval),
        None => 0,
    };
    for (index, interval) in intervals.iter().enumerate().skip(first) {
        let index = u32::try_from(index).ok().ok_or(Failure::rowless(Cause::Arithmetic(Arithmetic::RangeCount)))?;
        let mut range = interval.clone();
        if let Some(after) = after
            && index == after.interval
        {
            range.start = range.start.max(past(&after.after)?);
        }
        for _ in 0..=capacity {
            if let Some(end) = &range.end
                && range.start >= *end
            {
                break;
            }
            let rows = capacity.saturating_sub(found.len()).saturating_add(1).min(limits.page.rows);
            let page = load_rows(store, map, limits, range.clone(), rows, mapped)?;
            for row in page.rows {
                let size = row_bytes(store, &row)?;
                let Some(total) = encoded.checked_add(size) else {
                    return Err(Failure::at(Cause::Arithmetic(Arithmetic::PageBytes), restore::site(store, &row.key)));
                };
                if found.len() == capacity || total > u64::from(limits.page.bytes) {
                    let next = match found.last() {
                        Some(last) => last.cursor.clone(),
                        None => pages::resume(None, after),
                    };
                    return Ok(pages::Walked { rows: found.into_boxed(), next: Some(next) });
                }
                encoded = total;
                range.start = match past(&row.key) {
                    Ok(next) => next,
                    Err(failure) => return Err(Failure::at(failure.cause, restore::site(store, &row.key))),
                };
                let cursor = Cursor { interval: index, after: row.key.clone() };
                found.push(pages::WalkRow { row, cursor }).expect("encoded and row bounds checked");
            }
            if page.next.is_none() {
                break;
            }
        }
    }
    Ok(pages::Walked { rows: found.into_boxed(), next: None })
}

/// One page of stored map rows, without a view overlay or layout conversion.
pub(crate) fn load_stored(store: &mut Store, range: Range) -> Result<pages::Page, Failure> {
    let limits = store.limits.store;
    load_rows(store, KeptMap::Store, limits, range, limits.page.rows, false)
}

pub(crate) fn load_map(store: &mut Store, map: KeptMap, range: Range) -> Result<pages::Page, Failure> {
    load_rows(store, map, pages::map_limits(store, map), range, 1, true)
}

fn load_rows(
    store: &mut Store,
    map: KeptMap,
    mut limits: skein_kv::Limits,
    range: Range,
    rows: u32,
    mapped: bool,
) -> Result<pages::Page, Failure> {
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
            range: range.clone(),
            max: skein_kv::Page { rows, bytes: limits.page.bytes },
        },
        &mut above,
        &mut below,
    );
    assert!(below.is_empty(), "memory pages issue no files");
    let mut page = terminal(above.pop())?;
    if mapped && map == KeptMap::Store {
        page = crate::view::overlay(store, &range, page, rows)?;
        for row in &mut page.rows {
            crate::registry::read(store, &row.key, &mut row.value);
        }
    }
    Ok(page)
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
            | skein_kv::Event::LogRead { .. }
            | skein_kv::Event::LogUnread { .. }
            | skein_kv::Event::Got { .. }
            | skein_kv::Event::Held { .. }
            | skein_kv::Event::Released { .. }
            | skein_kv::Event::Closed { .. }
            | skein_kv::Event::Recovered { .. },
        ) => WalkFailure::UnexpectedTerminal,
    };
    Err(Failure::rowless(Cause::Walk(failure)))
}
