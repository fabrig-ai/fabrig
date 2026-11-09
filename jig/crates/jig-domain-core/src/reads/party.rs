//! Party-scope visibility stub, filled by `party-reads`
//! (`jig/docs/design/domain/engine.md`, 10.1).

use crate::Core;

/// Whether the reading or acting party may see this party's scope, without a project role.
pub(crate) fn scope_visible(_core: &Core, _reader: u64, _owner: u64) -> bool {
    false
}
