//! Join the worker vocabulary (temper/docs/design/protocol/services.md, 6).
#[cfg(feature = "worker")]
use crate::worker_alone::{End, Note, Stop};
#[cfg(not(feature = "worker"))]
#[path = "unbuilt_vocabulary.rs"]
mod unbuilt;
#[cfg(not(feature = "worker"))]
pub use unbuilt::{End, Note, Stop};
impl From<Note> for crate::Note {
    fn from(note: Note) -> Self {
        match note {
            Note::WorkerJob { run, attempt, kind, verb, ending, why } => {
                Self::WorkerJob { run, attempt, kind, verb, ending, why }
            }
            Note::Linked { worker, address } => Self::Linked { worker, address },
            Note::Lost { worker, address, why } => Self::Lost { worker, address, why },
            Note::Refused { worker, address, reason } => Self::Refused { worker, address, reason },
        }
    }
}
impl From<Stop> for crate::Stop {
    fn from(stop: Stop) -> Self {
        match stop {
            Stop::Start(problems) => Self::Start(problems),
        }
    }
}
impl From<End> for crate::End {
    fn from(end: End) -> Self {
        match end {
            End::Drained => Self::Drained,
            End::Aborted => Self::Aborted,
            End::Stopped(stop) => Self::Stopped(stop.into()),
        }
    }
}
