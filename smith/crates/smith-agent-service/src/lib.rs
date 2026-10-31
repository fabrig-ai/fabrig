//! One agent run composed from its domain, host channel, LLM and machine.
//! The service keeps the four machines, bounded routing tables, and the queues
//! between them. It never knows provider wire syntax, charter policy decisions,
//! or the meaning of a host's bearer token. [`iterate`] drives the framed service;
//! [`Effects`] drives lower effects for a host.
//! [`Service::completions`] and [`Service::submissions`] meet a
//! shell or simulated kernel. Contracts: smith/docs/design/protocol/agent.md, sections 4–6;
//! smith/docs/design/protocol/README.md, sections 5–6.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod live;
mod projection;
mod service;
mod terminal;

pub mod profile;

#[cfg(test)]
mod tests;

pub use projection::{event_limits, event_producer};
pub use service::teardown::{Answer, Close, Step, Teardown};
pub use terminal::{Last, last};

pub use service::{
    AdoptError, Config, ConfigError, Effects, Failure, Limits, ProgramBackend, Service, done, effects_valid, failure,
    io_valid, iterate, next_deadline, next_policy_deadline, program_operations, program_valid, valid, work_pending,
};
