//! The box's agent preflight for upgrade rehearsal (protocol/binaries.md).
use temper_engine_shell::serve::WorkerHalf;
use temper_service as service;
pub(crate) struct Half;
impl WorkerHalf for Half {
    fn preflight(&self, named: &service::startup::Named) -> Result<service::settings::Agent, String> {
        temper_worker_shell::agent::preflight_worker(named.worker.as_ref().expect("named worker"))
    }
}
