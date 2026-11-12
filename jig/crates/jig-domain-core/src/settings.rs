//! Deployment settings, defaults and revisions (domain/engine/settings.md).

pub use crate::below::settings::*;

pub mod application;
pub mod change;
pub mod charters;
pub mod check;
pub mod fleet_needs;
pub(crate) mod held;
pub(crate) mod restart;
pub mod revisions;
#[cfg(test)]
mod tests;

#[cfg(test)]
use crate::{Charter, Template};
use alloc::boxed::Box;
#[cfg(test)]
use jig_domain_authority as authority;
#[cfg(test)]
use jig_domain_people as people;
#[cfg(test)]
use skein_lib::List;
use skein_lib::Map;
#[cfg(test)]
use skein_lib::Wall;

pub(crate) use change::{apply, changes_current};
pub use check::check;
pub use held::{Held, VersionKey};
pub use restart::revision_writes;

/// One paged restore and its connector continuations (jig/docs/design/domain/engine/settings.md, 10).
#[derive(Hash, Debug)]
pub(crate) struct Restoring {
    pub(crate) pending: Option<Restoration>,
    pub(crate) groups: skein_lib::Slab<Option<RestoreGroup>>,
}
impl Restoring {
    pub(crate) fn new(kinds: &[Kind]) -> Self {
        Self {
            pending: None,
            groups: skein_lib::Slab::with_capacity(u32::try_from(kinds.len()).expect("declared kinds")),
        }
    }
}
#[derive(Hash, Debug)]
pub(crate) struct Restoration {
    pub(crate) party: crate::routing::party::dispatch::Party,
    pub(crate) revision: u32,
    pub(crate) newest: u32,
    pub(crate) reason: Box<[u8]>,
    pub(crate) items: Map<Setting, RestoreEdit>,
    pub(crate) left_out: Box<[Setting]>,
    pub(crate) proposal: Option<u64>,
}
#[derive(Hash, Debug)]
pub(crate) enum RestoreEdit {
    Core(Edit),
    Application(application::Restored),
}
#[derive(Hash, Debug)]
pub(crate) struct RestoreGroup {
    pub(crate) connector: u16,
    pub(crate) items: Box<[application::Restored]>,
    pub(crate) judged: Option<Box<[crate::effects::connector::Judged]>>,
}
