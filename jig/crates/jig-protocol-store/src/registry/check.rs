//! Readability and census-safe retirement, before writing (protocol/store/layouts.md, 5.3–5.4).
use super::{Layouts, Row, RowKey, texts};
use crate::{OpenRefusal, Store, Unplanned};
use alloc::boxed::Box;
use skein_codec::{
    layout::{Plan, Underived, Why, Workspace},
    schema::Span,
};
use skein_lib::List;

mod held;
pub use held::{HeldLayout, judge_held_layout};

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
    ty: &crate::ValueType,
    held: &List<super::Digest>,
) -> Result<Option<Plan>, Unplanned> {
    let text = layouts.texts.get(&row.value.text).ok_or(Unplanned::Text)?;
    if row.stored && !text.stored {
        return Err(Unplanned::Text);
    }
    let old = text.schema.as_ref().ok_or(Unplanned::Text)?;
    let old_held = texts::schemas(layouts, &row.value.held, row.stored).ok_or(Unplanned::Text)?;
    let variant = row.value.variant.as_deref();
    let old_root = old.root(&row.value.root, variant).ok_or(Unplanned::Text)?;
    let new = layouts.texts.get(&digest).ok_or(Unplanned::Text)?.schema.as_ref().ok_or(Unplanned::Text)?;
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
    match Plan::derive(old, old_root, old_held.as_slice(), new, new_root, new_held.as_slice()) {
        Ok(plan) => Ok(Some(plan)),
        Err(why) => Err(Unplanned::Underived(why)),
    }
}
fn conversion(job: &crate::Job, family: &str, number: u16) -> Option<crate::Layouts> {
    let converts = job.converts.as_ref()?;
    if converts.family == family && converts.value_type == number { Some(converts.from) } else { None }
}
fn job(store: &mut Store, key: &RowKey, row: &Row) -> Result<Option<&'static str>, OpenRefusal> {
    let mut one = None;
    for set in store.owner_sets.sets {
        for job in set.jobs {
            if let Some(from) = conversion(job, &key.0, key.1) {
                match from {
                    crate::Layouts::One(from) if row.value.layout == from => one = Some(job.name),
                    crate::Layouts::Unplanned if !crate::jobs::kept::done(store, set.owner, job)? => {
                        return Ok(Some(job.name));
                    }
                    crate::Layouts::One(_) | crate::Layouts::Unplanned => {}
                }
            }
        }
    }
    Ok(one)
}
/// Log layouts remain held even when the map census is empty (logs.md, 8).
pub(super) fn history_type(family: &str, number: u16) -> bool {
    for held in crate::patch_set::STORE_FAMILIES {
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
fn declaration(store: &Store, key: &RowKey) -> Option<(u32, &'static crate::ValueType)> {
    let mut index = 0_u32;
    for set in store.owner_sets.sets {
        for family in set.families {
            for ty in family.types {
                if family.name == key.0.as_ref() && ty.number == key.1 {
                    return Some((index, ty));
                }
                index = index.checked_add(1).expect("type count");
            }
        }
    }
    None
}

pub(super) fn plans(
    store: &mut Store,
    layouts: &mut Layouts,
    binary: &mut texts::Binary,
    work: &mut Workspace,
) -> Result<(), OpenRefusal> {
    let mut keys = List::uncapped();
    for (key, _) in &layouts.rows {
        keys.push(key.clone()).expect("uncapped row keys");
    }
    for key in &keys {
        let row = layouts.rows.get(key).expect("held row");
        if row.retire {
            continue;
        }
        let declared = declaration(store, key);
        let found = match declared {
            Some((index, _)) => Some(index),
            None => None,
        };
        let values = if history_type(&key.0, key.1) {
            None
        } else {
            match found {
                Some(index) => count(store, layouts, index, key.2),
                None => absent(store, key.2),
            }
        };
        if found.is_none() && values == Some(0) {
            layouts.rows.get_mut(key).expect("held row").retire = true;
            continue;
        }
        let mut why = Unplanned::Undeclared;
        if let Some(index) = found {
            let binary_id = layouts.types.get(index).expect("found type").binary;
            if binary_id != key.2 && values == Some(0) {
                layouts.rows.get_mut(key).expect("held row").retire = true;
                continue;
            }
            let (_, declaration) = declared.expect("binary type declared");
            let digest = binary.text(&key.0).expect("binary text");
            let held = binary.held(layouts, digest).ok_or(OpenRefusal::Failed)?;
            let readable = texts::row(layouts, key);

            let row = layouts.rows.get(key).expect("held row");
            let plan = if readable.is_some() {
                derive(binary, layouts, row, digest, declaration, &held)
            } else {
                Err(Unplanned::Text)
            };
            match plan {
                Ok(None) => {
                    if key.2 != binary_id {
                        layouts
                            .types
                            .get_mut(index)
                            .expect("found type")
                            .keeping
                            .push(key.2)
                            .expect("uncapped keeping ids");
                    }
                    continue;
                }
                Ok(Some(plan)) if key.2 == binary_id || plan.keeps() || work.fit(&plan) => {
                    let ty = layouts.types.get_mut(index).expect("found type");
                    // A binary id is reheaded before any plan is consulted, but
                    // its row still needs the text and fingerprint checks above.
                    if key.2 != ty.binary {
                        if plan.keeps() {
                            ty.keeping.push(key.2).expect("uncapped keeping ids");
                        } else {
                            ty.plans.insert(key.2, plan).expect("uncapped plans");
                        }
                    }
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
            && !history_type(&key.0, key.1)
            && let Some(name) = job(store, key, layouts.rows.get(key).expect("held row"))?
        {
            layouts.written.insert(key.clone(), name).expect("uncapped written ids");
            continue;
        }
        return Err(OpenRefusal::Layout { family: key.0.clone(), value_type: key.1, id: key.2, values, why });
    }
    texts::clear_all(layouts);
    Ok(())
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
