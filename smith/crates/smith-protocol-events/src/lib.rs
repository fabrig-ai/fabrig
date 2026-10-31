//! Projects domain facts and protocol measurements into the process event stream.
//! The component retains bounded joins and destination delivery state. It never
//! chooses domain outcomes or receives credentials. `Producer` accepts typed
//! observations; `Sinks` exchanges stream demand and terminals with its IO owner.
//! Contract: smith/docs/design/protocol/events.md, sections 3–5.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod answers;
mod calls;
mod children;
pub mod events;
mod inputs;
mod messages;
mod observations;
mod responses;
pub mod sinks;

pub use sinks::{Destination, GivenUp, Why};

pub use answers::{AnswerMeta, Answered, accepted, answer_record, failure_record, refusal_record};
pub use events::{EventLimits, Producer, Sources};
pub use observations::{
    ChildShare, budget_record, check_exit, child_completed, child_share, opening_model, reported_usage,
    request_tool_choice,
};

#[cfg(test)]
mod tests;

#[cfg(any(test, feature = "fixture"))]
pub mod fixture;
