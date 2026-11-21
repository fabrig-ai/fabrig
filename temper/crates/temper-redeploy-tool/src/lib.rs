//! A deployment's redeploy (temper/docs/design/redeploy.md, 7).
pub mod cli;
pub mod description;
#[doc(hidden)]
pub mod fake;
pub mod hosts;
pub mod openings;
pub mod record;
pub mod redeploy;
pub mod referee;
pub mod stage;
#[doc(hidden)]
#[expect(clippy::must_use_candidate, reason = "shared test fixtures retain their helpers' signatures")]
pub mod testing;

pub use cli::{TABLE, main};
