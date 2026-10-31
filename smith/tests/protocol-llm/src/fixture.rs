//! Shared protocol story setup (smith/docs/design/protocol/llm.md, section 10).
use smith_protocol_llm::{ComponentLimits, Limits};

/// The protocol world's client and workspace description preset.
#[must_use]
pub fn limits() -> Limits {
    smith_protocol_llm::fixture::small(skein_llm_world::limits(), crate::adapter::stated())
}

/// The same component capacities for manual and scripted peers.
#[must_use]
pub fn component() -> ComponentLimits {
    smith_protocol_llm::fixture::component(limits(), smith_agent_service::profile::standard().policy.connection_keep)
        .expect("fixture receiving bound")
}

/// The shared agent settings with protocol token pricing and model window.
#[must_use]
pub fn settings(seed: u64) -> smith_agent_world::Settings {
    smith_agent_world::Settings {
        window: crate::MODEL_WINDOW,
        limits: crate::LIMITS,
        ..smith_agent_world::Settings::calm(seed)
    }
}
