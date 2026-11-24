//! Stale-view fault kinds (`temper/docs/design/testing/workloads.md`, section 2.2).

use skein_world::faults::Kind;

use crate::forge::{CLOSED_BY_ANOTHER, HEAD_BEHIND, MERGE_LATE, MERGED_BY_ANOTHER};

/// The stale-view fault family.
pub const FAMILY: &str = "stale";

/// Another user reopens a pull.
pub const REOPENED: Kind = Kind { family: FAMILY, name: "reopened_by_another" };
/// A dependency is squashed.
pub const SQUASHED: Kind = Kind { family: FAMILY, name: "squashed_dependency" };
/// The base moves.
pub const MOVED_BASE: Kind = Kind { family: FAMILY, name: "moved_base" };

/// The stale sweep's kinds, in configuration order, including lagging and refusing kinds.
pub const STALE: [Kind; 7] =
    [HEAD_BEHIND, MERGE_LATE, REOPENED, SQUASHED, MOVED_BASE, CLOSED_BY_ANOTHER, MERGED_BY_ANOTHER];
