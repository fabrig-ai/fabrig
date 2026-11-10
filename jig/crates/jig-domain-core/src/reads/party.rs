//! A person's own preferences, read and kept without a project role
//! (`jig/docs/design/domain/engine/briefs.md`, 2.1).

use crate::Core;

/// Whether the reading or acting party may see this party's scope, without a project role.
pub(crate) fn scope_visible(core: &Core, reader: u64, owner: u64) -> bool {
    reader == owner && core.people.kind(reader) == Some(jig_domain_people::Kind::Person)
}
