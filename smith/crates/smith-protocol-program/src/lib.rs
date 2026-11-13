//! Lazy program conversations and source commands (protocol/programs.md, 3–5, 9).
//! `backend::Backend` owns startup files, MCP calls, program pipes and turns;
//! `Command` settles source commands, and `relay` bridges authenticated MCP IO.
//! Login contents and host policy stay outside this crate.
#![no_std]
#![forbid(unsafe_code)]
#![expect(
    clippy::manual_map,
    reason = "closures are out of step code (programming-model.md 10.3), so an Option is mapped by a match"
)]

extern crate alloc;

pub mod backend;
mod command;
pub mod mcp;
pub mod relay;
mod source;
mod status;

#[cfg(test)]
mod tests;

pub use backend::Given;
pub use command::{Command, Ended, Launch};
pub use source::{
    ANTHROPIC_DIRECTORY, ENVIRONMENT_BYTES, ENVIRONMENT_ENTRIES, Error, Programs, configuration_directory, environment,
    keeps,
};
pub use status::{EMAIL_BYTES, Method, SIGN_IN, SIGN_OUT, STATUS, STATUS_BYTES, SUBSCRIPTION, Status, email, status};

/// The agent binary mode that execs a program to die with its agent (protocol/programs.md, 5.1).
pub const PROGRAM_CHILD_FLAG: &str = "--program-child";
