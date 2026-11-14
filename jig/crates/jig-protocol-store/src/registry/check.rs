//! Readability and census-safe retirement, before writing (protocol/store/layouts.md, 5.3–5.4).
use super::{Layouts, Row, RowKey, texts};
use crate::{OpenRefusal, Store, Unplanned};
use alloc::boxed::Box;
use skein_codec::{
    layout::{Plan, Underived, Why, Workspace},
    schema::{Schema, Span},
};
use skein_lib::List;

pub(super) fn patches(store: &Store) -> Result<(), OpenRefusal> {
    let Some(pending) = &store.directory.patch else { return Ok(()) };
    for step in pending.plan.steps() {
        for index in 0..pending.machines.len() {
            let machine = pending.machines.get(crate::patching::machines::FamilyIx(index));
            if machine.block(*step).is_none() {
                continue;
            }
            let family = store
                .patch_sets
                .sets
                .get(machine.set)
                .expect("machine set")
                .families
                .get(machine.family)
                .expect("machine family");
            if !family.types.is_empty() {
                return Err(OpenRefusal::Set { owner: step.owner, patch: step.patch });
            }
        }
    }
    Ok(())
}
fn count(store: &Store, layouts: &Layouts, index: u32, id: u16) -> Option<u64> {
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
        for (from, range) in &layouts.ranges {
            if range.ty != index || from.as_ref() >= end.as_ref() || range.to.as_ref() <= space.prefix() {
                continue;
            }
            relevant = true;
            if from.as_ref() <= through {
                through = range.to.as_ref();
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
fn prefix_end(prefix: &[u8]) -> Option<Box<[u8]>> {
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
    layouts: &Layouts,
    row: &Row,
    new: &Schema,
    ty: &crate::ValueType,
    new_held: &[&Schema],
) -> Result<Plan, Unplanned> {
    let text = layouts.texts.get(row.value.text()).ok_or(Unplanned::Text)?;
    if row.stored && !text.stored {
        return Err(Unplanned::Text);
    }
    let old = text.schema.as_ref().ok_or(Unplanned::Text)?;
    let old_held = texts::schemas(layouts, row.value.held(), row.stored).ok_or(Unplanned::Text)?;
    let variant = match row.value.variant() {
        Some(variant) => Some(variant.as_ref()),
        None => None,
    };
    let old_root = old.root(row.value.root(), variant).ok_or(Unplanned::Text)?;
    let new_root = new.root(ty.root.as_bytes(), super::variant(ty.variant)).ok_or(Unplanned::Text)?;
    // Check that the kept text and the row agree, rather than trusting its id.
    let fingerprint = match old.layout(old_root, old_held.as_slice()) {
        Ok(layout) => layout.digest(),
        Err(_) => return Err(Unplanned::Text),
    };
    if fingerprint != *row.value.layout() {
        return Err(Unplanned::Text);
    }
    match Plan::derive(old, old_root, old_held.as_slice(), new, new_root, new_held) {
        Ok(plan) => Ok(plan),
        Err(why) => Err(Unplanned::Underived(why)),
    }
}
fn job(store: &Store, key: &RowKey, row: &Row) -> bool {
    for set in store.patch_sets.sets {
        for job in set.jobs {
            if let Some(converts) = &job.converts
                && key.0.as_ref() == converts.family
                && key.1 == converts.value_type
                && row.value.layout() == &converts.from
            {
                return true;
            }
        }
    }
    false
}
pub(super) fn plans(
    store: &Store,
    layouts: &mut Layouts,
    binary: &texts::Binary,
    work: &mut Workspace,
) -> Result<(), OpenRefusal> {
    let mut planned = List::uncapped();
    let mut retired = List::uncapped();
    for (key, row) in &layouts.rows {
        let mut found = None;
        for (index, ty) in layouts.types.as_slice().iter().enumerate() {
            if ty.family == key.0.as_ref() && ty.number == key.1 {
                found = Some(u32::try_from(index).expect("type count"));
                break;
            }
        }
        let patches = match &store.directory.patch {
            Some(pending) => !pending.plan.steps().is_empty(),
            None => false,
        };
        // A patch has not yet moved the active census's version heads to the seed's ids.
        let values = if !row.stored && patches {
            None
        } else {
            match found {
                Some(index) => count(store, layouts, index, key.2),
                None => absent(store, key.2),
            }
        };
        if found.is_none() && values == Some(0) {
            retired.push(key.clone()).expect("uncapped retirements");
            continue;
        }
        let mut why = Unplanned::Undeclared;
        if let Some(index) = found {
            let ty = layouts.types.get(index).expect("found type");
            if ty.binary != key.2 && values == Some(0) {
                retired.push(key.clone()).expect("uncapped retirements");
                continue;
            }
            let mut declaration = None;
            for set in store.patch_sets.sets {
                for family in set.families {
                    if family.name != ty.family {
                        continue;
                    }
                    for value in family.types {
                        if value.number == ty.number {
                            declaration = Some(value);
                        }
                    }
                }
            }
            let declaration = declaration.expect("binary type declared");
            let digest = binary.text(ty.family, true).expect("binary text");
            let new =
                layouts.texts.get(&digest).ok_or(OpenRefusal::Failed)?.schema.as_ref().ok_or(OpenRefusal::Failed)?;
            let held = binary.held(layouts, digest, true).ok_or(OpenRefusal::Failed)?;
            let held = texts::schemas(layouts, &held, false).ok_or(OpenRefusal::Failed)?;
            match derive(layouts, row, new, declaration, held.as_slice()) {
                Ok(plan) if work.fit(&plan) => {
                    planned.push((index, key.2, plan)).expect("uncapped plans");
                    continue;
                }
                Ok(_) => {
                    why = Unplanned::Underived(Underived {
                        why: Why::Deep,
                        decl: Span { start: 0, end: 0 },
                        part: None,
                        old: false,
                    });
                }
                Err(error) => why = error,
            }
        }
        if let Unplanned::Underived(_) = why
            && job(store, key, row)
        {
            continue;
        }
        return Err(OpenRefusal::Layout { family: key.0.clone(), value_type: key.1, id: key.2, values, why });
    }
    for key in &retired {
        layouts.rows.get_mut(key).expect("held row").retire = true;
    }
    for (index, id, plan) in planned.into_boxed() {
        layouts.types.get_mut(index).expect("type").plans.insert(id, plan).expect("uncapped plans");
    }
    Ok(())
}
