//! Sets of project charter words (jig/docs/design/domain/engine/project-charters.md, 2 and 6).

use super::check::{WordsFault, words_fault};
use crate::{CharterPart, Limits};
use alloc::boxed::Box;
use jig_domain_people::settings::{NAME_BYTES, is_name};
use skein_lib::{Map, Set, len_u64, usize_of};

pub const SOURCE_BYTES: u32 = 256;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProjectCharters {
    pub project: u32,
    pub revision: u64,
    pub source: Box<[u8]>,
    pub charters: Box<[ProjectCharter]>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ProjectCharter {
    pub name: Box<[u8]>,
    pub settings: Box<[u8]>,
    pub instructions: Box<[u8]>,
    pub parts: Box<[CharterPart]>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ProjectChartersFault {
    Unknown,
    Stale,
    Stopping,
    Source { bound: u32 },
    Count { charter: Box<[u8]>, bound: u32 },
    Name { charter: Box<[u8]>, bound: u32 },
    SettingsName { charter: Box<[u8]>, bound: u32 },
    Duplicate { charter: Box<[u8]> },
    Parts { charter: Box<[u8]>, bound: u32 },
    Reasons { charter: Box<[u8]> },
    Bytes { charter: Box<[u8]>, bound: u32 },
}

impl ProjectCharters {
    /// The set's deep allocations, excluding its outer record slot.
    #[must_use]
    pub fn owned_bytes(&self) -> Option<u64> {
        let mut bytes =
            len_u64(self.source.len()).checked_add(u64::try_from(size_of_val(self.charters.as_ref())).ok()?)?;
        for charter in &self.charters {
            bytes = bytes
                .checked_add(len_u64(charter.name.len()))?
                .checked_add(len_u64(charter.settings.len()))?
                .checked_add(len_u64(charter.instructions.len()))?
                .checked_add(crate::parts_owned_bytes(&charter.parts)?)?;
        }
        Some(bytes)
    }

    /// Validate the set's shape and words, without resolving settings names.
    pub fn check(&self, limits: &Limits) -> Result<(), ProjectChartersFault> {
        use ProjectChartersFault as Fault;
        if self.source.len() > usize_of(SOURCE_BYTES) {
            return Err(Fault::Source { bound: SOURCE_BYTES });
        }
        let count = limits.tasks.charters;
        if let Some(extra) = self.charters.get(usize_of(count)) {
            return Err(Fault::Count { charter: named(&extra.name), bound: count });
        }
        let mut names = Set::with_capacity(count);
        for charter in &self.charters {
            if !is_name(&charter.name) {
                return Err(Fault::Name { charter: named(&charter.name), bound: NAME_BYTES });
            }
            if !names.insert(charter.name.as_ref()).expect("set count checked") {
                return Err(Fault::Duplicate { charter: charter.name.clone() });
            }
            if !is_name(&charter.settings) {
                return Err(Fault::SettingsName { charter: charter.name.clone(), bound: NAME_BYTES });
            }
            match words_fault(&charter.parts, Some(len_u64(charter.instructions.len())), limits) {
                None => {}
                Some(WordsFault::Parts) => {
                    return Err(Fault::Parts { charter: charter.name.clone(), bound: limits.tasks.charter_parts });
                }
                Some(WordsFault::Reasons) => return Err(Fault::Reasons { charter: charter.name.clone() }),
                Some(WordsFault::Bytes { .. }) => {
                    return Err(Fault::Bytes { charter: charter.name.clone(), bound: limits.run_bytes });
                }
            }
        }
        Ok(())
    }
}

fn named(name: &[u8]) -> Box<[u8]> {
    Box::from(name.get(..usize_of(NAME_BYTES)).unwrap_or(name))
}

/// One row per live agent's set plus the newest set of each project.
#[must_use]
pub fn capacity(limits: &Limits) -> Option<u32> {
    limits.tasks.tasks.checked_add(limits.authority.projects)
}

/// The retained sets at their entrance bounds (excluding the child indexes).
#[must_use]
pub fn memory_bound(limits: &Limits) -> Option<u64> {
    let count = capacity(limits)?;
    let charter = u64::from(limits.run_bytes)
        .checked_add(u64::from(NAME_BYTES).checked_mul(2)?)?
        .checked_add(u64::try_from(size_of::<ProjectCharter>()).ok()?)?;
    let set = u64::from(SOURCE_BYTES).checked_add(charter.checked_mul(u64::from(limits.tasks.charters))?)?;
    Map::<(u32, u64), ProjectCharters>::price(count)?.checked_add(set.checked_mul(u64::from(count))?)
}
