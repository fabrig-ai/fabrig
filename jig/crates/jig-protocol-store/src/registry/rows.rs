//! Registry rows and their commit operations (protocol/store/layouts.md, 3 and 5).
use super::{Layouts, Range, Row, RowKey, Text, Type, texts};
use crate::{KeptFamily, OpenRefusal, Store, ValueType, keys, registry_v1 as wire};
use alloc::boxed::Box;
use skein_codec::Record as _;
use skein_lib::{List, Map, Reader};

/// Loading precedes the same registration refusal in a start and a rehearsal (layouts.md, 5.1).
pub(super) fn open(store: &mut Store, layouts: &mut Layouts, empty: bool) -> Result<bool, OpenRefusal> {
    load(store, layouts)?;
    let mut registered = false;
    for set in store.owner_sets.sets {
        for family in set.families {
            registered |= !family.types.is_empty() || !family.retired.is_empty();
        }
    }
    if layouts.rows.is_empty() && !empty && registered {
        return Err(OpenRefusal::OlderBuild { why: crate::Older::Unregistered });
    }
    Ok(registered)
}

fn load(store: &mut Store, layouts: &mut Layouts) -> Result<(), OpenRefusal> {
    let ranges = [skein_kv::Range { start: Box::from([0, 6]), end: Some(Box::from([0, 8])) }];
    let mut after = None;
    let mut count = 0_u64;
    for space in store.stores.store.census().keyspaces() {
        count = count.checked_add(space.rows).ok_or(OpenRefusal::Failed)?;
    }
    for _ in 0..=count {
        let page = match crate::pages::reader::walk(store, &ranges, after.as_ref(), store.limits.store.ops) {
            Ok(page) => page,
            Err(failure) => {
                store.open_failed(crate::restore::OpenFailure::Data(failure));
                return Err(OpenRefusal::Failed);
            }
        };
        for row in page.rows {
            match keys::decode_own(&row.row.key) {
                Some(keys::Own::Layout { family, number, id }) => {
                    let value = wire::Layout::decode(&wire::CEILINGS, &mut Reader::new(&row.row.value))
                        .ok()
                        .ok_or(OpenRefusal::Failed)?;
                    layouts
                        .rows
                        .insert((family, number, id), Row { value, stored: true, retire: false })
                        .expect("uncapped registry");
                }
                Some(keys::Own::Text(digest)) => {
                    let value = wire::Text::decode(&wire::CEILINGS, &mut Reader::new(&row.row.value))
                        .ok()
                        .ok_or(OpenRefusal::Failed)?;
                    let bytes = value.text;
                    layouts
                        .texts
                        .insert(digest, Text { bytes, schema: None, stored: true, current: false })
                        .expect("uncapped texts");
                }
                Some(
                    keys::Own::History { .. }
                    | keys::Own::TurnLog { .. }
                    | keys::Own::LogAttempt { .. }
                    | keys::Own::Format
                    | keys::Own::Floor
                    | keys::Own::Wires
                    | keys::Own::Opening(_)
                    | keys::Own::Job { .. }
                    | keys::Own::Transcript { .. }
                    | keys::Own::Attempt { .. },
                )
                | None => return Err(OpenRefusal::Failed),
            }
        }
        after = page.next;
        if after.is_none() {
            return Ok(());
        }
    }
    Err(OpenRefusal::Failed)
}
fn made(
    layouts: &mut Layouts,
    binary: &mut texts::Binary,
    family: &KeptFamily,
    ty: &ValueType,
) -> Option<wire::Layout> {
    let text = binary.text(family.name)?;
    let held = binary.held(layouts, text)?;
    let layout = binary.layout(layouts, text, ty.root.as_bytes(), super::variant(ty.variant), &held)?;
    let value = wire::Layout {
        root: Box::from(ty.root.as_bytes()),
        variant: match super::variant(ty.variant) {
            Some(v) => Some(Box::from(v)),
            None => None,
        },
        layout,
        text,
        held,
    };
    value.measure(&wire::CEILINGS).ok()?;
    Some(value)
}
pub(super) fn register(store: &Store, layouts: &mut Layouts, binary: &mut texts::Binary) -> Result<bool, OpenRefusal> {
    let mut changed = false;
    for set in store.owner_sets.sets {
        for family in set.families {
            for ty in family.types {
                let refusal = OpenRefusal::Set { owner: set.owner };
                let value = made(layouts, binary, family, ty).ok_or(refusal.clone())?;
                let version =
                    layouts.texts.get(&value.text).ok_or(refusal.clone())?.schema().ok_or(refusal.clone())?.version();
                let mut found = None;
                let mut greatest = None;
                let first = (Box::from(family.name), ty.number, u16::MIN);
                let last = (Box::from(family.name), ty.number, u16::MAX);
                for ((_, _, id), row) in layouts.rows.range(first..=last) {
                    greatest = Some(*id);
                    if row.value.layout == value.layout {
                        found = Some(*id);
                    }
                }
                let id = match found {
                    Some(id) => id,
                    None => {
                        let id = match greatest {
                            Some(id) => id.checked_add(1).ok_or(refusal.clone())?,
                            None => version,
                        };
                        layouts
                            .rows
                            .insert(
                                (Box::from(family.name), ty.number, id),
                                Row { value, stored: false, retire: false },
                            )
                            .expect("uncapped registration");
                        changed = true;
                        id
                    }
                };
                let index = layouts.types.len();
                layouts
                    .types
                    .push(Type { version, binary: id, keeping: List::uncapped(), plans: Map::uncapped() })
                    .expect("uncapped types");
                for range in ty.keys {
                    let from = super::whole(set.owner, range.from, store.limits.store.key).ok_or(refusal.clone())?;
                    let to = super::whole(set.owner, range.to, store.limits.store.key).ok_or(refusal.clone())?;
                    layouts.ranges.insert(from, Range { to, ty: index }).expect("uncapped ranges");
                }
            }
        }
    }
    Ok(changed)
}
fn key(key: &RowKey, limit: u32) -> Option<Box<[u8]>> {
    keys::encode_own(&keys::Own::Layout { family: key.0.clone(), number: key.1, id: key.2 }, limit)
}
pub(super) fn operations(bound: u32, layouts: &mut Layouts) -> Result<(), OpenRefusal> {
    let mut used = Map::uncapped();
    for (row_key, row) in &layouts.rows {
        let key = key(row_key, bound).ok_or(OpenRefusal::Failed)?;
        if row.retire {
            if row.stored {
                layouts.pending.push(skein_kv::Op::Erase { key });
            }
            continue;
        }
        used.insert(row.value.text, ()).expect("uncapped digests");
        for digest in &row.value.held {
            used.insert(*digest, ()).expect("uncapped digests");
        }
        if !row.stored {
            let value = row.value.encode(&wire::CEILINGS).ok().ok_or(OpenRefusal::Failed)?;
            layouts.pending.push(skein_kv::Op::Put { key, value });
        }
    }
    for (digest, text) in &layouts.texts {
        let key = keys::encode_own(&keys::Own::Text(*digest), bound).ok_or(OpenRefusal::Failed)?;
        if used.contains_key(digest) {
            if !text.stored {
                let value =
                    wire::Text { text: text.bytes.clone() }.encode(&wire::CEILINGS).ok().ok_or(OpenRefusal::Failed)?;
                layouts.pending.push(skein_kv::Op::Put { key, value });
            }
        } else if text.stored {
            layouts.pending.push(skein_kv::Op::Erase { key });
        }
    }
    // Stage the resulting registry for a later settling retirement. Keep the
    // rows and texts until jobs end: a written job's first page names them.
    let mut row_keys = List::uncapped();
    for (key, _) in &layouts.rows {
        row_keys.push(key.clone()).expect("uncapped row keys");
    }
    for key in &row_keys {
        let row = layouts.rows.get_mut(key).expect("held row");
        row.stored = !row.retire;
    }
    let mut digests = List::uncapped();
    for (digest, _) in &layouts.texts {
        digests.push(*digest).expect("uncapped text digests");
    }
    for digest in &digests {
        layouts.texts.get_mut(digest).expect("held text").stored = used.contains_key(digest);
    }
    Ok(())
}
pub(super) fn size(op: &skein_kv::Op) -> Option<u64> {
    let key = u32::try_from(op.key().len()).ok()?;
    Some(u64::from(match op {
        skein_kv::Op::Append { .. } | skein_kv::Op::Drop { .. } => return None,
        skein_kv::Op::Put { value, .. } => skein_kv::put_bytes(key, u32::try_from(value.len()).ok()?)?,
        skein_kv::Op::Erase { .. } => skein_kv::erase_bytes(key)?,
    }))
}
