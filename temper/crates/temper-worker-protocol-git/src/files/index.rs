//! The private paging index, streamed whole without following links (protocol/git.md, 4.7).
use super::{Action, count, fd};
use crate::{
    Limits, Settings,
    sequence::{self, Next},
};
use alloc::boxed::Box;
use skein_io::kernel::{self, Error, Fd, Op, OpenHow};
use skein_lib::bytes;
use temper_worker_domain_checkout::git;

#[derive(Debug)]
pub(crate) struct File {
    directory: Box<[u8]>,
    dir: Option<Fd>,
    source: Option<Fd>,
    copy: Option<Fd>,
    phase: Phase,
    next: Option<Box<Next>>,
    at: u64,
    buffer: Box<[u8]>,
    written: u32,
    removing: bool,
    failed: bool,
}
#[derive(Clone, Copy, Debug)]
enum Phase {
    Directory,
    Source,
    Copy,
    Read,
    Write,
    CloseCopy,
    CloseSource,
    Remove,
    CloseDirectory,
}
impl File {
    pub fn copy(directory: Box<[u8]>, next: Next) -> Self {
        Self {
            directory,
            dir: None,
            source: None,
            copy: None,
            phase: Phase::Directory,
            next: Some(Box::new(next)),
            at: 0,
            buffer: Box::new([]),
            written: 0,
            removing: false,
            failed: false,
        }
    }
    pub fn remove(directory: Box<[u8]>, next: Next) -> Self {
        Self { removing: true, ..Self::copy(directory, next) }
    }
    pub fn first(&self, settings: &Settings) -> Action {
        Action::Run(match self.phase {
            Phase::Directory => {
                Op::Open { root: settings.root, path: self.directory.clone(), how: OpenHow::DirectoryNoFollow }
            }
            Phase::Source => Op::Open {
                root: self.dir.expect("scan directory"),
                path: Box::from(b"index".as_slice()),
                how: OpenHow::ReadNoFollow,
            },
            Phase::Copy => Op::Open {
                root: self.dir.expect("scan directory"),
                path: Box::from(b"temper-scan".as_slice()),
                how: OpenHow::CreateNoFollow { mode: Some(0o600) },
            },
            Phase::Read => Op::Read { fd: self.source.expect("source index"), buf: self.buffer.clone(), at: self.at },
            Phase::Write => Op::Write {
                fd: self.copy.expect("scan index"),
                bytes: self.buffer.clone(),
                from: self.written,
                at: self.at,
            },
            Phase::CloseCopy => Op::Close { fd: self.copy.expect("scan index") },
            Phase::CloseSource => Op::Close { fd: self.source.expect("source index") },
            Phase::Remove => Op::Remove {
                dir: self.dir.expect("scan directory"),
                name: Box::from(b"temper-scan".as_slice()),
                directory: false,
            },
            Phase::CloseDirectory => Op::Close { fd: self.dir.expect("scan directory") },
        })
    }
    pub fn complete(&mut self, complete: kernel::Complete, settings: &Settings, limits: &Limits, stop: bool) -> Action {
        let missing = self.removing && complete.result == Err(Error::NotFound);
        self.failed |= (stop && !self.removing) || (complete.result.is_err() && !missing);
        match self.phase {
            Phase::Directory => {
                self.dir = fd(complete.result).ok();
                if self.dir.is_none() {
                    return self.end();
                }
                self.phase = if self.removing {
                    Phase::Remove
                } else if self.failed {
                    Phase::CloseDirectory
                } else {
                    Phase::Source
                };
            }
            Phase::Source => {
                self.source = fd(complete.result).ok();
                self.phase = if self.failed { self.closing() } else { Phase::Copy };
            }
            Phase::Copy => {
                self.copy = fd(complete.result).ok();
                self.phase = if self.failed { self.closing() } else { self.reading(limits) };
            }
            Phase::Read => {
                let n = count(complete.result).unwrap_or(0);
                if n == 0 || self.failed {
                    self.phase = self.closing();
                } else {
                    let buf = super::read_buffer(complete.kind);
                    self.buffer =
                        Box::from(buf.get(..usize::try_from(n).expect("read count fits")).expect("kernel read count"));
                    self.written = 0;
                    self.phase = Phase::Write;
                }
            }
            Phase::Write => {
                let n = count(complete.result).unwrap_or(0);
                self.failed |= n == 0;
                self.written = self.written.checked_add(n).expect("bounded write");
                self.at = self.at.checked_add(u64::from(n)).expect("index offset fits");
                self.phase = if self.failed {
                    self.closing()
                } else if usize::try_from(self.written).expect("write count fits") == self.buffer.len() {
                    self.reading(limits)
                } else {
                    Phase::Write
                };
            }
            Phase::CloseCopy => {
                self.copy = None;
                self.phase = self.closing();
            }
            Phase::CloseSource => {
                self.source = None;
                self.phase = self.closing();
            }
            Phase::Remove => self.phase = Phase::CloseDirectory,
            Phase::CloseDirectory => {
                self.dir = None;
                return self.end();
            }
        }
        self.first(settings)
    }
    fn reading(&mut self, limits: &Limits) -> Phase {
        self.buffer = bytes::zeroed(usize::try_from(limits.read_bytes).expect("read size fits"));
        Phase::Read
    }
    fn closing(&self) -> Phase {
        if self.copy.is_some() {
            Phase::CloseCopy
        } else if self.source.is_some() {
            Phase::CloseSource
        } else {
            Phase::CloseDirectory
        }
    }
    fn end(&mut self) -> Action {
        Action::Operation(if self.failed {
            sequence::failed(git::Fault::Broken, b"", 0)
        } else {
            *self.next.take().expect("index continuation")
        })
    }
}
