//! Owner-set admission and binary levels (protocol/store/opening-owners.md, 1).
use super::{
    schemas,
    sets::{Owner, OwnerSet, OwnerSets},
};
use skein_codec::schema::Schema;
use skein_lib::List;

pub(crate) use crate::codec::component::Levels;
pub(crate) fn level(levels: &Levels, owner: Owner) -> Option<u32> {
    match owner {
        Owner::Skein => levels.skein,
        Owner::Smith => levels.smith,
        Owner::Jig => levels.jig,
        Owner::Application => levels.application,
    }
}
pub(crate) fn set_level(levels: &mut Levels, owner: Owner, level: Option<u32>) {
    match owner {
        Owner::Skein => levels.skein = level,
        Owner::Smith => levels.smith = level,
        Owner::Jig => levels.jig = level,
        Owner::Application => levels.application = level,
    }
}
pub(crate) fn binary(sets: &OwnerSets) -> Levels {
    let mut levels = Levels { skein: None, smith: None, jig: None, application: None };
    for set in sets.sets {
        set_level(&mut levels, set.owner, set.level);
    }
    levels
}
#[expect(clippy::manual_find, reason = "step code uses no closures (programming-model.md 10.3)")]
pub(crate) fn set(sets: &OwnerSets, owner: Owner) -> Option<&OwnerSet> {
    for set in sets.sets {
        if set.owner == owner {
            return Some(set);
        }
    }
    None
}
/// One check per open (protocol/store/opening-owners.md, 1).
pub(crate) fn checked(store: &mut crate::Store) -> Result<(), Owner> {
    if let Some(verdict) = store.directory.checked {
        return verdict;
    }
    let verdict = bounds(&store.owner_sets, store.prepared_schemas);
    store.directory.checked = Some(verdict);
    verdict
}

/// The manifest counts each kept family by its name (protocol/store/README.md, 2, protocol/store/opening-owners.md, 1): the
/// set where the families pass its bounds, in number or a name's bytes, is
/// refused before a first start writes.
pub(crate) fn bounds(sets: &OwnerSets, prepared: &[Schema]) -> Result<(), Owner> {
    #[cfg(test)]
    counting::ran();
    let ceilings = &crate::component_v3::CEILINGS;
    let name = skein_lib::length::usize_of(ceilings.count_family);
    let mut families = 0_u32;
    for (index, set) in sets.sets.iter().enumerate() {
        for previous in sets.sets.get(..index).expect("earlier sets") {
            if previous.owner == set.owner {
                return Err(set.owner);
            }
        }
        for family in set.families {
            families = families.checked_add(1).expect("the ceiling stops the count first");
            if families > ceilings.manifest_counts || family.name.len() > name {
                return Err(set.owner);
            }
        }
    }
    let mut current = List::with_capacity(families);
    keys(sets, prepared, &mut current)?;
    let current = current.as_slice();
    super::types::check(sets, prepared, current)?;
    crate::jobs::admission::check(sets)?;
    check_frames(sets, prepared, current)?;
    holds(sets, prepared, current)?;
    Ok(())
}
/// The two transcript roots are owners' data, checked even at equal levels.
fn check_frames(sets: &OwnerSets, prepared: &[Schema], current: &[Schema]) -> Result<(), Owner> {
    let mut smith = None;
    for set in sets.sets {
        let refusal = set.owner;
        let mut seen = false;
        for family in set.families {
            if family.frames.is_none() {
                continue;
            }
            if seen || (set.owner != Owner::Jig && set.owner != Owner::Smith) {
                return Err(refusal);
            }
            seen = true;
            if set.owner == Owner::Smith {
                smith = Some(family.name.as_bytes());
            }
        }
    }
    for set in sets.sets {
        for family in set.families {
            let Some(frames) = &family.frames else { continue };
            let refusal = set.owner;
            if family.map != crate::KeptMap::Store {
                return Err(refusal);
            }
            for ty in family.types {
                if ty.number == frames.number {
                    return Err(refusal);
                }
            }
            for retired in family.retired {
                if retired.number == frames.number {
                    return Err(refusal);
                }
            }
            let mut parsed = None;
            let binary = schemas::frame(prepared, current, family.current.as_bytes(), &mut parsed).ok_or(refusal)?;
            if binary.spelled(binary.family()) != family.name.as_bytes() {
                return Err(refusal);
            }
            let root = binary.find(frames.root.as_bytes()).ok_or(refusal)?;
            check_frame_root(binary, root, set.owner, smith)?;
            let mut previous = None;
            for text in frames.texts {
                let mut parsed = None;
                let schema = schemas::frame(prepared, current, text.as_bytes(), &mut parsed).ok_or(refusal)?;
                let version = schema.version();
                if let Some(before) = previous
                    && Some(version) != u16::checked_add(before, 1)
                {
                    return Err(refusal);
                }
                if schema.spelled(schema.family()) != family.name.as_bytes() {
                    return Err(refusal);
                }
                let root = seed_root(schema, binary, frames.root).ok_or(refusal)?;
                check_frame_root(schema, root, set.owner, smith)?;
                previous = Some(version);
            }
            if let Some(last) = previous
                && last != binary.version()
            {
                return Err(refusal);
            }
        }
    }
    Ok(())
}

/// Find the seed's declaration, including a former spelling of the binary's root.
pub(crate) fn seed_root(schema: &Schema, binary: &Schema, name: &str) -> Option<skein_codec::schema::DeclIx> {
    if let Some(root) = schema.find(name.as_bytes()) {
        return Some(root);
    }
    let current = binary.find(name.as_bytes())?;
    for former in binary.was(skein_codec::schema::Named::Declaration(current)) {
        if let Some(root) = schema.find(binary.spelled(*former)) {
            return Some(root);
        }
    }
    None
}

