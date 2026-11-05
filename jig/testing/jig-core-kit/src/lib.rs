//! The core's driver, peers and referee (jig/docs/design/domain/testing.md, 7).

#![forbid(unsafe_code)]

mod charters;
pub mod world;

mod application_observations;
pub mod client_peer;
pub mod peers;
pub mod protocol;
pub mod store_peer;

/// The testing application's real store and world faults.
pub use jig_test_store_files::{Fault, Store};

pub mod oracle;

pub mod conformance;
pub mod referee;

pub mod faults;

pub mod watches;

pub mod reads;

pub mod stories;

pub mod fixtures;
