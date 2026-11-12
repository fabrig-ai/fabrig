//! Temper's engine and worker protocol services.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
extern crate alloc;
use alloc::boxed::Box;
#[path = "worker/job_ending.rs"]
mod worker_job_ending;
#[path = "worker_alone/vocabulary.rs"]
mod worker_vocabulary;
pub use temper_engine_domain::{JobDiagnostic, JobDiagnosticEnding, RunRefusalDiagnostic};
pub use worker_job_ending::JobEnding as WorkerJobEnding;
pub mod diagnostics;
pub mod forge;
pub mod forges;
pub use forge::ForgeRuntime;
pub mod limits;
pub use temper_service_core::long_pass;
pub mod settings;
pub mod startup;
pub mod stock;
pub use startup::{Named, StoreLimits, named, startup, store_limits};
pub use temper_protocol_records_forge::NotConnected;
pub mod system;
pub use system::{STAGES, Service, Stage, iterate, iterate_stage};
pub mod store;
#[cfg(test)]
mod tests;
pub use temper_service_core::tokens;

pub mod accounts;
pub mod assignment;
pub mod clients;
pub mod family;
#[cfg(feature = "worker")]
pub mod identities;
#[cfg(feature = "worker")]
pub mod program_sources;
pub use jig_protocol_engine_clients_http::server;
pub mod translate;
pub mod values;
#[cfg(feature = "worker")]
mod worker;
#[cfg(feature = "worker")]
pub mod worker_alone;
#[cfg(feature = "worker")]
mod worker_git;
#[cfg(feature = "worker")]
pub mod worker_route;
#[cfg(feature = "worker")]
pub mod worker_settings;
#[cfg(feature = "worker")]
pub use worker::{Worker, WorkerRuntime};
#[cfg(not(feature = "worker"))]
mod unbuilt;
#[cfg(not(feature = "worker"))]
pub use unbuilt::{Worker, WorkerRuntime, worker_alone};
pub mod workers;
pub use accounts::AccountsRuntime;
mod core_wait;
pub use core_wait::CoreWait;
pub use temper_engine_domain::JournalStop;
mod engine;
mod route;
mod setup;
mod start_refusal;
mod stop;
pub mod upgrade;
pub use start_refusal::{refused_row_start, refused_start};
pub use store::UnreadableRow;
mod tools;
pub use temper_engine_domain::loads::{Failure as LoadFailure, Kind as RestoreRangeKind};
pub use tools::{render_for_task, render_for_task_page, settings_note};
/// How the service ended (protocol/README.md, 4).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum End {
    Drained,
    Aborted,
    Stopped(Stop),
}
/// Why the service stopped; failed startup names its configuration paths.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Stop {
    /// A full encoded transcript frame exceeded the page bound before data IO.
    FrameTooLarge {
        number: u64,
        task: u64,
        bytes: u32,
        limit: u32,
    },
    Journal {
        why: JournalStop,
    },
    CommitFailed {
        number: u64,
    },
    Start(Problems),
    /// jig's start refused a fit at `phase` (jig jig/docs/design/domain/engine/restart.md).
    StartRefused {
        phase: Option<jig_domain_core::start::Phase>,
        failure: jig_domain_core::start::Failure,
    },
    /// A restored charter leaves too little brief room (temper/docs/design/domain/agent/tools.md).
    Room(temper_engine_domain::room::Short),
    /// A startup page could not be read: the root's range kind, and the store's
    /// typed cause where the store gave one (jig-protocol-store `restore::Failure`).
    RestorePage {
        range: RestoreRangeKind,
        failure: Option<jig_protocol_store::restore::Failure>,
    },
    /// The forge's restored records do not fit its limits, at `phase`.
    ForgeFit {
        phase: Option<jig_domain_core::start::Phase>,
        failure: temper_engine_domain_forge::RestoreFit,
    },
    /// The deployment's namespace binding was refused.
    Binding,
    /// A current stored setting names a retired value or number.
    Setting(temper_engine_domain_forge::settings::SettingKey),
    /// The store refuses this start (jig's protocol/store/records.md, 3 and protocol/store/opening.md).
    StoreOpen {
        why: jig_protocol_store::OpenRefusal,
    },
    /// A temper row refused decoding at restart (protocol/store.md, 5).
    StoreRow {
        row: UnreadableRow,
    },
    Listener {
        listener: Listening,
        address: Option<skein_io::Addr>,
        error: Option<skein_io::Error>,
    },
    /// The webhooks listener failed.
    Hooks,
}
use jig_protocol_config::Problems;

