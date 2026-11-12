//! The agent's document helpers keep no state and never know the agent's run.
//! `config` reads, resolves and checks the document; `trace` holds its settings.
//! `program` checks a program's directories and prepares its terminal sign-in.
//! Contract: smith/docs/design/shell/README.md, sections 2.1 and 2.6;
//! smith/docs/design/protocol/programs.md, sections 4.1 and 4.2.

#[cfg(unix)]
pub mod program;

pub mod config;
pub mod trace;

pub use smith_agent_profile::host_limits;

#[cfg(test)]
mod fixture;
