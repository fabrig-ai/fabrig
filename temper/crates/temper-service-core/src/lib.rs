//! Vocabulary shared by the services (temper/docs/design/protocol/services.md, 3).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

pub mod agent;
pub mod limits;
pub mod long_pass;
pub mod process;
pub mod settings;
pub mod startup;
pub mod tokens;
pub mod values;

/// One stage of a pass, in the loop's order (protocol/README.md, 6.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stage {
    Io,
    Route,
    Engine,
    Worker,
    Out,
}
pub const STAGES: [Stage; 5] = [Stage::Io, Stage::Route, Stage::Engine, Stage::Worker, Stage::Out];

/// The peer's refusal code, or a local opening/decoding refusal.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refusal {
    Peer(u16),
    Terms,
    Malformed,
}

/// A documentation answer's byte bound (protocol/services.md, 2).
pub const DOC_ANSWER: u32 = 2_048;

#[cfg(test)]
mod tests;
