//! Durable record replacement and its one committed retry (protocol/hosts.md, 5.3).
//! The pending state retains retry bytes and the continuation until durability settles.
use super::Terminal;
use super::{
    Action, ChatState, Digest, Expect, File, Fresh, Queue, RecordName, Resume, State, Store, StoreFault, digest, local,
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
pub(super) struct Writing {
    pub(super) record: RecordName,
    pub(super) file: File,
    pub(super) expected: Expect,
    pub(super) after: AfterWrite,
    pub(super) retried: bool,
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
        State::Write(Writing { record, file, expected, after, retried: false })
    }

    pub(super) fn write_event(&mut self, writing: Writing, event: Terminal, out: &mut Queue<Action>) -> State {
        match event {
            Terminal::Stored(digest) => self.stored(writing, digest, out),
            Terminal::Committed(_) => self.retry_committed(writing, out),
            event @ (Terminal::Loaded(_)
            | Terminal::Scanned { .. }
            | Terminal::Removed
            | Terminal::Synced
            | Terminal::Closed
            | Terminal::Missing
            | Terminal::Fault(_)) => {
                Self::failure(writing.record, super::event_fault(writing.record, event), writing.after.loading(), out)
            }
        }
    }

    fn retry_committed(&mut self, mut writing: Writing, out: &mut Queue<Action>) -> State {
        self.retain_written(writing.file.name.clone(), digest(&writing.file.bytes));
        if writing.retried {
            Self::failure(writing.record, StoreFault::NotDurable { file: writing.record }, writing.after.loading(), out)
        } else {
            writing.retried = true;
            writing.expected = Expect::Digest(digest(&writing.file.bytes));
            State::Write(writing)
        }
    }

    fn stored(&mut self, writing: Writing, digest: Digest, out: &mut Queue<Action>) -> State {
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
