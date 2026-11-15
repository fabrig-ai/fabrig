//! Requests and their terminal events.

#![expect(clippy::disallowed_methods, reason = "prefix successor needs a mutable copy of the caller's bytes")]

use alloc::boxed::Box;
use skein_io::kernel::Fd;
use skein_lib::Token;

#[derive(PartialEq, Eq, Debug)]
pub enum Request {
    Open { owner: Token, root: Fd },
    Commit { owner: Token, ops: Box<[Op]> },
    Get { owner: Token, key: Box<[u8]> },
    Load { owner: Token, range: Range, max: Page },
    ReadLog { owner: Token, log: u64, at: u64, direction: Direction, max: u32 },
    Close { owner: Token },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Op {
    Put { key: Box<[u8]>, value: Box<[u8]> },
    Erase { key: Box<[u8]> },
    Append { log: u64, frame: crate::LogFrame },
    Drop { log: u64 },
}

impl Op {
    #[must_use]
    pub fn key(&self) -> &[u8] {
        match self {
            Op::Put { key, .. } | Op::Erase { key } => key,
            Op::Append { .. } | Op::Drop { .. } => {
                unreachable!("payload ops are translated to extent rows before map operations")
            }
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Range {
    pub start: Box<[u8]>,
    pub end: Option<Box<[u8]>>,
}

impl Range {
    #[must_use]
    pub fn prefix(prefix: &[u8]) -> Range {
        let mut end = prefix.to_vec();
        let mut successor = None;
        while let Some(last) = end.pop() {
            if last < u8::MAX {
                end.push(last.checked_add(1).expect("a byte below 255 has a successor"));
                successor = Some(end.into_boxed_slice());
                break;
            }
        }
        Range { start: Box::from(prefix), end: successor }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Page {
    pub rows: u32,
    pub bytes: u32,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Row {
    pub key: Box<[u8]>,
    pub value: Box<[u8]>,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Event {
    Opened {
        owner: Token,
        last: u64,
    },
    Committed {
        owner: Token,
        number: u64,
    },
    Refused {
        owner: Token,
        refusal: Refusal,
    },
    Failed {
        owner: Token,
        number: u64,
    },
    Got {
        owner: Token,
        value: Option<Box<[u8]>>,
    },
    Loaded {
        owner: Token,
        rows: Box<[Row]>,
        next: Option<Box<[u8]>>,
    },
    LogRead {
        owner: Token,
        records: Box<[LogRecord]>,
        next: Option<u64>,
    },
    LogUnread {
        owner: Token,
        at: u64,
    },
    Closed {
        owner: Token,
    },
    OpenFailed {
        owner: Token,
        failure: Failure,
    },
    /// A fatal file failure was repaired; the recovered commit prefix is usable.
    Recovered {
        last: u64,
    },
    /// Recovery after a previously opened store could not complete.
    RecoveryFailed {
        failure: Failure,
    },
}

/// Why a request was refused, as bounded data without a key's or value's bytes
/// (skein/docs/design/data/kv.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refusal {
    TooLarge(Large),
    Full(Exhausted),
    Busy,
    Unavailable,
    Logs,
}

/// What passed its limit (skein/docs/design/data/kv.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Large {
    Key,
    Value,
    Ops,
    Frame,
    Page,
    Range,
    Record,
    Log,
}

/// What the commit would exhaust (skein/docs/design/data/kv.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Exhausted {
    Budget { used: u64, budget: u64, projected: u64 },
    Entries { capacity: u32 },
    Numbers,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Failure {
    Io,
    Corrupt,
    Full,
}

/// Payload log traversal (skein/docs/design/data/kv.md, 6.3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Forward,
    Backward,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LogRecord {
    pub at: u64,
    pub end: u64,
    pub length: u32,
    pub record: Option<Box<[u8]>>,
}

/// The store alone writes rows under this prefix (data/kv.md, 6.1).
#[derive(Debug)]
pub struct LogRule {
    pub prefix: Box<[u8]>,
}
