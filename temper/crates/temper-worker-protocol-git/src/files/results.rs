//! Bounded read, then removal, ahead of delivery (domain/forge/reds.md, 5.6).
use super::{Action, count, fd};
use crate::{Settings, sequence::Next};
use alloc::boxed::Box;
use skein_io::kernel::{self, Error, Fd, Op, OpenHow};
use skein_lib::{Intake, bytes, stream};
use temper_worker_domain_checkout::git;

const CAPACITY: u32 = 262_145;
#[derive(Debug)]
pub(crate) struct File {
    directory: Box<[u8]>,
    phase: Phase,
    dir: Option<Fd>,
    opened: Option<Fd>,
    bytes: Intake,
    found: bool,
    failed: bool,
}
#[derive(Clone, Copy, Debug)]
enum Phase {
    Directory,
    Clear,
    Open,
    Read,
    Close,
    Remove,
    CloseDirectory,
}
impl File {
    pub fn new(directory: Box<[u8]>) -> Self {
        Self {
            directory,
            phase: Phase::Directory,
            dir: None,
            opened: None,
            bytes: Intake::with_capacity(CAPACITY),
            found: false,
            failed: false,
        }
    }
    pub fn first(&self, settings: &Settings) -> Action {
        Action::Run(match self.phase {
            Phase::Directory => {
                Op::Open { root: settings.root, path: self.directory.clone(), how: OpenHow::DirectoryNoFollow }
            }
            Phase::Clear => Op::Remove {
                dir: self.dir.expect("opened directory"),
                name: Box::from(b"reds".as_slice()),
                directory: false,
            },
            Phase::Open => Op::Open {
                root: self.dir.expect("opened directory"),
                path: Box::from(b"check-results".as_slice()),
                how: OpenHow::ReadNoFollow,
            },
            Phase::Read => Op::Read {
                fd: self.opened.expect("opened results"),
                buf: bytes::zeroed(usize::try_from(self.bytes.room().min(4096)).expect("bounded read")),
                at: u64::from(self.bytes.len()),
            },
            Phase::Close => Op::Close { fd: self.opened.expect("opened results") },
            Phase::Remove => Op::Remove {
                dir: self.dir.expect("opened directory"),
                name: Box::from(b"check-results".as_slice()),
                directory: false,
            },
            Phase::CloseDirectory => Op::Close { fd: self.dir.expect("opened directory") },
        })
    }
    pub fn complete(&mut self, complete: kernel::Complete, settings: &Settings, stop: bool) -> Action {
        let missing = complete.result == Err(Error::NotFound);
        self.failed |= stop || (complete.result.is_err() && !missing);
        match self.phase {
            Phase::Directory => {
                if let Ok(fd) = fd(complete.result) {
                    self.dir = Some(fd);
                }
                if self.dir.is_none() {
                    return self.end();
                }
                self.phase = if self.failed { Phase::Remove } else { Phase::Clear };
            }
            Phase::Clear => self.phase = if self.failed { Phase::Remove } else { Phase::Open },
            Phase::Open => {
                if let Ok(fd) = fd(complete.result) {
                    self.opened = Some(fd);
                    self.found = true;
                }
                self.phase = if self.opened.is_some() && !self.failed {
                    Phase::Read
                } else if self.opened.is_some() {
                    Phase::Close
                } else {
                    Phase::Remove
                };
            }
            Phase::Read => {
                let n = count(complete.result).unwrap_or(0);
                let Op::Read { buf: bytes, .. } = complete.kind else { unreachable!("results read") };
                let prefix = bytes.get(..usize::try_from(n).expect("kernel read count")).expect("read buffer");
                self.bytes.append(prefix).expect("demand bounded by room");
                self.phase = if n == 0 || self.bytes.room() == 0 || self.failed { Phase::Close } else { Phase::Read };
            }
            Phase::Close => {
                self.opened = None;
                self.phase = Phase::Remove;
            }
            Phase::Remove => self.phase = Phase::CloseDirectory,
            Phase::CloseDirectory => {
                self.dir = None;
                return self.end();
            }
        }
        self.first(settings)
    }
    fn end(&mut self) -> Action {
        let bytes = if self.found && !self.failed {
            let size = self.bytes.len();
            Some(self.bytes.meet(stream::Read::Fill(size)).expect("buffered file"))
        } else {
            None
        };
        Action::Operation(Next::End(git::Done::Results { bytes }))
    }
}
