//! Keyed configuration-fit failures retained by a selected start
//! (jig/docs/design/domain/engine.md, 6).
use crate::{Key, ModelEffort};

/// The first live stored row that this configuration cannot retain.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RestoreFailure {
    pub key: Key,
    pub cause: RestoreCause,
}

/// A measured capacity, unrepresentable accounting, or unavailable executor choice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RestoreCause {
    Bound { which: RestoreBound, held: u64, limit: u64 },
    Accounting,
    LlmChoice { chosen: ModelEffort, why: RestoreLlmWhy },
}

/// Why configuration no longer offers a retained model and effort.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RestoreLlmWhy {
    Unconfigured,
    Charter,
    Effort,
}

/// The existing configured or retained envelope exceeded by a stored row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RestoreBound {
    Tasks,
    Stubs,
    Writers,
    Pools,
    Funders,
    PlacedHolds,
    PersonProposals,
    HeldRevisions,
    ProjectTasks,
    ProjectGoals,
    ProjectPlacedHolds,
    PartyPlacedHolds,
    Parameters,
    Inputs,
    SpecBytes,
    InboxBytes,
    AuthorityGrants,
    ExecutorKinds,
    AuthoritySegments,
    AuthorityBytes,
    ResultBytes,
    ResultFields,
    ContractChoices,
    Batch,
    HoldSegments,
    HoldPathCapacity,
    HoldBytes,
    AccountOriginBytes,
    Circle,
    Dependencies,
    Holdings,
    Subagents,
    MessageBytes,
    SettingsItems,
    Exemptions,
    Delegates,
    References,
    Subscriptions,
    InboxMessages,
    SavedResources,
    RootHoldings,
    RefusedRows,
    ReminderBytes,
    ZeroBytes,
    AmendmentSlots,
    MetSlots,
}
