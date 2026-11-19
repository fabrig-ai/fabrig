//! The `github-conformance::suite` integration target's surface, not the tool's CLI.
//! Only the cases, hosted fixture, runner and journal needed by that suite are exported.
//! Captured requests and fault injection are opt-in through `Runner::for_suite`;
//! `run_fake`, `run_live` and `clean` keep recording only their durable outputs.
pub use crate::live::Fixtures;
pub use crate::runner::{Runner, Target};
pub mod cases {
    pub use crate::cases::{NAMES, bot_permission_is_not_a_role, issues_since_epoch, logs, queue, run};
}
pub mod fake {
    pub use crate::fake::{BOT, PERSON, World, boxed};
}
pub mod journal {
    pub use crate::journal::{Journal, Made};
}
