//! Native protocol backend for the shared agent world (protocol/llm.md, section 10).
pub mod adapter;
pub mod pool_peer;
pub mod referee;
mod translate;
pub mod wire;
mod world;

pub use smith_agent_world::{
    Boundary, CompletionObservation, CompletionTerminal, DeliverySubmission, HostReply, HostSchedule, JOBS, Job,
    Settings, delivered, host_referee, messages_referee, script,
};
pub use world::World;

/// Window leaving one 4096-token output above its four-fifths threshold.
pub const MODEL_WINDOW: u32 = 20_480;
/// The agent fixture's protocol admission overrides.
pub const LIMITS: smith_domain::Limits = smith_domain::Limits {
    run: smith_domain::run::Limits { host_part: u64::MAX, ..smith_agent_world::LIMITS.run },
    session: smith_domain::session::Limits { token_bytes: skein_llm::TOKEN_BYTES, ..smith_agent_world::LIMITS.session },
    ..smith_agent_world::LIMITS
};
/// Tight protocol admission bounds.
pub const TIGHT: smith_domain::Limits = smith_domain::Limits {
    run: smith_domain::run::Limits { host_part: u64::MAX, ..smith_agent_world::TIGHT.run },
    session: smith_domain::session::Limits { token_bytes: skein_llm::TOKEN_BYTES, ..smith_agent_world::TIGHT.session },
    ..smith_agent_world::TIGHT
};
pub use smith_agent_world::BUDGET;
