//! Top-owned state and coherent current facts (domain/landing.md, 1–10).
use alloc::boxed::Box;
use skein_lib::{Duration, Wall};

use crate::Verdict;

/// A forge object id: 32 bytes, a SHA-1 one padded.
pub type Commit = [u8; 32];
/// A gate's key (domain/landing.md, 6.1).
pub type Key = [u8; 32];
/// Bound of a branch-name-safe gate or after-landing tier name.
pub const CHECK_NAME_BYTES: usize = 32;
/// Bound of a workflow file name.
pub const WORKFLOW_BYTES: usize = 255;

/// A repository's forge number and forge id.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Repository {
    pub forge: u16,
    pub repository: u32,
}

/// A repository's test gate (domain/landing.md, 13.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Gate {
    /// Unique branch-name-safe final segment of `temper/gate/<name>`.
    pub name: Box<[u8]>,
    /// The workflow file as named by dispatch.
    pub workflow: Box<[u8]>,
    /// Path prefixes of whole segments, never patterns.
    pub ignored: Box<[Box<[u8]>]>,
    pub blocking: bool,
    /// Include the round’s base in the key and dispatch inputs.
    pub base: bool,
}

/// The project's landing settings needed by the policy.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Project {
    pub gate_retries: u32,
    pub gate_timeout: Duration,
}
impl Project {
    pub const DEFAULT: Self = Self { gate_retries: 2, gate_timeout: Duration::from_millis(3_600_000) };
}

/// Whether a fact holds, as for a change.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Status {
    Unknown,
    Pending,
    Passed,
    Failed,
}

/// A landing's committed place (domain/landing.md, 3, 5–8).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Phase {
    Waiting,
    Preparing,
    Proving,
    Proved { at: Wall },
    Turn { at: Wall },
    Landed,
    Held { was: Box<Phase>, why: Hold },
    Cancelled,
}

/// Why a release is needed (domain/landing.md, 7.4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Hold {
    Faults { member: u16, gate: u16 },
    Prepare,
    Record { member: u16 },
    Timeout { member: u16, gate: u16, run: u64 },
    Turn,
}

/// One gate's proof at a member's candidate (domain/landing.md, 7).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Proof {
    Unkeyed,
    Keyed { key: Key },
    Dispatched { key: Key, try_: u16 },
    Passed { key: Key },
    Failed { key: Key },
}

/// What this landing has asked to post on the candidate.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Posted {
    Nothing,
    Pending,
    Verdict,
}

/// One gate's proof and posting, owned by the top.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct GateProof {
    pub proof: Proof,
    pub posted: Posted,
    /// Runner faults for this key, against `gate_retries`.
    pub faults: u32,
}

/// One change into one repository (domain/landing.md, 4.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[expect(clippy::struct_excessive_bools, reason = "independent per-member effect and judgement receipts")]
pub struct Member {
    pub change: u64,
    pub repository: Repository,
    pub pull: u64,
    /// Head and main from which the candidate was prepared.
    pub head: Option<Commit>,
    pub base: Option<Commit>,
    pub candidate: Option<Commit>,
    /// One per repository gate, in configuration order.
    pub gates: Box<[GateProof]>,
    /// Preparation last asked of the change.
    pub asked: Option<u16>,
    /// Main's commit once landed: the candidate.
    pub landed: Option<Commit>,
    pub left: Option<Leave>,
    pub unit: u16,
    pub branch: Box<[u8]>,
    pub combined: Option<Combine>,
    pub recorded: bool,
    pub told: bool,
    pub judging: bool,
    /// The leave decision has been emitted, including for cascaded leaves.
    pub leave_told: bool,
}

/// Why a member leaves, retaining its first-ready time (domain/landing.md, 4.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Leave {
    Conflict,
    HeadMoved { head: Commit },
    Cancelled,
    Held,
    Refused { gate: u64 },
    Failed { gate: u16, run: u64 },
    EarlierConflict,
    Unit { change: u64 },
    Dependency { change: u64 },
    Blamed { gate: u16, run: u64, red: Commit, green: Commit },
}

/// The top-owned landing task record (domain/landing.md, 12).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Landing {
    /// Last member to leave, in stack order within a decision (landing-history.md, 3.3).
    pub last_left: Option<u16>,
    pub task: u64,
    pub project: u32,
    pub phase: Phase,
    /// Upstream first: a member follows every member it depends on (domain/landing.md, 4.1, 8).
    pub members: Box<[Member]>,
    /// Candidates' preparation, counted from 1.
    pub preparation: u16,
    pub stale_proofs: u32,
    /// The train's landing branch (domain/landing.md, 9, 12).
    pub branch: Box<[u8]>,
    pub round: Round,
}