pub use skein_forge_setup::reconcile::{Stage as SetupStage, Why as SetupWhy};

/// A startup setup failure contains only closed, credential-free values.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SetupStopped {
    NotPrepared,
    Stopped { plan: SetupPlan, step: SetupStep },
    Missing,
    Busy,
    Removal { path: Box<[u8]>, errno: i32 },
    Unreadable { path: Box<[u8]>, why: HandoverUnread },
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SetupPlan {
    Create,
    Reconcile,
    TakeOver,
    Adopt,
}
/// Closed file refusals carry no file contents.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HandoverUnread {
    NotAFile,
    TooLarge,
    Unsafe(skein_io::file::Unsafe),
    Kernel(i32),
}
#[derive(Debug)]
pub struct HandoverRead {
    pub forge: u16,
    pub run: skein_lib::Token,
    pub path: Box<[u8]>,
    pub max: u32,
}
#[derive(Debug)]
pub struct HandoverRemoval {
    pub forge: u16,
    pub run: skein_lib::Token,
    pub path: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SetupStep {
    pub stage: skein_forge_setup::reconcile::Stage,
    pub why: skein_forge_setup::reconcile::Why,
}

/// Operator diagnostics taken by the shell (protocol/forge/engine.md, 3).
#[derive(PartialEq, Eq, Debug)]
pub enum Note {
    /// A client stream closed on its queue or translation bound (client/engine.md).
    ClientStreamFailed {
        sign_in: u64,
        why: StreamFailure,
    },
    /// A configured charter may differ from the store's settings (protocol/configuration/README.md, 3).
    StartupWarning {
        line: Box<[u8]>,
    },
    /// The gate has kept a party's input for at least 1 s (protocol/README.md, 6.2).
    CoreWaiting {
        why: CoreWait,
    },
    CoreAnswers,
    /// A forge's link changed or was refused (temper/docs/design/domain/forge/adoption.md, 6).
    ForgeLink {
        forge: u16,
        note: temper_engine_domain_forge::LinkNote,
    },
    /// The engine's hello refusal, suppressed until its words change (protocol/cluster.md, 3).
    HelloRefused {
        channel: u64,
        text: Box<[u8]>,
    },
    /// A failed answer accepted by the hub, for stderr only (domain/worker/README.md, 4.3).
    RunFailed(temper_engine_domain::RunDiagnostic),
    /// A refused assignment accepted by the hub, for stderr only (domain/worker/README.md, 4.3).
    RunRefused(RunRefusalDiagnostic),
    /// A failed or refused job accepted by the engine (domain/worker/jobs.md).
    JobAnswered(JobDiagnostic),
    /// The worker's own job evidence, kept outside facts and results (domain/worker/jobs.md).
    WorkerJob {
        run: u64,
        attempt: u64,
        kind: u32,
        verb: Box<[u8]>,
        ending: WorkerJobEnding,
        why: Box<[u8]>,
    },
    /// A forge read refusal with its task, tool, layer and authority findings.
    ReadRefused {
        task: u64,
        attempt: u64,
        tool: Box<[u8]>,
        layer: &'static str,
        reason: &'static str,
        findings: Box<[jig_domain_authority::Finding]>,
        error: Option<skein_forge::api::Error>,
        job: Option<skein_forge::api::JobAttempt>,
    },
    /// A brief section's forge read failed (temper/docs/design/domain/forge/change/repairs.md).
    BriefReadRefused {
        task: u64,
        section: &'static [u8],
        stage: &'static str,
        reason: &'static str,
        heads: Option<(skein_forge::api::Commit, skein_forge::api::Commit)>,
        error: Option<skein_forge::api::Error>,
    },
    /// A whole brief failed with its count and fixed reason (jig/docs/design/domain/engine/briefs.md, 1.1).
    BriefFailed {
        task: u64,
        counts: &'static str,
        reason: Box<[u8]>,
    },
    /// A decoded call that could not enter the engine's admission room.
    CallAdmissionRefused {
        task: u64,
        attempt: u64,
        tool: Box<[u8]>,
        layer: &'static str,
        reason: &'static str,
    },
    /// One patch finished before the store opened (jig's protocol/store/opening.md, 2).
    Patched {
        of: jig_protocol_store::Owner,
        patch: u32,
        counts: jig_protocol_store::Counts,
    },
    /// A bounded reset diagnostic (temper/docs/design/protocol/setup/playground.md).
    Playground {
        line: Box<[u8]>,
    },
    /// A listener bound its address (protocol/cluster.md, 3).
    Listening {
        listener: Listening,
        addr: core::net::SocketAddr,
    },
    Linked {
        worker: Box<[u8]>,
        address: Box<core::net::SocketAddr>,
    },
    /// Once per linked link lost (protocol/cluster.md, 3).
    Lost {
        worker: Box<[u8]>,
        address: Box<core::net::SocketAddr>,
        why: WorkerLoss,
    },
    /// A channel that said hello ended (protocol/cluster.md, 3).
    WorkerLost {
        worker: Option<Box<[u8]>>,
        channel: skein_lib::Token,
        why: EngineWorkerLoss,
    },
    /// A record the engine's link did not send (protocol/cluster.md, 3).
    WorkerUnsent {
        worker: Box<[u8]>,
        channel: skein_lib::Token,
        unsent: jig_protocol_engine_workers::Unsent,
    },
    /// An opening ended, suppressed by name and reason until hello (protocol/cluster.md, 3).
    WorkerOpeningEnded {
        worker: Option<Box<[u8]>>,
        why: EngineWorkerLoss,
    },
    /// A workers' listener failed, reported at once (protocol/cluster.md, 3).
    WorkerListenerFailed {
        address: Option<skein_io::Addr>,
        error: Option<skein_io::Error>,
    },
    Refused {
        worker: Box<[u8]>,
        address: Box<core::net::SocketAddr>,
        reason: Refusal,
    },
    ForgeLacking {
        number: u16,
        names: Box<[u8]>,
    },
    /// A set-up ask refused by admission or the box link (protocol/forge/engine.md, 8).
    SetUpRefused {
        number: u16,
        why: temper_engine_domain_forge::SetUpRefusal,
    },
    /// A diagnostic from the box link (protocol/forge/engine.md, 8).
    BoxLink {
        line: Box<[u8]>,
    },
    ForgeStopped {
        number: u16,
        why: skein_forge_connection::Stopped,
    },
    ForgeSetupStopped {
        number: u16,
        why: SetupStopped,
    },
    ForgeNotConnected {
        number: u16,
        why: NotConnected,
    },
    /// An entry newly unable to serve a forge sign-in meta (protocol/openid-provider.md, 7).
    ForgeSignIn {
        number: u16,
        client: Box<[u8]>,
        why: clients::forge_sign_in::Unserved,
    },
    /// A refused job-log redirect, with no signed path or query (protocol/forge/README.md, 2.6).
    ForgeOrigin {
        number: u16,
        origin: skein_forge::github::redirect::Origin,
    },
    ForgeUnkept {
        number: u16,
        installation: u64,
    },
}

/// The engine's listeners (protocol/cluster.md, 3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Listening {
    /// The clients' HTTP server.
    Clients,
    /// The workers' socket listener.
    Workers,
    /// The forge webhooks listener.
    Webhooks,
}

