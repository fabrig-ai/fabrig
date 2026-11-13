//! Complete close-disposal identities and their owner funding
//! (`jig/docs/design/client/domain/objects.md`, 11).

use super::Chip;

/// A direct live delegate and the reservation charged if a party keeps it.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct LiveDelegate {
    pub chip: Chip,
    /// Budget minus spend by this task and ended descendants; live reservations stay included.
    pub takeover_budget: u64,
}
