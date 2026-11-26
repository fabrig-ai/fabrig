//! The engine's held loads (`jig/docs/design/domain/engine/vocabulary.md`, 2.4).

use crate::{Key, Record, TranscriptRow, TurnRecord, reads};
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::Token;

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Out {
    Load { load: Token, range: Range },
}

#[derive(Hash, Debug)]
pub enum In {
    Loaded { load: Token, result: Answer },
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Range {
    Record(Key),
    Proposal { task: u64, proposal: u64 },
    Window { task: u64, from: (u64, u32) },
    Read(Box<reads::Range>),
}

#[derive(Hash, Debug)]
pub enum Answer {
    Rows(Rows),
    Cut { turn: TurnRecord, bytes: u64, more: bool },
    Failed(Failure),
}

#[derive(Hash, Debug)]
pub enum Rows {
    Records { records: Box<[Record]>, more: bool },
    Turns { turns: Box<[TranscriptRow]>, more: bool },
    Read { rows: Box<[reads::Row]>, next: Option<reads::Position> },
}

#[derive(Hash, Debug)]
pub enum Failure {
    Busy,
    Gone,
    TooLinked { joins: u32 },
    TurnProblem { task: u64, attempt: u64, turn: u32, owner: Box<tasks::TaskRecord>, why: reads::TurnFailure },
    Unread,
    Failed,
}
