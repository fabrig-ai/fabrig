//! Read-only directory inspection before a source check (protocol/host.md, 2).
//! Keeps one file request in flight and closes each opened directory before
//! answering. No login contents are read and no directory is made.
use alloc::boxed::Box;
use skein_io::{file, kernel};
use skein_lib::{Queue, Time, Token, Writer};
use smith_host_domain::{FailureReason, ProgramCheck, ProgramFailure, SourceName};

#[derive(Debug)]
enum Stage {
    Opening,
    Stating(Token),
    Closing,
    Done,
}

/// Inspect both login directory levels without creating or following either.
#[derive(Debug)]
pub struct SourceInspection {
    owner: Token,
    root: kernel::Fd,
    programs: Box<[u8]>,
    source: SourceName,
    user: u32,
    deadline: Time,
    expired: bool,
    second: bool,
    stage: Stage,
    result: Option<Result<(), ProgramCheck>>,
}

impl SourceInspection {
    /// Open the programs name beneath its shell-opened parent, without links.
    #[must_use]
    pub fn new(
        owner: Token,
        root: kernel::Fd,
        programs: Box<[u8]>,
        source: SourceName,
        user: u32,
        deadline: Time,
        below: &mut Queue<file::Request>,
    ) -> Self {
        below.push(file::Request::OpenDirectory { owner, root, name: programs.clone(), no_follow: true });
        Self {
            owner,
            root,
            programs,
            source,
            user,
            deadline,
            expired: false,
            second: false,
            stage: Stage::Opening,
            result: None,
        }
    }

    /// Whether this file event owner belongs to the inspection.
    #[must_use]
    pub fn owns(&self, owner: Token) -> bool {
        self.owner == owner
    }

    /// Consume the outstanding file result, retaining no open directory at answer.
    pub fn from_file(&mut self, event: file::Event, below: &mut Queue<file::Request>) {
        match event {
            file::Event::Opened { file, .. } => {
                if self.expired {
                    self.close(file, below);
                } else {
                    self.stage = Stage::Stating(file);
                    below.push(file::Request::Stat { owner: self.owner, file });
                }
            }
            file::Event::Stated { stat, .. } => {
                #[expect(clippy::verbose_bit_mask, reason = "the permission mask names group and other bits")]
                let private = stat.kind == kernel::Kind::Directory && stat.owner == self.user && stat.mode & 0o077 == 0;
                if !private && !self.expired {
                    self.fail(FailureReason::Unreadable, b"program source directory is not private");
                }
                let file = match self.stage {
                    Stage::Stating(file) => file,
                    Stage::Opening | Stage::Closing | Stage::Done => unreachable!("stat follows open"),
                };
                self.close(file, below);
            }
            file::Event::Closed { .. } => self.closed(below),
            file::Event::Failed { error, .. } => self.failed(error, below),
            file::Event::Cancelled { .. } => self.failed(kernel::Error::Cancelled, below),
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
            | file::Event::Listed { .. } => unreachable!("inspection opens, stats and closes only"),
        }
    }

    /// Include directory inspection in the host's check deadline.
    pub fn fire(&mut self, now: Time) {
        match self.stage {
            Stage::Done => {}
            Stage::Opening | Stage::Stating(_) | Stage::Closing => {
                if !self.expired && now >= self.deadline {
                    self.expired = true;
                    self.fail(FailureReason::Deadline, b"program source inspection deadline");
                }
            }
        }
    }

    /// The check deadline until it has fired or the inspection has settled.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        match self.stage {
            Stage::Done => None,
            Stage::Opening | Stage::Stating(_) | Stage::Closing => {
                if self.expired {
                    None
                } else {
                    Some(self.deadline)
                }
            }
        }
    }

    /// Take readiness, or the host check answer, once all file requests settle.
    pub fn answer(&mut self) -> Option<Result<(), ProgramCheck>> {
        match self.stage {
            Stage::Done => self.result.take(),
            Stage::Opening | Stage::Stating(_) | Stage::Closing => None,
        }
    }

    fn close(&mut self, file: Token, below: &mut Queue<file::Request>) {
        self.stage = Stage::Closing;
        below.push(file::Request::Close { owner: self.owner, file });
    }

    fn closed(&mut self, below: &mut Queue<file::Request>) {
        if self.result.is_some() || self.second {
            self.stage = Stage::Done;
            if self.result.is_none() {
                self.result = Some(Ok(()));
            }
            return;
        }
        self.second = true;
        let length = self
            .programs
            .len()
            .checked_add(1)
            .expect("separator length fits")
            .checked_add(self.source.as_bytes().len())
            .expect("validated source path length");
        let mut path = Writer::new(length);
        path.put(&self.programs).expect("programs fits");
        path.put(b"/").expect("separator fits");
        path.put(self.source.as_bytes()).expect("source fits");
        self.stage = Stage::Opening;
        below.push(file::Request::OpenDirectory {
            owner: self.owner,
            root: self.root,
            name: path.finish(),
            no_follow: true,
        });
    }

    fn failed(&mut self, error: kernel::Error, below: &mut Queue<file::Request>) {
        match self.stage {
            Stage::Opening => {
                if !self.expired && error == kernel::Error::NotFound {
                    self.result = Some(Err(ProgramCheck::NotSignedIn));
                } else if !self.expired {
                    self.fail(FailureReason::Unreadable, b"cannot open program source directory");
                }
                self.stage = Stage::Done;
            }
            Stage::Stating(file) => {
                if !self.expired {
                    self.fail(FailureReason::Unreadable, b"cannot stat program source directory");
                }
                self.close(file, below);
            }
            Stage::Closing => {
                if !self.expired {
                    self.fail(FailureReason::Unreadable, b"cannot close program source directory");
                }
                self.stage = Stage::Done;
            }
            Stage::Done => unreachable!("one terminal per file request"),
        }
    }

    fn fail(&mut self, reason: FailureReason, detail: &[u8]) {
        self.result = Some(Err(ProgramCheck::Failed(ProgramFailure { reason, detail: Box::from(detail) })));
    }
}
