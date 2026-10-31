//! Shared story frames for the local process worlds (testing.md, section 2.1).

use crate::Placement;
use crate::fixture::contains;

/// Both ways the local host runs its agent.
pub const PLACEMENTS: [Placement; 2] = [Placement::Spawned, Placement::InProcess];

/// Find a byte string without repeating its length at the call site.
#[must_use]
pub fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    contains(haystack, needle)
}
