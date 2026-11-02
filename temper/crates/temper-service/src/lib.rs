//! Temper's engine and worker protocol services.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;
use alloc::boxed::Box;
pub mod diagnostics;
pub mod forge;
pub mod forges;
pub use forge::ForgeRuntime;
pub mod limits;
pub mod settings;
pub mod startup;
pub mod stock;
pub use startup::{Named, StoreLimits, named, startup, store_limits};
pub use temper_protocol_records::NotConnected;
pub mod system;
pub use system::{Service, iterate};
pub mod store;
#[cfg(test)]
mod tests;
pub mod tokens;

pub mod accounts;
pub mod assignment;
pub mod clients;
pub mod family;
pub mod identities;
pub mod program_sources;
pub mod server;
pub mod translate;
pub mod values;
mod worker;
mod worker_git;
pub mod worker_route;
pub mod worker_settings;
pub use worker::{Worker, WorkerRuntime};
pub mod workers;
pub use accounts::AccountsRuntime;
mod engine;
mod route;
mod setup;
mod start_refusal;
mod stop;
pub use start_refusal::refused_start;
mod tools;
pub use temper_engine_domain::loads::{Failure as LoadFailure, Kind as RestoreRangeKind};
/// How the service ended (protocol/README.md, 6).
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
    CommitFailed {
        number: u64,
    },
    Start(Problems),
    /// jig's start refused a fit at `phase` (jig engine.md 6).
    StartRefused {
        phase: Option<jig_domain_core::start::Phase>,
        failure: jig_domain_core::start::Failure,
    },
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
    /// The store refuses this start (jig's protocol/store.md, 5.3 and 10).
    StoreOpen {
        why: jig_protocol_store::OpenRefusal,
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

/// Operator diagnostics taken by the shell (protocol/forge.md, 2.3).
#[derive(PartialEq, Eq, Debug)]
pub enum Note {
    /// A forge read refusal with its task, tool, layer and authority findings.
    ReadRefused {
        task: u64,
        attempt: u64,
        tool: Box<[u8]>,
        layer: &'static str,
        reason: &'static str,
        findings: Box<[jig_domain_authority::Finding]>,
        error: Option<skein_forge::api::Error>,
    },
    /// A decoded call that could not enter the engine's admission room.
    CallAdmissionRefused {
        task: u64,
        attempt: u64,
        tool: Box<[u8]>,
        layer: &'static str,
        reason: &'static str,
    },
    /// One patch finished before the store opened (jig's protocol/store.md, 10.2).
    Patched {
        of: jig_protocol_store::Owner,
        patch: u32,
        counts: jig_protocol_store::Counts,
    },
    /// A bounded reset diagnostic (temper/docs/design/draft/playground-reset.md, 6.2).
    Playground {
        line: Box<[u8]>,
    },
    /// A listener bound its address (protocol/README.md, 15.3).
    Listening {
        listener: Listening,
        addr: core::net::SocketAddr,
    },
    Linked {
        worker: Box<[u8]>,
        address: Box<core::net::SocketAddr>,
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
    /// A set-up ask refused by admission or the box link (protocol/forge.md, 2.8).
    SetUpRefused {
        number: u16,
        why: temper_engine_domain_forge::SetUpRefusal,
    },
    /// A diagnostic from the box link (protocol/forge.md, 2.8).
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
    /// A refused job-log redirect, with no signed path or query (protocol/forge.md, 3.6).
    ForgeOrigin {
        number: u16,
        origin: skein_forge::github::redirect::Origin,
    },
    ForgeUnkept {
        number: u16,
        installation: u64,
    },
}

/// The engine's listeners (protocol/README.md, 15.3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Listening {
    /// The clients' HTTP server.
    Clients,
    /// The workers' socket listener.
    Workers,
    /// The forge webhooks listener.
    Webhooks,
}

/// The peer's refusal code, or a local opening/decoding refusal.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refusal {
    Peer(u16),
    Terms,
    Malformed,
}

pub use jig_protocol_worker_wire::Reason as WorkerRefusalReason;

pub use temper_engine_domain::Unsettled;

impl Service {
    /// Machine inputs loaded once by the shell (temper/docs/design/draft/playground-reset.md, 3).
    pub fn configure_forge_runtime(&mut self, github: ForgeRuntime, roots: skein_tls::Config) {
        if let Some(engine) = &mut self.engine {
            assert!(engine.forge.serving.is_none(), "machine inputs precede serving");
            engine.forge.github = Some(github);
            engine.forge.roots = Some(roots);
        }
    }

    /// Kept names awaiting the shell's startup resolution (playground-reset.md, 3).
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

    /// Re-add a kept forge with the shell's answer before serving (playground-reset.md, 3).
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
