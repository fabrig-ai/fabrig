//! Readability and census-safe retirement, before writing (protocol/store/layouts.md, 5.3–5.4).
use super::{Frame, Layouts, Row, RowKey, texts};
use crate::{OpenRefusal, OwnerSets, Store, Unplanned, ValueType};
use alloc::boxed::Box;
use skein_codec::{
    layout::{Mode, Plan, Underived, Why, Workspace},
    schema::Span,
};
use skein_lib::List;

mod audit;
mod held;
pub use held::{HeldLayout, judge_held_layout};

pub(super) fn count(store: &Store, layouts: &Layouts, index: u32, id: u16) -> Option<u64> {
    // Only this type's ranges can contribute to its count. Select them once,
    // in key order, rather than searching all registry ranges per keyspace.
    let mut ranges = List::uncapped();
    for (from, range) in &layouts.ranges {
        if range.ty == index {
            ranges.push((from.as_ref(), range.to.as_ref())).expect("uncapped type ranges");
        }
    }
    let mut count = 0_u64;
    for space in store.stores.store.census().keyspaces() {
        if space.rows == 0 {
            continue;
        }
        if space.prefix().is_empty() {
            return None;
        }
        let end = prefix_end(space.prefix())?;
        let mut relevant = false;
        let mut through = space.prefix();
        for &(from, to) in &ranges {
            if from >= end.as_ref() {
                break;
            }
            if to <= space.prefix() {
                continue;
            }
            relevant = true;
            if from <= through {
                through = to;
            }
        }
        if !relevant {
            continue;
        }
        if through < end.as_ref() {
            return None;
        }
        for head in space.heads() {
            match head.head {
                Some(head_id) if head_id == id => count = count.checked_add(head.rows)?,
                None if head.rows != 0 => return None,
                Some(_) | None => {}
            }
        }
    }
    Some(count)
}
fn absent(store: &Store, id: u16) -> Option<u64> {
    for space in store.stores.store.census().keyspaces() {
        if space.prefix().is_empty() && space.rows != 0 {
            return None;
        }
        for head in space.heads() {
            if (head.head == Some(id) || head.head.is_none()) && head.rows != 0 {
                return None;
            }
        }
    }
    Some(0)
}
pub(super) fn prefix_end(prefix: &[u8]) -> Option<Box<[u8]>> {
    for index in (0..prefix.len()).rev() {
        let byte = *prefix.get(index)?;
        if let Some(next) = byte.checked_add(1) {
            let mut end: Box<[u8]> = Box::from(prefix.get(..index.checked_add(1)?)?);
            *end.last_mut()? = next;
            return Some(end);
        }
    }
    None
}
fn derive(
    binary: &mut texts::Binary,
    layouts: &Layouts,
    row: &Row,
    digest: super::Digest,
    ty: &ValueType,
    held: &List<super::Digest>,
    mode: Option<Mode>,
) -> Result<Option<Plan>, Unplanned> {
    let text = layouts.texts.get(&row.value.text).ok_or(Unplanned::Text)?;
    if row.stored && !text.stored {
        return Err(Unplanned::Text);
    }
    let old = text.schema().ok_or(Unplanned::Text)?;
    let old_held = texts::schemas(layouts, &row.value.held, row.stored).ok_or(Unplanned::Text)?;
    let variant = row.value.variant.as_deref();
    let old_root = old.root(&row.value.root, variant).ok_or(Unplanned::Text)?;
    let new = layouts.texts.get(&digest).ok_or(Unplanned::Text)?.schema().ok_or(Unplanned::Text)?;
    let new_root = new.root(ty.root.as_bytes(), super::variant(ty.variant)).ok_or(Unplanned::Text)?;
    let new_held = texts::schemas(layouts, held, false).ok_or(Unplanned::Text)?;
    // Check that the kept text and the row agree, rather than trusting its id.
    let fingerprint =
        binary.layout(layouts, row.value.text, &row.value.root, variant, &row.value.held).ok_or(Unplanned::Text)?;
    if fingerprint != row.value.layout {
        return Err(Unplanned::Text);
    }
    // The id or fingerprint alone cannot prove identical conversion inputs.
    // Digest checks have validated the stored bytes, including every held text.
    if row.value.text == digest
        && row.value.root.as_ref() == ty.root.as_bytes()
        && variant == super::variant(ty.variant)
        && &row.value.held == held
        && binary.shallow(layouts, digest, new_root.decl, held)
    {
        return Ok(None);
    }
    #[cfg(test)]
    super::counting::plan();
    let plan = match mode {
        Some(mode) => Plan::between(mode, old, old_root, old_held.as_slice(), new, new_root, new_held.as_slice()),
        None => Plan::derive(old, old_root, old_held.as_slice(), new, new_root, new_held.as_slice()),
    };
    match plan {
        Ok(plan) => Ok(Some(plan)),
        Err(why) => Err(Unplanned::Underived(why)),
    }
}
fn conversion(job: &crate::Job, family: &str, number: u16) -> Option<crate::Layouts> {
    let converts = job.converts.as_ref()?;
    if converts.family == family && converts.value_type == number { Some(converts.from) } else { None }
}
fn job(store: &mut Store, key: &RowKey, row: &Row, unplanned: bool) -> Result<Option<&'static str>, OpenRefusal> {
    let mut one = None;
    for set in store.owner_sets.sets {
        for job in set.jobs {
            if let Some(from) = conversion(job, &key.0, key.1) {
                match from {
                    crate::Layouts::One(from) if row.value.layout == from => one = Some(job.name),
                    crate::Layouts::Unplanned if unplanned && !crate::jobs::kept::done(store, set.owner, job)? => {
                        return Ok(Some(job.name));
                    }
                    crate::Layouts::One(_) | crate::Layouts::Unplanned => {}
                }
            }
        }
    }
    Ok(one)
}
/// Files and logs stay held even when the map census is empty (layouts.md, 1).
pub(super) fn history_type(sets: &OwnerSets, family: &str, number: u16) -> bool {
    if frame_type(sets, family, number).is_some() {
        return true;
    }
    for held in crate::owner_set::STORE_FAMILIES {
        if held.name != family {
            continue;
        }
        for ty in held.types {
            if ty.number != number {
                continue;
            }
            if family == "jig-store-tasks" && ty.root == "Row" && ty.variant.is_none() {
                return true;
            }
            for range in ty.keys {
                for logged in crate::keys::LOGGED {
                    if range.from < logged.to && logged.from < range.to {
                        return true;
                    }
                }
            }
        }
    }
    false
}
/// The two frame types have no key ranges and are planned separately from
/// any map type of the same family, including map records that hold turns.
fn frame_type(sets: &OwnerSets, name: &str, number: u16) -> Option<(Frame, ValueType)> {
    for set in sets.sets {
        for family in set.families {
            if family.name == name
                && let Some(frames) = family.frames
                && frames.number == number
            {
                return Some((
                    Frame::owner(set.owner)?,
                    ValueType { number, root: frames.root, variant: None, keys: &[] },
                ));
            }
        }
    }
    None
}
fn declaration(
    store: &Store,
    layouts: &Layouts,
    key: &RowKey,
    frame: Option<(Frame, ValueType)>,
) -> Option<(u32, ValueType)> {
    if let Some((slot, ty)) = frame {
        return Some((layouts.frame_types.get(slot).expect("registered frame type"), ty));
    }
    let mut index = 0_u32;
    for set in store.owner_sets.sets {
        for family in set.families {
            for ty in family.types {
                if family.name == key.0.as_ref() && ty.number == key.1 {
                    return Some((index, *ty));
                }
                index = index.checked_add(1).expect("type count");
            }
        }
    }
    None
}

