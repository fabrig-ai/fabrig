//! Configured restoration fit before linking (temper/docs/design/domain/forge/README.md, 9).
use crate::RecordKind;

/// A connector-owned retained or reconstructed set.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RestoreSet {
    Rows(RecordKind),
    NamedAnswerBytes,
    NamedAnswerRows,
    BindingAddressBytes,
    BindingSubjectBytes,
    SettingGates,
    SettingApprovals,
    SettingSegments,
    ChangeTimers,
    ProjectionTimers,
    References,
    Watches,
    FollowQueue,
    LandingFlights,
    LandingMembers,
    LandingsKept,
    LandingGates,
    LandingAsks,
    LandingEntries,
    NameBytes,
    ContextProjects,
}

/// The first configured bound a restored row or derived set could not fit.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct RestoreFit {
    pub family: Option<RecordKind>,
    pub set: RestoreSet,
    pub held: u64,
    pub additional: u64,
    pub limit: u64,
}
