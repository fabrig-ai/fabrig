//! Agent tools as bounded values (client/ctl/README.md, 4).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
extern crate alloc;
pub mod adopt;
mod amend;
mod change;
mod demo;
mod forge;
mod finding;
mod forges;
pub mod jig;
pub mod key;
mod landing;
pub mod limits;
mod making;
pub mod recovery;
pub mod root;
mod settings;
pub mod table;
#[cfg(test)]
mod tests;

pub mod help;
