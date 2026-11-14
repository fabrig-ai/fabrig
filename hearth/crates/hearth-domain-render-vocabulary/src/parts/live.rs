//! Live paths and their ordered consumers (hearth/docs/design/domain/render/spec.md, 1.4).
use alloc::boxed::Box;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Live {
    pub paths: Box<[Covered]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Covered {
    pub path: Box<[u8]>,
    pub consumers: Box<[Consumer]>,
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Consumer {
    pub task: Box<[u8]>,
    pub unit: Option<Box<[u8]>>,
    pub action: Action,
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Action {
    None,
    Reload,
    ReloadOrRestart,
    Restart,
    Start,
}
impl Action {
    #[must_use]
    pub fn name(self) -> &'static [u8] {
        match self {
            Self::Reload => b"reload",
            Self::Restart => b"restart",
            Self::ReloadOrRestart => b"reload-or-restart",
            Self::Start => b"start",
            Self::None => b"none",
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Change {
    pub paths: Box<[ConsumersKeyed]>,
}
/// Keyed changes (hearth/docs/design/domain/render/README.md, 4.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum ConsumersKeyed {
    Added { key: Box<[u8]>, value: Box<[Consumer]> },
    Removed { key: Box<[u8]>, value: Box<[Consumer]> },
    Changed { key: Box<[u8]>, before: Box<[Consumer]>, after: Box<[Consumer]> },
}
