//! Application roots on the worker channel (jig/docs/design/protocol/channel/bridges.md, 8).
use crate::Spoken;
use skein_codec::bridge::Bridge;
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
    pub bridges: List<Bridge>,
    pub roots: List<Root>,
}
impl Application {
    /// An application whose opaque parts cross unchanged.
    #[must_use]
    pub fn opaque(spoken: List<Spoken>) -> Self {
        Self { spoken, bridges: List::uncapped(), roots: List::uncapped() }
    }
}