/// A round's own branch, preparation and pull request (domain/landing.md, 5.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Round {
    pub branch: Box<[u8]>,
    pub pull: Option<u64>,
    pub upstream: Option<Commit>,
    /// One-member execution's projection, retained for the top's single-member views.
    pub combined: Option<Combine>,
    pub recorded: bool,
    pub told: bool,
    pub judging: bool,
    pub ending: Option<RoundEnd>,
    pub failure: Box<[u8]>,
    /// First branch to delete when the round ends unlanded.
    pub changed: Option<u16>,
    pub search: Option<Search>,
}
/// Observed unit boundaries, numbered by members from 1; 0 is the base.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Search {
    pub green: u16,
    pub red: u16,
}

impl Round {
    #[must_use]
    pub fn new(branch: Box<[u8]>) -> Self {
        Self {
            branch,
            pull: None,
            upstream: None,
            combined: None,
            recorded: false,
            told: false,
            judging: false,
            ending: None,
            failure: Box::new([]),
            changed: None,
            search: None,
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Combine {
    Merge,
    Squash,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RoundEnd {
    Cut,
    Stale,
    Leave,
    Conflict,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Preparation {
    Idle,
    Running,
    Conflict,
    Failed,
}

/// A prepare job's outcome for a member (domain/landing.md, 5.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Outcome {
    Combined,
    Conflict,
    EarlierConflict,
}

/// Current dispatch of this key. The top follows the current try, clearing
/// its old run when committing a retry (domain/landing.md, 7.4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Flight {
    None,
    Mine { run: Option<Verdict> },
    Other,
}

/// Current facts for the gate at this candidate.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct GateFacts {
    /// From the repository's configured gate at this position.
    pub blocking: bool,
    pub key: Option<Key>,
    /// A receipt of this key: Passed or Failed, otherwise Unknown.
    pub receipt: Status,
    pub flight: Flight,
    /// Status of this landing's most recently requested posting effect.
    pub posting: Status,
    pub expired: Option<u64>,
    pub run: u64,
}

/// Fresh facts for one member, in the landing's member order.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[expect(clippy::struct_excessive_bools, reason = "independent member observations")]
pub struct MemberFacts {
    pub ready: bool,
    pub cancelled: bool,
    pub head: Commit,
    pub main: Commit,
    /// Whether head contains main.
    pub contains_main: Status,
    pub asked: bool,
    /// This landing's own merge effect: Unknown until asked, then Pending,
    /// Passed when made or Failed when refused.
    pub merge: Status,
    pub merged: Option<Commit>,
    pub gates: Box<[GateFacts]>,
    /// The base already contains the head.
    pub contained: Status,
    pub candidate: Option<(Commit, Combine)>,
    pub preparation: Preparation,
    pub record: Status,
    pub held: bool,
    pub judgement: Status,
    pub refused: Option<u64>,
    pub outcome: Option<Outcome>,
    /// Positions of other members this member depends on.
    pub dependencies: Box<[u16]>,
    pub deleted: Status,
}

/// The coherent snapshot supplied by the top.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Facts {
    pub members: Box<[MemberFacts]>,
    pub turn_free: bool,
    pub released: bool,
    pub opened: Option<u64>,
    pub opening: Status,
    pub closed: Status,
    pub preparation: Preparation,
}

/// The candidate's gate status (domain/landing.md, 10).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Posting {
    Pending,
    Ran { passed: bool },
    Received,
}

/// One thing due in this snapshot, as for a change.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Decision {
    None,
    Wait {
        until: Option<Wall>,
    },
    /// Stack from `main`, the round's base or the last kept prefix, starting at `member`.
    Prepare {
        member: u16,
        head: Commit,
        main: Commit,
    },
    Key {
        member: u16,
        gate: u16,
        candidate: Commit,
    },
    Dispatch {
        member: u16,
        gate: u16,
        key: Key,
        try_: u16,
    },
    Retry {
        member: u16,
        gate: u16,
        key: Key,
        try_: u16,
    },
    Post {
        member: u16,
        gate: u16,
        posting: Posting,
    },
    Repair {
        member: u16,
        gate: u16,
    },
    Proved,
    Merge {
        member: u16,
        candidate: Commit,
    },
    Stale,
    Finish,
    Cancel,
    Hold(Hold),
    Open,
    Record {
        member: u16,
        prefix: Commit,
    },
    Landed {
        member: u16,
        prefix: Commit,
    },
    Close,
    Delete {
        member: u16,
    },
    Resolve {
        member: u16,
        base: Commit,
    },
    Leave {
        member: u16,
        why: Leave,
    },
    Judge {
        member: u16,
    },
}

/// State to commit atomically with the decision.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Stepped {
    pub landing: Landing,
    pub decision: Decision,
}

/// Admission bounds and the run polling interval.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub members: u32,
    pub gates: u32,
    pub poll: Duration,
}
