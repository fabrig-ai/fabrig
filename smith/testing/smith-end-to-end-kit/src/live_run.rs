//! The shared outcome policy for live exec and simulated terminal captures.

pub use smith_local_process_world::referee::live::{
    Outcome as Expectation, carries_delivery_identity, delivered_finish, matches_file,
};
