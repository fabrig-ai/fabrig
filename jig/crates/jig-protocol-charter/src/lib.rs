//! Translation of a core-owned charter into Smith's run vocabulary
//! (domain/hosts/charters.md). The application's root supplies its
//! tools, endpoints, deployment and bounds. This crate owns jig's contract text ceiling,
//! required checks and the application of core-decided workspace tool grants.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

mod assignment;
mod boundary;
mod call;
mod charter;
mod encode;
mod hosted;
mod identity;
mod result;
#[cfg(test)]
mod tests;

pub use assignment::{CONTRACT_CHECKS_MUST_PASS, CONTRACT_TEXT_MAX, WAIT_GRANTED, assignment};
pub use boundary::{
    Budget, ChangeRule, Charter, Contract, Conventions, FieldRule, ItemRule, Items, Llm, Model, Prices, Reserve,
    Section, TextRule, Tool, ToolEffect, VerdictRule, WorkspaceTools,
};
pub use call::{CallKey, call_key, call_name};
pub use charter::charter;
pub use encode::{EndpointName, encode};
pub use hosted::{RootParts, hosted, message, now_title};
pub use identity::run_identity;
pub use result::{Field, RunResult, Unreadable, result};
pub use smith_domain_run::Mode;

/// The encoding ceilings already owned by this kit (protocol/channel/README.md, 5.1).
#[must_use]
pub const fn limits() -> smith_charter::v5::Limits {
    smith_charter::v5::CEILINGS
}
