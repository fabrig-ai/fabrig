//! Read-only, payload-free snapshots, constructed only when explicitly requested.


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

pub use temper_engine_service::diagnostics::{Engine, EngineEvent, WaitEngine, WaitOwner, WaitAccounts};
pub use temper_worker_service::diagnostics::{LinkEvent, Worker, WorkerEvent};


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

impl From<temper_worker_service::diagnostics::Pending> for Pending {
    fn from(value: temper_worker_service::diagnostics::Pending) -> Self {
        Self {
            worker_runs: true,
            box_pipe_pending: false,
            engine: None,
            worker: Some(value.worker),
            work_pending: value.work_pending,
            io_ready: value.io_ready,
            io_due: value.io_due,
            completions: value.completions,
            told: value.told,
            raw_submissions: value.raw_submissions,
            ended: value.ended,
            signals: value.signals,
            signals_closing: value.signals_closing,
        }
    }
}


impl From<temper_worker_service::diagnostics::Wait> for Wait {
    fn from(value: temper_worker_service::diagnostics::Wait) -> Self {
        Self {
            engine: None,
            worker_empty: value.worker_empty,
            held_listeners: value.held_listeners,
            listener_closes: value.listener_closes,
            signals_closing: value.signals_closing,
            operations: value.operations,
            submissions: value.submissions,
        }
    }
}
