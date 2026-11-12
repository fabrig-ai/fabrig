//! Landed QA's ledger and commands (temper/docs/design/testing/qa/ledger.md).
mod brief;
pub mod build;
mod case;
mod cli;
mod config;
mod document;
pub mod faces;
mod findings;
mod home;
mod ledger;
mod pass_document;
mod pass_end;
mod passes;
pub mod place;
mod recording;
mod report;
pub mod runner;
mod text;
pub mod watch;

pub use case::{CaseInput, Step};
pub use cli::{HOME, SESSION, TABLE, main};
pub use config::{Agent, Bounds, Configuration, Git, RequestConfiguration};
pub use findings::{Check, Finding, FindingAnswer, FindingInput, FindingState, Note};
pub use home::Home;
pub use ledger::{Answer, Ask, Build, BuildState, Case, CaseAnswer, Exit, Landing, Ledger, Line, Pages, Session};
pub use passes::{
    Before, CaseBefore, CaseState, Outcome, OutcomeResult, Pass, PassStart, PassState, Request, SessionKind,
};

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
