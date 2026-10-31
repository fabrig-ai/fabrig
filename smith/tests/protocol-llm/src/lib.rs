//! The agent, run, session and tools together on a scripted typed host
//! (domain/run.md, section 13; domain/host.md, sections 2 and 10;
//! testing-strategy.md, sections 2.2, 6 and 7).
//!
//! The world owns time, seeds, a fake checkout and the fake LLM domain.
//! Its opt-in wire backend adopts prepared shared Clients and byte peers,
//! passes each root Complete receiving contract through the adapter, and
//! synchronizes their clocks with the root iteration. Logical call owners and
//! physical close rights remain distinct until the lower Closed terminal.
//! Its parent-delivery bridge exposes root requests and accepts sealed terminals
//! through the shared schedule and flight ledger (domain/run.md, section 8.2;
//! domain/host.md, section 9).
//! It calls the agent's step, fire and resume entrances under bounded output
//! pressure. Its host starts one charter, receives one answer and supplies typed
//! push replies. The world composes the domain and optional protocol peers.
//! Opaque host objects are attested by the skein JSON parser; fixture schemas
//! are caller data. Skein owns provider grammar and transport.
//!
//! A fixture translator recognizes the scripts' finite argument vocabulary.
//! The independent referee checks boundary observations after settlement; those
//! observations never drive the host. Replay and memory use skein's shared kit.

pub mod adapter;

/// Observed Smith component peer for shared-pool and model-progress stories.
pub mod pool_peer;

mod fixture;

pub mod host_referee;

mod limits;

pub mod messages_referee;

pub mod referee;

pub mod script;

mod translate;

pub mod wire;

mod world;

pub use limits::{BUDGET, LIMITS, MODEL_WINDOW, TIGHT};

pub use script::{JOBS, Job};

pub use world::{
    Boundary, CompletionObservation, CompletionTerminal, DeliverySubmission, HostReply, HostSchedule, Settings, World,
    delivered,
};
