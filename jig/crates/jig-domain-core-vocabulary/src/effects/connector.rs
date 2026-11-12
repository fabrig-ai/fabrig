//! Connector vocabulary crossing the application's root in jig's terms.
//! Closed at once (jig/docs/design/domain/people/acts.md, 2).
//! The root translates numbered connectors' own types to these values;
//! connector-owned payloads remain with their owner (domain/connectors.md, 2).

use crate::settings;
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Token, Wall};

/// Core-authorized task identity and funding for one connector repair.
#[derive(Hash, Debug)]
pub struct RepairSeed {
    pub number: u64,
    pub project: u32,
    pub period: u64,
    pub period_budget: u64,
    pub open_period: bool,
    pub authority: tasks::Authority,
    pub budget: u64,
}

impl RepairSeed {
    /// The allocated task number for root-owned connector holding translation.
    #[must_use]
    pub const fn number(&self) -> u64 {
        self.number
    }
}

/// What one adopted resource permits this project to do.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ResourceRole {
    /// The deployment owns the resource.
    Owned,
    /// It writes as a participant of a shared resource.
    Participant,
    /// It may read, but may not write.
    Context,
    /// It is known but unavailable.
    Unavailable,
}

/// How a named resource is held while a task uses it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum HoldKind {
    /// No task hold is needed.
    Shared,
    /// One task holds the resource at a time.
    Exclusive { wait: bool },
    /// At most the current number of slots may be held.
    Pooled { slots: u32, wait: bool },
}

/// What a connector's effect does to its resource.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EffectForm {
    Creation,
    Transition,
    Set,
}

/// Promise available after an effect's uncertain outcome.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Recovery {
    Keyed,
    Conditional,
    Idempotent,
    Unrecoverable,
}

/// The inspectable part of a connector-owned effect.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct EffectDescription {
    /// Whether this kind waits while its task has pending dependencies.
    pub waits_for_dependencies: bool,
    /// Connector number assigned by the application root.
    pub connector: u16,
    /// Stable purpose within the asking task.
    pub purpose: u64,
    /// Authority's resource and state description.
    pub effect: authority::Effect,
    /// Effect's shape for outbox and recovery.
    pub form: EffectForm,
    /// Connector's declared recovery class.
    pub recovery: Recovery,
}

/// Why a described effect was not admitted, including refusals before authority checked it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EffectRefusal {
    Authority,
    Obsolete,
    Description,
    Capacity,
    Dependencies,
    Proposal,
    Unavailable,
}

/// A default a connector stored that the application no longer gives
/// (jig/docs/design/domain/engine/settings.md, 5; jig/docs/design/domain/connectors.md, section 17).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DroppedDefault {
    pub kind: u32,
    pub scope: u32,
    pub number: u32,
}

/// A connector's answer in jig's vocabulary, after root translation.
/// Application settings: jig/docs/design/domain/engine/settings.md, 5;
/// jig/docs/design/domain/connectors.md, section 17.
#[derive(Hash, Debug)]
pub enum Event {
    /// A restore's judged items (jig/docs/design/domain/engine/settings.md, 10; jig/docs/design/domain/connectors.md, 17).
    SettingsRestored {
        owner: Token,
        judged: Box<[Judged]>,
    },
    SettingsJudged {
        owner: Token,
        judged: Box<[Judged]>,
    },
    SettingsDefaults {
        connector: u16,
        defaults: Box<[DefaultState]>,
        dropped: Box<[DroppedDefault]>,
    },
    Resource {
        project: u32,
        name: tasks::Name,
        role: ResourceRole,
        hold: HoldKind,
    },
    PoolSlots {
        name: tasks::Name,
        slots: u32,
    },
    Described {
        owner: Token,
        description: Box<EffectDescription>,
    },
    DescribeRefused {
        owner: Token,
    },
    DescribeBusy {
        owner: Token,
    },
    Verdict {
        owner: Token,
        judge: authority::Judge,
        verdict: authority::Verdict,
        at: Wall,
        guarded: bool,
        state: [u8; 32],
    },
    Procedure {
        task: u64,
        step: u64,
        decision: Box<tasks::ProcedureDecision>,
    },
    Outbox {
        /// Number of the connector that owns this outbox entry.
        connector: u16,
        entry: u64,
        task: u64,
        outcome: OutboxOutcome,
    },
    /// A projection's outcome settles writers independently of any task or call.
    ProjectionOutbox {
        connector: u16,
        entry: u64,
        outcome: OutboxOutcome,
    },
    News {
        task: u64,
        subscription: u64,
        topic: u64,
        class: tasks::NewsClass,
        words: Box<[u8]>,
    },
    SectionReady {
        section: Token,
        size: Option<u32>,
    },
    WorkspaceReady {
        task: u64,
        size: u32,
    },
    Adopted {
        project: u32,
        name: tasks::Name,
        role: AdoptionRole,
        home: bool,
    },
    Drift {
        task: u64,
        resource: tasks::Name,
    },
    /// This connector has settled every effect of a closing task.
    Closed {
        task: u64,
        connector: u16,
    },
    RestartDone {
        stage: RestartStage,
    },
}

/// An outbox terminal or uncertain write.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum OutboxOutcome {
    Made,
    Failed,
    Withdrawn,
    /// A person settled an uncertain effect as not made.
    NotMade,
    Uncertain,
    /// The connector cannot safely retry this entry; a person must resolve it.
    Held {
        entry: u64,
    },
}

