//! A cut run's private mark and the start's selective sweep (worker/checkouts.md).
use super::{Action, Directories, count, fd, page, read_buffer};
use crate::{Settings, invocation, sequence::Next};
use alloc::boxed::Box;
use skein_io::kernel::{self, Error, Fd, Op, OpenHow};
use skein_lib::{Decimal, Intake, List, Reader, Token, Writer, bytes, stream};
use temper_worker_domain_checkout::{Limits, git};

const VERSION: &[u8] = b"temper-kept 1\n";

fn put(writer: &mut Writer, value: &[u8]) {
    writer.put(value).expect("measured mark");
}
fn field(writer: &mut Writer, value: &[u8]) {
    put(writer, &u32::try_from(value.len()).expect("bounded field").to_be_bytes());
    put(writer, value);
}
fn encode(mark: &git::Kept) -> Box<[u8]> {
    let mut size = VERSION.len().checked_add(16).expect("mark header").checked_add(mark.key.len()).expect("key size");
    for repository in &mark.repositories {
        size = size
            .checked_add(40)
            .expect("mark header")
            .checked_add(repository.name.len())
            .expect("mark name")
            .checked_add(repository.remote.len())
            .expect("bounded mark");
    }
    let mut writer = Writer::new(size);
    put(&mut writer, VERSION);
    put(&mut writer, &mark.when.to_be_bytes());
    field(&mut writer, &mark.key);
    put(&mut writer, &u32::try_from(mark.repositories.len()).expect("repository count").to_be_bytes());
    for repository in &mark.repositories {
        field(&mut writer, &repository.name);
        field(&mut writer, &repository.remote);
        put(&mut writer, &repository.commit.raw());
    }
    writer.finish()
}
fn read_field(input: &mut Reader<'_>, limit: u32) -> Option<Box<[u8]>> {
    let n = input.u32()?;
    if n == 0 || n > limit {
        return None;
    }
    Some(Box::from(input.bytes(n)?))
}
fn decode(workspace: Token, input: &[u8], limits: &Limits) -> Option<git::Kept> {
    workspace.raw().checked_add(1_u64 << 32)?;
    let mut input = Reader::new(input);
    if input.bytes(u32::try_from(VERSION.len()).expect("short header"))? != VERSION {
        return None;
    }
    let when = input.u64()?;
    let key = read_field(&mut input, limits.name_bytes)?;
    let n = input.u32()?;
    if n == 0 || n > limits.repositories {
        return None;
    }
    let mut repositories = List::with_capacity(n);
    for _ in 0..n {
        let name = read_field(&mut input, limits.name_bytes)?;
        if name.as_ref() == b"." || name.as_ref() == b".." || name.contains(&b'/') || name.contains(&0) {
            return None;
        }
        let remote = read_field(&mut input, limits.name_bytes)?;
        for previous in repositories.as_slice() {
            let previous: &git::KeptRepository = previous;
            if previous.name == name {
                return None;
            }
        }
        let commit = git::Commit::new(input.bytes(32)?.try_into().ok()?);
        repositories.push(git::KeptRepository { name, remote, commit }).expect("bounded repositories");
    }
    if !input.is_empty() {
        return None;
    }
    Some(git::Kept { workspace, key, repositories: repositories.into_boxed(), when })
}
fn token(name: &[u8]) -> Option<Token> {
    let mut n = 0_u64;
    if name.is_empty() {
        return None;
    }
    for byte in name {
        if !byte.is_ascii_digit() {
            return None;
        }
        n = n.checked_mul(10)?.checked_add(u64::from(byte.checked_sub(b'0')?))?;
    }
    if Decimal::of(n).as_bytes() != name {
        return None;
    }
    Some(Token::new(n))
}

