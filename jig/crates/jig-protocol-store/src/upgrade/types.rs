//! Value types checked at set admission (protocol/store/layouts.md, 2).
use crate::{KeptFamily, KeptMap, Owner, OwnerSets};
use skein_codec::schema;
use skein_lib::Map;

pub(crate) fn check(sets: &OwnerSets, prepared: &[schema::Schema]) -> Result<(), Owner> {
    for set in sets.sets {
        let refusal = set.owner;
        for family in set.families {
            if family.types.is_empty() && family.retired.is_empty() {
                continue;
            }
            if family.map != KeptMap::Store {
                return Err(refusal);
            }
            let mut parsed = None;
            let Some(current) = super::schemas::get(prepared, family.current.as_bytes(), &mut parsed) else {
                return Err(refusal);
            };
            let mut ranges = Map::uncapped();
            let mut first = None;
            for (index, ty) in family.types.iter().enumerate() {
                for previous in family.types.get(..index).expect("earlier types") {
                    if previous.number == ty.number {
                        return Err(refusal);
                    }
                }
                let variant = match ty.variant {
                    Some(name) => Some(name.as_bytes()),
                    None => None,
                };
                let root = current.root(ty.root.as_bytes(), variant).ok_or(refusal)?;
                if current.layout(root, &[]).is_err() {
                    return Err(refusal);
                }
                let start = ty.keys.first().ok_or(refusal)?.from;
                if let Some(previous) = first
                    && previous >= start
                {
                    return Err(refusal);
                }
                first = Some(start);
                let mut through = None;
                for range in ty.keys {
                    if range.from >= range.to {
                        return Err(refusal);
                    }
                    if let Some(previous) = through
                        && previous > range.from
                    {
                        return Err(refusal);
                    }
                    through = Some(range.to);
                    if ranges.insert(range.from, range.to).expect("uncapped ranges").is_some() {
                        return Err(refusal);
                    }
                }
            }
            let mut first = None;
            for (index, retired) in family.retired.iter().enumerate() {
                for ty in family.types {
                    if ty.number == retired.number {
                        return Err(refusal);
                    }
                }
                for previous in family.retired.get(..index).expect("earlier retired types") {
                    if previous.number == retired.number {
                        return Err(refusal);
                    }
                }
                let start = retired.keys.first().ok_or(refusal)?.from;
                if let Some(previous) = first
                    && previous >= start
                {
                    return Err(refusal);
                }
                first = Some(start);
                let mut through = None;
                for range in retired.keys {
                    if range.from >= range.to {
                        return Err(refusal);
                    }
                    if let Some(previous) = through
                        && previous > range.from
                    {
                        return Err(refusal);
                    }
                    through = Some(range.to);
                    if ranges.insert(range.from, range.to).expect("uncapped ranges").is_some() {
                        return Err(refusal);
                    }
                }
            }
            if !covers(family, &ranges) {
                return Err(refusal);
            }
        }
    }
    Ok(())
}

// Types may interleave: settings' Changed and Revision each occupy several
// ranges. Compare their ordered union to Keys to match the family's ranges.
fn covers(family: &KeptFamily, ranges: &Map<&'static [u8], &'static [u8]>) -> bool {
    let mut keys = family.keys.iter();
    let mut key = keys.next();
    let mut from = match key {
        Some(key) => Some(key.from),
        None => None,
    };
    for (&start, &to) in ranges {
        if from != Some(start) {
            return false;
        }
        // Every turn consumes a declared range or ends this type's range.
        for _ in 0..family.keys.len() {
            let Some(current) = key else { return false };
            if to < current.to {
                from = Some(to);
                break;
            }
            let end = current.to;
            key = keys.next();
            from = match key {
                Some(key) => Some(key.from),
                None => None,
            };
            if to == end {
                break;
            }
            if from != Some(end) {
                return false;
            }
        }
    }
    key.is_none()
}
