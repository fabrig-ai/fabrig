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
    CannotSendWhole { wire: &'static str, version: u16 },
    Untranslated { kind: u16, halt: skein_codec::patch::Halt },
    StopBoundPassed { abandoned: u64 },
    WorkerJob { run: u64, attempt: u64, kind: u32, verb: Box<[u8]>, ending: crate::WorkerJobEnding, why: Box<[u8]> },
    Linked { worker: Box<[u8]>, address: Box<core::net::SocketAddr> },
    Lost { worker: Box<[u8]>, address: Box<core::net::SocketAddr>, why: crate::WorkerLoss },
    OpeningRefused { reason: u16, text: Box<[u8]> },
    Refused { worker: Box<[u8]>, address: Box<core::net::SocketAddr>, reason: crate::Refusal },
}