#[derive(Debug)]
pub(crate) struct File {
    name: Box<[u8]>,
    content: Option<Box<[u8]>>,
    phase: Write,
    opened: Option<Fd>,
    written: u32,
    failed: bool,
}
#[derive(Clone, Copy, Debug)]
enum Write {
    Remove,
    Discard,
    Open,
    Content,
    Sync,
    Close,
    Rename,
    Invalidate,
    Cleanup,
    End,
}
impl File {
    #[expect(clippy::manual_map, reason = "the step subset spells out optional payloads")]
    pub fn new(workspace: Token, mark: Option<&git::Kept>) -> Self {
        Self {
            name: Box::from(Decimal::of(workspace.raw()).as_bytes()),
            content: match mark {
                Some(mark) => Some(encode(mark)),
                None => None,
            },
            phase: Write::Remove,
            opened: None,
            written: 0,
            failed: false,
        }
    }
    fn temporary(&self) -> Box<[u8]> {
        invocation::joined(&[&self.name, b".new"])
    }
    pub fn first(&self, dirs: &Directories) -> Action {
        let dir = dirs.kept.expect("layout opened kept");
        Action::Run(match self.phase {
            Write::Remove | Write::Invalidate => Op::Remove { dir, name: self.name.clone(), directory: false },
            Write::Discard | Write::Cleanup => Op::Remove { dir, name: self.temporary(), directory: false },
            Write::Open => {
                Op::Open { root: dir, path: self.temporary(), how: OpenHow::CreateNoFollow { mode: Some(0o600) } }
            }
            Write::Content => Op::Write {
                fd: self.opened.expect("open mark"),
                bytes: self.content.clone().expect("mark content"),
                from: self.written,
                at: u64::from(self.written),
            },
            Write::Sync => Op::Sync { fd: self.opened.expect("open mark") },
            Write::Close => Op::Close { fd: self.opened.expect("open mark") },
            Write::Rename => Op::Rename {
                from_dir: dir,
                from: self.temporary(),
                to_dir: dir,
                to: self.name.clone(),
                no_replace: false,
            },
            Write::End => {
                return Action::Operation(Next::End(if self.failed {
                    git::Done::Failed { fault: git::Fault::Broken }
                } else {
                    git::Done::Succeeded
                }));
            }
        })
    }
    pub fn complete(&mut self, result: Result<kernel::Done, Error>, dirs: &Directories, stop: bool) -> Action {
        let missing_ok = match self.phase {
            Write::Remove | Write::Discard | Write::Cleanup => result == Err(Error::NotFound),
            Write::Open
            | Write::Content
            | Write::Sync
            | Write::Close
            | Write::Rename
            | Write::Invalidate
            | Write::End => false,
        };
        self.failed |= stop || (result.is_err() && !missing_ok);
        self.phase = match self.phase {
            Write::Remove => {
                if self.failed || self.content.is_none() {
                    Write::End
                } else {
                    Write::Discard
                }
            }
            Write::Discard => {
                if self.failed {
                    Write::End
                } else {
                    Write::Open
                }
            }
            Write::Open => {
                self.opened = fd(result).ok();
                if self.opened.is_some() {
                    if self.failed { Write::Close } else { Write::Content }
                } else {
                    Write::Cleanup
                }
            }
            Write::Content => {
                let n = count(result).unwrap_or(0);
                self.failed |= n == 0;
                self.written = self.written.checked_add(n).expect("bounded file");
                if self.failed {
                    Write::Close
                } else if usize::try_from(self.written).expect("bounded file")
                    == self.content.as_ref().expect("mark").len()
                {
                    Write::Sync
                } else {
                    Write::Content
                }
            }
            Write::Sync => Write::Close,
            Write::Close => {
                self.opened = None;
                if self.failed { Write::Cleanup } else { Write::Rename }
            }
            Write::Rename => {
                if self.failed {
                    Write::Invalidate
                } else {
                    Write::End
                }
            }
            Write::Invalidate => Write::Cleanup,
            Write::Cleanup => Write::End,
            Write::End => unreachable!("terminal mark"),
        };
        self.first(dirs)
    }
}

