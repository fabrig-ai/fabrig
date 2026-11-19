//! The client contract's records for both ends (protocol/client/wire.md, section 1, and protocol/client/README.md, section 6).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

mod approval;
pub use approval::{ApprovalCode, CODE_ALPHABET, CODE_LETTERS, CODE_TYPED_BYTES};

#[expect(clippy::struct_field_names, reason = "The Accounts wire record retains the schema’s accounts field.")]
#[rustfmt::skip]
#[path = "generated/client.rs"]
pub mod v37;
pub use v37::*;

#[rustfmt::skip]
#[path = "generated/public.rs"]
pub mod public_v1;
mod public;
pub use public::{PublicIndexBounds, public_bounds, public_limits};

/// The independently versioned public wire (client/domain/contract/README.md, section 10).
pub const PUBLIC_VERSION: u16 = public_v1::VERSION;
/// The public projects answer kind.
pub const PUBLIC_PROJECTS: u16 = 1;
/// The public board answer kind.
pub const PUBLIC_BOARD: u16 = 2;
/// The public task card answer kind.
pub const PUBLIC_CARD: u16 = 3;

/// The client wire version this release reads and writes.
pub const VERSION: u32 = {
    let [high, low] = v37::VERSION.to_be_bytes();
    u32::from_be_bytes([0, 0, high, low])
};

#[cfg(test)]
mod tests;
