//! Exact retained chip owner metadata (client/domain/contract.md, 5.1;
//! objects.md, 3). Optional observations never attest placement or clocks.
use super::{
    Budget, ClosingStage, DocumentText, Executor, HoldCategory, Part, Requester, ResultValue, RetryNotice, RunStage,
    WakePolicy,
};
use alloc::boxed::Box;
use skein_lib::{Duration, List, Token, Wall};

/// All metadata supplied by the chip owner, independent of its coarse phase.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChipDetails {
    pub dependencies: List<Dependency>,
    pub project: u32,
    pub executor: Executor,
    pub phase_since: Wall,
    pub run_since: Option<Wall>,
    pub attempt: Option<u64>,
    pub retry: Option<RetryNotice>,
    pub wake: WakePolicy,
    pub person_wait: Option<PersonWait>,
    pub hold: Option<ChipHold>,
    pub budget: Budget,
    pub activity: u64,
    pub resources: List<ResourceLine>,
    pub resources_cut: bool,
    pub resources_observed: bool,
    pub procedure: Option<Part>,
    pub live_delegates: u32,
    pub closing_connectors: Option<List<u16>>,
    pub ending: Option<ChipEnding>,
    pub ended_at: Option<Wall>,
    pub observation: Option<ChipObservation>,
}
/// A row's absent holder/writer means unknown when `resources_observed` is false.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceLine {
    pub part: Part,
    pub kind: u16,
    pub taken: bool,
    pub holder: Option<u64>,
    pub writer: Option<ResourceWriter>,
    pub writer_lost: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResourceWriter {
    Run { task: u64, attempt: u64 },
    Effect { entry: u64 },
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PersonWait {
    pub since: Wall,
    pub stall: Duration,
    pub deadline: Wall,
    pub notice: Option<PersonStallNotice>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PersonStallNotice {
    pub message: u64,
    pub at: Wall,
    pub activity: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChipHold {
    pub placed: Option<super::PlacedHold>,
    /// The connector's retained explanation of its effect or procedure decision.
    pub words: Option<DocumentText>,
    pub circle: Option<Circle>,
    /// Separate account supplied by the core; never inferred from the origin.
    pub account: Option<Part>,
    pub source: Option<Part>,
    pub reason: HoldCategory,
    pub revision: u64,
    pub before: PriorPhase,
    pub escalation: Option<HoldEscalation>,
    pub tries: FailureTries,
    pub hold_wait_since: Option<Wall>,
}
/// Prior owner stages have no phase clock or observed run facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PriorPhase {
    Waiting,
    Active(RunStage),
    Closing(ClosingStage),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FailureTries {
    pub transient: u32,
    pub permanent: u32,
    pub run: u32,
    pub agent: u32,
    pub lost: u32,
    pub invalid: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum HoldEscalation {
    Routing,
    Waiting { holder: EscalationHolder, entry: u64, since: Wall },
    Rejected { by: u64, reason: DocumentText },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EscalationHolder {
    Task(u64),
    Person(u64),
    Role { project: u32, role: u32 },
}
/// Whole result and cancellation requester supplied by the owner.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ChipEnding {
    Done(ResultValue),
    Failed(DocumentText),
    Cancelled { reason: DocumentText, result: Option<ResultValue>, by: Requester, at: Wall },
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChipObservation {
    pub channel: Token,
    pub work: ChipWork,
    pub since: Wall,
    pub reported_at: Wall,
    pub started: Option<Wall>,
}
/// Chip work's raw host-routing bytes have no completion/provider identity.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ChipWork {
    Preparing,
    Starting,
    Active,
    Writing,
    Tool { call: Box<[u8]>, tool: Box<[u8]> },
    Waiting,
    Ending,
}

/// Added-dependency facts carried beside task links (contract.md, 5.1).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Dependency {
    pub number: u64,
    pub pending: bool,
    pub added: Option<DependencyAdded>,
    pub keeps: bool,
    pub status: Option<super::TaskStatus>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DependencyAdded {
    pub at: Wall,
    pub by: Requester,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Circle {
    pub tasks: List<u64>,
    pub cut: bool,
}
