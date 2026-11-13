//! A deployment's redeploy (temper/docs/design/redeploy.md, 7).
pub mod cli;
pub mod description;
#[doc(hidden)]
pub mod fake;
pub mod hosts;
pub mod record;
pub mod redeploy;
pub mod referee;
pub mod stage;

pub use cli::{TABLE, main};
