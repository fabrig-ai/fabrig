//! Durable record replacement and its one committed retry (protocol/hosts.md, 5.3).
//! The pending state retains retry bytes and the continuation until durability settles.
use super::Terminal;
use super::{
    Action, ChatState, Digest, Expect, File, Fresh, Queue, RecordName, Resume, State, Store, StoreFault, Token, digest,
    file, local,
};

#[derive(Debug)]
pub(super) enum AfterWrite {
    Answer(local::Event),
    Journal { state: ChatState, resume: Resume },
    FreshState { state: ChatState, resume: Resume },
}

impl AfterWrite {
    fn loading(&self) -> bool {
        match self {
            Self::Answer(_) => false,
            Self::Journal { resume, .. } | Self::FreshState { resume, .. } => resume.loading(),
        }
    }
}

#[derive(Debug)]
pub(super) struct Pending {
    pub(super) record: RecordName,
    pub(super) file: File,
    pub(super) expected: Expect,
    pub(super) after: AfterWrite,
}

#[derive(Debug)]
pub(super) enum Writing {
    First(Pending),
    Retry(Pending),
}

impl Writing {
    pub(super) fn request(&self, owner: Token, root: Token) -> file::Request {
        let pending = match self {
            Self::First(pending) | Self::Retry(pending) => pending,
        };
        file::Request::Store {
            owner,
            root,
            path: pending.file.name.clone(),
            bytes: pending.file.bytes.clone(),
            expected: pending.expected,
            no_follow: true,
        }
    }

    fn into_pending(self) -> Pending {
        match self {
            Self::First(pending) | Self::Retry(pending) => pending,
        }
    }
}

impl Store {
    pub(super) fn begin_write(
        &self,
        record: RecordName,
        file: File,
        after: AfterWrite,
        out: &mut Queue<Action>,
    ) -> State {
        if file.bytes.len() > usize::try_from(self.limits.file_bytes).expect("file bound fits")
            || (self.ephemeral.is_some()
                && !self.versions.contains_key(&file.name)
                && self.versions.len() == self.versions.capacity())
        {
            return Self::failure(record, StoreFault::TooLarge { file: record }, after.loading(), out);
        }
        let expected = match record {
            RecordName::Turn(_) => Expect::Absent,
            RecordName::State | RecordName::Delivery(_) | RecordName::Journal => match self.versions.get(&file.name) {
                Some(version) => Expect::Digest(*version),
                None => Expect::Absent,
            },
        };
        State::Write(Writing::First(Pending { record, file, expected, after }))
    }

    pub(super) fn write_event(&mut self, writing: Writing, event: Terminal, out: &mut Queue<Action>) -> State {
        match event {
            Terminal::Stored(digest) => self.stored(writing.into_pending(), digest, out),
            Terminal::Committed(_) => match writing {
                Writing::First(pending) => self.retry_committed(pending),
                Writing::Retry(pending) => Self::failure(
                    pending.record,
                    StoreFault::NotDurable { file: pending.record },
                    pending.after.loading(),
                    out,
                ),
            },
            event @ (Terminal::Loaded(_)
            | Terminal::Scanned { .. }
            | Terminal::Removed
            | Terminal::Synced
            | Terminal::Closed
            | Terminal::Missing
            | Terminal::Fault(_)) => {
                let pending = writing.into_pending();
                Self::failure(pending.record, super::event_fault(pending.record, event), pending.after.loading(), out)
            }
        }
    }

    fn retry_committed(&mut self, mut pending: Pending) -> State {
        self.retain_written(pending.file.name.clone(), digest(&pending.file.bytes));
        pending.expected = Expect::Digest(digest(&pending.file.bytes));
        State::Write(Writing::Retry(pending))
    }

    fn stored(&mut self, writing: Pending, digest: Digest, out: &mut Queue<Action>) -> State {
        self.retain_written(writing.file.name.clone(), digest);
        let retained = match writing.record {
            RecordName::Turn(_) => true,
            RecordName::State | RecordName::Journal | RecordName::Delivery(_) => {
                self.versions.insert(writing.file.name, digest).is_ok()
            }
        };
        if !retained {
            return Self::failure(
                writing.record,
                StoreFault::TooLarge { file: writing.record },
                writing.after.loading(),
                out,
            );
        }
        match writing.after {
            AfterWrite::Answer(terminal) => Self::finished(terminal, out),
            AfterWrite::Journal { state, resume } => State::Fresh(Fresh::Scan { state, resume }),
            AfterWrite::FreshState { state, resume } => State::Fresh(Fresh::RemoveJournal { state, resume }),
        }
    }
}