fn mode(at_binary: bool, later: bool, held: bool) -> Option<Mode> {
    if at_binary {
        return None;
    }
    if later {
        Some(if held { Mode::Read } else { Mode::Keep })
    } else if held {
        None
    } else {
        Some(Mode::Read)
    }
}

fn is_later(layouts: &Layouts, key: &RowKey, row: &Row, binary: u16) -> bool {
    let binary = layouts.rows.get(&(key.0.clone(), key.1, binary)).expect("registered binary row").value.layout;
    let Some(order) = layouts.orders.get(&(key.0.clone(), key.1)) else { return false };
    let mut after_binary = false;
    for fingerprint in &order.value.layouts {
        if *fingerprint == row.value.layout {
            return after_binary;
        }
        if *fingerprint == binary {
            after_binary = true;
        }
    }
    false
}

pub(super) fn plans(
    store: &mut Store,
    layouts: &mut Layouts,
    binary: &mut texts::Binary,
    work: &mut Workspace,
    frame_work: &mut Workspace,
) -> Result<audit::Check, OpenRefusal> {
    let mut audit = audit::Check::new();
    let mut keys = List::uncapped();
    for (key, _) in &layouts.rows {
        keys.push(key.clone()).expect("uncapped row keys");
    }
    for key in &keys {
        let row = layouts.rows.get(key).expect("held row");
        if row.retire {
            continue;
        }
        let frame = frame_type(&store.owner_sets, &key.0, key.1);
        let held_outside_map = history_type(&store.owner_sets, &key.0, key.1);
        let Some((index, declaration)) = declaration(store, layouts, key, frame) else {
            let values = if held_outside_map { None } else { absent(store, key.2) };
            if values == Some(0) {
                layouts.rows.get_mut(key).expect("held row").retire = true;
                continue;
            }
            return Err(OpenRefusal::Layout {
                family: key.0.clone(),
                value_type: key.1,
                id: key.2,
                values,
                why: Unplanned::Undeclared,
            });
        };
        let binary_id = layouts.types.get(index).expect("found type").binary;
        let at_binary = key.2 == binary_id;
        let later = !at_binary && is_later(layouts, key, row, binary_id);
        layouts.rows.get_mut(key).expect("held row").later = later;
        let values = if held_outside_map { None } else { count(store, layouts, index, key.2) };
        if !at_binary && values == Some(0) {
            layouts.rows.get_mut(key).expect("held row").retire = true;
            continue;
        }
        let digest = binary.text(&key.0).expect("binary text");
        let held =
            if frame.is_some() { List::uncapped() } else { binary.held(layouts, digest).ok_or(OpenRefusal::Failed)? };
        let readable = texts::row(layouts, key);
        let row = layouts.rows.get(key).expect("held row");
        let mode = mode(at_binary, later, held_outside_map);
        let plan = if readable.is_some() && (frame.is_none() || row.value.held.is_empty()) {
            derive(binary, layouts, row, digest, &declaration, &held, mode)
        } else {
            Err(Unplanned::Text)
        };
        let work = if frame.is_some() { &mut *frame_work } else { &mut *work };
        let why = match plan {
            Ok(None) => {
                if !at_binary {
                    let ty = layouts.types.get_mut(index).expect("found type");
                    ty.keeping.push(key.2).expect("uncapped keeping ids");
                }
                continue;
            }
            Ok(Some(plan)) if at_binary || plan.keeps() || work.fit(&plan) => {
                if !held_outside_map && let Some(mode) = mode {
                    // A named job may take a layout even when a plan reaches it.
                    let refuses = plan.refuses();
                    if let Some(name) = job(store, key, row, refuses)? {
                        layouts.written.insert(key.clone(), name).expect("uncapped written ids");
                        continue;
                    }
                    if refuses {
                        audit.add(index, key, values, mode);
                    }
                }
                // A binary id is reheaded before any plan is consulted, but
                // its row still needs the text and fingerprint checks above.
                if !at_binary {
                    let ty = layouts.types.get_mut(index).expect("found type");
                    if plan.keeps() {
                        ty.keeping.push(key.2).expect("uncapped keeping ids");
                    } else {
                        ty.plans.insert(key.2, plan).expect("uncapped plans");
                    }
                }
                continue;
            }
            Ok(Some(_)) => Unplanned::Underived(Underived {
                why: Why::Deep,
                decl: Span { start: 0, end: 0 },
                part: None,
                old: false,
            }),
            Err(why) => why,
        };
        if let Unplanned::Underived(_) = why
            && !held_outside_map
            && !at_binary
            && let Some(name) = job(store, key, row, true)?
        {
            layouts.written.insert(key.clone(), name).expect("uncapped written ids");
            continue;
        }
        return Err(OpenRefusal::Layout { family: key.0.clone(), value_type: key.1, id: key.2, values, why });
    }
    texts::clear_all(layouts);
    Ok(audit)
}

