//! One process composing the engine and worker (temper/docs/design/protocol/services.md, 6).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
extern crate alloc;
pub use temper_engine_service::{
    AccountsRuntime, ForgeRuntime, HandoverRead, HandoverRemoval, HandoverUnread, JobDiagnostic, JobDiagnosticEnding,
    Listening, STAGES, Stage, Unsettled, clients, stock,
};
#[cfg(test)]
use temper_engine_service::{assignment, family, forges, workers};
use temper_engine_service::{forge, store, tokens, values};
use temper_worker_service::Worker;
#[cfg(test)]
use temper_worker_service::worker_route;
pub use temper_worker_service::{WorkerJobEnding, WorkerRuntime, program_sources, worker_alone, worker_settings};
pub mod startup;
pub use startup::{Named, StoreLimits, named, startup, store_limits};
pub mod diagnostics;
pub mod limits;
pub mod settings;
pub mod system;
pub use system::{Service, iterate, iterate_stage};
mod worker_joined;
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum End {
    Engine(temper_engine_service::End),
    Worker(worker_alone::End),
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Stop {
    Engine(temper_engine_service::Stop),
    Worker(worker_alone::Stop),
}
#[derive(PartialEq, Eq, Debug)]
pub enum Note {
    Engine(temper_engine_service::Note),
    Worker(worker_alone::Note),
}
impl From<worker_alone::Note> for Note {
    fn from(note: worker_alone::Note) -> Self {
        Self::Worker(note)
    }
}

pub use temper_engine_service::{End as EngineEnd, Note as EngineNote, Stop as EngineStop};
#[cfg(test)]
mod translate {
    pub use temper_engine_service::translate::{identity, items, limits, parts};
    pub use temper_worker_service::worker_route::translate::{preparation, workspace};
}
impl From<worker_alone::End> for End {
    fn from(end: worker_alone::End) -> Self {
        Self::Worker(end)
    }
}
impl From<worker_alone::Stop> for Stop {
    fn from(stop: worker_alone::Stop) -> Self {
        Self::Worker(stop)
    }
}

#[cfg(test)]
mod tests;
