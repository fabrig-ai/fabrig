//! Reference UI record codecs (jig/docs/design/client/domain/view-state.md, 8; tools/crates/design.md, 3.2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

/// The current view records schema text (skein/docs/design/data/codec/golden.md, section 1).
#[expect(clippy::disallowed_macros, reason = "the codec exposes its owner's schema without the UI")]
pub const SCHEMA: &str = include_str!("../../jig-client-view/schema/records-v1.schema");

#[rustfmt::skip]
#[path = "generated/records.rs"]
pub mod records_v1;
