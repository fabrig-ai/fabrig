//! The job capability's boundary (temper/docs/design/domain/worker/jobs.md;
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
    /// Whether to ask the command for a typed refusal.
    pub refusal: bool,
    /// The engine’s limit on a typed refusal result.
    pub result: u32,
    /// Whether to ask the command for an outdated finding.
    pub outdated: bool,
    pub hooks: Hooks,
}

/// Discovery, when requested, and the ceiling of commands run as hooks.
#[derive(PartialEq, Eq, Debug)]
pub struct Hooks {
    /// `Some` asks to find the moment's hook, with an empty path for no fallback.
    pub fallback: Option<Fallback>,
    pub ceiling: u32,
    /// What the prepared merge says about the hook's path.
    pub conflict: Option<Conflict>,
}

impl Hooks {
    #[must_use]
    pub fn new(fallback: Option<Box<[u8]>>, ceiling: u32) -> Self {
        let mut hooks = Self { fallback: None, ceiling, conflict: None };
        if let Some(program) = fallback {
            hooks.fallback = Some(Fallback { program, conflict: None });
        }
        hooks
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Fallback {
    pub program: Box<[u8]>,
    pub conflict: Option<Conflict>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Conflict {
    Path,
    Cut,
}

impl Conflict {
    #[must_use]
    pub const fn why(self) -> &'static [u8] {
        match self {
            Self::Path => b"left in conflict",
            Self::Cut => b"could not check for conflicts: the list was cut",
        }
    }
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
    /// The root completed temper's own prepare without a process (temper/docs/design/domain/worker/jobs.md).
    Prepared {
        owner: Token,
        result: Box<[u8]>,
        code: u8,
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
    /// An explicit hook time was cut at the project ceiling.
    TimeCut {
        owner: Token,
        line: Box<[u8]>,
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
    /// A check of the command failed before it could be spawned. The tail is the failure line.
    NotRun,
    /// Discovery found neither a hook nor an applicable fallback.
    Absent,
    /// The command ran past its own time; its tail starts with the reason.
    Time,
    /// The protocol layer reached the job's deadline.
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
    /// Find or check the command where applicable, and kill its group at its time or `deadline`.
    Spawn {
        owner: Token,
        program: Box<[u8]>,
        args: Box<[Box<[u8]>]>,
        #[expect(clippy::type_complexity, reason = "the job boundary carries the contract's exact environment pairs")]
        env: Box<[(Box<[u8]>, Box<[u8]>)]>,
        dir: Box<[u8]>,
        deadline: Time,
        hooks: Hooks,
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
    Absent,
    Failed { detail: Box<[u8]> },
    Deadline,
}
