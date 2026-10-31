//! Fresh-history recovery, from journal through both directory syncs (protocol/hosts.md, 5.3).
//! Recovery retains its new metadata and whether its terminal resumes a load or a save.
use super::Terminal;
use super::{Action, AfterWrite, Box, ChatState, Queue, RecordName, State, Store, file, local, save_state};

#[derive(Clone, Copy, Debug)]
pub(super) enum Resume {
    Load,
    Save,
}

impl Resume {
    pub(super) fn loading(self) -> bool {
        match self {
            Self::Load => true,
            Self::Save => false,
        }
    }
}

#[derive(Debug)]
pub(super) enum Fresh {
    Scan { state: ChatState, resume: Resume },
    Remove { state: ChatState, resume: Resume, entries: Box<[file::Entry]>, index: usize },
    SyncOld { state: ChatState, resume: Resume },
    RemoveJournal { state: ChatState, resume: Resume },
    SyncJournal { state: ChatState, resume: Resume },
}

impl Store {
    pub(super) fn fresh_request(&self, fresh: &Fresh) -> file::Request {
        match fresh {
            Fresh::Scan { .. } => self.scan_request(false),
            Fresh::Remove { entries, index, .. } => {
                self.remove_file(entries.get(*index).expect("selected removal").name.clone())
            }
            Fresh::SyncOld { .. } | Fresh::SyncJournal { .. } => {
                file::Request::SyncDirectory { owner: self.owner, root: self.descriptor }
            }
            Fresh::RemoveJournal { .. } => self.remove_file(b"fresh".as_slice().into()),
        }
    }

    pub(super) fn fresh_event(&mut self, fresh: Fresh, event: Terminal, out: &mut Queue<Action>) -> State {
        match fresh {
            Fresh::Scan { state, resume } => match event {
                Terminal::Scanned { entries, more: 0 } => remove_next(state, resume, entries, 0),
                Terminal::Scanned { .. } => Self::failure(
                    RecordName::State,
                    super::StoreFault::TooLarge { file: RecordName::State },
                    resume.loading(),
                    out,
                ),
                event @ (Terminal::Loaded(_)
                | Terminal::Stored(_)
                | Terminal::Removed
                | Terminal::Synced
                | Terminal::Closed
                | Terminal::Missing
                | Terminal::Committed(_)
                | Terminal::Fault(_)) => fresh_failed(RecordName::State, resume, event, out),
            },
            Fresh::Remove { state, resume, entries, index } => match event {
                Terminal::Removed => remove_next(state, resume, entries, index.checked_add(1).expect("bounded scan")),
                event @ (Terminal::Loaded(_)
                | Terminal::Scanned { .. }
                | Terminal::Stored(_)
                | Terminal::Synced
                | Terminal::Closed
                | Terminal::Missing
                | Terminal::Committed(_)
                | Terminal::Fault(_)) => fresh_failed(
                    super::record_name(&entries.get(index).expect("selected removal").name).expect("record removal"),
                    resume,
                    event,
                    out,
                ),
            },
            Fresh::SyncOld { state, resume } => match event {
                Terminal::Synced => self.begin_write(
                    RecordName::State,
                    save_state(state),
                    AfterWrite::FreshState { state, resume },
                    out,
                ),
                event @ (Terminal::Loaded(_)
                | Terminal::Scanned { .. }
                | Terminal::Stored(_)
                | Terminal::Removed
                | Terminal::Closed
                | Terminal::Missing
                | Terminal::Committed(_)
                | Terminal::Fault(_)) => fresh_failed(RecordName::State, resume, event, out),
            },
            Fresh::RemoveJournal { state, resume } => match event {
                Terminal::Removed => {
                    self.versions.remove(&Box::from(&b"fresh"[..]));
                    State::Fresh(Fresh::SyncJournal { state, resume })
                }
                event @ (Terminal::Loaded(_)
                | Terminal::Scanned { .. }
                | Terminal::Stored(_)
                | Terminal::Synced
                | Terminal::Closed
                | Terminal::Missing
                | Terminal::Committed(_)
                | Terminal::Fault(_)) => fresh_failed(RecordName::Journal, resume, event, out),
            },
            Fresh::SyncJournal { state, resume } => match event {
                Terminal::Synced => match resume {
                    Resume::Load => State::Load(super::Loading::Scan { state: Some(state) }),
                    Resume::Save => Self::finished(local::Event::StateSaved, out),
                },
                event @ (Terminal::Loaded(_)
                | Terminal::Scanned { .. }
                | Terminal::Stored(_)
                | Terminal::Removed
                | Terminal::Closed
                | Terminal::Missing
                | Terminal::Committed(_)
                | Terminal::Fault(_)) => fresh_failed(RecordName::State, resume, event, out),
            },
        }
    }
}

fn remove_next(state: ChatState, resume: Resume, entries: Box<[file::Entry]>, start: usize) -> State {
    for index in start..entries.len() {
        match super::record_name(&entries.get(index).expect("bounded removal index").name) {
            Some(RecordName::Turn(_) | RecordName::Delivery(_)) => {
                return State::Fresh(Fresh::Remove { state, resume, entries, index });
            }
            Some(RecordName::State | RecordName::Journal) | None => {}
        }
    }
    State::Fresh(Fresh::SyncOld { state, resume })
}

fn fresh_failed(record: RecordName, resume: Resume, event: Terminal, out: &mut Queue<Action>) -> State {
    Store::failure(record, super::event_fault(record, event), resume.loading(), out)
}