pub use temper_service_core::Refusal;

pub use jig_protocol_engine_workers::Loss as EngineWorkerLoss;
pub use jig_protocol_worker_wire::Loss as WorkerLoss;
pub use jig_protocol_worker_wire::Reason as WorkerRefusalReason;
pub use skein_channel::Closed as WorkerClosed;

pub use temper_engine_domain::Unsettled;

impl Service {
    /// Machine inputs loaded once by the shell (temper/docs/design/protocol/configuration/engine.md).
    pub fn configure_forge_runtime(&mut self, github: ForgeRuntime, roots: skein_tls::Config) {
        if let Some(engine) = &mut self.engine {
            assert!(engine.forge.serving.is_none(), "machine inputs precede serving");
            engine.forge.github = Some(github);
            engine.clients.pushes.roots = Some(roots.clone());
            engine.forge.roots = Some(roots);
        }
    }

    /// Push hosts awaiting this send's resolution (domain/forge/attention.md, 7.3).
    #[must_use]
    #[expect(clippy::type_complexity, reason = "resolution pairs an exchange token with its host and port")]
    pub fn unresolved_pushes(&self) -> Box<[(skein_lib::Token, Box<[u8]>, u16)]> {
        match &self.engine {
            Some(engine) => engine.clients.pushes.unresolved(),
            None => Box::new([]),
        }
    }
    pub fn resolved_push(&mut self, token: skein_lib::Token, address: Option<core::net::SocketAddr>) {
        if let Some(engine) = &mut self.engine {
            engine.clients.pushes.resolved(token, address, engine.env.now, &mut engine.clients.io);
        }
    }
    /// Kept names awaiting the shell's startup resolution (temper/docs/design/protocol/forge/engine.md, 2).
    #[must_use]
    #[expect(clippy::type_complexity, reason = "the startup resolution contract returns numbered host and port tuples")]
    pub fn unresolved(&self) -> Box<[(u16, Box<[u8]>, u16)]> {
        let Some(engine) = &self.engine else { return Box::new([]) };
        if engine.forge.serving.is_some() {
            return Box::new([]);
        }
        let mut names = skein_lib::List::with_capacity(engine.workers.forges.len());
        for forge in engine.workers.forges.iter() {
            if engine.forge.resolving.contains_key(&forge.number)
                && let jig_protocol_config::Address::Host { name, port } = &forge.api
            {
                names.push((forge.number, name.clone(), *port)).expect("kept forges");
            }
        }
        names.into_boxed()
    }

