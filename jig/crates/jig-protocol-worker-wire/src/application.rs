//! Application roots on the worker channel (jig/docs/design/protocol/channel/layouts.md, 7).
use crate::Spoken;
use skein_lib::List;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Carried {
    Items,
    Left,
    Saved,
    Preparation,
    JobSpec,
    JobResult,
    Grant,
    Ask,
    Answer,
}

#[derive(Clone, Copy, Debug)]
pub struct Root {
    pub carried: Carried,
    pub wire: &'static str,
    pub declaration: &'static str,
}

#[derive(Debug)]
pub struct Application {
    pub spoken: List<Spoken>,
    pub roots: List<Root>,
}
impl Application {
    /// An application whose opaque parts cross unchanged.
    #[must_use]
    pub fn opaque(spoken: List<Spoken>) -> Self {
        Self { spoken, roots: List::uncapped() }
    }
}
