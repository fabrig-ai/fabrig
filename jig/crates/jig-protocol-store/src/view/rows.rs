//! The three serving read edges share this view (protocol/store/layouts.md, 4).
use super::{State, Task, covered};
use crate::{Store, keys, logs, pages, restore};
use alloc::boxed::Box;
use core::ops::Bound;
use skein_kv::Range;
use skein_lib::{List, Map, Queue, Token, length::len_u64};

pub(crate) fn raw_get(store: &mut Store, key: Box<[u8]>) -> Result<Option<Box<[u8]>>, restore::JoinFailure> {
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    skein_kv::down(
        &mut store.stores.store,
        &crate::memory_env(store.limits.store),
        skein_kv::Request::Get { owner: Token::new(0), key },
        &mut above,
        &mut below,
    );
    assert!(below.is_empty(), "map gets issue no files");
    restore::reader::joined(above.pop())
}

fn register(store: &mut Store, range: &Range) -> Result<(), restore::JoinFailure> {
    if store.view.active.is_none() {
        return Ok(());
    }
    let Some((task, _)) = keys::logged_prefix(&range.start) else { return Ok(()) };
    let Some(log) = logs::history(task) else { return Ok(()) };
    if !store.view.active.as_ref().expect("held read").tasks.contains_key(&task) {
        let key = logs::extent_key(log);
        let Some(extent) = raw_get(store, key.clone())? else { return Ok(()) };
        logs::extent_row(&key, &extent).ok_or(restore::JoinFailure::Decode(restore::DecodeFailure::Value))?;
        store
            .view
            .active
            .as_mut()
            .expect("held read")
            .tasks
            .insert(task, Task { asked: List::uncapped(), covered: List::uncapped() })
            .expect("read joins bound tasks");
    }
    let task = store.view.active.as_mut().expect("held read").tasks.get_mut(&task).expect("registered task");
    if !covered(&task.asked, range) {
        task.asked.push(range.clone()).expect("read intervals bound requests");
    }
    Ok(())
}

pub(crate) fn get(store: &mut Store, key: Box<[u8]>) -> Result<Option<Box<[u8]>>, restore::JoinFailure> {
    if keys::logged(&key).is_some() {
        let end = pages::past(&key).expect("bounded key successor");
        register(store, &Range { start: key.clone(), end: Some(end) })?;
    }
    let map = raw_get(store, key.clone())?;
    let fetched = match &store.view.active {
        Some(active) => active.rows.get(key.as_ref()).cloned(),
        None => None,
    };
    if map.is_some() && fetched.is_some() {
        return Err(restore::JoinFailure::Decode(restore::DecodeFailure::KeyValueDisagree));
    }
    Ok(map.or(fetched))
}

pub(crate) fn overlay(
    store: &mut Store,
    range: &Range,
    page: pages::Page,
    max: u32,
) -> Result<pages::Page, restore::Failure> {
    if let Err(failure) = register(store, range) {
        return Err(restore::Failure::at(
            restore::Cause::Join { target: restore::site(store, &range.start), failure },
            restore::site(store, &range.start),
        ));
    }
    merge(&store.view, range, page, max, store.limits.store.page.bytes)
}
fn merge(
    view: &State,
    range: &Range,
    page: pages::Page,
    max: u32,
    bytes: u32,
) -> Result<pages::Page, restore::Failure> {
    let Some(active) = &view.active else { return Ok(page) };
    if active.rows.is_empty() {
        return Ok(page);
    }
    let mut rows = Map::uncapped();
    for row in page.rows {
        rows.insert(row.key, row.value).expect("map page bounds rows");
    }
    let end = match &range.end {
        Some(end) => Bound::Excluded(end.as_ref()),
        None => Bound::Unbounded,
    };
    for (key, value) in active
        .rows
        .range::<[u8], _>((Bound::Included(range.start.as_ref()), end))
        .take(skein_lib::length::usize_of(max.saturating_add(1)))
    {
        if rows.contains_key(key.as_ref()) {
            return Err(restore::Failure::rowless(restore::Cause::Decode(restore::DecodeFailure::KeyValueDisagree)));
        }
        rows.insert(key.clone(), value.clone()).expect("fetched bytes bound rows");
    }
    let mut found = List::with_capacity(max);
    let mut used = 0_u64;
    let mut next = page.next;
    for (key, value) in &rows {
        if let Some(next) = &next
            && key >= next
        {
            break;
        }
        let size = len_u64(key.len()).checked_add(len_u64(value.len())).expect("bounded row bytes");
        if found.len() == max || used.saturating_add(size) > u64::from(bytes) {
            if found.is_empty() {
                return Err(restore::Failure::rowless(restore::Cause::Walk(restore::WalkFailure::Refused(
                    skein_kv::Refusal::TooLarge(skein_kv::Large::Page),
                ))));
            }
            next = Some(key.clone());
            break;
        }
        used = used.checked_add(size).expect("page bytes checked");
        found.push(skein_kv::Row { key: key.clone(), value: value.clone() }).expect("page rows checked");
    }
    Ok(pages::Page { rows: found.into_boxed(), next })
}
