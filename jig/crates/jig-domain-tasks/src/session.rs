//! A standing task's session forms (jig/docs/design/domain/tasks/standing.md, 7).
use crate::Reasons;
use skein_lib::Wall;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Session {
    pub first: u64,
    pub reasons: Reasons,
    pub last: Wall,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SessionEnd {
    Quiet,
    Bound,
    Unoffered,
    Fresh,
    Closed,
    Rules,
}
