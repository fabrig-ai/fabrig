//! The testing application's row-part connector (domain/engine/views.md, 2).
//! Owner 7: task number, a deliberately small kind, busy, and requested version.

use alloc::boxed::Box;
use jig_domain_contract::parts::{MadePart, PartFailure, PartsAsk};
use skein_lib::List;

#[must_use]
pub fn make(ask: &PartsAsk) -> List<MadePart> {
    let mut parts = List::with_capacity(ask.rows.len().checked_mul(ask.kinds.len()).expect("bounded parts"));
    for row in &ask.rows {
        for kind in &ask.kinds {
            let part = match (kind.owner, kind.kind) {
                (7, 1 | 2) => MadePart::Made(Box::from(row.chip.number.to_le_bytes())),
                (7, 3) => MadePart::Failed(PartFailure::Busy),
                (7, 4) => MadePart::Made(Box::from(u64::from(kind.version).to_le_bytes())),
                _ => MadePart::Failed(PartFailure::Unreadable),
            };
            parts.push(part).expect("one part per row and kind");
        }
    }
    parts
}
