//! The scratch's maps decoded before planning (protocol/store/opening-rehearsal.md, 1).
use crate::registry::rehearsal::Decoder;
use crate::{Event, KeptFamily, OpenRefusal, OwnerSet, Phase, Store, Undecoded, open, pages, restore};
use alloc::boxed::Box;
use skein_codec::{schema, value::Value};
use skein_lib::{List, Queue, Token};

/// Starts do not walk. A rehearsal retains the stored layout texts, one kept
/// family's schemas and one reusable value; pages have the count walk's bounds.
pub(crate) fn records(store: &mut Store, owner: Token, above: &mut Queue<Event>) -> bool {
    if store.rehearsal.is_none() {
        return true;
    }
    let mut layouts = match Decoder::load(store) {
        Ok(layouts) => layouts,
        Err(why) => {
            open::refuse(store, owner, why, above);
            return false;
        }
    };
    let mut records = 0_u64;
    for set in store.owner_sets.sets {
        for family in set.families {
            match family_records(store, set, family, &mut layouts) {
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

#[derive(Debug)]
enum Root {
    Kept(schema::DeclIx),
    Layout(u16),
}

fn family_records(
    store: &mut Store,
    set: &OwnerSet,
    family: &KeptFamily,
    layouts: &mut Decoder,
) -> Result<u64, Failure> {
    layouts.family();
    let registered = !family.types.is_empty();
    let mut parsed = None;
    let kept = if registered {
        None
    } else {
        Some(
            crate::upgrade::schemas::get(store.prepared_schemas, family.current.as_bytes(), &mut parsed)
                .ok_or(Failure::Open(OpenRefusal::Set { owner: set.owner }))?,
        )
    };
    let mut ranges = List::uncapped();
    if registered {
        for ty in family.types {
            for keys in ty.keys {
                let keys = crate::Keys { from: keys.from, to: keys.to, root: ty.root };
                let range = pages::kept_interval(store, set.owner, family.map, &keys)
                    .ok_or(Failure::Open(OpenRefusal::Failed))?;
                ranges.push((range, Root::Layout(ty.number))).expect("uncapped ranges");
            }
        }
    } else {
        let schema = kept.expect("unregistered family's current decoder");
        for keys in family.keys {
            let root = schema.find(keys.root.as_bytes()).ok_or(Failure::Open(OpenRefusal::Set { owner: set.owner }))?;
            let range =
                pages::kept_interval(store, set.owner, family.map, keys).ok_or(Failure::Open(OpenRefusal::Failed))?;
            ranges.push((range, Root::Kept(root))).expect("uncapped ranges");
        }
    }
    let mut value = Value::with_room(pages::map_limits(store, family.map).value);
    let mut count = 0_u64;
    for (range, root) in ranges.into_boxed() {
        let mut cursor = None;
        for _ in 0..=rows(store, family) {
            let rows = pages::map_limits(store, family.map).page.rows;
            let walked = if registered {
                pages::reader::walk_stored_map(store, family.map, core::slice::from_ref(&range), cursor.as_ref(), rows)
            } else {
                pages::reader::walk_map(store, family.map, core::slice::from_ref(&range), cursor.as_ref(), rows)
            };
            let page = match walked {
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
                let decoded = match root {
                    Root::Layout(number) => layouts.decode(family.name, number, &mut value, item.row.value),
                    Root::Kept(root) => {
                        let schema = kept.expect("current root's decoder");
                        match value.decode(schema, root, item.row.value) {
                            Ok(_) => Ok(()),
                            Err(_) => Err(schema.version()),
                        }
                    }
                };
                if let Err(version) = decoded {
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
