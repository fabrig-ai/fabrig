//! One listing's directory open, close and spawn (protocol/agent.md, 3).
//! Holds the spawn until the checked directory is closed; cancellation never
//! starts a child. Each file terminal advances the state and follows its effect.

use super::Spawn;
use crate::Below;
use skein_io::{file, kernel};
use skein_lib::{Queue, Time, Token};
use smith_domain::tools;

#[derive(Debug)]
enum State {
    Opening { root: Token, spawn: Spawn },
    Closing { root: Token, spawn: Spawn, file: Token },
    Ready { root: Token, spawn: Spawn },
    Spawned,
    Failed(tools::Done),
    Stopped,
}

/// The file operation's owner is stable throughout the directory lifecycle.
#[derive(Debug)]
pub(crate) struct Directory {
    owner: Token,
    state: State,
}

impl Directory {
    pub(super) fn open(owner: Token, root: Token, spawn: Spawn, deadline: Time) -> (Self, Below) {
        let path = spawn.dir.clone();
        let directory = Self { owner, state: State::Opening { root, spawn } };
        (directory, Below::OpenDirectory { owner, root, path, no_follow: false, deadline })
    }

    #[expect(clippy::wildcard_enum_match_arm, reason = "file events are a foreign vocabulary")]
    pub(super) fn event(&mut self, event: file::Event, stopping: bool, below: &mut Queue<Below>) {
        let state = core::mem::replace(&mut self.state, State::Stopped);
        self.state = match state {
            State::Opening { root, spawn } => match event {
                file::Event::Opened { file, .. } => State::Closing { root, spawn, file },
                file::Event::Failed { error, .. } => State::Failed(directory_failure(error)),
                file::Event::Cancelled { .. } => State::Failed(tools::Done::TimedOut),
                _ => unreachable!("directory-open terminal"),
            },
            State::Closing { root, spawn, .. } => match event {
                file::Event::Closed { .. } => {
                    if stopping {
                        State::Stopped
                    } else {
                        State::Ready { root, spawn }
                    }
                }
                file::Event::Failed { .. } | file::Event::Cancelled { .. } => {
                    State::Failed(tools::Done::Failed { fault: tools::Fault::Other })
                }
                _ => unreachable!("directory-close terminal"),
            },
            State::Ready { .. } | State::Spawned | State::Failed(_) | State::Stopped => {
                unreachable!("file event belongs to a pending listing directory")
            }
        };
        self.follow(below);
    }

    fn follow(&mut self, below: &mut Queue<Below>) {
        let state = core::mem::replace(&mut self.state, State::Stopped);
        self.state = match state {
            State::Closing { root, spawn, file } => {
                below.push(Below::File {
                    request: file::Request::Close { owner: self.owner, file },
                    deadline: Time::from_nanos(u64::MAX),
                });
                State::Closing { root, spawn, file }
            }
            State::Ready { root, spawn } => {
                below.push(Below::Spawn { owner: self.owner, root, spawn });
                State::Spawned
            }
            resting @ (State::Opening { .. } | State::Spawned | State::Failed(_) | State::Stopped) => resting,
        };
    }

    pub(super) fn cancel(&self, below: &mut Queue<Below>) {
        match self.state {
            State::Opening { .. } => below.push(Below::CancelFile { owner: self.owner }),
            State::Closing { .. } | State::Ready { .. } | State::Spawned | State::Failed(_) | State::Stopped => {}
        }
    }

    pub(super) fn unspawned(&self) -> bool {
        match self.state {
            State::Failed(_) | State::Stopped => true,
            State::Opening { .. } | State::Closing { .. } | State::Ready { .. } | State::Spawned => false,
        }
    }

    pub(super) fn failure(self) -> Option<tools::Done> {
        match self.state {
            State::Failed(done) => Some(done),
            State::Opening { .. } | State::Closing { .. } | State::Ready { .. } | State::Spawned | State::Stopped => {
                None
            }
        }
    }
}

#[expect(clippy::wildcard_enum_match_arm, reason = "kernel errors are a foreign vocabulary")]
fn directory_failure(error: kernel::Error) -> tools::Done {
    match error {
        kernel::Error::NotFound => tools::Done::Missing,
        kernel::Error::NotADirectory | kernel::Error::IsADirectory | kernel::Error::NotAFile => {
            tools::Done::NotDirectory
        }
        kernel::Error::Escape => tools::Done::Escapes,
        kernel::Error::Permission | kernel::Error::ReadOnly => tools::Done::Failed { fault: tools::Fault::Denied },
        kernel::Error::NoSpace => tools::Done::Failed { fault: tools::Fault::NoSpace },
        kernel::Error::TimedOut => tools::Done::TimedOut,
        kernel::Error::Cancelled => tools::Done::Cancelled,
        _ => tools::Done::Failed { fault: tools::Fault::Other },
    }
}
