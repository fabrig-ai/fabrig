//! Away spells and decisions (temper/docs/design/domain/forge/attention.md, 4, 5).
use alloc::boxed::Box;
use skein_lib::Wall;
use temper_protocol_reaches::{Kind, Reach};

pub const AWAY_SUBJECT: u64 = 1 << 60;
pub const DECIDED_DAY_NS: u64 = 86_400_000_000_000;
pub const DECIDED_KEPT_NS: u64 = 604_800_000_000_000;
pub const DECISIONS_SHOWN: usize = 128;
pub const DECIDED_QUESTION_BYTES: usize = 400;
pub const DECIDED_OPTION_BYTES: usize = 120;
pub const DECIDED_WHY_BYTES: usize = 400;
pub const DECIDED_OPTIONS: core::ops::RangeInclusive<usize> = 2..=6;
#[must_use]
pub fn decided_subject(project: u32) -> u64 {
    (1 << 62) | (1 << 35) | u64::from(project)
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Away {
    pub party: u64,
    pub project: u32,
    pub since: Wall,
    pub until: Option<Wall>,
    pub landed: u64,
    pub off: Option<Wall>,
}
impl Away {
    #[must_use]
    pub fn on_at(&self, wall: Wall) -> bool {
        self.off.is_none()
            && match self.until {
                Some(until) => until > wall,
                None => true,
            }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Decided {
    pub project: u32,
    pub number: u32,
    pub party: u64,
    pub task: u64,
    pub at: Wall,
    pub question: Box<[u8]>,
    pub options: Box<[Box<[u8]>]>,
    pub took: Option<u8>,
    pub why: Box<[u8]>,
    pub overturned: bool,
    pub reach: Reach,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DecidedCall {
    pub question: Box<[u8]>,
    pub options: Box<[Box<[u8]>]>,
    pub took: u8,
    pub why: Box<[u8]>,
    pub kind: Kind,
    pub paths: Box<[Box<[u8]>]>,
    pub under: u8,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SetAwayAnswer {
    Away,
    TooLarge,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum OverturnAnswer {
    Overturned,
    Already,
    Unknown,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DecisionView {
    pub number: u32,
    pub task: u64,
    pub at: Wall,
    pub question: Box<[u8]>,
    pub took: Box<[u8]>,
    pub why: Box<[u8]>,
    pub overturned: bool,
    pub reach: Reach,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AskedView {
    pub number: u32,
    pub task: u64,
    pub at: Wall,
    pub reach: Reach,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AwaysView {
    pub rows: Box<[Away]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DecisionsView {
    pub project: u32,
    pub rows: Box<[DecisionView]>,
    pub asked: Box<[AskedView]>,
    pub cut: bool,
}
