//! Ops's connector worlds: the shared production below and a scripted root above.
#![forbid(unsafe_code)]

#[cfg(test)]
use skein_world::domain::heap;

#[cfg(test)]
#[global_allocator]
static HEAP: heap::Counting = heap::Counting;

mod infrastructure_world;
mod world;

pub use infrastructure_world::{InfrastructureWorld, infra_environment, infra_service, infrastructure_limits};
pub use world::{World, limits, service};

pub mod conformance;
pub mod direct;
pub mod protocol;

/// Ops client adapters on the reusable client world.
pub mod client;

/// Ops clients in front of the actual conformance engine.
pub mod joint;

pub use conformance::agent_bounds;
