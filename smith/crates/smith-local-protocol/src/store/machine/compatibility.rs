//! The pre-recovery layout check (domain/host.md, section 8).
//! Only complete empty scans and regular reserved marker residue permit creation.
use super::Terminal;
use super::{Action, Queue, RecordName, State, Store, file, kernel, local};

#[derive(Debug)]
pub(super) enum Compatibility {
    Format,
    Admit,
    CreateFormat,
}

impl Store {
    pub(super) fn compatibility_request(&self, state: &Compatibility) -> file::Request {
        match state {
            Compatibility::Format => self.load_file(b"format".as_slice().into(), crate::format::max_bytes()),
            Compatibility::Admit => self.scan_request(true),
            Compatibility::CreateFormat => {
                let marker = crate::format::file();
                file::Request::Store {
                    owner: self.owner,
                    root: self.root,
                    path: marker.name,
                    bytes: marker.bytes,
                    expected: file::Expect::Absent,
                    no_follow: true,
                }
            }
        }
    }

    pub(super) fn compatibility_event(
        &mut self,
        state: Compatibility,
        event: Terminal,
        out: &mut Queue<Action>,
    ) -> State {
        match state {
            Compatibility::Format => match event {
                Terminal::Loaded(bytes) => match crate::format::check(&bytes) {
                    Ok(()) => {
                        self.checked = true;
                        State::Load(super::Loading::Journal)
                    }
                    Err(why) => older(why, out),
                },
                Terminal::Missing => State::Compatibility(Compatibility::Admit),
                event @ (Terminal::Scanned { .. }
                | Terminal::Stored(_)
                | Terminal::Removed
                | Terminal::Synced
                | Terminal::Closed
                | Terminal::Committed(_)
                | Terminal::Fault(_)) => {
                    Self::failure(RecordName::State, super::event_fault(RecordName::State, event), true, out)
                }
            },
            Compatibility::Admit => match event {
                Terminal::Scanned { entries, more } => {
                    if more != 0 {
                        return older(local::Older::Unrecorded, out);
                    }
                    for entry in &entries {
                        if entry.kind != kernel::Kind::File || !marker_residue(&entry.name) {
                            return older(local::Older::Unrecorded, out);
                        }
                    }
                    State::Compatibility(Compatibility::CreateFormat)
                }
                Terminal::Fault(super::Fault::Conflict) => {
                    Self::failure(RecordName::State, super::StoreFault::Conflict { file: RecordName::State }, true, out)
                }
                event @ (Terminal::Loaded(_)
                | Terminal::Stored(_)
                | Terminal::Removed
                | Terminal::Synced
                | Terminal::Closed
                | Terminal::Missing
                | Terminal::Committed(_)
                | Terminal::Fault(_)) => {
                    let _: super::StoreFault = super::event_fault(RecordName::State, event);
                    older(local::Older::Unrecorded, out)
                }
            },
            Compatibility::CreateFormat => match event {
                Terminal::Stored(_) => {
                    self.checked = true;
                    Self::empty(out)
                }
                event @ (Terminal::Loaded(_)
                | Terminal::Scanned { .. }
                | Terminal::Removed
                | Terminal::Synced
                | Terminal::Closed
                | Terminal::Missing
                | Terminal::Committed(_)
                | Terminal::Fault(_)) => {
                    Self::failure(RecordName::State, super::event_fault(RecordName::State, event), true, out)
                }
            },
        }
    }
}

fn older(why: local::Older, out: &mut Queue<Action>) -> State {
    Store::finished(local::Event::StoreFailed { turn: None, reason: local::StoreFailure::OlderBuild { why } }, out)
}

fn marker_residue(name: &[u8]) -> bool {
    if name.len() != 29 || !name.starts_with(b"format.skein-") {
        return false;
    }
    for byte in name.get(13..).expect("reserved prefix length") {
        if !byte.is_ascii_digit() && !(b'a'..=b'f').contains(byte) {
            return false;
        }
    }
    true
}
