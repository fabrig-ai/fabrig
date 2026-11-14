//! The client contract's records for both ends (protocol/client/wire.md, section 1, and protocol/client/README.md, section 6).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

mod approval;
pub use approval::{ApprovalCode, CODE_ALPHABET, CODE_LETTERS, CODE_TYPED_BYTES};

#[expect(clippy::struct_field_names, reason = "The Accounts wire record retains the schema’s accounts field.")]
#[rustfmt::skip]
#[path = "generated/v22.rs"]
pub mod v22;
pub use v22::*;

#[rustfmt::skip]
#[path = "generated/public_v1.rs"]
pub mod public_v1;
mod public;
pub use public::{PublicIndexBounds, public_bounds, public_limits};

/// The independently versioned public wire (client/domain/contract/README.md, section 10).
pub const PUBLIC_VERSION: u16 = 1;
/// The public projects answer kind.
pub const PUBLIC_PROJECTS: u16 = 1;
/// The public board answer kind.
pub const PUBLIC_BOARD: u16 = 2;
/// The public task card answer kind.
pub const PUBLIC_CARD: u16 = 3;

/// The client wire version this release reads and writes.
pub const VERSION: u32 = 22;

#[cfg(test)]
mod tests;
