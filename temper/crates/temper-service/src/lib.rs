//! One process composing the engine and worker (temper/docs/design/protocol/services.md, 6).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
extern crate alloc;
pub use temper_engine_service::*;
pub use temper_worker_service::{Worker, WorkerRuntime, WorkerJobEnding, identities, program_sources, worker_alone, worker_route, worker_settings};
pub mod startup;
pub use startup::{Named, StoreLimits, named, startup, store_limits};
pub mod settings;
pub mod limits;
pub mod diagnostics;
pub mod system;
pub use system::{Service, iterate, iterate_stage};
mod worker_joined;
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum End { Engine(temper_engine_service::End), Worker(worker_alone::End) }
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Stop { Engine(temper_engine_service::Stop), Worker(worker_alone::Stop) }
#[derive(PartialEq, Eq, Debug)]
pub enum Note { Engine(temper_engine_service::Note), Worker(worker_alone::Note) }
impl From<temper_engine_service::Note> for Note { fn from(note: temper_engine_service::Note) -> Self { Self::Engine(note) } }
impl From<worker_alone::Note> for Note { fn from(note: worker_alone::Note) -> Self { Self::Worker(note) } }
