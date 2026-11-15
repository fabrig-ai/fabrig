//! Single-flight payload I/O (skein/docs/design/data/kv.md, 6.2–6.4).

#![expect(
    clippy::disallowed_types,
    reason = "scripts hold one bounded batch or directory page; cached files are bounded by Limits::logs"
)]

use super::{Done, IO_OWNER, Pending, Step, Store};
use crate::{
    Event,
    frame::u32_at,
    logs::{self, Read},
};
use alloc::{boxed::Box, vec::Vec};
use core::mem;
use skein_io::{
    file,
    kernel::{Error, Fd},
};
use skein_lib::{Map, Queue, Token, Writer};

#[derive(Debug)]
struct Cached {
    file: Token,
    used: u64,
}
#[derive(Debug)]
pub(super) struct Cache {
    files: Map<u64, Cached>,
    clock: u64,
}
impl Cache {
    pub(super) fn new() -> Self {
        Self { files: Map::uncapped(), clock: 0 }
    }
    pub(super) fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
    fn get(&mut self, log: u64) -> Option<Token> {
        self.clock = self.clock.saturating_add(1);
        let entry = self.files.get_mut(&log)?;
        entry.used = self.clock;
        Some(entry.file)
    }
    fn insert(&mut self, log: u64, file: Token) {
        self.clock = self.clock.saturating_add(1);
        assert!(self.files.insert(log, Cached { file, used: self.clock }).is_ok(), "cache has a bounded file per log");
    }
    fn oldest(&mut self) -> Option<Token> {
        let mut oldest = None;
        for (&log, entry) in &self.files {
            if match oldest {
                Some((_, used)) => entry.used < used,
                None => true,
            } {
                oldest = Some((log, entry.used));
            }
        }
        let (log, _) = oldest?;
        Some(self.files.remove(&log).expect("oldest file is held").file)
    }
}

#[derive(Debug)]
struct Write {
    log: u64,
    at: u64,
    bytes: Box<[u8]>,
}
#[derive(Debug)]
enum Command {
    Ensure { path: Box<[u8]> },
    Append(Write),
    Directory { path: Box<[u8]> },
    Drop { log: u64 },
    Read(Read),
    Sweep,
    Close { file: Token },
}
#[derive(Debug)]
enum Io {
    Next,
    Root,
    Make,
    Evict { command: Command },
    OpenWrite { write: Write },
    Create { write: Write },
    CreatedClose { write: Write },
    Write { file: Token },
    Sync,
    Directory,
    DirectorySync,
    Close,
    CacheClose,
    DropOpen { log: u64 },
    Remove { directory: Token },
    ReadOpen { read: Read },
    ReadLength { read: Read, file: Token },
    ReadFrame { read: Read, file: Token, at: u64, length: u32 },
    SweepOpen,
    SweepPage { directory: Token },
    ShardOpen { name: Box<[u8]> },
    ShardPage { directory: Token, name: Box<[u8]> },
    SweepRemove { directory: Token, name: Box<[u8]> },
    ShardSync { directory: Token },
    ShardClose,
}
#[derive(Debug)]
pub(super) struct Task {
    commands: Queue<Command>,
    front: Option<Command>,
    io: Io,
    held: Vec<Token>,
    root_token: Option<Token>,
    failed: bool,
    pub(super) answer: Option<Event>,
    shards: Queue<Box<[u8]>>,
    entries: Queue<Box<[u8]>>,
    top: Option<Token>,
}
impl Task {
    fn new() -> Self {
        Self {
            commands: Queue::uncapped(),
            front: None,
            io: Io::Next,
            held: Vec::new(),
            root_token: None,
            failed: false,
            answer: None,
            shards: Queue::uncapped(),
            entries: Queue::uncapped(),
            top: None,
        }
    }
    pub(super) fn batch(batch: &Queue<Pending>) -> Self {
        let mut task = Self::new();
        let mut names = Vec::new();
        for pending in batch {
            for append in &pending.appends {
                if !names.contains(&append.log) {
                    names.push(append.log);
                }
            }
        }
        if !names.is_empty() {
            task.commands.push(Command::Ensure { path: Box::from(&b"logs"[..]) });
        }
        for log in names {
            let mut at = None;
            let mut size = 0_usize;
            for pending in batch {
                for append in &pending.appends {
                    if append.log == log {
                        if at.is_none() {
                            at = Some(append.at);
                        }
                        size = size.checked_add(append.frame.bytes.len()).expect("batch bytes fit usize");
                    }
                }
            }
            let mut bytes = Writer::new(size);
            for pending in batch {
                for append in &pending.appends {
                    if append.log == log {
                        bytes.put(&append.frame.bytes).expect("sized log batch");
                    }
                }
            }
            task.commands.push(Command::Ensure { path: logs::directory(log) });
            task.commands.push(Command::Append(Write {
                log,
                at: at.expect("log has an append"),
                bytes: bytes.finish(),
            }));
            task.commands.push(Command::Directory { path: logs::directory(log) });
        }
        // A surviving, but unsynced directory from an earlier failed batch is also announced.
        if !task.commands.is_empty() {
            task.commands.push(Command::Directory { path: Box::from(&b"logs"[..]) });
            task.commands.push(Command::Directory { path: Box::from(&b"."[..]) });
        }
        task
    }
    pub(super) fn drops(mut drops: Queue<u64>) -> Self {
        let mut task = Self::new();
        for _ in 0..drops.len() {
            let log = drops.pop().expect("initial drop count");
            task.commands.push(Command::Drop { log });
        }
        task
    }
    pub(super) fn read(read: Read) -> Self {
        let mut task = Self::new();
        task.answer = Some(read.unread());
        task.commands.push(Command::Read(read));
        task
    }
    pub(super) fn sweep() -> Self {
        let mut task = Self::new();
        task.commands.push(Command::Sweep);
        task
    }
    pub(super) fn close(cache: &mut Cache) -> Self {
        let mut task = Self::new();
        for _ in 0..cache.files.len() {
            let file = cache.oldest().expect("cache holds its initial length");
            task.commands.push(Command::Close { file });
        }
        task
    }