fn check_frame_root(
    schema: &Schema,
    root: skein_codec::schema::DeclIx,
    owner: Owner,
    smith: Option<&[u8]>,
) -> Result<(), Owner> {
    use skein_codec::schema::{FieldIx, Shape};
    let (first, count) = match schema.declaration(root).shape {
        Shape::Record { versioned: true, first, count } => (first, count),
        Shape::Record { versioned: false, .. } | Shape::Enum { .. } => return Err(owner),
    };
    if owner != Owner::Jig {
        return Ok(());
    }
    let mut held = false;
    for offset in 0..count {
        let field = FieldIx::new(first.index().checked_add(offset).expect("field"));
        if let Some(other) = held_family(schema, schema.field(field).ty) {
            if smith != Some(other) || held {
                return Err(owner);
            }
            held = true;
        }
    }
    if held { Ok(()) } else { Err(owner) }
}

fn held_family(schema: &Schema, mut ty: skein_codec::schema::TypeIx) -> Option<&[u8]> {
    use skein_codec::schema::{Held, Type};
    for _ in 0..schema.text().len() {
        match schema.ty(ty) {
            Type::Option(item) | Type::List { item, .. } => ty = item,
            Type::Bytes { held: Held::Record { family, .. }, .. }
            | Type::Secret { held: Held::Record { family, .. }, .. } => return Some(schema.spelled(family)),
            Type::U8
            | Type::U16
            | Type::U32
            | Type::U64
            | Type::Bool
            | Type::Duration
            | Type::Fixed(_)
            | Type::Secret { held: Held::Opaque | Held::Pieces(_), .. }
            | Type::Text(_)
            | Type::Named(_)
            | Type::Bytes { held: Held::Opaque | Held::Pieces(_), .. } => return None,
        }
    }
    None
}

/// Incremental transitive closure. Adding i -> j closes a cycle exactly when
/// j already reaches i; each row is a manifest-bounded, 64-family bit set.
fn holds(sets: &OwnerSets, prepared: &[Schema], current: &[Schema]) -> Result<(), Owner> {
    use skein_codec::schema::{DeclIx, FieldIx, Shape};
    let mut families = List::with_capacity(crate::component_v3::CEILINGS.manifest_counts);
    for set in sets.sets {
        for family in set.families {
            families.push(family).expect("bounds checked the manifest count");
        }
    }
    let mut reach = List::with_capacity(families.len());
    for _ in &families {
        reach.push(0_u64).expect("one row per family");
    }
    let mut index = 0_u32;
    for set in sets.sets {
        let refusal = set.owner;
        for family in set.families {
            let own = 1_u64.checked_shl(index).expect("at most 64 families");
            let mut texts = List::uncapped();
            texts.push(family.current).expect("uncapped admission texts");
            if let Some(frames) = &family.frames {
                for text in frames.texts {
                    texts.push(*text).expect("uncapped admission texts");
                }
            }
            for text in &texts {
                let mut parsed = None;
                let Some(schema) = schemas::frame(prepared, current, text.as_bytes(), &mut parsed) else {
                    return Err(refusal);
                };
                for decl in 0..schema.declarations() {
                    match schema.declaration(DeclIx::new(decl)).shape {
                        Shape::Enum { .. } => {}
                        Shape::Record { first, count, .. } => {
                            for offset in 0..count {
                                let field = FieldIx::new(first.index().checked_add(offset).expect("schema field"));
                                let Some(name) = held_family(schema, schema.field(field).ty) else { continue };
                                for target in 0..families.len() {
                                    if families.get(target).expect("family").name.as_bytes() != name {
                                        continue;
                                    }
                                    let to = 1_u64.checked_shl(target).expect("at most 64 families");
                                    let reached = *reach.get(target).expect("target row");
                                    if index == target || reached & own != 0 {
                                        return Err(refusal);
                                    }
                                    let added = to | reached;
                                    for source in 0..families.len() {
                                        let row = reach.get_mut(source).expect("source row");
                                        if source == index || *row & own != 0 {
                                            *row |= added;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            index = index.checked_add(1).expect("manifest-bound families");
        }
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod counting;

/// Each range belongs to a declaration in the binary's text, and no two
/// families claim the same key (protocol/store/opening-owners.md, 1).
fn keys(sets: &OwnerSets, prepared: &[Schema], current: &mut List<Schema>) -> Result<(), Owner> {
    for set in sets.sets {
        let refusal = set.owner;
        for family in set.families {
            let schema = schemas::current(prepared, current, family.current.as_bytes()).ok_or(refusal)?;
            let mut through = None;
            for keys in family.keys {
                if keys.from >= keys.to || schema.find(keys.root.as_bytes()).is_none() {
                    return Err(refusal);
                }
                if let Some(previous) = through
                    && previous > keys.from
                {
                    return Err(refusal);
                }
                through = Some(keys.to);
                if set.owner != Owner::Application {
                    let reserved: &[&[u8]] = match family.map {
                        crate::KeptMap::Store => &[&[1, 4, 3]],
                        crate::KeptMap::Secrets => &[],
                    };
                    for key in reserved {
                        if keys.from <= *key && *key < keys.to {
                            return Err(refusal);
                        }
                    }
                }
                for other in sets.sets {
                    // Application ranges are relative to their map's reserved prefix.
                    if (set.owner == Owner::Application) != (other.owner == Owner::Application) {
                        continue;
                    }
                    for other_family in other.families {
                        if core::ptr::eq(family, other_family) || family.map != other_family.map {
                            continue;
                        }
                        for other_keys in other_family.keys {
                            if keys.from < other_keys.to && other_keys.from < keys.to {
                                return Err(refusal);
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