    /// Re-add a kept forge with the shell's answer before serving (temper/docs/design/protocol/forge/engine.md, 2).
    pub fn resolved(&mut self, forge: u16, address: Option<core::net::SocketAddr>) {
        let Some(engine) = &mut self.engine else { return };
        if engine.forge.serving.is_some() || engine.forge.unreachable(forge).is_none() {
            return;
        }
        engine.forge.resolving.remove(&forge);
        let Some(address) = address else { return };
        let Some(configured) = engine.workers.forges.remove(forge) else { return };
        engine.forge.down(skein_forge_connection::Request::Remove { forge });
        if let jig_protocol_config::Address::Host { name, .. } = &configured.api {
            engine.forge.names.insert(name.clone(), address.ip()).expect("kept host capacity");
        }
        let Some(mut endpoint) = forges::endpoint(engine, &configured) else {
            assert!(engine.workers.forges.insert(configured), "removed forge has table room");
            return;
        };
        endpoint.address = skein_io::Addr::Ip(address);
        engine.forge.endpoints.insert(forge, endpoint).expect("kept forge capacity");
        if forges::keep(engine, configured) {
            engine.setup.replace_endpoint(forge, address);
            if engine.forge_records.done() {
                forge::records::connect(engine, forge);
            }
        }
    }
}

/// A stream edge or translation refusal (jig/docs/design/protocol/client/README.md, 5).
pub use jig_protocol_engine_clients::StreamFailure;