#[derive(Debug)]
pub(crate) struct Layout {
    phase: Read,
    directory: Option<Fd>,
    file: Option<Fd>,
    name: Box<[u8]>,
    content: Intake,
    limits: Limits,
    candidate: Option<git::Kept>,
    tree: u8,
}
#[derive(Clone, Copy, Debug)]
enum Read {
    Make,
    Open,
    List,
    File,
    Bytes,
    CloseFile,
    Workspace,
    CloseWorkspace,
    Discard,
    Trees,
    ListTree,
    Move,
    CloseTree,
    End,
}
impl Layout {
    pub fn new(limits: Limits) -> Self {
        let capacity = limits
            .repositories
            .checked_mul(
                limits.name_bytes.checked_mul(2).expect("name bound").checked_add(40).expect("repository bound"),
            )
            .expect("repository count")
            .checked_add(limits.name_bytes)
            .expect("key bound")
            .checked_add(64)
            .expect("mark bound");
        Self {
            phase: Read::Make,
            directory: None,
            file: None,
            name: Box::new([]),
            content: Intake::with_capacity(capacity),
            limits,
            candidate: None,
            tree: 0,
        }
    }
    pub fn first(&self, dirs: &mut Directories, settings: &Settings) -> Action {
        Action::Run(match self.phase {
            Read::Make => Op::MakeDirectory { dir: settings.root, name: Box::from(b"kept".as_slice()), mode: 0o700 },
            Read::Open => {
                Op::Open { root: settings.root, path: Box::from(b"kept".as_slice()), how: OpenHow::DirectoryNoFollow }
            }
            Read::List | Read::ListTree => Op::List {
                fd: self.directory.expect("listing directory"),
                entries: Box::new([kernel::Entry::BLANK]),
                names: bytes::zeroed(kernel::LONGEST_NAME),
            },
            Read::File => {
                Op::Open { root: dirs.kept.expect("kept open"), path: self.name.clone(), how: OpenHow::ReadNoFollow }
            }
            Read::Bytes => Op::Read {
                fd: self.file.expect("mark open"),
                buf: bytes::zeroed(usize::try_from(self.content.room().min(4096)).expect("bounded read")),
                at: u64::from(self.content.len()),
            },
            Read::CloseFile | Read::CloseWorkspace => Op::Close { fd: self.file.expect("file open") },
            Read::Workspace => Op::Open {
                root: dirs.workspaces.expect("workspaces open"),
                path: self.name.clone(),
                how: OpenHow::DirectoryNoFollow,
            },
            Read::Discard => {
                Op::Remove { dir: dirs.kept.expect("kept open"), name: self.name.clone(), directory: false }
            }
            Read::Trees => Op::Open {
                root: settings.root,
                path: Box::from(if self.tree == 0 { b"workspaces".as_slice() } else { b"gitdirs".as_slice() }),
                how: OpenHow::DirectoryNoFollow,
            },
            Read::Move => dirs.rename(self.directory.expect("tree open"), self.name.clone()),
            Read::CloseTree => Op::Close { fd: self.directory.expect("tree open") },
            Read::End => return Action::Layout(Ok(())),
        })
    }
    #[expect(clippy::too_many_lines, reason = "the startup table lists each bounded file step and its terminal")]
    pub fn complete(&mut self, complete: kernel::Complete, dirs: &mut Directories, settings: &Settings) -> Action {
        match self.phase {
            Read::Make => {
                if complete.result.is_err() && complete.result != Err(Error::Exists) {
                    return Action::Layout(Err(complete.result.expect_err("failed mkdir")));
                }
                self.phase = Read::Open;
            }
            Read::Open => {
                let opened = match fd(complete.result) {
                    Ok(fd) => fd,
                    Err(error) => return Action::Layout(Err(error)),
                };
                dirs.kept = Some(opened);
                self.directory = Some(opened);
                self.phase = Read::List;
            }
            Read::List => {
                self.name = match page(complete) {
                    Ok(name) => name,
                    Err(error) => return Action::Layout(Err(error)),
                };
                self.phase = if self.name.is_empty() {
                    Read::Trees
                } else if token(&self.name).is_some() && dirs.marks.len() < self.limits.workspaces {
                    Read::File
                } else {
                    Read::Discard
                };
            }
            Read::File => {
                self.file = fd(complete.result).ok();
                dirs.mark_file = self.file;
                self.phase = if self.file.is_some() { Read::Bytes } else { Read::Discard };
            }
            Read::Bytes => {
                let failed = complete.result.is_err();
                let n = count(complete.result).unwrap_or(0);
                let buffer = read_buffer(complete.kind);
                self.content
                    .append(buffer.get(..usize::try_from(n).expect("read count")).expect("read buffer"))
                    .expect("bounded read");
                if n == 0 || self.content.room() == 0 {
                    let n = self.content.len();
                    let input = self.content.meet(stream::Read::Fill(n)).expect("buffered mark");
                    self.candidate = if failed {
                        None
                    } else {
                        decode(token(&self.name).expect("canonical token"), &input, &self.limits)
                    };
                    self.phase = Read::CloseFile;
                }
            }
            Read::CloseFile => {
                if complete.result.is_err() {
                    self.candidate = None;
                }
                self.file = None;
                dirs.mark_file = None;
                self.phase = if self.candidate.is_some() { Read::Workspace } else { Read::Discard };
            }
            Read::Workspace => {
                self.file = fd(complete.result).ok();
                dirs.mark_file = self.file;
                self.phase = if self.file.is_some() { Read::CloseWorkspace } else { Read::Discard };
            }
            Read::CloseWorkspace => {
                self.file = None;
                dirs.mark_file = None;
                if let Err(error) = complete.result {
                    return Action::Layout(Err(error));
                }
                dirs.marks.push(self.candidate.take().expect("validated mark")).expect("workspace limit");
                self.phase = Read::List;
            }
            Read::Discard => {
                if let Err(error) = complete.result
                    && error != Error::NotFound
                {
                    return Action::Layout(Err(error));
                }
                self.candidate = None;
                self.phase = Read::List;
            }
            Read::Trees => {
                self.directory = match fd(complete.result) {
                    Ok(fd) => Some(fd),
                    Err(error) => return Action::Layout(Err(error)),
                };
                dirs.scanning = self.directory;
                self.phase = Read::ListTree;
            }
            Read::ListTree => {
                self.name = match page(complete) {
                    Ok(name) => name,
                    Err(error) => return Action::Layout(Err(error)),
                };
                self.phase = if self.name.is_empty() {
                    Read::CloseTree
                } else if dirs.keep || contains(&dirs.marks, &self.name) {
                    Read::ListTree
                } else {
                    Read::Move
                };
            }
            Read::Move => {
                if let Err(error) = complete.result {
                    return Action::Layout(Err(error));
                }
                self.phase = Read::ListTree;
            }
            Read::CloseTree => {
                self.directory = None;
                dirs.scanning = None;
                if let Err(error) = complete.result {
                    return Action::Layout(Err(error));
                }
                if self.tree == 0 {
                    self.tree = 1;
                    self.phase = Read::Trees;
                } else {
                    self.phase = Read::End;
                }
            }
            Read::End => unreachable!("terminal layout"),
        }
        self.first(dirs, settings)
    }
}
fn contains(marks: &List<git::Kept>, name: &[u8]) -> bool {
    let Some(token) = token(name) else {
        return false;
    };
    for mark in marks.as_slice() {
        if mark.workspace == token {
            return true;
        }
    }
    false
}