    /// Advance until exactly one file request is issued, or all work is finished.
    pub(super) fn advance(&mut self, store: &mut Store, root: Fd, step: &mut Step<'_>) -> bool {
        let command = match self.front.take() {
            Some(command) => Some(command),
            None => self.commands.pop(),
        };
        if let Some(command) = command {
            self.issue(command, store, root, step);
            true
        } else if let Some(file) = self.held.pop() {
            step.below.push(file::Request::Close { owner: IO_OWNER, file });
            self.io = Io::Close;
            true
        } else {
            false
        }
    }
    fn issue(&mut self, command: Command, store: &mut Store, root: Fd, step: &mut Step<'_>) {
        self.io = match command {
            Command::Ensure { path } => match self.root_token {
                Some(token) => {
                    step.below.push(file::Request::MakeDirectory { owner: IO_OWNER, root: token, path, mode: 0o700 });
                    Io::Make
                }
                None => {
                    self.front = Some(Command::Ensure { path });
                    open_directory(root, Box::from(&b"."[..]), step);
                    Io::Root
                }
            },
            Command::Append(write) => match store.cache.get(write.log) {
                Some(file) => Self::write(write, file, step),
                None => Self::open_write(write, store, root, step),
            },
            Command::Directory { path } => {
                open_directory(root, path, step);
                Io::Directory
            }
            Command::Drop { log } => {
                if let Some(entry) = store.cache.files.remove(&log) {
                    step.below.push(file::Request::Close { owner: IO_OWNER, file: entry.file });
                    self.front = Some(Command::Drop { log });
                    Io::Close
                } else {
                    open_directory(root, logs::directory(log), step);
                    Io::DropOpen { log }
                }
            }
            Command::Read(read) => match store.cache.get(read.log) {
                Some(file) => Self::read_length(read, file, step),
                None => {
                    if store.cache.files.len() >= step.env.limits.logs {
                        Self::evict(Command::Read(read), store, step)
                    } else {
                        step.below.push(file::Request::OpenWriteNoFollow {
                            owner: IO_OWNER,
                            root,
                            name: logs::path(read.log),
                        });
                        Io::ReadOpen { read }
                    }
                }
            },
            Command::Sweep => {
                open_directory(root, Box::from(&b"logs"[..]), step);
                Io::SweepOpen
            }
            Command::Close { file } => {
                step.below.push(file::Request::Close { owner: IO_OWNER, file });
                Io::CacheClose
            }
        };
    }
    fn evict(command: Command, store: &mut Store, step: &mut Step<'_>) -> Io {
        let file = store.cache.oldest().expect("a full nonzero cache holds a file");
        step.below.push(file::Request::Close { owner: IO_OWNER, file });
        Io::Evict { command }
    }
    fn open_write(write: Write, store: &mut Store, root: Fd, step: &mut Step<'_>) -> Io {
        if store.cache.files.len() >= step.env.limits.logs {
            Self::evict(Command::Append(write), store, step)
        } else {
            step.below.push(file::Request::OpenWriteNoFollow { owner: IO_OWNER, root, name: logs::path(write.log) });
            Io::OpenWrite { write }
        }
    }
    fn write(write: Write, file: Token, step: &mut Step<'_>) -> Io {
        step.below.push(file::Request::WriteAt { owner: IO_OWNER, file, offset: write.at, bytes: write.bytes });
        Io::Write { file }
    }
    fn read_length(read: Read, file: Token, step: &mut Step<'_>) -> Io {
        let offset = read.header().expect("read header is checked against its captured extent");
        step.below.push(file::Request::ReadAt { owner: IO_OWNER, file, offset, max: 4 });
        Io::ReadLength { read, file }
    }
    fn fail(&mut self) {
        self.failed = true;
        self.commands.clear();
        self.front = None;
    }
    pub(super) fn failed(&self) -> bool {
        self.failed
    }

