//! The job capability's boundary (temper/docs/design/domain/worker.md, 4.4;
//! temper/docs/design/protocol/tooling.md, 2 and 3).

use alloc::boxed::Box;
use jig_domain_link::{JobBound, JobKind};
use skein_lib::{Duration, Time, Token};

/// One kind the capability runs.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Kind {
    pub bound: JobBound,
    /// The command's second argument.
    pub verb: Box<[u8]>,
    /// Whether exit 0 delivers its writable repository's head.
    pub delivers: bool,
}

/// A job's spec, decoded by the service from temper's payload record.
#[derive(PartialEq, Eq, Debug)]
pub struct Spec {
    /// A relative path in the job's directory.
    pub command: Box<[u8]>,
    /// The request file's bytes.
    pub request: Box<[u8]>,
}

/// Parent to capability.
#[derive(PartialEq, Eq, Debug)]
pub enum Event {
    Run {
        owner: Token,
        kind: JobKind,
        spec: Spec,
        deadline: Duration,
        directory: Box<[u8]>,
    },
    Stop {
        owner: Token,
    },
    /// Terminal for `Write`.
    Written {
        owner: Token,
        written: bool,
    },
    Spawned {
        owner: Token,
    },
    /// The command's exit and the bounded end of its error output.
    Exited {
        owner: Token,
        exit: Exit,
        tail: Box<[u8]>,
    },
    /// Terminal for `Read`: missing or oversized files yield `None`.
    Read {
        owner: Token,
        bytes: Option<Box<[u8]>>,
    },
    /// Terminal for `Spawn`: every process is gone.
    Gone {
        owner: Token,
    },
}

/// How the command's process ended.
#[derive(PartialEq, Eq, Debug)]
pub enum Exit {
    Code(u8),
    Signal,
    /// The protocol layer killed the process group at its deadline.
    Deadline,
}

/// Capability to parent, including operations routed to the protocol layer.
#[derive(PartialEq, Eq, Debug)]
pub enum Request {
    Write {
        owner: Token,
        path: Box<[u8]>,
        bytes: Box<[u8]>,
    },
    /// Kill the process group at `deadline`, answering `Exited(Deadline)`.
    Spawn {
        owner: Token,
        program: Box<[u8]>,
        args: Box<[Box<[u8]>]>,
        #[expect(clippy::type_complexity, reason = "the job boundary carries the contract's exact environment pairs")]
        env: Box<[(Box<[u8]>, Box<[u8]>)]>,
        dir: Box<[u8]>,
        deadline: Time,
    },
    Kill {
        owner: Token,
    },
    Read {
        owner: Token,
        path: Box<[u8]>,
        most: u32,
    },
    Started {
        owner: Token,
    },
    StartRefused {
        owner: Token,
        why: Box<[u8]>,
    },
    Ended {
        owner: Token,
        end: End,
    },
    Gone {
        owner: Token,
        detail: Box<[u8]>,
    },
}

/// A job's ending; a stopped job may say nothing.
#[derive(PartialEq, Eq, Debug)]
pub enum End {
    Finished { result: Box<[u8]>, deliver: bool },
    Failed { detail: Box<[u8]> },
    Deadline,
}
