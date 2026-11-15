//! Registry rows, their seed and commit operations (protocol/store/layouts.md, 3 and 5).
use super::{Layouts, Range, Row, RowKey, Text, Type, texts};
use crate::{KeptFamily, OpenRefusal, Store, ValueType, keys, registry_v1 as wire};
use alloc::boxed::Box;
use skein_lib::{List, Map, Reader, Writer};

pub(super) fn load(store: &mut Store, layouts: &mut Layouts) -> Result<(), OpenRefusal> {
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
                    let bytes = Box::from(value.text());
                    layouts.texts.insert(digest, Text { bytes, schema: None, stored: true }).expect("uncapped texts");
                }
                Some(
                    keys::Own::Format
                    | keys::Own::Layouts
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
    binary: &texts::Binary,
    family: &KeptFamily,
    ty: &ValueType,
    current: bool,
) -> Option<wire::Layout> {
    let text = binary.text(family.name, current)?;
    let held = binary.held(layouts, text, current)?;
    let schema = layouts.texts.get(&text)?.schema.as_ref()?;
    let root = schema.root(ty.root.as_bytes(), super::variant(ty.variant))?;
    let schemas = texts::schemas(layouts, &held, false)?;
    let layout = schema.layout(root, schemas.as_slice()).ok()?.digest();
    wire::Layout::new(
        &wire::CEILINGS,
        wire::LayoutParts {
            root: Box::from(ty.root.as_bytes()),
            variant: match super::variant(ty.variant) {
                Some(v) => Some(Box::from(v)),
                None => None,
            },
            layout,
            text,
            held,
        },
    )
    .ok()
}
pub(super) fn seed(store: &Store, layouts: &mut Layouts, binary: &mut texts::Binary) -> Result<(), OpenRefusal> {
    for set in store.patch_sets.sets {
        for family in set.families {
            for ty in family.types {
                let refusal = OpenRefusal::Set { owner: set.owner, patch: 0 };
                let text = binary.text(family.name, false).ok_or(refusal.clone())?;
                texts::parse(layouts, text).ok_or(refusal.clone())?;
                let schema = layouts.texts.get(&text).ok_or(refusal.clone())?.schema.as_ref().ok_or(refusal.clone())?;
                // A value type introduced after the kept text has no seed layout.
                // Registration gives it the binary's layout, with no old values to convert.
                if schema.root(ty.root.as_bytes(), super::variant(ty.variant)).is_none() {
                    if !empty(store, set.owner, ty).ok_or(refusal.clone())? {
                        return Err(refusal);
                    }
                    continue;
                }
                let value = made(layouts, binary, family, ty, false).ok_or(refusal)?;
                let id = layouts
                    .texts
                    .get(value.text())
                    .ok_or(OpenRefusal::Set { owner: set.owner, patch: 0 })?
                    .schema
                    .as_ref()
                    .ok_or(OpenRefusal::Set { owner: set.owner, patch: 0 })?
                    .version();
                layouts
                    .rows
                    .insert((Box::from(family.name), ty.number, id), Row { value, stored: false, retire: false })
                    .expect("uncapped seed");
            }
            texts::clear(layouts);
        }
    }
    Ok(())
}
/// An absent kept type cannot have values (protocol/store/layouts.md, 5.1).
fn empty(store: &Store, owner: crate::Owner, ty: &ValueType) -> Option<bool> {
    for range in ty.keys {
        let from = super::whole(owner, range.from, store.limits.store.key)?;
        let to = super::whole(owner, range.to, store.limits.store.key)?;
        for space in store.stores.store.census().keyspaces() {
            if space.rows == 0 || space.prefix() >= to.as_ref() {
                continue;
            }
            let end = super::check::prefix_end(space.prefix());
            if let Some(end) = end
                && end.as_ref() <= from.as_ref()
            {
                continue;
            }
            return Some(false);
        }
    }
    Some(true)
}

pub(super) fn register(store: &Store, layouts: &mut Layouts, binary: &mut texts::Binary) -> Result<bool, OpenRefusal> {
    let mut changed = false;
    for set in store.patch_sets.sets {
        for family in set.families {
            for ty in family.types {
                let refusal = OpenRefusal::Set { owner: set.owner, patch: 0 };
                let value = made(layouts, binary, family, ty, true).ok_or(refusal.clone())?;
                let version = layouts
                    .texts
                    .get(value.text())
                    .ok_or(refusal.clone())?
                    .schema
                    .as_ref()
                    .ok_or(refusal.clone())?
                    .version();
                let mut found = None;
                let mut greatest = None;
                for ((name, number, id), row) in &layouts.rows {
                    if name.as_ref() == family.name && *number == ty.number {
                        greatest = Some(greatest.unwrap_or(0_u16).max(*id));
                        if row.value.layout() == value.layout() {
                            found = Some(*id);
                        }
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
            texts::clear(layouts);
        }
    }
    Ok(changed)
}
fn key(key: &RowKey, limit: u32) -> Option<Box<[u8]>> {
    keys::encode_own(&keys::Own::Layout { family: key.0.clone(), number: key.1, id: key.2 }, limit)
}
pub(super) fn operations(store: &Store, layouts: &mut Layouts) -> Result<(), OpenRefusal> {
    let mut used = Map::uncapped();
    for (row_key, row) in &layouts.rows {
        let key = key(row_key, store.limits.store.key).ok_or(OpenRefusal::Failed)?;
        if row.retire {
            if row.stored {
                layouts.pending.push(skein_kv::Op::Erase { key });
            }
            continue;
        }
        used.insert(*row.value.text(), ()).expect("uncapped digests");
        for digest in row.value.held() {
            used.insert(*digest, ()).expect("uncapped digests");
        }
        if !row.stored {
            let mut writer = Writer::new(skein_lib::usize_of(row.value.measure()));
            row.value.encode(&mut writer).ok().ok_or(OpenRefusal::Failed)?;
            layouts.pending.push(skein_kv::Op::Put { key, value: writer.finish() });
        }
    }
    for (digest, text) in &layouts.texts {
        let key = keys::encode_own(&keys::Own::Text(*digest), store.limits.store.key).ok_or(OpenRefusal::Failed)?;
        if used.contains_key(digest) {
            if !text.stored {
                let value = wire::Text::new(&wire::CEILINGS, wire::TextParts { text: text.bytes.clone() })
                    .ok()
                    .ok_or(OpenRefusal::Failed)?;
                let mut writer = Writer::new(skein_lib::usize_of(value.measure()));
                value.encode(&mut writer).ok().ok_or(OpenRefusal::Failed)?;
                layouts.pending.push(skein_kv::Op::Put { key, value: writer.finish() });
            }
        } else if text.stored {
            layouts.pending.push(skein_kv::Op::Erase { key });
        }
    }
    Ok(())
}
pub(super) fn size(op: &skein_kv::Op) -> Option<u64> {
    let key = u32::try_from(op.key().len()).ok()?;
    Some(u64::from(match op {
        skein_kv::Op::Put { value, .. } => skein_kv::put_bytes(key, u32::try_from(value.len()).ok()?)?,
        skein_kv::Op::Erase { .. } => skein_kv::erase_bytes(key)?,
    }))
}
