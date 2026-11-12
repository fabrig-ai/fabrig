//! The worker's own vocabulary (temper/docs/design/protocol/services.md, 5).
use alloc::boxed::Box;
use jig_protocol_config::Problems;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum End {
    Drained,
    Aborted,
    Stopped(Stop),
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Stop {
    Start(Problems),
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Note {
    StopBoundPassed { abandoned: u64 },
    WorkerJob { run: u64, attempt: u64, kind: u32, verb: Box<[u8]>, ending: crate::WorkerJobEnding, why: Box<[u8]> },
    Linked { worker: Box<[u8]>, address: Box<core::net::SocketAddr> },
    Lost { worker: Box<[u8]>, address: Box<core::net::SocketAddr>, why: crate::WorkerLoss },
    Refused { worker: Box<[u8]>, address: Box<core::net::SocketAddr>, reason: crate::Refusal },
}
impl From<Note> for crate::Note {
    fn from(note: Note) -> Self {
        match note {
            Note::StopBoundPassed { abandoned } => Self::StopBoundPassed { abandoned },
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
