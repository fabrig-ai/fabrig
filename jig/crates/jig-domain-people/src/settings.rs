//! Setting names and check faults carried by keyed answers (jig/docs/design/domain/engine/settings.md, 1, 5).
/// A core or application setting (jig/docs/design/domain/engine/settings.md, 1, 5).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Setting {
    Charter(u32),
    Template(u32),
    Policy(u32),
    Rules,
    /// Scope 0 is the deployment; otherwise it is a project number.
    Application {
        kind: u32,
        scope: u32,
        number: u32,
    },
    /// A project's own policy, an item of its scope (jig/docs/design/domain/engine/settings.md, 2).
    ProjectPolicy {
        project: u32,
    },
    Model(u32),
}
/// A check fault, including a connector's judgment (jig/docs/design/domain/engine/settings.md, 1, 5).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SettingFault {
    Shape,
    Count,
    Reference,
    Rules,
    Budget,
    Grace,
    Owner,
    Judged,
}

/// Whether an accepted settings revision moves running work (domain/engine/settings.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Running {
    Stays,
    Moves,
}

/// The charter member rejected by the settings check (domain/engine/settings.md, 9).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SettingMember {
    /// The charter's document count.
    Documents,
    /// A document resource name's segment count.
    NameSegments,
    /// One segment's byte length in a document resource name.
    NameSegmentBytes,
    /// The document path's byte length.
    Path,
    /// The optional heading's byte length.
    Heading,
    /// The document connector's registration.
    Connector,
    /// The charter's stored policy and parts byte length.
    StoredPolicy,
}
/// The failed setting rule; scalar bounds are carried beside it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SettingRule {
    /// The supplied count or length exceeds the configured maximum.
    Maximum,
    /// The supplied length falls below the required minimum.
    Minimum,
    /// The supplied connector number is not registered.
    Registered,
}
/// Bounded refusal facts, without copying any of the rejected charter's bytes.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SettingDetail {
    /// The member relative to the charter.
    pub member: SettingMember,
    /// Document index; zero for documents or stored policy.
    pub document: u32,
    /// Name segment index; zero for members other than `NameSegmentBytes`.
    pub segment: u32,
    /// The failed check, interpreted with the member and bound.
    pub rule: SettingRule,
    /// The supplied count, byte length or connector number.
    pub supplied: u32,
    /// The minimum or maximum; zero for `Registered`.
    pub bound: u32,
}
