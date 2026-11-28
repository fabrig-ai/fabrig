//! Procedure wait reports (jig/docs/design/domain/engine/waits.md, 6).

use crate::ops::{AtBound, Unbounded, WaitKind, WaitObject};
use skein_lib::Wall;

/// One wait in a connector's replacement report (jig/docs/design/domain/engine/waits.md, 6).
/// Use `Procedure` objects for `Step`, `Room`, and `SignOff`, and `Task` for `Turn`.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Reported {
    pub kind: WaitKind,
    pub object: WaitObject,
    pub since: Wall,
    pub bound: ReportedBound,
}

/// Why the core refused a whole replacement report. The previous report is unchanged.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ReportRefusal {
    NotLive,
    NotProcedure,
    WrongConnector,
    TooManyWaits,
    TooManyThings,
    WrongKindOrObject,
    NoFigure,
}

/// The clock or reason named by a connector, before the core projects the wait.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ReportedBound {
    /// The connector's own clock, kept as reported.
    At {
        at: Wall,
        then: AtBound,
    },
    /// Count this kind's configured figure from the floored since, capped by the task's deadline.
    /// Only `Step`, `Room`, and `SignOff` have figures.
    Figure(AtBound),
    None(Unbounded),
}
