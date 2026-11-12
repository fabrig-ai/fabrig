//! Landed QA's ledger and commands (temper/docs/design/testing/qa/ledger.md).
mod brief;
mod case;
mod cli;
mod config;
mod document;
pub mod faces;
mod home;
mod ledger;
pub mod runner;
mod text;
pub mod watch;

pub use case::{CaseInput, Step};
pub use cli::{HOME, SESSION, TABLE, main};
pub use config::{Agent, Bounds, Configuration, Git};
pub use home::Home;
pub use ledger::{Answer, Ask, Build, BuildState, Case, CaseAnswer, Exit, Landing, Ledger, Line, Pages, Session};

/// A command or ledger refusal. Its display is the words of ledger.md, 5,
/// or the named configuration, document or IO failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal(pub String);

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Refusal {}

pub(crate) fn refused(words: impl Into<String>) -> Refusal {
    Refusal(words.into())
}