/// The connector's place in the core-owned restart order.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RestartStage {
    Restored,
    ReadAfresh,
    Settled,
}

/// A project's durable adoption role (domain/model.md, 6.1; domain/connectors.md, 12).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AdoptionRole {
    Owned,
    Participant,
    Fork,
    Context,
}

/// One party the connector learned while adopting a resource.
#[derive(Debug)]
pub struct AdoptedParty {
    pub subject: Box<[u8]>,
    pub role: people::Role,
}

/// A connector's judgment of one item (jig/docs/design/domain/engine/settings.md, 5;
/// jig/docs/design/domain/connectors.md, section 17).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Judged {
    Accept { size: u32, charters: Box<[u32]> },
    Same,
    Refuse { why: Box<[u8]> },
}

/// A connector's default metadata (jig/docs/design/domain/engine/settings.md, 5;
/// jig/docs/design/domain/connectors.md, section 17).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DefaultState {
    pub kind: u32,
    pub scope: u32,
    pub number: u32,
    pub size: u32,
    pub charters: Box<[u32]>,
    pub differs: bool,
}

/// What the core asks a numbered connector to do.
/// Application settings: jig/docs/design/domain/engine/settings.md, 5;
/// jig/docs/design/domain/connectors.md, section 17.
#[derive(Hash, Debug)]
pub enum Ask {
    /// Erase replaced values below the history cutoff (jig/docs/design/domain/engine/settings.md, 10; jig/docs/design/domain/connectors.md, 17).
    ForgetReplaced {
        before: u64,
    },
    /// Keep restored values as proposal items (jig/docs/design/domain/engine/settings.md, 10; jig/docs/design/domain/connectors.md, 17).
    RestoreSettings {
        owner: Token,
        proposal: u64,
        items: Box<[settings::application::Restored]>,
    },
    /// Judge items in this decision, in their order, for the revision they would make.
    JudgeSettings {
        owner: Token,
        revision: u64,
        /// Accepted proposal whose kept values Set items name, with empty bytes.
        proposal: Option<u64>,
        items: Box<[settings::application::Item]>,
    },
    /// Keep committed changes in the same decision, taking a proposal's kept values when named.
    KeepSettings {
        revision: u64,
        proposal: Option<u64>,
        items: Box<[settings::application::Item]>,
    },
    /// Keep a proposal's values, checked against the kinds' bounds alone.
    KeepProposalSettings {
        proposal: u64,
        items: Box<[settings::application::Item]>,
    },
    /// Erase the values of a withdrawn or rejected proposal.
    DropProposalSettings {
        proposal: u64,
    },
    /// Save defaults in the release's decision; none means no revision was needed.
    SaveSettingsDefaults {
        revision: Option<u64>,
    },
    /// The live task names these resources.
    Names {
        task: u64,
        resources: Box<[tasks::Name]>,
    },
    /// The task is erased whole: nothing it named need be kept for it
    /// (jig/docs/design/domain/connectors.md, 3.4).
    Unname {
        task: u64,
    },
    Hold {
        task: u64,
        resource: tasks::Name,
        from: Option<u64>,
    },
    ReleaseHold {
        task: u64,
        resource: tasks::Name,
    },
    Writer {
        task: u64,
        attempt: u64,
        resources: Box<[tasks::Name]>,
    },
    Describe {
        owner: Token,
    },
    /// Stage the payload durably retained under this proposal's number.
    DescribeProposal {
        owner: Token,
        proposal: u64,
    },
    /// Retain an explicitly proposed payload without making an outbox entry.
    KeepProposal {
        owner: Token,
        proposal: u64,
        task: u64,
        project: u32,
    },
    /// Remove the connector payload of a terminal proposal.
    DropProposal {
        proposal: u64,
    },
    Keep {
        owner: Token,
        entry: u64,
        task: u64,
        key: crate::EffectKey,
    },
    Drop {
        owner: Token,
        answer: authority::Answer,
        findings: Box<[authority::Finding]>,
        why: EffectRefusal,
    },
    Make {
        entry: u64,
    },
    /// Settle a held entry as not made, or prepare a new attempt, within the release decision.
    ResolveUncertain {
        entry: u64,
        again: bool,
    },
    Judge {
        owner: Token,
        judge: authority::Judge,
        state: [u8; 32],
        resources: Box<[authority::Name]>,
    },
    ProcedureMade {
        task: u64,
    },
    ProcedureActivate {
        task: u64,
    },
    ProcedureMessage {
        task: u64,
        words: Box<[u8]>,
    },
    ProcedureClose {
        task: u64,
    },
    Project {
        goal: u64,
    },
    Release {
        task: u64,
    },
    Subscribe {
        task: u64,
        topic: u64,
    },
    Unsubscribe {
        task: u64,
        topic: u64,
    },
    Read {
        owner: Token,
    },
    Gather {
        section: Token,
        budget: u32,
    },
    CutTo {
        section: Token,
        size: u32,
    },
    Take {
        section: Token,
    },
    Prepare {
        task: u64,
        attempt: u64,
    },
    Left {
        task: u64,
        attempt: u64,
    },
    Adopt {
        owner: Token,
        project: u32,
        resource: tasks::Name,
    },
    Restore {
        owner: Token,
    },
    ReadAfresh,
    SettleOutbox,
}
