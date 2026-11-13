//! Goal-filter membership carried with the inbox's original row cursor
//! (`client/domain/pages.md`, section 3). Live membership follows the bounded
//! requester walk; retained membership uses the supplied original ancestry.

use super::Cursor;
use skein_lib::List;

/// Complete qualifying goal roots for one inbox row, including its task itself.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct InboxSelection {
    pub cursor: Cursor,
    pub goals: List<u64>,
}

impl InboxSelection {
    /// Heap held by the roots, including reserved unused capacity.
    #[must_use]
    pub fn owned_bytes(&self) -> Option<u64> {
        List::<u64>::price(self.goals.capacity())
    }
}

/// Heap bound of a whole parallel context list and all its root allocations.
#[must_use]
pub fn inbox_selection_bound(rows: u32, depth: u32) -> Option<u64> {
    let roots = List::<u64>::price(depth.checked_add(1)?)?;
    List::<InboxSelection>::price(rows)?.checked_add(u64::from(rows).checked_mul(roots)?)
}
