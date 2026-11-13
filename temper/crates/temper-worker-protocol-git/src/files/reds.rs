//! Whole check files installed by rename (domain/forge/reds.md, 5.2).
use super::{Action, count, fd};
use crate::{Settings, sequence::Next};
use alloc::boxed::Box;
use skein_io::kernel::{Done, Error, Fd, Op, OpenHow};
use temper_worker_domain_checkout::git;
#[derive(Debug)]
pub(crate) struct Files {
    directory: Box<[u8]>,
    content: Option<Box<[u8]>>,
    marker: bool,
    phase: Phase,
    parent: Option<Fd>,
    directory_fd: Option<Fd>,
    file: Option<Fd>,
    written: u32,
    temporary: bool,
    failed: bool,
}
#[derive(Clone, Copy, Debug)]
enum Phase {
    Parent,
    Directory,
    OpenDirectory,
    Clear,
    Discard,
    Open,
    Write,
    Close,
    Rename,
    Cleanup,
    CloseDirectory,
    CloseParent,
}
impl Files {
    pub fn new(directory: Box<[u8]>, content: Option<Box<[u8]>>, marker: bool) -> Self {
        Self {
            directory,
            content,
            marker,
            phase: Phase::Parent,
            parent: None,
            directory_fd: None,
            file: None,
            written: 0,
            temporary: false,
            failed: false,
        }
    }
    fn name(&self, temporary: bool) -> Box<[u8]> {
        Box::from(match (self.marker, temporary) {
            (false, false) => b"reds".as_slice(),
            (false, true) => b"reds.tmp",
            (true, false) => b"reds-wait",
            (true, true) => b"reds-wait.tmp",
        })
    }
    pub fn first(&self, settings: &Settings) -> Action {
        let dir = self.directory_fd;
        Action::Run(match self.phase {
            Phase::Parent => {
                let Some(parent) = self.directory.strip_suffix(b"/temper") else { return Self::end(true) };
                Op::Open { root: settings.root, path: Box::from(parent), how: OpenHow::DirectoryNoFollow }
            }
            Phase::Directory => Op::MakeDirectory {
                dir: self.parent.expect("git directory opened"),
                name: Box::from(b"temper".as_slice()),
                mode: 0o700,
            },
            Phase::OpenDirectory => Op::Open {
                root: self.parent.expect("git directory opened"),
                path: Box::from(b"temper".as_slice()),
                how: OpenHow::DirectoryNoFollow,
            },
            Phase::Clear => {
                Op::Remove { dir: dir.expect("temper opened"), name: Box::from(b"reds".as_slice()), directory: false }
            }
            Phase::Discard | Phase::Cleanup => {
                Op::Remove { dir: dir.expect("temper opened"), name: self.name(true), directory: false }
            }
            Phase::Open => Op::Open {
                root: dir.expect("temper opened"),
                path: self.name(true),
                how: OpenHow::CreateNoFollow { mode: Some(0o600) },
            },
            Phase::Write => Op::Write {
                fd: self.file.expect("opened file"),
                bytes: self.content.clone().expect("write content"),
                from: self.written,
                at: u64::from(self.written),
            },
            Phase::Close => Op::Close { fd: self.file.expect("opened file") },
            Phase::Rename => Op::Rename {
                from_dir: dir.expect("temper opened"),
                from: self.name(true),
                to_dir: dir.expect("temper opened"),
                to: self.name(false),
                no_replace: false,
            },
            Phase::CloseDirectory => Op::Close { fd: dir.expect("temper opened") },
            Phase::CloseParent => Op::Close { fd: self.parent.expect("git directory opened") },
        })
    }
    pub fn complete(&mut self, result: Result<Done, Error>, settings: &Settings, stop: bool) -> Action {
        let allowed = match self.phase {
            Phase::Clear | Phase::Discard | Phase::Cleanup => result == Err(Error::NotFound),
            Phase::Directory => result == Err(Error::Exists),
            Phase::Parent
            | Phase::OpenDirectory
            | Phase::Open
            | Phase::Write
            | Phase::Close
            | Phase::Rename
            | Phase::CloseDirectory
            | Phase::CloseParent => false,
        };
        self.failed |= stop || (result.is_err() && !allowed);
        match self.phase {
            Phase::Parent => {
                if let Ok(fd) = fd(result) {
                    self.parent = Some(fd);
                }
            }
            Phase::OpenDirectory => {
                if let Ok(fd) = fd(result) {
                    self.directory_fd = Some(fd);
                }
            }
            Phase::Open => {
                if let Ok(fd) = fd(result) {
                    self.file = Some(fd);
                    self.temporary = true;
                }
            }
            Phase::Write => match count(result) {
                Ok(0) | Err(_) => self.failed = true,
                Ok(count) => self.written = self.written.checked_add(count).expect("bounded check bytes"),
            },
            Phase::Close => self.file = None,
            Phase::Cleanup => self.temporary = false,
            Phase::Rename if result.is_ok() => self.temporary = false,
            Phase::CloseDirectory => self.directory_fd = None,
            Phase::CloseParent => {
                self.parent = None;
                return Self::end(self.failed);
            }
            Phase::Clear | Phase::Directory | Phase::Discard | Phase::Rename => {}
        }
        if self.failed {
            return self.drain(settings);
        }
        self.phase = match self.phase {
            Phase::Parent => Phase::Directory,
            Phase::Directory => Phase::OpenDirectory,
            Phase::OpenDirectory => {
                if self.marker || self.content.is_none() {
                    Phase::Clear
                } else {
                    Phase::Discard
                }
            }
            Phase::Clear if self.content.is_none() => Phase::CloseDirectory,
            Phase::Clear => Phase::Discard,
            Phase::Discard => Phase::Open,
            Phase::Open => Phase::Write,
            Phase::Write => {
                if usize::try_from(self.written).expect("file size")
                    == self.content.as_ref().expect("write bytes").len()
                {
                    Phase::Close
                } else {
                    Phase::Write
                }
            }
            Phase::Close => Phase::Rename,
            Phase::Rename | Phase::Cleanup => Phase::CloseDirectory,
            Phase::CloseDirectory => Phase::CloseParent,
            Phase::CloseParent => unreachable!("last close settled above"),
        };
        self.first(settings)
    }
    fn drain(&mut self, settings: &Settings) -> Action {
        self.phase = if self.file.is_some() {
            Phase::Close
        } else if self.temporary {
            Phase::Cleanup
        } else if self.directory_fd.is_some() {
            Phase::CloseDirectory
        } else if self.parent.is_some() {
            Phase::CloseParent
        } else {
            return Self::end(true);
        };
        self.first(settings)
    }
    fn end(failed: bool) -> Action {
        Action::Operation(Next::End(if failed {
            git::Done::Failed { fault: git::Fault::Broken }
        } else {
            git::Done::Succeeded
        }))
    }
}
