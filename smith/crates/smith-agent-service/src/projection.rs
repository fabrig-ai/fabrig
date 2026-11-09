//! Adapts composed source limits and program inventory to event projection.
//! No event component retains the service configuration or its domain owner.
//! Contract: smith/docs/design/protocol/events.md, sections 4–5.

use alloc::boxed::Box;
pub use smith_agent_profile::event_limits;
use smith_agent_profile::events::sources;

/// Construct the process event projection from the service's admitted configuration.
#[must_use]
pub fn event_producer(config: &crate::Config) -> Option<smith_protocol_events::Producer> {
    config.limits.events?;
    let programs = match crate::service::bound(config.program.as_ref()) {
        Some(program) => program.endpoints.clone(),
        None => Box::default(),
    };
    Some(smith_protocol_events::Producer::new(&sources(&config.limits), config.channel_endpoints.clone(), programs))
}
