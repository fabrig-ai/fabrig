//! Projection from the engine's aggregate to the worker (protocol/services.md, 6).
pub use temper_engine_service::limits::*;
pub use temper_service_core::limits::{TOOLING_FILE_BYTES, TOOLING_LOG_BYTES, TOOLING_PATH_BYTES};
pub use temper_worker_service::worker_settings::job_kinds;
#[must_use]
pub fn worker(limits: &Limits) -> crate::worker_settings::Limits {
    crate::worker_settings::Limits {
        checkout: limits.checkout,
        git_component: limits.git_component,
        peer: temper_service_core::limits::Peer {
            forges: limits.forge.forges,
            installations: limits.forges.installations,
            token_bytes: limits.forge_component.token_bytes,
        },
    }
}

/// The joined document's engine and worker limits table.
pub fn engine_reference(out: &mut skein_lib::Writer) -> Result<(), skein_lib::Overflow> {
    engine_reference_with(out, b"# Engine limits\n\nKind: reference\n\nEvery limit engine.limits.overrides and a worker's limits.overrides take: its unit, the small profile's value and what it bounds.\n")
}
