//! Bounded content-free observations (`domain/engine.md`, 11).
//! Dropping a fact changes no watch or delivery.
use crate::{End, Refusal};

/// One observation of pacing or admission.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Fact {
    /// A watch was admitted.
    Watching,
    /// A watch was refused at a bound.
    Refused { refusal: Refusal },
    /// A document was requested.
    Making,
    /// A document was delivered.
    Delivered,
    /// Subjects changed for this many watches.
    Changed { watchers: u32 },
    /// A watch ended.
    Ended { end: End },
}
