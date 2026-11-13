//! Landing history's bounded rows and the writer's input
//! (temper/docs/design/domain/landing-history.md, 2, 3 and 4).
use alloc::boxed::Box;
use skein_forge::api::{Commit, Repository};
use skein_lib::Wall;

pub const MARKS: u32 = 2048;
pub const HOUR_NS: u64 = 3_600_000_000_000;
pub const REASON_BYTES: u32 = 256;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct InLanding {
    pub task: u64,
    pub round: u16,
    pub unit: u16,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Step {
    pub what: What,
    pub landing: Option<InLanding>,
    pub head: Option<Commit>,
    pub base: Option<Commit>,
    pub operation: Operation,
    pub attempt: u32,
    pub source_at: Option<Wall>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Operation {
    Own,
    Of(u64),
}
/// A job's durable attempt transition, supplied by the root; never stored here.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct JobStep {
    pub task: u64,
    pub attempt: u32,
    pub job: Job,
    pub outcome: Option<JobOutcome>,
    pub head: Option<Commit>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Event {
    pub project: u32,
    pub number: u64,
    pub at: Wall,
    pub source_at: Option<Wall>,
    pub change: u64,
    pub repository: Repository,
    pub landing: Option<InLanding>,
    pub head: Option<Commit>,
    pub base: Option<Commit>,
    pub operation: u64,
    pub attempt: u32,
    pub what: What,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ChangeHistory {
    pub last: u64,
    pub ready: Option<u64>,
    pub queued: Option<u64>,
    pub paused: Option<u64>,
    pub held: Option<u64>,
    pub landing_held: Option<u64>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Project {
    pub project: u32,
    pub next: u64,
    pub kept: u64,
    pub whole_since: Option<Wall>,
    pub marks: Box<[Mark]>,
    pub sweeping: Option<u64>,
    pub removing: bool,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Mark {
    pub hour: Wall,
    pub first: u64,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ChangeIndex {
    pub project: u32,
    pub change: u64,
    pub number: u64,
    pub landing: Option<u64>,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct LandingIndex {
    pub project: u32,
    pub landing: u64,
    pub number: u64,
    pub change: u64,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Last {
    pub project: u32,
    pub number: u64,
    pub change: u64,
    pub at: Wall,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Member {
    pub change: u64,
    pub unit: u16,
    pub head: Commit,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Proof {
    pub gate: u16,
    pub run: u64,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Unready {
    Held,
    HeadMoved,
    Refused,
    Dependency,
    Approval,
    Conflict,
    Closed,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Line {
    Queue,
    Landing,
    ForgeQueue,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum UpdateOutcome {
    Updated,
    Conflict,
    Failed,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Job {
    Prepare,
    Merge,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum JobOutcome {
    Done,
    Conflict,
    Failed,
    Lost,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum With {
    Branch,
    Earlier,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Ask {
    Resolution,
    Repair,
    Judgement,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Answer {
    Pushed,
    Passed,
    Refused,
    Nothing,
    Failed,
    Cancelled,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Pause {
    BranchRed,
    TierRed,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RoundCause {
    Turned,
    Left,
    Stale,
    Cancelled,
    Cut,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GateVerdict {
    Passed,
    Failed,
    Fault,
    Timeout,
    Superseded,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RecordedState {
    Merged,
    Closed,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Ending {
    NoChange,
    Cancelled,
    Failed,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LandingOutcome {
    Done,
    Cancelled,
    Held,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum On {
    Change,
    Landing,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AfterVerdict {
    Verified,
    Red,
    Cleared,
    Uncleared,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Departure {
    Landing(temper_engine_domain_forge_landing::Leave),
    Unready,
    Conflict,
    Repair,
    Ejected,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum By {
    Party(u64),
    Task(u64),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum What {
    Ready,
    Unready { why: Unready },
    Joined { line: Line, place: Option<u32>, priority: Option<i32> },
    Departed { line: Line, cause: Departure },
    Selected,
    Update { style: skein_forge::api::UpdateStyle },
    Updated { outcome: UpdateOutcome },
    Job { job: Job, task: u64 },
    JobEnded { job: Job, outcome: JobOutcome },
    Conflict { with: With, files: u32 },
    Asked { ask: Ask, task: u64, gate: Option<u16> },
    Answered { ask: Ask, outcome: Answer },
    Pushed,
    Delivered { checked: bool },
    Paused { why: Pause },
    Resumed,
    Formed { members: Box<[Member]>, place: u16 },
    RoundEnded { cause: RoundCause },
    Gate { gate: u16, key: [u8; 32] },
    GateEnded { gate: u16, key: [u8; 32], verdict: GateVerdict },
    Proved { proofs: Box<[Proof]> },
    Search,
    Searched { green: Option<Commit>, red: Commit, blamed: u16 },
    Probe,
    ProbeEnded { green: bool },
    Turn { members: u16 },
    Landed,
    Recorded { state: RecordedState },
    Ended { ending: Ending },
    LandingEnded { outcome: LandingOutcome },
    Held { on: On, reason: Box<[u8]>, by: Option<By> },
    Released { on: On, reason: Box<[u8]>, by: Option<By> },
    After { verdict: AfterVerdict },
}

/// Heap payload held by an event, excluding its inline row.
#[must_use]
pub fn event_bytes(event: &Event) -> Option<u64> {
    use core::mem::size_of;
    match &event.what {
        What::Held { reason, .. } | What::Released { reason, .. } => Some(skein_lib::len_u64(reason.len())),
        What::Formed { members, .. } => {
            skein_lib::len_u64(members.len()).checked_mul(u64::try_from(size_of::<Member>()).ok()?)
        }
        What::Proved { proofs } => {
            skein_lib::len_u64(proofs.len()).checked_mul(u64::try_from(size_of::<Proof>()).ok()?)
        }
        What::Ready
        | What::Unready { .. }
        | What::Joined { .. }
        | What::Departed { .. }
        | What::Selected
        | What::Update { .. }
        | What::Updated { .. }
        | What::Job { .. }
        | What::JobEnded { .. }
        | What::Conflict { .. }
        | What::Asked { .. }
        | What::Answered { .. }
        | What::Pushed
        | What::Delivered { .. }
        | What::Paused { .. }
        | What::Resumed
        | What::RoundEnded { .. }
        | What::Gate { .. }
        | What::GateEnded { .. }
        | What::Search
        | What::Searched { .. }
        | What::Probe
        | What::ProbeEnded { .. }
        | What::Turn { .. }
        | What::Landed
        | What::Recorded { .. }
        | What::Ended { .. }
        | What::LandingEnded { .. }
        | What::After { .. } => Some(0),
    }
}

mod read;
pub use read::{Cursor, HistoryRange, Of, Query, Reply};
