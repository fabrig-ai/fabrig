//! One native model declaration shared with the service's profile scenarios.

pub fn declarations(environment_bytes: u32) -> super::profile_service::profile::Configuration {
    super::profile_service::fixture::declarations(environment_bytes)
}