    #[expect(
        clippy::too_many_lines,
        clippy::manual_map,
        reason = "each I/O phase names its terminal event and continuation without closures"
    )]
    pub(super) fn finished(&mut self, store: &mut Store, root: Fd, event: Done, step: &mut Step<'_>) {
        let phase = mem::replace(&mut self.io, Io::Next);
        match phase {
            Io::Next => unreachable!("an issued request has an I/O phase"),
            Io::Root => match event {
                Done::Opened { file, .. } => {
                    self.root_token = Some(file);
                    self.held.push(file);
                }
                Done::Failed { .. } => self.fail(),
                Done::Listed { .. } | Done::Read { .. } | Done::Finished => super::wrong_answer(),
            },
            Io::Make => match event {
                Done::Finished | Done::Failed { error: Error::Exists } => {}
                Done::Failed { .. } => self.fail(),
                Done::Listed { .. } | Done::Read { .. } | Done::Opened { .. } => super::wrong_answer(),
            },
            Io::Evict { command } => match event {
                Done::Finished => self.front = Some(command),
                Done::Failed { .. } => self.fail(),
                Done::Listed { .. } | Done::Read { .. } | Done::Opened { .. } => super::wrong_answer(),
            },
            Io::OpenWrite { write } => match event {
                Done::Opened { file, .. } => {
                    store.cache.insert(write.log, file);
                    self.io = Self::write(write, file, step);
                }
                Done::Failed { error: Error::NotFound } => {
                    step.below.push(file::Request::CreateNoFollow {
                        owner: IO_OWNER,
                        root,
                        name: logs::path(write.log),
                        mode: 0o600,
                    });
                    self.io = Io::Create { write };
                }
                Done::Failed { .. } => self.fail(),
                Done::Listed { .. } | Done::Read { .. } | Done::Finished => super::wrong_answer(),
            },
            Io::Create { write } => match event {
                Done::Opened { file, .. } => {
                    step.below.push(file::Request::Close { owner: IO_OWNER, file });
                    self.io = Io::CreatedClose { write };
                }
                Done::Failed { .. } => self.fail(),
                Done::Listed { .. } | Done::Read { .. } | Done::Finished => super::wrong_answer(),
            },
            Io::CreatedClose { write } => match event {
                Done::Finished => {
                    self.io = Self::open_write(write, store, root, step);
                }
                Done::Failed { .. } => self.fail(),
                Done::Listed { .. } | Done::Read { .. } | Done::Opened { .. } => super::wrong_answer(),
            },
            Io::Write { file } => match event {
                Done::Finished => {
                    step.below.push(file::Request::Sync { owner: IO_OWNER, file });
                    self.io = Io::Sync;
                }
                Done::Failed { .. } => self.fail(),
                Done::Listed { .. } | Done::Read { .. } | Done::Opened { .. } => super::wrong_answer(),
            },
            Io::Sync => match event {
                Done::Finished => {}
                Done::Failed { .. } => self.fail(),
                Done::Listed { .. } | Done::Read { .. } | Done::Opened { .. } => super::wrong_answer(),
            },
            Io::DirectorySync => match event {
                Done::Finished => {
                    let file = self.held.pop().expect("directory sync holds its handle");
                    step.below.push(file::Request::Close { owner: IO_OWNER, file });
                    self.io = Io::Close;
                }
                Done::Failed { .. } => self.fail(),
                Done::Listed { .. } | Done::Read { .. } | Done::Opened { .. } => super::wrong_answer(),
            },
            Io::Directory => match event {
                Done::Opened { file, .. } => {
                    self.held.push(file);
                    step.below.push(file::Request::Sync { owner: IO_OWNER, file });
                    self.io = Io::DirectorySync;
                }
                Done::Failed { error: Error::NotFound } => {}
                Done::Failed { .. } => self.fail(),
                Done::Listed { .. } | Done::Read { .. } | Done::Finished => super::wrong_answer(),
            },
            Io::CacheClose => match event {
                Done::Finished => {}
                Done::Failed { .. } => self.failed = true,
                Done::Listed { .. } | Done::Read { .. } | Done::Opened { .. } => super::wrong_answer(),
            },
            Io::Close => match event {
                Done::Finished => {}
                Done::Failed { .. } => self.fail(),
                Done::Listed { .. } | Done::Read { .. } | Done::Opened { .. } => super::wrong_answer(),
            },
            Io::DropOpen { log } => match event {
                Done::Opened { file, .. } => {
                    self.held.push(file);
                    step.below.push(file::Request::RemoveIn {
                        owner: IO_OWNER,
                        root: file,
                        name: logs::name(log),
                        directory: false,
                    });
                    self.io = Io::Remove { directory: file };
                }
                Done::Failed { error: Error::NotFound } => {}
                Done::Failed { .. } => self.fail(),
                Done::Listed { .. } | Done::Read { .. } | Done::Finished => super::wrong_answer(),
            },
            Io::Remove { directory } => match event {
                Done::Finished | Done::Failed { error: Error::NotFound } => {
                    step.below.push(file::Request::Sync { owner: IO_OWNER, file: directory });
                    self.io = Io::DirectorySync;
                }
                Done::Failed { .. } => self.fail(),
                Done::Listed { .. } | Done::Read { .. } | Done::Opened { .. } => super::wrong_answer(),
            },
            Io::ReadOpen { read } => match event {
                Done::Opened { file, .. } => {
                    store.cache.insert(read.log, file);
                    self.io = Self::read_length(read, file, step);
                }
                Done::Failed { .. } => self.fail(),
                Done::Listed { .. } | Done::Read { .. } | Done::Finished => super::wrong_answer(),
            },
            Io::ReadLength { read, file } => match event {
                Done::Read { bytes } => {
                    let span = match u32_at(&bytes, 0) {
                        Some(length) => match read.span(length) {
                            Some(span) => Some((length, span)),
                            None => None,
                        },
                        None => None,
                    };
                    if let Some((length, (at, _end))) = span {
                        if length > step.env.limits.record.saturating_add(17) {
                            self.fail();
                            return;
                        }
                        self.answer = Some(Event::LogUnread { owner: read.owner, at });
                        step.below.push(file::Request::ReadAt { owner: IO_OWNER, file, offset: at, max: length });
                        self.io = Io::ReadFrame { read, file, at, length };
                    } else {
                        self.fail();
                    }
                }
                Done::Failed { .. } => self.fail(),
                Done::Listed { .. } | Done::Opened { .. } | Done::Finished => super::wrong_answer(),
            },
            Io::ReadFrame { mut read, file, at, length } => match event {
                Done::Read { bytes } => {
                    if bytes.len() != skein_lib::usize_of(length) || !read.accept(&bytes, step.env.limits.record) {
                        self.answer = Some(Event::LogUnread { owner: read.owner, at });
                        self.fail();
                    } else if read.finished() {
                        self.answer = Some(read.answer());
                    } else if read.header().is_some() {
                        self.answer = Some(read.unread());
                        self.io = Self::read_length(read, file, step);
                    } else {
                        self.answer = Some(read.unread());
                        self.fail();
                    }
                }
                Done::Failed { .. } => self.fail(),
                Done::Listed { .. } | Done::Opened { .. } | Done::Finished => super::wrong_answer(),
            },
            Io::SweepOpen => match event {
                Done::Opened { file, .. } => {
                    self.held.push(file);
                    self.top = Some(file);
                    self.page(file, Io::SweepPage { directory: file }, step);
                }
                Done::Failed { error: Error::NotFound } => {}
                Done::Failed { .. } => self.fail(),
                Done::Listed { .. } | Done::Read { .. } | Done::Finished => super::wrong_answer(),
            },
            Io::SweepPage { directory } => match event {
                Done::Listed { entries } => {
                    if entries.is_empty() {
                        return;
                    }
                    for entry in entries {
                        if logs::number(&entry.name).is_some() {
                            self.shards.push(entry.name);
                        }
                    }
                    self.next_shard(root, directory, step);
                }
                Done::Failed { .. } => self.fail(),
                Done::Opened { .. } | Done::Read { .. } | Done::Finished => super::wrong_answer(),
            },
            Io::ShardOpen { name } => match event {
                Done::Opened { file, .. } => {
                    self.held.push(file);
                    self.page(file, Io::ShardPage { directory: file, name }, step);
                }
                Done::Failed { .. } => self.fail(),
                Done::Listed { .. } | Done::Read { .. } | Done::Finished => super::wrong_answer(),
            },
            Io::ShardPage { directory, name } => match event {
                Done::Listed { entries } => {
                    if entries.is_empty() {
                        step.below.push(file::Request::Sync { owner: IO_OWNER, file: directory });
                        self.io = Io::ShardSync { directory };
                    } else {
                        for entry in entries {
                            if let Some(log) = logs::number(&entry.name)
                                && !store.logs.exists(&store.map, log)
                            {
                                self.entries.push(entry.name);
                            }
                        }
                        self.next_removal(directory, name, step);
                    }
                }
                Done::Failed { .. } => self.fail(),
                Done::Opened { .. } | Done::Read { .. } | Done::Finished => super::wrong_answer(),
            },
            Io::SweepRemove { directory, name } => match event {
                Done::Finished | Done::Failed { error: Error::NotFound } => self.next_removal(directory, name, step),
                Done::Failed { .. } => self.fail(),
                Done::Listed { .. } | Done::Read { .. } | Done::Opened { .. } => super::wrong_answer(),
            },
            Io::ShardClose => match event {
                Done::Finished => self.next_shard(root, self.top.expect("sweep holds its top directory"), step),
                Done::Failed { .. } => self.fail(),
                Done::Listed { .. } | Done::Read { .. } | Done::Opened { .. } => super::wrong_answer(),
            },
            Io::ShardSync { directory } => match event {
                Done::Finished => {
                    // Close this shard before opening another; the top directory stays for its next page.
                    assert!(self.held.pop() == Some(directory), "shard is the last temporary handle");
                    step.below.push(file::Request::Close { owner: IO_OWNER, file: directory });
                    self.io = Io::ShardClose;
                }
                Done::Failed { .. } => self.fail(),
                Done::Listed { .. } | Done::Read { .. } | Done::Opened { .. } => super::wrong_answer(),
            },
        }
    }
    fn page(&mut self, directory: Token, phase: Io, step: &mut Step<'_>) {
        step.below.push(file::Request::ListPage { owner: IO_OWNER, directory, max: 32 });
        self.io = phase;
    }
    fn next_shard(&mut self, root: Fd, directory: Token, step: &mut Step<'_>) {
        if let Some(shard) = self.shards.pop() {
            let mut path = Writer::new(21);
            path.put(b"logs/").expect("sized shard");
            path.put(&shard).expect("sized shard");
            let name = path.finish();
            open_directory(root, name.clone(), step);
            self.io = Io::ShardOpen { name };
        } else {
            self.page(directory, Io::SweepPage { directory }, step);
        }
    }
    fn next_removal(&mut self, directory: Token, name: Box<[u8]>, step: &mut Step<'_>) {
        if let Some(path) = self.entries.pop() {
            step.below.push(file::Request::RemoveIn { owner: IO_OWNER, root: directory, name: path, directory: false });
            self.io = Io::SweepRemove { directory, name };
        } else {
            self.page(directory, Io::ShardPage { directory, name }, step);
        }
    }
    pub(super) fn outstanding(&self) -> bool {
        match &self.io {
            Io::Next => false,
            Io::Root
            | Io::Make
            | Io::Evict { .. }
            | Io::OpenWrite { .. }
            | Io::Create { .. }
            | Io::Write { .. }
            | Io::CreatedClose { .. }
            | Io::Sync
            | Io::Directory
            | Io::DirectorySync
            | Io::Close
            | Io::CacheClose
            | Io::DropOpen { .. }
            | Io::Remove { .. }
            | Io::ReadOpen { .. }
            | Io::ReadLength { .. }
            | Io::ReadFrame { .. }
            | Io::SweepOpen
            | Io::SweepPage { .. }
            | Io::ShardOpen { .. }
            | Io::ShardPage { .. }
            | Io::SweepRemove { .. }
            | Io::ShardSync { .. }
            | Io::ShardClose => true,
        }
    }
}
fn open_directory(root: Fd, name: Box<[u8]>, step: &mut Step<'_>) {
    step.below.push(file::Request::OpenDirectory { owner: IO_OWNER, root, name, no_follow: true });
}
