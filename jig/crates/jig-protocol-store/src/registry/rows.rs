//! Registry rows and their commit operations (protocol/store/layouts.md, 3 and 5).
use super::{Digest, Frame, Layouts, Order, Range, Row, RowKey, Text, Type, texts};
use crate::{Frames, KeptFamily, OpenRefusal, Store, ValueType, keys, registry_v1 as wire};
use alloc::boxed::Box;
use skein_codec::Record as _;
use skein_lib::{List, Map, Reader};

/// Loading precedes the same registration refusal in a start and a rehearsal (layouts.md, 5.1).
pub(super) fn open(store: &mut Store, layouts: &mut Layouts, empty: bool) -> Result<bool, OpenRefusal> {
    load(store, layouts)?;
    let mut registered = false;
    let mut frames = false;
    for set in store.owner_sets.sets {
        for family in set.families {
            registered |= !family.types.is_empty() || !family.retired.is_empty();
            frames |= family.frames.is_some();
        }
    }
    if layouts.rows.is_empty() && !empty && registered {
        return Err(OpenRefusal::OlderBuild { why: crate::Older::Unregistered });
    }
    Ok(registered || frames)
}

fn load(store: &mut Store, layouts: &mut Layouts) -> Result<(), OpenRefusal> {
    let ranges = [
        skein_kv::Range { start: Box::from([0, 6]), end: Some(Box::from([0, 8])) },
        skein_kv::Range { start: Box::from([0, 14]), end: Some(Box::from([0, 15])) },
    ];
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
            if row.row.key.starts_with(&[0, 14]) {
                let key = keys::decode_order(&row.row.key).ok_or(OpenRefusal::Failed)?;
                let value = wire::Order::decode(&wire::CEILINGS, &mut Reader::new(&row.row.value))
                    .ok()
                    .ok_or(OpenRefusal::Failed)?;
                layouts.orders.insert(key, Order { value, dirty: false }).expect("uncapped orders");
                continue;
            }
            match keys::decode_own(&row.row.key) {
                Some(keys::Own::Layout { family, number, id }) => {
                    let value = wire::Layout::decode(&wire::CEILINGS, &mut Reader::new(&row.row.value))
                        .ok()
                        .ok_or(OpenRefusal::Failed)?;
                    layouts
                        .rows
                        .insert((family, number, id), Row { value, stored: true, retire: false, later: false })
                        .expect("uncapped registry");
                }
                Some(keys::Own::Text(digest)) => {
                    let value = wire::Text::decode(&wire::CEILINGS, &mut Reader::new(&row.row.value))
                        .ok()
                        .ok_or(OpenRefusal::Failed)?;
                    let bytes = value.text;
                    layouts.texts.insert(digest, Text::stored(bytes)).expect("uncapped texts");
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
    layout_row(layouts, binary, text, ty.root.as_bytes(), ty.variant, held)
}

fn layout_row(
    layouts: &Layouts,
    binary: &mut texts::Binary,
    text: Digest,
    root: &[u8],
    variant: Option<&str>,
    held: List<Digest>,
) -> Option<wire::Layout> {
    let variant = super::variant(variant);
    let layout = binary.layout(layouts, text, root, variant, &held)?;
    let value = wire::Layout {
        root: Box::from(root),
        variant: match variant {
            Some(name) => Some(Box::from(name)),
            None => None,
        },
        layout,
        text,
        held,
    };
    value.measure(&wire::CEILINGS).ok()?;
    Some(value)
}

/// Pre-registry frame heads spell their versions. Seed these ids even if
/// two versions have the same fingerprint: both heads may occur on disk.
pub(super) fn seed_frames(
    layouts: &mut Layouts,
    binary: &mut texts::Binary,
    family: &KeptFamily,
    frames: &Frames,
) -> Option<bool> {
    if in_type(layouts, family.name, frames.number).next().is_some() {
        return Some(false);
    }
    let current = binary.text(family.name)?;
    texts::parse(layouts, current)?;
    let mut changed = false;
    for &seed in frames.texts {
        let text = texts::keep(layouts, seed.as_bytes(), false)?;
        texts::parse(layouts, text)?;
        let schema = layouts.texts.get(&text)?.schema()?;
        let current = layouts.texts.get(&current)?.schema()?;
        let root = crate::upgrade::admission::seed_root(schema, current, frames.root)?;
        let value =
            layout_row(layouts, binary, text, schema.spelled(schema.declaration(root).name), None, List::uncapped())?;
        let version = schema.version();
        insert(layouts, family.name, frames.number, version, value);
        changed = true;
    }
    Some(changed)
}

pub(super) fn register(store: &Store, layouts: &mut Layouts, binary: &mut texts::Binary) -> Result<bool, OpenRefusal> {
    layouts.owners = store.owner_sets;
    let mut changed = false;
    for set in store.owner_sets.sets {
        for family in set.families {
            for ty in family.types {
                let refusal = OpenRefusal::Set { owner: set.owner };
                let value = made(layouts, binary, family, ty).ok_or(refusal.clone())?;
                let (index, added) = register_type(layouts, family.name, ty.number, value).ok_or(refusal.clone())?;
                changed |= added;
                for range in ty.keys {
                    let from = super::whole(set.owner, range.from, store.limits.store.key).ok_or(refusal.clone())?;
                    let to = super::whole(set.owner, range.to, store.limits.store.key).ok_or(refusal.clone())?;
                    layouts.ranges.insert(from, Range { to, ty: index }).expect("uncapped ranges");
                }
            }
        }
    }
    for set in store.owner_sets.sets {
        for family in set.families {
            let Some(frames) = &family.frames else { continue };
            let refusal = OpenRefusal::Set { owner: set.owner };
            let slot = Frame::owner(set.owner).ok_or(refusal.clone())?;
            changed |= seed_frames(layouts, binary, family, frames).ok_or(refusal.clone())?;
            let text = binary.text(family.name).ok_or(refusal.clone())?;
            texts::parse(layouts, text).ok_or(refusal.clone())?;
            // The turn is opaque to the body plan; it moves by its own id.
            let value = layout_row(layouts, binary, text, frames.root.as_bytes(), None, List::uncapped())
                .ok_or(refusal.clone())?;
            let (index, added) = register_type(layouts, family.name, frames.number, value).ok_or(refusal)?;
            layouts.frame_types.set(slot, index);
            changed |= added;
        }
    }
    Ok(changed)
}

fn in_type<'a>(
    layouts: &'a Layouts,
    family: &str,
    number: u16,
) -> alloc::collections::btree_map::Range<'a, RowKey, Row> {
    let first = (Box::from(family), number, u16::MIN);
    let last = (Box::from(family), number, u16::MAX);
    layouts.rows.range(first..=last)
}

