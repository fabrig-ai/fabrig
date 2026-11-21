//! Worker pause requests (jig/docs/design/domain/engine/workers.md, 5).
use crate::{
    Ask, Event, Key,
    contract::{self, WorkerPauseMode},
};
use alloc::boxed::Box;
use skein_lib::RequestKey;

/// Ask a linked worker to pause, drain or resume, under the caller's engine name.
#[must_use]
pub fn pause_worker(key: Key, request: RequestKey, worker: u32, mode: WorkerPauseMode) -> Event {
    Event::Send { key, name: Some(request), request: Ask::Jig(Box::new(contract::Ask::PauseWorker { worker, mode })) }
}
