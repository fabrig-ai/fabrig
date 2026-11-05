//! Client domain world and reusable shell harness (client/domain/testing.md, 4).
#![forbid(unsafe_code)]
pub const PROJECT: u32 = 1;
pub const TASK: u64 = 9;
pub const PARTY: u64 = 7;
pub const SIGNING_PARTY: u64 = 1;

pub mod harness;
pub mod joint;
pub mod model;
pub mod random;
pub mod scripted;
pub mod stories;
pub mod test;
pub use harness::{App, ClientId, Fault, Front, FrontFault, Outcome, Settings, Until, World};
pub use scripted::Scripted;
pub use test::Test;

pub mod observe;