fn insert(layouts: &mut Layouts, family: &str, number: u16, id: u16, value: wire::Layout) {
    layouts
        .rows
        .insert((Box::from(family), number, id), Row { value, stored: false, retire: false, later: false })
        .expect("uncapped registration");
}

fn register_type(layouts: &mut Layouts, family: &'static str, number: u16, value: wire::Layout) -> Option<(u32, bool)> {
    let version = layouts.texts.get(&value.text)?.schema()?.version();
    let ordered = order(layouts, family, number, value.layout);
    let mut found = None;
    let mut greatest = None;
    for ((_, _, id), row) in in_type(layouts, family, number) {
        greatest = Some(*id);
        if row.value.layout == value.layout {
            found = Some(*id);
        }
    }
    let id = match found {
        Some(id) => id,
        None => {
            let id = match greatest {
                Some(id) => id.checked_add(1)?,
                None => {
                    layouts.introduced.push((family, number)).expect("uncapped value types");
                    version
                }
            };
            insert(layouts, family, number, id, value);
            id
        }
    };
    let index = layouts.types.len();
    layouts
        .types
        .push(Type { version, binary: id, keeping: List::uncapped(), plans: Map::uncapped() })
        .expect("uncapped types");
    Some((index, ordered || found.is_none()))
}

// Missing orders are seeded from the stored ids, before registration can insert a new row.
fn order(layouts: &mut Layouts, family: &str, number: u16, fingerprint: Digest) -> bool {
    let key = (Box::from(family), number);
    if let Some(order) = layouts.orders.get_mut(&key) {
        if remember(&mut order.value, fingerprint) {
            order.dirty = true;
            return true;
        }
        return false;
    }
    let mut value = wire::Order { layouts: List::with_capacity(wire::CEILINGS.order_layouts) };
    for (_, row) in in_type(layouts, family, number) {
        remember(&mut value, row.value.layout);
    }
    remember(&mut value, fingerprint);
    layouts.orders.insert(key, Order { value, dirty: true }).expect("uncapped orders");
    true
}

fn remember(order: &mut wire::Order, fingerprint: Digest) -> bool {
    if order.layouts.as_slice().contains(&fingerprint) {
        return false;
    }
    order.layouts.raise(wire::CEILINGS.order_layouts);
    if order.layouts.push(fingerprint).is_err() {
        let mut next = fingerprint;
        for old in order.layouts.iter_mut().rev() {
            core::mem::swap(old, &mut next);
        }
    }
    true
}

pub(super) fn key(key: &RowKey, limit: u32) -> Option<Box<[u8]>> {
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
    let orders = core::mem::replace(&mut layouts.orders, Map::uncapped());
    for ((family, number), order) in &orders {
        if order.dirty {
            let key = keys::order(family, *number, bound).ok_or(OpenRefusal::Failed)?;
            let value = order.value.encode(&wire::CEILINGS).ok().ok_or(OpenRefusal::Failed)?;
            layouts.pending.push(skein_kv::Op::Put { key, value });
        }
    }
    // Keep the rows and texts until jobs end: a written job's first page names them.
    // A retirement does not stop protecting its text unless its erase enters a writing open's batch.
    let mut row_keys = List::uncapped();
    for (key, _) in &layouts.rows {
        row_keys.push(key.clone()).expect("uncapped row keys");
    }
    for key in &row_keys {
        let row = layouts.rows.get_mut(key).expect("held row");
        row.stored |= !row.retire;
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
