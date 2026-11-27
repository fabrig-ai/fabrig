//! Temper's own branch and pull writes (temper/docs/design/domain/forge/writes.md, sections 1 and 4).
use crate::Name;
use alloc::boxed::Box;
use skein_forge::api::Commit;

pub const WRITES: usize = 32;

/// One branch's record of temper's writes, in ascending admission order.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Writes {
    pub name: Name,
    pub head: Option<Commit>,
    pub next: u64,
    pub entries: Box<[Written]>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Written {
    pub order: u64,
    pub operation: Operation,
    pub sender: Sender,
    pub state: WriteState,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Operation {
    Push { before: Option<Commit>, after: Commit },
    Create { after: Commit },
    Delete { before: Option<Commit> },
    Update { pull: u64, before: Commit, base: Commit, after: Option<Commit> },
    MergedInto { pull: u64, before: Option<Commit>, after: Option<Commit> },
    Merge { pull: u64, head: Commit },
    Open { pull: Option<u64>, base: Box<[u8]>, head: Commit },
    Close { pull: u64 },
    Reopen { pull: u64 },
    Retarget { pull: u64, from: Box<[u8]>, to: Box<[u8]> },
    Writer { before: Option<Commit> },
    Compare { after: Commit },
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Sender {
    Attempt { task: u64, attempt: u64 },
    Entry { task: u64, entry: u64 },
    Previous,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum WriteState {
    Admitted,
    Sent,
    Confirmed,
    Refused,
    Uncertain,
}

impl WriteState {
    #[must_use]
    pub const fn unsettled(self) -> bool {
        match self {
            Self::Admitted | Self::Sent | Self::Uncertain => true,
            Self::Confirmed | Self::Refused => false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Actor {
    Unknown,
    Writer,
    Other,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Seen {
    Current,
    Explained { order: u64 },
    Behind { order: u64 },
    Owed,
    Drift { nearest: Option<u64> },
}

/// A branch whose record cannot take another write without forgetting one.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Full {
    pub name: Name,
    pub senders: Box<[Sender]>,
}
