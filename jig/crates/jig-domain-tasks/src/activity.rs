//! Immutable task activity in commit order (domain/engine/README.md, sections 5.4 and 5.5, and domain/engine/executors.md, section 2).
//! Only the last position is live. Messages retain their original admission,
//! before unread hints merge; history and lifecycle rows are named for demand loads.
use crate::{MessageFamily, MessageKind, Party, Request, Stored, TaskRecord, Word};
use alloc::boxed::Box;
use skein_lib::{Queue, Wall, length::len_u64};

/// One immutable line; its position never depends on wall-clock order.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ActivityLine {
    pub task: u64,
    pub position: u64,
    pub at: Wall,
    pub line: Line,
}

/// Ordered semantic activity; larger retained records are loaded by their exact identity.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Line {
    TalkOpened {
        party: u64,
    },
    TalkEnded {
        party: u64,
        lapsed: bool,
    },
    Words {
        message: u64,
        by: Party,
        kind: MessageKind,
        words: Box<[u8]>,
    },
    Heard(Word),
    /// A notice sent by this task, kept after its recipient's waiting entry clears.
    Notified {
        recipient: Party,
        word: Word,
    },
    RunBegan {
        attempt: u64,
        why: Why,
        resumed: bool,
    },
    RunEnded {
        attempt: u64,
        end: EndKind,
        spent: u64,
    },
    Turn {
        reply: u64,
        attempt: u64,
        turn: u32,
        read: Option<u64>,
    },
    Decided {
        revision: u64,
        proposal: Option<DecidedProposal>,
    },
    Milestone {
        position: u64,
    },
    NowWritten {
        attempt: u64,
        line: Box<[u8]>,
    },
    Stood {
        attempt: u64,
        words: Box<[u8]>,
    },
    SessionEnded {
        attempt: u64,
        end: crate::SessionEnd,
    },
    /// The run finished done while its task was kept, and parked: the
    /// dependencies it waits for, at most `Limits.dependencies`
    /// (domain/tasks/lifecycle.md, 8; domain/engine/executors.md).
    Kept {
        attempt: u64,
        pending: Box<[u64]>,
        question: Option<u64>,
    },
}

/// Maximum bytes retained from a standing run's result (jig/docs/design/domain/tasks/standing.md, 7).
pub const STOOD_WORDS: u32 = 512;

/// Maximum bytes retained from a Now's first line (jig/docs/design/domain/tasks/standing.md, 7).
pub const NOW_LINE: u32 = 512;

/// Which proposal a decision line names, and whether it records its making.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DecidedProposal {
    pub number: u64,
    pub made: bool,
}

/// The durable cause carried through preparation to the committed claim.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Why {
    Made,
    Ready,
    Message {
        message: u64,
    },
    Released,
    Retry {
        tries: crate::Tries,
    },
    Narrowed,
    /// A procedure's connector or writer inputs changed.
    Changed,
}

/// The accepted terminal outcome; result bytes remain in their existing rows.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EndKind {
    Parked,
    Finished,
    Cancelled,
    Failed {
        /// Failure class used by the retry policy.
        class: crate::Class,
        /// Contract failure kind retained for this attempt, independent of later runs.
        invalid_result: Option<crate::InvalidResult>,
    },
    Refused,
}

pub(crate) fn append(task: &mut TaskRecord, at: Wall, line: Line, out: &mut Queue<Request>) {
    task.activity = task.activity.checked_add(1).expect("task activity position fits");
    out.push(Request::Save {
        record: Stored::Activity(ActivityLine { task: task.number, position: task.activity, at, line }),
    });
}

pub(crate) fn now_written(
    domain: &mut crate::Domain,
    env: &skein_lib::Env<crate::Limits>,
    number: u64,
    attempt: u64,
    line: &[u8],
    out: &mut Queue<Request>,
) {
    let Some(task) = crate::domain::task_mut(domain, number) else { return };
    if task.record.flags.ended_by != crate::EndedBy::Standing
        || crate::run::run_attempt(&task.record.phase) != Some(attempt)
    {
        return;
    }
    let line = Box::from(skein_lib::bytes::utf8_prefix(line, skein_lib::length::usize_of(NOW_LINE)));
    append(&mut task.record, env.wall, Line::NowWritten { attempt, line }, out);
    crate::domain::publish(domain, env, number, out);
}

pub(crate) fn message(task: &mut TaskRecord, word: &Word, bound: u32, out: &mut Queue<Request>) {
    let line = match word.kind.family() {
        MessageFamily::Spoken(_) => {
            Line::Words { message: word.number, by: word.from, kind: word.kind.clone(), words: word.words.clone() }
        }
        MessageFamily::Hint(_)
        | MessageFamily::Slot(_)
        | MessageFamily::Separate(_)
        | MessageFamily::Result
        | MessageFamily::Virtual => Line::Heard(word.clone()),
    };
    append(task, word.at, line, out);
    crate::positions::message(task, word, bound);
}

/// Maximum complete bytes in an emitted line; referenced history/turn rows have their own bounds.
#[must_use]
pub fn line_bytes_limit(limits: &crate::Limits) -> Option<u64> {
    let caller = u64::from(crate::VERDICT_FIELDS.min(limits.result_bytes))
        .checked_mul(u64::try_from(size_of::<crate::ResultField>()).ok()?)?
        .checked_add(u64::from(limits.message_bytes.max(limits.result_bytes)))?;
    let notice = len_u64(crate::run::NOTHING_AWAITED_WORDS.len());
    u64::try_from(size_of::<ActivityLine>())
        .ok()?
        .checked_add(caller.max(notice).max(u64::from(STOOD_WORDS)).max(u64::from(NOW_LINE)))
}

/// Whole-decision line ceiling: admission, decisions and lifecycle cascades,
/// plus one notice for each bounded standing interest.
#[must_use]
pub fn lines_per_decision(limits: &crate::Limits) -> Option<u32> {
    limits.tasks.checked_mul(12)?.checked_add(limits.tasks.checked_mul(limits.subscriptions)?)
}
