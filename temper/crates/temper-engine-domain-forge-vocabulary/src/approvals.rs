//! Changes waiting on a person's approval (domain/forge/change/questions.md, 2).
use alloc::boxed::Box;
use skein_lib::Wall;

/// A live change whose merge waits on a person's approval, before the people are joined
/// (domain/forge/change/questions.md, 2).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ApprovalWait {
    pub task: u64,
    pub project: u32,
    pub forge: u16,
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub pull: u64,
    pub title: Box<[u8]>,
    /// When its review facts at its last head were first kept.
    pub since: Wall,
    /// Whether its reviews were read whole.
    pub whole: bool,
    pub size: Option<skein_forge::api::Size>,
    /// Its enforcing approvals still short, in its panel's order, each with its people counted.
    pub short: Box<[ShortApproval]>,
}

/// A short approval and the sets of its rule touched at this head
/// (domain/forge/change/questions.md, 2).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ShortApproval {
    pub scope: u32,
    pub rule: u32,
    pub role: u32,
    pub required: u32,
    pub counted: Box<[crate::Reviewer]>,
    pub sets: u8,
}
