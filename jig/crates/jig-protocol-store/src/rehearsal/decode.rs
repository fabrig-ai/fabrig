//! The scratch's maps decoded before planning (protocol/store/opening.md, 5).
use crate::patching::{machines, plan::Levels};
use crate::{Event, KeptFamily, OpenRefusal, PatchSet, Phase, Store, Undecoded, open, pages, restore};
use alloc::boxed::Box;
use skein_codec::{patch::Patcher, value::Value};
use skein_lib::{Queue, Token};

/// Starts do not walk. A rehearsal retains only one family's schemas and one
/// reusable value; map pages have the same bounds as the count walk.
pub(crate) fn records(store: &mut Store, owner: Token, found: Levels, above: &mut Queue<Event>) -> bool {
    if store.rehearsal.is_none() {
        return true;
    }
    let mut records = 0_u64;
    for set in store.patch_sets.sets {
        let Some(level) = found.of(set.owner) else { continue };
        for (ix, family) in set.families.iter().enumerate() {
            if family.texts.is_empty() {
                continue;
            }
            match family_records(store, set, ix, level) {
                Ok(count) => records = records.checked_add(count).expect("map row counts fit u64"),
                Err(Failure::Record { version, key }) => {
                    store.phase = Phase::Stopped;
                    above.push(Event::Undecodable {
                        owner,
                        family: Box::from(family.name),
                        version: u32::from(version),
                        at: Undecoded::Record { map: family.map, key },
                    });
                    return false;
                }
                Err(Failure::Open(why)) => {
                    open::refuse(store, owner, why, above);
                    return false;
                }
            }
        }
    }
    above.push(Event::Decoded { owner, records });
    true
}

enum Failure {
    Record { version: u16, key: Box<[u8]> },
    Open(OpenRefusal),
}

fn decoder(store: &Store, set: &PatchSet, ix: usize, level: u32) -> Result<(Box<Patcher>, u16), Failure> {
    let invalid = OpenRefusal::Set { owner: set.owner, patch: 0 };
    let family = set.families.get(ix).expect("family being walked");
    let room = machines::Rooms::of(&store.limits).of_map(family.map);
    let capacity = machines::capacity(set, ix, room).ok_or(Failure::Open(invalid.clone()))?;
    let mut patcher = Box::new(Patcher::new(capacity));
    let mut version = None;
    for text in family.texts {
        let Ok(kept) = patcher.keep(text.as_bytes()) else { return Err(Failure::Open(invalid)) };
        if version.is_none() {
            version = Some(kept);
        }
    }
    let mut version = version.ok_or(Failure::Open(invalid.clone()))?;
    for text in set.patches.iter().take(skein_lib::length::usize_of(level)) {
        match patcher.plan(text.as_bytes()) {
            Ok(Some(plan)) => version = patcher.versions(plan).to,
            Ok(None) => {}
            Err(_) => return Err(Failure::Open(invalid)),
        }
    }
    Ok((patcher, version))
}

fn family_records(store: &mut Store, set: &PatchSet, ix: usize, level: u32) -> Result<u64, Failure> {
    let family = set.families.get(ix).expect("family being walked");
    let (patcher, version) = decoder(store, set, ix, level)?;
    let schema = patcher.schema(version).expect("reached version kept");
    let mut value = Value::with_room(machines::Rooms::of(&store.limits).of_map(family.map).nodes);
    let mut count = 0_u64;
    for keys in family.keys {
        let root =
            schema.find(keys.root.as_bytes()).ok_or(Failure::Open(OpenRefusal::Set { owner: set.owner, patch: 0 }))?;
        let range =
            pages::kept_interval(store, set.owner, family.map, keys).ok_or(Failure::Open(OpenRefusal::Failed))?;
        let mut cursor = None;
        for _ in 0..=rows(store, family) {
            let page = match pages::reader::walk_map(
                store,
                family.map,
                core::slice::from_ref(&range),
                cursor.as_ref(),
                pages::map_limits(store, family.map).page.rows,
            ) {
                Ok(page) => page,
                Err(failure) => {
                    store.open_failed(restore::OpenFailure::Data(failure));
                    return Err(Failure::Open(OpenRefusal::Failed));
                }
            };
            if page.rows.is_empty() && page.next.is_some() {
                return Err(Failure::Open(OpenRefusal::Failed));
            }
            for item in page.rows {
                if value.decode(schema, root, item.row.value).is_err() {
                    return Err(Failure::Record { version, key: item.row.key });
                }
                count = count.checked_add(1).expect("map row counts fit u64");
            }
            cursor = page.next;
            if cursor.is_none() {
                break;
            }
        }
        if cursor.is_some() {
            return Err(Failure::Open(OpenRefusal::Failed));
        }
    }
    Ok(count)
}

fn rows(store: &Store, family: &KeptFamily) -> u32 {
    match family.map {
        crate::KeptMap::Store => store.stores.store.len(),
        crate::KeptMap::Secrets => store.stores.secrets.len(),
    }
}
