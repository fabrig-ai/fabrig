//! Jobs are checked with their sets, before the open writes (protocol/store/jobs.md, 1).
use crate::{Job, KeptMap, KeyRange, Layouts, Owner, OwnerSet, OwnerSets};

pub(crate) fn check(sets: &OwnerSets) -> Result<(), Owner> {
    for (set_index, set) in sets.sets.iter().enumerate() {
        let refusal = set.owner;
        for (index, job) in set.jobs.iter().enumerate() {
            if job.name.len() > 64 || job.keys.is_empty() {
                return Err(refusal);
            }
            for previous in set.jobs.get(..index).expect("earlier jobs") {
                if previous.name == job.name {
                    return Err(refusal);
                }
            }
            if !counts(job) || !ranges(set, job) || !converts(set, job) {
                return Err(refusal);
            }
            for earlier_set in sets.sets.get(..=set_index).expect("sets through this one") {
                let earlier = if earlier_set.owner == set.owner {
                    set.jobs.get(..index).expect("earlier jobs")
                } else {
                    earlier_set.jobs
                };
                for previous in earlier {
                    if overlaps(previous, job) {
                        return Err(refusal);
                    }
                }
            }
        }
    }
    Ok(())
}
fn overlaps(previous: &Job, job: &Job) -> bool {
    let (Some(previous), Some(converts)) = (previous.converts, job.converts) else { return false };
    if previous.family != converts.family || previous.value_type != converts.value_type {
        return false;
    }
    match (previous.from, converts.from) {
        (Layouts::Unplanned, Layouts::Unplanned) => true,
        (Layouts::One(a), Layouts::One(b)) => a == b,
        (Layouts::One(_), Layouts::Unplanned) | (Layouts::Unplanned, Layouts::One(_)) => false,
    }
}
fn counts(job: &Job) -> bool {
    if job.counts.len() > 16 {
        return false;
    }
    for (index, name) in job.counts.iter().enumerate() {
        if name.is_empty() || name.len() > 32 {
            return false;
        }
        for byte in name.bytes() {
            if !byte.is_ascii_alphanumeric() && byte != b'-' {
                return false;
            }
        }
        if job.counts.get(..index).expect("earlier count names").contains(name) {
            return false;
        }
    }
    true
}
fn ranges(set: &OwnerSet, job: &Job) -> bool {
    let mut through: Option<&[u8]> = None;
    for range in job.keys {
        if range.from >= range.to || !in_map(set, range) {
            return false;
        }
        if let Some(through) = through
            && through > range.from
        {
            return false;
        }
        through = Some(range.to);
    }
    true
}
fn in_map(set: &OwnerSet, range: &KeyRange) -> bool {
    let mut from = range.from;
    // Each turn crosses a declared map range, even when families interleave.
    let mut turns = 0_usize;
    for family in set.families {
        turns = turns.saturating_add(family.keys.len());
    }
    for _ in 0..turns {
        let mut to = from;
        for family in set.families {
            if family.map != KeptMap::Store {
                continue;
            }
            for keys in family.keys {
                if keys.from <= from && from < keys.to {
                    to = to.max(keys.to);
                }
            }
        }
        if to >= range.to {
            return true;
        }
        if to == from {
            return false;
        }
        from = to;
    }
    false
}
fn converts(set: &OwnerSet, job: &Job) -> bool {
    let Some(converts) = job.converts else { return true };
    for family in set.families {
        if family.name == converts.family {
            if family.map != KeptMap::Store {
                return false;
            }
            for ty in family.types {
                if ty.number == converts.value_type {
                    if ty.keys.len() != job.keys.len() {
                        return false;
                    }
                    for (range, written) in ty.keys.iter().zip(job.keys) {
                        if range.from != written.from || range.to != written.to {
                            return false;
                        }
                    }
                    return true;
                }
            }
            return false;
        }
    }
    false
}
