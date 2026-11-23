//! Shared engine/worker link fixtures (temper/docs/design/testing/tiers.md, 1.2).
pub use temper_engine_service::{assignment, limits, startup, stock};
pub use temper_worker_service::{Worker, WorkerRuntime, worker_settings};
pub use temper_worker_service_fixtures::{
    NOW, pinned_agent, worker, worker_agent, worker_files, worker_limits, worker_settings,
};

pub mod settings {
    pub use temper_engine_service::settings::*;
    pub use temper_worker_service::settings::Worker;
    pub use temper_worker_service::worker_settings::Agent;
}

pub mod link_peer;
pub mod link_profiles;
pub mod link_setup;
pub mod worker_git;
pub use link_setup::{assignment_with_settings, encoding, engine, forges, repository, worker_runtime};
pub use worker_git::socket_stand_with_limits;

#[must_use]
pub fn address() -> skein_io::Addr {
    skein_io::Addr::Ip((::core::net::Ipv4Addr::LOCALHOST, 7100).into())
}
