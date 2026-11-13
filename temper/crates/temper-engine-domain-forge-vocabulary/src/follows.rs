//! Hearing source landings (temper/docs/design/domain/forge/follows.md, 3, 7 and 9).
use crate::Limits;
use alloc::boxed::Box;
use skein_forge::api::{Commit, Repository};
pub mod acts;
pub mod words;
pub use acts::FollowAct;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FollowRow {
    pub project: u32,
    pub follow: u32,
    pub source: Repository,
    pub branch: Box<[u8]>,
    pub heard: Option<Commit>,
    pub task: Option<u64>,
    pub stopped: Option<Box<[u8]>>,
    pub queue: Box<[FollowEntry]>,
    pub skips: Box<[FollowSkip]>,
    pub parity: Option<FollowParity>,
    pub questions: Box<[FollowQuestion]>,
    pub pending: Option<FollowAct>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FollowEntry {
    pub before: Commit,
    pub after: Commit,
    pub pull: Option<u64>,
    pub commits: Box<[FollowCommit]>,
    pub more_commits: u32,
    pub files: Option<Box<[Box<[u8]>]>>,
    pub more_files: u32,
    pub change: Option<u64>,
    pub state: FollowState,
    pub word: bool,
    pub remake: bool,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FollowQuestion {
    pub change: u64,
    pub number: u64,
    pub words: Box<[u8]>,
    pub person: Option<u64>,
    pub relayed: Option<u64>,
    pub answer: Option<Box<[u8]>>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FollowCommit {
    pub id: Commit,
    pub message: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum FollowState {
    NotMade,
    WaitsWord,
    HeldByOwner(Box<[u8]>),
    WaitsFor(u64),
    Producing,
    Held(Box<[u8]>),
    Ended(Box<[u8]>),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FollowSkip {
    pub after: Commit,
    pub reason: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FollowParity {
    pub source: Commit,
    pub head: Commit,
    pub verdict: FollowVerdict,
    pub run: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum FollowVerdict {
    Passed,
    Failing(u32),
    Many,
}
/// Pure snapshot, in the queue's oldest-first order.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FollowLag {
    pub count: u32,
    pub entries: Box<[FollowEntry]>,
}
pub fn capacity(l: &Limits) -> Option<u32> {
    l.judge_projects.checked_mul(l.follows)
}
/// Refusals of the domain's owner-word entrance (follows.md, 5).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FollowRefusal {
    UnknownFollow,
    Stopped,
    UnknownEntry,
    Skipped,
    AlreadyStarted,
    NotConsecutive,
    MergeBound,
    ReasonBound,
    NoWord,
    Pending,
    NotOwner,
}
pub(crate) fn row_bytes(row: &FollowRow) -> Option<u64> {
    let mut bytes = skein_lib::len_u64(row.branch.len());
    let pending = match &row.pending {
        Some(FollowAct::Hold { reason, .. } | FollowAct::Skip { reason, .. }) => skein_lib::len_u64(reason.len()),
        Some(FollowAct::Merge { afters }) => skein_lib::len_u64(afters.len()).checked_mul(32)?,
        Some(FollowAct::Start { .. }) | None => 0,
    };
    bytes = bytes.checked_add(pending)?;
    bytes = bytes.checked_add(
        skein_lib::len_u64(row.questions.len()).checked_mul(u64::try_from(size_of::<FollowQuestion>()).ok()?)?,
    )?;
    for question in &row.questions {
        bytes = bytes.checked_add(skein_lib::len_u64(question.words.len()))?;
        if let Some(answer) = &question.answer {
            bytes = bytes.checked_add(skein_lib::len_u64(answer.len()))?;
        }
    }
    if let Some(stopped) = &row.stopped {
        bytes = bytes.checked_add(skein_lib::len_u64(stopped.len()))?;
    }
    bytes = bytes
        .checked_add(skein_lib::len_u64(row.queue.len()).checked_mul(u64::try_from(size_of::<FollowEntry>()).ok()?)?)?;
    for entry in &row.queue {
        bytes = bytes.checked_add(
            skein_lib::len_u64(entry.commits.len()).checked_mul(u64::try_from(size_of::<FollowCommit>()).ok()?)?,
        )?;
        for commit in &entry.commits {
            bytes = bytes.checked_add(skein_lib::len_u64(commit.message.len()))?;
        }
        if let Some(files) = &entry.files {
            bytes = bytes.checked_add(
                skein_lib::len_u64(files.len()).checked_mul(u64::try_from(size_of::<Box<[u8]>>()).ok()?)?,
            )?;
            for file in files {
                bytes = bytes.checked_add(skein_lib::len_u64(file.len()))?;
            }
        }
        match &entry.state {
            FollowState::HeldByOwner(words) | FollowState::Held(words) | FollowState::Ended(words) => {
                bytes = bytes.checked_add(skein_lib::len_u64(words.len()))?;
            }
            FollowState::NotMade | FollowState::WaitsWord | FollowState::WaitsFor(_) | FollowState::Producing => {}
        }
    }
    bytes = bytes
        .checked_add(skein_lib::len_u64(row.skips.len()).checked_mul(u64::try_from(size_of::<FollowSkip>()).ok()?)?)?;
    for skip in &row.skips {
        bytes = bytes.checked_add(skein_lib::len_u64(skip.reason.len()))?;
    }
    if let Some(parity) = &row.parity {
        bytes = bytes.checked_add(skein_lib::len_u64(parity.run.len()))?;
    }
    Some(bytes)
}

/// A new requester remakes only the prefix before an independently ended entry (follows.md, 4).
pub fn remaking(row: &mut FollowRow) {
    row.questions = Box::new([]);
    for entry in &mut row.queue {
        if let FollowState::Ended(_) = entry.state {
            break;
        }
        entry.change = None;
        if let FollowState::HeldByOwner(_) = entry.state {
        } else {
            entry.state = FollowState::NotMade;
        }
        entry.remake = false;
    }
}