pub(super) fn retired(store: &mut Store, layouts: &mut Layouts) -> Result<(), OpenRefusal> {
    for set in store.owner_sets.sets {
        for family in set.families {
            for ty in family.retired {
                for range in ty.keys {
                    let from =
                        super::whole(set.owner, range.from, store.limits.store.key).ok_or(OpenRefusal::Failed)?;
                    let to = super::whole(set.owner, range.to, store.limits.store.key).ok_or(OpenRefusal::Failed)?;
                    let ranges = [skein_kv::Range { start: from.clone(), end: Some(to.clone()) }];
                    let page = match crate::pages::reader::walk_stored(store, &ranges, None, 1) {
                        Ok(page) => page,
                        Err(failure) => {
                            store.open_failed(crate::restore::OpenFailure::Data(failure));
                            return Err(OpenRefusal::Failed);
                        }
                    };
                    if let Some(row) = page.rows.first() {
                        return Err(OpenRefusal::Layout {
                            family: Box::from(family.name),
                            value_type: ty.number,
                            id: super::head(&row.row.value).unwrap_or(0),
                            values: None,
                            why: Unplanned::Retired,
                        });
                    }
                    layouts.retired.insert(from, to).expect("uncapped retired ranges");
                }
                let mut retired = List::uncapped();
                for (key, _) in &layouts.rows {
                    if key.0.as_ref() == family.name && key.1 == ty.number {
                        retired.push(key.clone()).expect("uncapped retired rows");
                    }
                }
                for key in &retired {
                    layouts.rows.get_mut(key).expect("retired row").retire = true;
                }
            }
        }
    }
    Ok(())
}
