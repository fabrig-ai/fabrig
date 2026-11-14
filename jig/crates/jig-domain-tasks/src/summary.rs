//! Ended task summaries and their next retention stage (protocol/store/retention.md, 3–4).
use alloc::boxed::Box;
use crate::{Budget, Ending, Endings, Executor, Flags, Funder, Numbers, Party, Phase, TaskRecord, TaskResult};
use skein_lib::{Wall, bytes::utf8_prefix, length::usize_of};

pub const SUMMARY_TEXT: u32 = 256;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum RetentionStage {
    Turns,
    Record,
    Summary,
    Forgotten,
    Proposal,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct RetentionRow {
    pub stage: RetentionStage,
    pub since: Wall,
    pub number: u64,
}
impl RetentionRow {
    #[must_use]
    pub const fn of(record: &TaskRecord) -> Self {
        Self {
            stage: if record.ever_turned { RetentionStage::Turns } else { RetentionStage::Record },
            since: match record.ended_at { Some(at) => at, None => record.changed },
            number: record.number,
        }
    }
    #[must_use]
    pub const fn key(self) -> crate::Key {
        crate::Key::Retention { stage: self.stage, since: self.since, number: self.number }
    }
}

/// The chip's terminal class, without any full result payload.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SummaryEnding {
    Done,
    Failed,
    Cancelled { by: Party, at: Wall },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SummaryResultKind {
    Job,
    Approved,
    Report,
    Verdict { code: u32 },
    Change { connector: u16, kind: u16, resource: u64 },
    NoChange,
    Failed,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SummaryResult {
    pub kind: SummaryResultKind,
    pub words: Box<[u8]>,
    pub words_cut: bool,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TaskSummary {
    pub number: u64,
    pub root: u64,
    pub depth: u32,
    pub requester: Party,
    pub project: u32,
    pub label: u32,
    pub flags: Flags,
    pub tracked: Option<u32>,
    pub executor: Executor,
    pub title: Box<[u8]>,
    pub title_cut: bool,
    pub ending: SummaryEnding,
    pub reason: Box<[u8]>,
    pub reason_cut: bool,
    pub result: Option<SummaryResult>,
    pub created_at: Wall,
    pub changed: Wall,
    pub last_activity: Wall,
    pub ended_at: Wall,
    pub numbers: Numbers,
    pub budget: Budget,
    pub funder: Funder,
    pub ended_delegates: Endings,
    pub attempt: u64,
    pub activity: u64,
    pub result_position: u64,
    pub last_reply: u64,
}
impl TaskSummary {
    #[must_use]
    pub fn of(record: &TaskRecord) -> Self {
        let end = skein_lib::bytes::find(&record.spec.words, b"\n").unwrap_or(record.spec.words.len());
        let line = record.spec.words.get(..end).expect("first line within words");
        let (title, title_cut) = text(line);
        let (ending, reason, result) = match &record.phase {
            Phase::Ended(Ending::Done(result)) => (SummaryEnding::Done, &[][..], Some(result)),
            Phase::Ended(Ending::Failed { reason }) => (SummaryEnding::Failed, reason.as_ref(), None),
            Phase::Ended(Ending::Cancelled { by, at, reason, result }) => (
                SummaryEnding::Cancelled { by: *by, at: *at }, reason.as_ref(), result.as_ref(),
            ),
            Phase::Waiting | Phase::Active(_) | Phase::Held { .. } | Phase::Closing(_) => {
                unreachable!("a summary is made only for an ended task")
            }
        };
        let (reason, reason_cut) = text(reason);
        let result = match result { Some(value) => Some(SummaryResult::of(value)), None => None };
        Self {
            number: record.number, root: record.root, depth: record.depth,
            requester: record.requester, project: record.project, label: record.label,
            flags: record.flags, tracked: record.tracked, executor: record.executor,
            title, title_cut, ending, reason, reason_cut, result,
            created_at: record.created_at, changed: record.changed, last_activity: record.last_activity,
            ended_at: RetentionRow::of(record).since,
            numbers: record.numbers, budget: record.authority.budget, funder: record.funder,
            ended_delegates: record.ended_delegates, attempt: record.attempt, activity: record.activity,
            result_position: record.result_position, last_reply: record.last_reply,
        }
    }
}
impl SummaryResult {
    fn of(result: &TaskResult) -> Self {
        let (kind, words) = match result {
            TaskResult::Job { .. } => (SummaryResultKind::Job, &[][..]),
            TaskResult::Approved { words, .. } => (SummaryResultKind::Approved, match words {
                Some(words) => words.as_ref(), None => &[],
            }),
            TaskResult::Report { words } => (SummaryResultKind::Report, words.as_ref()),
            TaskResult::Verdict { code, words, .. } => (SummaryResultKind::Verdict { code: *code }, words.as_ref()),
            TaskResult::Change { connector, kind, resource, title, .. } => (
                SummaryResultKind::Change { connector: *connector, kind: *kind, resource: *resource }, title.words.as_ref(),
            ),
            TaskResult::NoChange { words } => (SummaryResultKind::NoChange, words.as_ref()),
            TaskResult::Failure { reason } => (SummaryResultKind::Failed, reason.as_ref()),
        };
        let (words, words_cut) = text(words);
        Self { kind, words, words_cut }
    }
}
fn text(words: &[u8]) -> (Box<[u8]>, bool) {
    let prefix = utf8_prefix(words, usize_of(SUMMARY_TEXT));
    (Box::from(prefix), prefix.len() < words.len())
}
