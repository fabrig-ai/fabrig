//! Facts about the independent revisions a party decides about
//! (client/domain/contract/documents.md, section 2). The parent translates task state
//! into these values; people keeps them with a keyed refusal for replay.

pub use jig_domain_contract::ops::party::{Bound, Exceeded, FailureClass, HoldReason, Moved};
