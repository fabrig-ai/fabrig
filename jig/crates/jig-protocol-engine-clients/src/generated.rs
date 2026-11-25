//! Generated translations of the client wire, public wire and cursors (skein/docs/design/data/codec/maps.md, 9).
#[rustfmt::skip]
#[expect(clippy::trivially_copy_pass_by_ref, reason = "maps take borrowed domain values even for small Copy types")]
pub(crate) mod client;
#[rustfmt::skip]
#[expect(clippy::trivially_copy_pass_by_ref, reason = "maps to-record functions borrow domain values uniformly")]
pub(crate) mod cursor;
#[rustfmt::skip]
#[expect(clippy::trivially_copy_pass_by_ref, reason = "maps to-record functions borrow domain values uniformly")]
pub(crate) mod public;
