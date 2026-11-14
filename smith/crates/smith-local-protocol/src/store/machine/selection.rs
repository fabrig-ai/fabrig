//! One chat directory selected between serial store operations (protocol/host.md, section 5.3).
//! The old root stays owned until its replacement opens; both roots settle on close.
use super::{Action, Box, Map, Queue, State, Store, Token, file, kernel, local};

#[derive(Debug)]
pub(super) enum Selection {
    Making { name: Box<[u8]> },
    Opening { name: Box<[u8]>, descriptor: Option<kernel::Fd> },
    Retiring { name: Box<[u8]>, root: Token, descriptor: kernel::Fd },
}

impl Store {
    /// Supply the descriptor of the directory token reported by the shared file layer.
    pub fn opened_directory_descriptor(&mut self, opened: kernel::Fd) {
        match &mut self.phase {
            State::Selecting(Selection::Opening { descriptor, .. }) => *descriptor = Some(opened),
            State::Idle
            | State::Selecting(_)
            | State::Compatibility(_)
            | State::Load(_)
            | State::Fresh(_)
            | State::Write(_)
            | State::Cleaning
            | State::Closing
            | State::Closed => unreachable!("only selection opens a directory"),
        }
    }

    pub(super) fn select_chat(&self, name: Box<[u8]>, create: bool, out: &mut Queue<Action>) -> State {
        let refused = if self.ephemeral.is_some() {
            Some(local::ChatRefusal::Ephemeral)
        } else if name.len() > 255 || !kernel::is_name(&name) {
            Some(local::ChatRefusal::Name)
        } else {
            None
        };
        if let Some(reason) = refused {
            return Self::finished(local::Event::ChatSelected { name, result: Err(reason) }, out);
        }
        State::Selecting(if create {
            Selection::Making { name }
        } else {
            Selection::Opening { name, descriptor: None }
        })
    }

    pub(super) fn selection_request(&self, state: &Selection) -> file::Request {
        match state {
            Selection::Making { name } => file::Request::MakeDirectory {
                owner: self.owner,
                root: self.chats_root,
                path: name.clone(),
                mode: 0o700,
            },
            Selection::Opening { name, .. } => file::Request::OpenDirectory {
                owner: self.owner,
                root: self.chats_descriptor,
                name: name.clone(),
                no_follow: true,
            },
            Selection::Retiring { .. } => file::Request::Close { owner: self.owner, file: self.root },
        }
    }

    pub(super) fn selection_event(
        &mut self,
        selection: Selection,
        event: file::Event,
        out: &mut Queue<Action>,
    ) -> State {
        match event {
            file::Event::Made { .. } => match selection {
                Selection::Making { name } => State::Selecting(Selection::Opening { name, descriptor: None }),
                Selection::Opening { .. } | Selection::Retiring { .. } => {
                    unreachable!("only directory creation emits Made")
                }
            },
            file::Event::Opened { file, .. } => match selection {
                Selection::Opening { name, descriptor } => {
                    let descriptor = descriptor.expect("file layer's opened descriptor");
                    if self.root == self.chats_root {
                        self.selected(name, file, descriptor, out)
                    } else {
                        State::Selecting(Selection::Retiring { name, root: file, descriptor })
                    }
                }
                Selection::Making { .. } | Selection::Retiring { .. } => {
                    unreachable!("only selection's open emits Opened")
                }
            },
            file::Event::Closed { .. } => match selection {
                Selection::Retiring { name, root, descriptor } => self.selected(name, root, descriptor, out),
                Selection::Making { .. } | Selection::Opening { .. } => {
                    unreachable!("only the previous root emits Closed")
                }
            },
            file::Event::Failed { error, .. } => {
                let reason = match error {
                    kernel::Error::NotFound => local::ChatRefusal::Missing,
                    kernel::Error::Refused
                    | kernel::Error::Busy
                    | kernel::Error::Reset
                    | kernel::Error::BrokenPipe
                    | kernel::Error::NotConnected
                    | kernel::Error::AddressInUse
                    | kernel::Error::AddressNotAvailable
                    | kernel::Error::Unreachable
                    | kernel::Error::TimedOut
                    | kernel::Error::TooManyOpenFiles
                    | kernel::Error::NoBufferSpace
                    | kernel::Error::Cancelled
                    | kernel::Error::TooLate
                    | kernel::Error::Exists
                    | kernel::Error::NotADirectory
                    | kernel::Error::IsADirectory
                    | kernel::Error::NotEmpty
                    | kernel::Error::Permission
                    | kernel::Error::NoSpace
                    | kernel::Error::ReadOnly
                    | kernel::Error::TooManyLinks
                    | kernel::Error::NameTooLong
                    | kernel::Error::Escape
                    | kernel::Error::NotAFile
                    | kernel::Error::InvalidArgument
                    | kernel::Error::Other(_) => local::ChatRefusal::Io,
                };
                self.selection_failed(selection, reason, out)
            }
            file::Event::Cancelled { .. } | file::Event::Refused { .. } => {
                self.selection_failed(selection, local::ChatRefusal::Io, out)
            }
            file::Event::Locked { .. }
            | file::Event::Busy { .. }
            | file::Event::Loaded { .. }
            | file::Event::Scanned { .. }
            | file::Event::Stored { .. }
            | file::Event::Conflict { .. }
            | file::Event::TooLarge { .. }
            | file::Event::Stated { .. }
            | file::Event::Written { .. }
            | file::Event::Read { .. }
            | file::Event::Cut { .. }
            | file::Event::Synced { .. }
            | file::Event::Linked { .. }
            | file::Event::Renamed { .. }
            | file::Event::Listed { .. }
            | file::Event::Removed { .. } => unreachable!("selection only creates, opens and retires directories"),
        }
    }

    fn selected(&mut self, name: Box<[u8]>, root: Token, descriptor: kernel::Fd, out: &mut Queue<Action>) -> State {
        self.root = root;
        self.descriptor = descriptor;
        self.checked = false;
        self.versions = Map::with_capacity(self.versions.capacity());
        Self::finished(local::Event::ChatSelected { name, result: Ok(()) }, out)
    }

    fn selection_failed(&mut self, selection: Selection, reason: local::ChatRefusal, out: &mut Queue<Action>) -> State {
        let name = match selection {
            Selection::Making { name } | Selection::Opening { name, .. } => name,
            Selection::Retiring { name, root, descriptor } => {
                self.checked = false;
                self.root = root;
                self.descriptor = descriptor;
                out.push(Action::Domain(local::Event::StoreFailed { turn: None, reason: local::StoreFailure::Write }));
                name
            }
        };
        Self::finished(local::Event::ChatSelected { name, result: Err(reason) }, out)
    }
}
