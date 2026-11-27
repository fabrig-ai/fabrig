//! Delivery hook discovery (smith/docs/design/domain/run-delivery.md, 1.1).
//! Holds only the open directory, the current entry and the first result;
//! the file layer keeps descriptors and enforces one sibling link at most.

use alloc::boxed::Box;
use skein_io::{file, kernel};
use skein_lib::{Queue, Time, Token};
use smith_domain::run::{Hook, HookProblem, Hooks};

use crate::boundary::{Below, ToDomain};
use crate::guide::{Guide, Next};

#[derive(Debug)]
pub(crate) enum Probe {
    Opening { deadline: Time },
    Entry { directory: Token, deadline: Time, deliver: Option<Hook> },
    Stating { directory: Token, deadline: Time, deliver: Option<Hook>, file: Token },
    ClosingEntry { directory: Token, deadline: Time, deliver: Option<Hook>, hook: Hook },
    ClosingDirectory { hooks: Hooks },
}

impl Probe {
    pub(crate) const fn cancellable(&self) -> bool {
        match self {
            Self::Opening { .. } | Self::Entry { .. } | Self::Stating { .. } => true,
            Self::ClosingEntry { .. } | Self::ClosingDirectory { .. } => false,
        }
    }

    pub(crate) fn event(self, owner: Token, event: file::Event, below: &mut Queue<Below>) -> Next {
        let event = observed(event);
        match self {
            Self::Opening { deadline } => match event {
                Observed::Opened(file) => entry(owner, file, deadline, None, below),
                Observed::Failed(
                    kernel::Error::NotFound | kernel::Error::NotADirectory | kernel::Error::TooManyLinks,
                ) => Next::Done(ToDomain::HooksProbed { owner, hooks: None }),
                Observed::Failed(_) => Next::Done(unreadable(owner)),
                Observed::Closed | Observed::Stated(_) => unreachable!("hooks directory open terminal"),
            },
            Self::Entry { directory, deadline, deliver } => match event {
                Observed::Opened(file) => {
                    below.push(Below::File { request: file::Request::Stat { owner, file }, deadline });
                    keep(Self::Stating { directory, deadline, deliver, file })
                }
                Observed::Failed(kernel::Error::NotFound) => {
                    found(owner, directory, deadline, deliver, Hook::Missing, below)
                }
                Observed::Failed(kernel::Error::TooManyLinks) => {
                    found(owner, directory, deadline, deliver, Hook::Broken(HookProblem::BadLink), below)
                }
                Observed::Failed(_) => {
                    found(owner, directory, deadline, deliver, Hook::Broken(HookProblem::Unreadable), below)
                }
                Observed::Closed | Observed::Stated(_) => unreachable!("hook open terminal"),
            },
            Self::Stating { directory, deadline, deliver, file } => {
                let hook = match event {
                    Observed::Stated(stat) => classify(stat),
                    Observed::Failed(_) => Hook::Broken(HookProblem::Unreadable),
                    Observed::Closed | Observed::Opened(_) => unreachable!("hook stat terminal"),
                };
                close(owner, file, below);
                keep(Self::ClosingEntry { directory, deadline, deliver, hook })
            }
            Self::ClosingEntry { directory, deadline, deliver, hook } => {
                closed(event);
                found(owner, directory, deadline, deliver, hook, below)
            }
            Self::ClosingDirectory { hooks } => {
                closed(event);
                Next::Done(ToDomain::HooksProbed { owner, hooks: Some(hooks) })
            }
        }
    }
}

pub(crate) fn unreadable(owner: Token) -> ToDomain {
    ToDomain::HooksProbed { owner, hooks: Some(Hooks::UNREADABLE) }
}

fn keep(probe: Probe) -> Next {
    Next::Continue(Guide::Hooks(probe))
}

fn entry(owner: Token, directory: Token, deadline: Time, deliver: Option<Hook>, below: &mut Queue<Below>) -> Next {
    let name = if deliver.is_none() { Box::from(&b"deliver"[..]) } else { Box::from(&b"finish"[..]) };
    below.push(Below::File { request: file::Request::OpenSiblingFile { owner, directory, name }, deadline });
    keep(Probe::Entry { directory, deadline, deliver })
}

fn found(
    owner: Token,
    directory: Token,
    deadline: Time,
    deliver: Option<Hook>,
    hook: Hook,
    below: &mut Queue<Below>,
) -> Next {
    match deliver {
        None => entry(owner, directory, deadline, Some(hook), below),
        Some(deliver) => {
            close(owner, directory, below);
            keep(Probe::ClosingDirectory { hooks: Hooks { deliver, finish: hook } })
        }
    }
}

fn classify(stat: kernel::Stat) -> Hook {
    match stat.kind {
        kernel::Kind::File if stat.mode & 0o111 != 0 => Hook::Executable,
        kernel::Kind::File => Hook::Broken(HookProblem::NotExecutable),
        kernel::Kind::Directory | kernel::Kind::Other => Hook::Broken(HookProblem::NotFile),
        kernel::Kind::Symlink => Hook::Broken(HookProblem::BadLink),
    }
}

fn close(owner: Token, file: Token, below: &mut Queue<Below>) {
    below.push(Below::File { request: file::Request::Close { owner, file }, deadline: Time::from_nanos(u64::MAX) });
}

fn closed(event: Observed) {
    match event {
        Observed::Closed | Observed::Failed(_) => {}
        Observed::Opened(_) | Observed::Stated(_) => unreachable!("close terminal"),
    }
}

/// File terminals that can advance a probe.
enum Observed {
    Opened(Token),
    Stated(kernel::Stat),
    Closed,
    Failed(kernel::Error),
}

fn observed(event: file::Event) -> Observed {
    match event {
        file::Event::Opened { file, .. } => Observed::Opened(file),
        file::Event::Stated { stat, .. } => Observed::Stated(stat),
        file::Event::Closed { .. } => Observed::Closed,
        file::Event::Failed { error, .. } => Observed::Failed(error),
        file::Event::Cancelled { .. } => Observed::Failed(kernel::Error::Cancelled),
        file::Event::Locked { .. }
        | file::Event::Busy { .. }
        | file::Event::Made { .. }
        | file::Event::Refused { .. }
        | file::Event::Loaded { .. }
        | file::Event::Scanned { .. }
        | file::Event::Stored { .. }
        | file::Event::Conflict { .. }
        | file::Event::TooLarge { .. }
        | file::Event::Written { .. }
        | file::Event::Read { .. }
        | file::Event::Cut { .. }
        | file::Event::Synced { .. }
        | file::Event::Linked { .. }
        | file::Event::Renamed { .. }
        | file::Event::Removed { .. }
        | file::Event::Listed { .. } => unreachable!("a probe only opens, stats and closes"),
    }
}
