//! Openings are read only for rehearsal and erased by Wires (protocol/store/records.md 3).
use crate::{Store, WorkerOpening, codec, keys, pages};
use alloc::boxed::Box;
use skein_lib::List;

fn range(start: Box<[u8]>) -> skein_kv::Range {
    skein_kv::Range { start, end: Some(Box::new([keys::COMPONENT, 6])) }
}

pub(crate) fn clear(store: &mut Store, ops: &mut List<skein_kv::Op>) -> Option<()> {
    let mut start: Box<[u8]> = Box::new([keys::COMPONENT, 5]);
    for _ in 0..=store.stores.store.len() {
        let page = pages::load(store, range(start)).ok()?;
        for row in page.rows {
            erase(ops, row.key)?;
        }
        match page.next {
            Some(next) => start = next,
            None => {
                // Include accepted openings whose commit is not durable yet.
                for (key, ()) in &store.opening_keys {
                    erase(ops, key.clone())?;
                }
                store.opening_keys = skein_lib::Map::uncapped();
                return Some(());
            }
        }
    }
    None
}

fn erase(ops: &mut List<skein_kv::Op>, key: Box<[u8]>) -> Option<()> {
    for op in ops.iter_mut() {
        if op.key() == key.as_ref() {
            *op = skein_kv::Op::Erase { key };
            return Some(());
        }
    }
    ops.push(skein_kv::Op::Erase { key }).ok()
}

pub(crate) fn kept(store: &mut Store) -> Option<Box<[WorkerOpening]>> {
    let mut openings = List::with_capacity(store.stores.store.len());
    let mut start: Box<[u8]> = Box::new([keys::COMPONENT, 5]);
    for _ in 0..=store.stores.store.len() {
        let page = pages::load(store, range(start)).ok()?;
        for row in page.rows {
            let opening = codec::openings::decode(&row.value)?;
            if keys::decode_own(&row.key)? != keys::Own::Opening(opening.worker.clone()) {
                return None;
            }
            // The escaped name suffix preserves the workers' names' order.
            openings.push(opening).ok()?;
        }
        match page.next {
            Some(next) => start = next,
            None => return Some(openings.into_boxed()),
        }
    }
    None
}
