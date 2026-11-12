//! Read-only, payload-free snapshots, constructed only when explicitly requested.
use skein_lib::{List, Time};

#[derive(Debug)]
#[expect(clippy::struct_excessive_bools, reason = "each flag names a separate service readiness term")]
pub struct Pending {
    pub work_pending: bool,
    pub io_ready: bool,
    pub io_due: bool,
    pub completions: u32,
    pub told: u32,
    pub raw_submissions: u32,
    pub ended: bool,
    pub signals: bool,
    pub signals_closing: bool,
    pub engine: Option<Engine>,
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


/// Closure and queue ownership beside the shell's unbounded wait decision.
#[derive(Debug)]
pub struct Wait {
    pub engine: Option<WaitEngine>,
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




