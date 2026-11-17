//! The worker component and its service (temper/docs/design/protocol/services.md, 5).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
pub mod diagnostics;
pub mod fetch;
pub mod figures;
pub mod identities;
pub mod program_sources;
pub mod settings;
pub mod startup;
mod worker;
pub mod worker_alone;
mod worker_git;
#[path = "worker/job_ending.rs"]
mod worker_job_ending;
pub mod worker_route;
pub mod worker_settings;
#[path = "worker_alone/vocabulary.rs"]
mod worker_vocabulary;
pub use jig_protocol_worker_wire::{Loss as WorkerLoss, Reason as WorkerRefusalReason};
pub use skein_channel::Closed as WorkerClosed;
pub use temper_service_core::{Refusal, STAGES, Stage, long_pass, tokens};
pub use worker::{Worker, WorkerRuntime, translate};
pub use worker_job_ending::JobEnding as WorkerJobEnding;
#[cfg(test)]
mod tests;
