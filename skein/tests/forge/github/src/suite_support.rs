//! The `github-conformance::suite` integration target's surface, not the tool's CLI.
//! The cases, hosted fake, runner, journal and directory face needed by that suite.
//! Captured requests and fault injection are opt-in through `Runner::for_suite`;
//! `run_fake`, `run_live` and `clean` keep recording only their durable outputs.
pub use crate::directory::Directory;
pub use crate::runner::{Runner, Target, clean_in, run_fake_in};
pub mod cases {
    pub use crate::cases::{NAMES, bot_permission_is_not_a_role, issues_since_epoch, logs, queue, run};
}
pub mod fake {
    pub use crate::fake::{BOT, PERSON, World, boxed};
}
pub mod journal {
    pub use crate::journal::{Journal, Made};
}
