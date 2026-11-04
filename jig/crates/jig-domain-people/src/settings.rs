//! Setting names and check faults carried by keyed answers (jig/docs/design/domain/engine.md, 17.1, 17.5).
/// A core or application setting (jig/docs/design/domain/engine.md, 17.1, 17.5).
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
    /// A project's own policy, an item of its scope (jig/docs/design/domain/engine.md, 17.2).
    ProjectPolicy {
        project: u32,
    },
}
/// A check fault, including a connector's judgment (jig/docs/design/domain/engine.md, 17.1, 17.5).
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

/// Whether an accepted settings revision moves running work (engine.md, 17.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Running {
    Stays,
    Moves,
}
