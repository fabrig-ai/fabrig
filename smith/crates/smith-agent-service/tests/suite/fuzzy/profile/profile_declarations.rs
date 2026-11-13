//! One native model declaration shared with the service's profile scenarios.

pub fn declarations(environment_bytes: u32) -> super::profile_service::profile::Configuration {
    smith_agent_profile_fixtures::declarations(environment_bytes)
}
