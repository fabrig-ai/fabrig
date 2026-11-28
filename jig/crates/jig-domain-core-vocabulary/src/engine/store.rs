//! The engine's store port (`jig/docs/design/domain/engine/vocabulary.md`, 2.4 and 2.6).

use crate::{Key, Record, TranscriptRow, TurnRecord, documents::deployment::StoreCensus, reads, start::RestoreRange};
use alloc::boxed::Box;
use jig_domain_notes as notes;
use jig_domain_tasks as tasks;
use skein_lib::Token;

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Out {
    Load {
        load: Token,
        range: Range,
    },
    /// Switch the pending generation, if any, after earlier commits.
    Switch,
}

#[derive(Hash, Debug)]
pub enum In {
    Loaded {
        load: Token,
        result: Answer,
    },
    /// The store has opened; the start begins.
    Opened {
        pending: bool,
    },
    /// The generation switched, or none was pending.
    Switched,
    /// The store's fill, as each commit is answered.
    Fill {
        bytes: u64,
    },
    /// The store's census of its keyspaces.
    Census(Box<StoreCensus>),
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Range {
    Record(Key),
    Proposal {
        task: u64,
        proposal: u64,
    },
    Window {
        task: u64,
        from: (u64, u32),
    },
    Notes(notes::Range),
    Read(Box<reads::Range>),
    /// A page of the start's ordered walk, at the store's place.
    Start {
        ranges: Box<[RestoreRange]>,
        after: Option<Cursor>,
    },
}

#[derive(Hash, Debug)]
pub enum Answer {
    Rows(Rows),
    Cut { turn: TurnRecord, bytes: u64, more: bool },
    Failed(Failure),
}

#[derive(Hash, Debug)]
pub enum Rows {
    Records {
        records: Box<[Record]>,
        more: bool,
    },
    Turns {
        turns: Box<[TranscriptRow]>,
        more: bool,
    },
    Read {
        rows: Box<[reads::Row]>,
        next: Option<reads::Position>,
    },
    /// The start's page in store order, with its next place if any.
    Start {
        items: Box<[Item]>,
        next: Option<Cursor>,
    },
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

/// One item in the start's store walk.
#[derive(Hash, Debug)]
pub enum Item {
    /// A decoded row to restore or sweep.
    Record(Record),
    /// A row whose owner the store could not load.
    Missing { record: Key, target: Key },
}

/// The store's place in a start walk, read by the store alone.
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Cursor {
    pub interval: u32,
    pub after: Box<[u8]>,
}

/// A store write of the core or its start.
#[derive(Hash, Debug)]
#[cfg_attr(
    target_pointer_width = "64",
    expect(
        clippy::large_enum_variant,
        reason = "durable run proofs carry placement evidence; writes retain the shared record vocabulary"
    )
)]
pub enum Write {
    /// Drop a task transcript whole with its places (jig/docs/design/protocol/store/transcripts.md, 5, jig/docs/design/protocol/store/growth.md, 1.1;
    /// jig/docs/design/domain/people/acts.md, 2).
    DropTranscript { task: u64 },
    /// Drop a task history log whole (jig/docs/design/protocol/store/logs.md, 3 and 9).
    DropHistory { task: u64 },
    /// Copy the old held revision to history without materialising it in the core.
    CopySettingsRevision(u32),
    /// Save one authentic core or child record.
    Save(Record),
    /// Erase one authentic core or child key.
    Erase(Key),
    /// Commit the start's mark before wires are served.
    Wires,
}
