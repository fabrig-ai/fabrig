//! The core's driver, peers and referee (jig/docs/design/domain/testing-fakes.md, 1).

#![forbid(unsafe_code)]

mod charters;
pub mod lists;
pub mod open;
pub mod world;

#[doc(hidden)]
pub mod testing {
    pub use crate::world::testing::fork_cut;

    #[must_use]
    pub fn oracle_closed(world: &crate::world::World) -> bool {
        world.oracle_closed
    }
}

mod application_observations;
pub mod client_peer;
pub mod peers;
pub mod protocol;
pub mod store_peer;

/// The testing application's real store and world faults.
pub use jig_test_store_files::{Fault, Store};

pub mod oracle;

/// The exact configuration clone used by the oracle's restart.
#[must_use]
pub fn clone_configuration(source: &jig_test_domain::Config) -> jig_test_domain::Config {
    oracle::configuration(source)
}

pub mod conformance;
pub mod referee;

pub mod faults;

pub mod watches;

pub mod reads;

pub mod stories;

pub mod fixtures;
