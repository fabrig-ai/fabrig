//! Jobs are checked with their sets, before the open writes (protocol/store/jobs.md, 1).
use crate::patching::plan::Unplanned;
use crate::{Job, KeptMap, KeyRange, PatchSet, PatchSets};

pub(crate) fn check(sets: &PatchSets) -> Result<(), Unplanned> {
    for set in sets.sets {
        let refusal = Unplanned { owner: set.owner, patch: 0 };
        for (index, job) in set.jobs.iter().enumerate() {
            if job.name.len() > 64 || (!job.settles && job.keys.is_empty()) {
                return Err(refusal);
            }
            for previous in set.jobs.get(..index).expect("earlier jobs") {
                if previous.name == job.name {
                    return Err(refusal);
                }
            }
            if job.settles {
                if !job.keys.is_empty() || job.converts.is_some() || !job.counts.is_empty() {
                    return Err(refusal);
                }
                let mut registered = false;
                for family in set.families {
                    registered |= !family.types.is_empty() || !family.retired.is_empty();
                }
                if !registered {
                    return Err(refusal);
                }
                for previous in set.jobs.get(..index).expect("earlier jobs") {
                    if previous.settles {
                        return Err(refusal);
                    }
                }
            }
            if !counts(job) || !ranges(set, job) || !converts(set, job) {
                return Err(refusal);
            }
        }
    }
    Ok(())
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
fn ranges(set: &PatchSet, job: &Job) -> bool {
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
fn in_map(set: &PatchSet, range: &KeyRange) -> bool {
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
fn converts(set: &PatchSet, job: &Job) -> bool {
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
