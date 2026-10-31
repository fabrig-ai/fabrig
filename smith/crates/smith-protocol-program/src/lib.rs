//! Lazy program conversations and source commands (protocol/programs.md, 3–5, 9).
//! `backend::Backend` owns startup files, MCP calls, program pipes and turns;
//! `Command` settles source commands, and `relay` bridges authenticated MCP IO.
//! Login contents and host policy stay outside this crate.
#![no_std]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod backend;
mod command;
pub mod mcp;
pub mod relay;
mod source;
mod status;

#[cfg(test)]
mod tests;

pub use command::{Command, Ended, Launch};
pub use source::{
    ENVIRONMENT_BYTES, ENVIRONMENT_ENTRIES, Error, Programs, configuration_directory, environment, keeps,
};
pub use status::{Method, SIGN_IN, SIGN_OUT, STATUS, STATUS_BYTES, SUBSCRIPTION, Status, status};
