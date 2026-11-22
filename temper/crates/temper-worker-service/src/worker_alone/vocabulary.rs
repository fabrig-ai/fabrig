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
    /// Each fetch and check out the checkout asks (domain/worker/hosted-runs.md, 1.3).
    Git {
        run: u64,
        attempt: u64,
        job: Option<(u32, Box<[u8]>)>,
        repository: Box<[u8]>,
        what: temper_worker_domain::Git,
    },
    /// A link pause's operator line (domain/worker/README.md, 2).
    Paused {
        state: jig_domain_link::PauseState,
        drain: skein_lib::Duration,
    },
    /// The three parts of a figures change (configuration/figures.md, 3.2).
    FiguresTaken {
        figures: Box<[jig_protocol_worker_wire::Limit]>,
    },
    FiguresWaiting {
        figures: Box<[jig_protocol_worker_wire::Limit]>,
    },
    FiguresRefused {
        problem: Box<[u8]>,
    },
    Cancelling {
        run: u64,
        attempt: u64,
    },
    /// The longer of the word and this worker's grace (protocol/wires.md, 4).
    Restarting {
        keep: skein_lib::Duration,
    },
    /// An agent signal left to this worker to decide; no stop follows it.
    AgentSignalled {
        run: u64,
        attempt: u64,
        signal: &'static str,
        from: Option<u32>,
    },
    /// An agent fault, when reported, once per attempt (domain/worker/hosted-runs.md, 1.3).
    AgentStopped {
        run: u64,
        attempt: u64,
        failure: jig_domain_host::AgentFailure,
        no_progress_ms: Option<u64>,
    },
    /// A failed preparation, once when its answer first goes up (domain/worker/checkouts.md, 1.3).
    Unprepared {
        run: u64,
        attempt: u64,
        failure: jig_domain_host::Failure,
        detail: Box<[u8]>,
    },
    /// An entry the git sweep could not remove (protocol/git-sweep.md, 3).
    Unswept {
        path: Box<[u8]>,
        error: skein_io::kernel::Error,
    },
    ResultsIgnored {
        owner: u64,
        tag: u64,
        reason: Box<[u8]>,
    },
    CannotSendWhole {
        wire: &'static str,
        version: u16,
    },
    Untranslated {
        kind: u16,
        halt: skein_codec::patch::Halt,
    },
    StopBoundPassed {
        abandoned: u64,
    },
    WorkerJob {
        run: u64,
        attempt: u64,
        kind: u32,
        verb: Box<[u8]>,
        ending: crate::WorkerJobEnding,
        why: Box<[u8]>,
    },
    Linked {
        worker: Box<[u8]>,
        address: Box<core::net::SocketAddr>,
    },
    Lost {
        worker: Box<[u8]>,
        address: Box<core::net::SocketAddr>,
        why: crate::WorkerLoss,
    },
    OpeningRefused {
        reason: u16,
        text: Box<[u8]>,
    },
    Refused {
        worker: Box<[u8]>,
        address: Box<core::net::SocketAddr>,
        reason: crate::Refusal,
    },
}
