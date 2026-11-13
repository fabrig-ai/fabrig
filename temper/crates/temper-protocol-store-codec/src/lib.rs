//! Temper's generated store records (temper/docs/design/protocol/store.md, section 4).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

#[rustfmt::skip]
#[expect(clippy::similar_names, reason = "the generated fields head/held and ending/pending name distinct durable facts")]
#[path = "generated/v16.rs"]
pub mod v16;
#[rustfmt::skip]
#[expect(clippy::similar_names, reason = "the generated fields head/held and ending/pending name distinct durable facts")]
#[path = "generated/v17.rs"]
pub mod v17;
#[rustfmt::skip]
#[expect(clippy::similar_names, reason = "the generated fields head/held and ending/pending name distinct durable facts")]
#[path = "generated/v18.rs"]
pub mod v18;

#[rustfmt::skip]
#[expect(clippy::similar_names, reason = "generated head/held and ending/pending name distinct durable facts")]
#[path = "generated/v19.rs"]
pub mod v19;

#[rustfmt::skip]
#[expect(clippy::similar_names, reason = "generated head/held and ending/pending name distinct durable facts")]
#[path = "generated/v20.rs"]
pub mod v20;

#[rustfmt::skip]
#[expect(clippy::similar_names, reason = "generated head/held and ending/pending name distinct durable facts")]
#[path = "generated/v21.rs"]
pub mod v21;
