//! Read-only, payload-free snapshots, constructed only when explicitly requested.
use skein_lib::{List, Time};

#[derive(Debug)]
#[expect(clippy::struct_excessive_bools, reason = "each flag names a separate service readiness term")]
pub struct Pending {
    pub work_pending: bool,
    pub worker_runs: bool,
    pub io_ready: bool,
    pub io_due: bool,
    pub completions: u32,
    pub told: u32,
    pub raw_submissions: u32,
    pub ended: bool,
    pub signals: bool,
    pub signals_closing: bool,
    pub box_pipe_pending: bool,
    pub engine: Option<Engine>,
    pub worker: Option<Worker>,
}

#[derive(Debug)]
#[expect(clippy::struct_excessive_bools, reason = "the snapshot exposes the engine's independent readiness gates")]
pub struct Engine {
    pub work_pending: bool,
    pub effects: bool,
    pub clients: bool,
    pub setup: bool,
    pub setup_lifecycle: bool,
    pub store: bool,
    pub forge: bool,
    pub workers: bool,
    pub accounts: bool,
    pub store_opened: bool,
    pub ended: bool,
    pub room: bool,
    pub begin: bool,
    pub ready: bool,
    pub domain_ready: bool,
    pub release_pending: bool,
    pub blocked: bool,
    pub pending: Option<EngineEvent>,
    pub events: List<EngineEvent>,
    pub due: bool,
    pub first_periods: u32,
    pub settled_revision: Option<u64>,
    pub revision: u64,
    pub next_deadline: Option<Time>,
    pub requests: u32,
    pub workers_opened: bool,
    pub clients_opened: bool,
}

#[derive(Debug)]
pub enum EngineEvent {
    Setup,
    Store,
    Worker,
    Party,
    Account,
    Credential,
    Forge,
    Released,
    Restart,
    Resume,
    Stop,
}

#[cfg(feature = "worker")]
pub use temper_worker_service::diagnostics::{Worker, WorkerEvent, LinkEvent};
#[cfg(not(feature = "worker"))]
#[path = "unbuilt_diagnostics.rs"]
mod unbuilt;
#[cfg(not(feature = "worker"))]
pub use unbuilt::{Worker, WorkerEvent, LinkEvent};

/// Closure and queue ownership beside the shell's unbounded wait decision.
#[derive(Debug)]
pub struct Wait {
    pub engine: Option<WaitEngine>,
    pub worker_empty: bool,
    pub held_listeners: u32,
    pub listener_closes: u32,
    pub signals_closing: bool,
    pub operations: bool,
    pub submissions: u32,
}

#[derive(Debug)]
pub struct WaitEngine {
    pub restart: Option<jig_domain_core::start::Phase>,
    pub mark_durable: bool,
    pub store_closed: bool,
    pub clients: WaitOwner,
    pub workers: WaitOwner,
    pub setup: WaitOwner,
    pub setup_above: u32,
    pub forge: WaitOwner,
    pub forge_above: u32,
    pub accounts: Option<WaitAccounts>,
}

#[derive(Debug)]
pub struct WaitOwner {
    pub closed: bool,
    pub io: u32,
}

#[derive(Debug)]
#[expect(clippy::struct_excessive_bools, reason = "each flag names an independent account lifecycle or readiness term")]
pub struct WaitAccounts {
    pub opened: bool,
    pub work_pending: bool,
    pub effects: u32,
    pub pending: bool,
    pub io: u32,
    pub empty: bool,
}

#[cfg(feature = "worker")]
impl From<temper_worker_service::diagnostics::Pending> for Pending {
 fn from(value: temper_worker_service::diagnostics::Pending) -> Self { Self {worker_runs: true, box_pipe_pending: false, engine: None, worker: Some(value.worker),work_pending: value.work_pending,io_ready: value.io_ready,io_due: value.io_due,completions: value.completions,told: value.told,raw_submissions: value.raw_submissions,ended: value.ended,signals: value.signals,signals_closing: value.signals_closing,} }
}

#[cfg(feature = "worker")]
impl From<temper_worker_service::diagnostics::Wait> for Wait {
 fn from(value: temper_worker_service::diagnostics::Wait) -> Self { Self {engine: None,worker_empty: value.worker_empty,held_listeners: value.held_listeners,listener_closes: value.listener_closes,signals_closing: value.signals_closing,operations: value.operations,submissions: value.submissions,} }
}
