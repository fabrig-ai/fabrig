//! A project's whole queues and retained titles (temper/docs/design/domain/forge/queues.md;
//! temper/docs/design/client/forge/records.md; temper/docs/design/client/forge/README.md, 3).
use alloc::boxed::Box;
use skein_lib::Wall;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ChangesView {
    pub project: u32,
    pub landing: Box<[u8]>,
    pub queues: Box<[ChangesQueue]>,
    pub more: u32,
    pub landed: Box<[ChangesLanded]>,
    pub landed_more: u32,
    pub landed_default: Box<[ChangesLanded]>,
    pub landed_default_more: u32,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ChangesQueue {
    pub repository: Box<[u8]>,
    pub branch: Box<[u8]>,
    pub red: bool,
    pub changes: Box<[ChangesQueued]>,
    pub way: temper_engine_domain_forge_change::Way,
    pub in_flight: Option<TrainLanding>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TrainLanding {
    pub task: u64,
    pub member: u64,
    pub round: u16,
    pub step: super::LandingStep,
    pub probe: Option<[u8; 32]>,
    pub title: Box<[u8]>,
    pub ready_since: Wall,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ChangesQueued {
    pub task: u64,
    pub title: Box<[u8]>,
    pub ready_since: Wall,
    pub checks_since: Option<Wall>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ChangesLanded {
    pub signed: Box<[crate::SignOff]>,
    pub task: u64,
    pub title: Box<[u8]>,
    pub at: Wall,
    pub commit: [u8; 32],
    pub default_branch: bool,
}
