//! The run's disposable git view (temper/docs/design/protocol/git-operations.md, 1.13).
use super::{Action, Directories, count, fd, mkdir, open};
use crate::{
    Limits, Settings,
    invocation::{self, Format},
    sequence::Next,
};
use alloc::boxed::Box;
use skein_io::kernel::{self, Error, Fd, Op, OpenHow};
use temper_worker_domain_checkout::git;

const NAMES: [&[u8]; 8] = [
    b"HEAD",
    b"index",
    b"MERGE_HEAD",
    b"MERGE_MSG",
    b"MERGE_MODE",
    b"refs/temper/fetched",
    b"refs/temper/landing",
    b"objects/info/alternates",
];
const TEMP: &[u8] = b"temper-view.tmp";

#[derive(Debug)]
pub(crate) struct View {
    place: git::Place,
    done: Box<git::Done>,
    format: Option<Format>,
    retry: bool,
    phase: Phase,
    git: Option<Fd>,
    parent: Option<Fd>,
    directory: Option<Fd>,
    source: Option<Fd>,
    target: Option<Fd>,
    index: usize,
    offset: u64,
    content: Box<[u8]>,
    written: u32,
    ending: Ending,
}
#[derive(Clone, Copy, Debug)]
enum Phase {
    Git,
    Parent,
    OpenParent,
    Directory,
    OpenDirectory,
    Source,
    Remove,
    Discard,
    Target,
    Read,
    Write,
    CloseSource,
    CloseTarget,
    Rename,
    CloseDirectory,
    CloseParent,
    DrainSource,
    DrainTarget,
    DrainDirectory,
    DrainParent,
    DrainGit,
    WorkTree,
    Aside,
    CloseWorkTree,
}
#[derive(Clone, Copy, Debug)]
enum Ending {
    Success,
    Repair,
    Broken,
}
impl View {
    pub fn new(place: &git::Place, done: Box<git::Done>, retry: bool, remake: bool, format: Option<Format>) -> Self {
        Self {
            place: place.clone(),
            done,
            format,
            retry,
            phase: if remake { Phase::WorkTree } else { Phase::Git },
            git: None,
            parent: None,
            directory: None,
            source: None,
            target: None,
            index: 0,
            offset: 0,
            content: Box::new([]),
            written: 0,
            ending: Ending::Success,
        }
    }
    fn name(&self) -> &'static [u8] {
        NAMES.get(self.index).copied().expect("view file")
    }
    fn leaf(&self) -> Box<[u8]> {
        Box::from(
            self.name()
                .strip_prefix(b"refs/temper/")
                .or(self.name().strip_prefix(b"objects/info/"))
                .unwrap_or(self.name()),
        )
    }
    fn parent_name(&self) -> &'static [u8] {
        if self.index == 7 { b"objects" } else { b"refs" }
    }
    fn directory_name(&self) -> &'static [u8] {
        if self.index == 7 { b"info" } else { b"temper" }
    }
    fn destination(&self) -> Fd {
        self.directory.or(self.git).expect("view directory")
    }
    pub fn first(&self, dirs: &mut Directories, settings: &Settings, limits: &Limits) -> Action {
        let dest = self.directory.or(self.git);
        Action::Run(match self.phase {
            Phase::Git => open(
                settings.root,
                invocation::joined(&[&invocation::place(&self.place), b"/.git"]),
                OpenHow::DirectoryNoFollow,
            ),
            Phase::Parent => mkdir(self.git.expect("git opened"), self.parent_name()),
            Phase::OpenParent => {
                open(self.git.expect("git opened"), Box::from(self.parent_name()), OpenHow::DirectoryNoFollow)
            }
            Phase::Directory => mkdir(self.parent.expect("parent opened"), self.directory_name()),
            Phase::OpenDirectory => {
                open(self.parent.expect("parent opened"), Box::from(self.directory_name()), OpenHow::DirectoryNoFollow)
            }
            Phase::Source => open(
                settings.root,
                invocation::joined(&[&invocation::git_dir(&self.place), b"/", self.name()]),
                OpenHow::ReadNoFollow,
            ),
            Phase::Remove => Op::Remove { dir: self.destination(), name: self.leaf(), directory: false },
            Phase::Discard => Op::Remove { dir: self.destination(), name: Box::from(TEMP), directory: false },
            Phase::Target => open(self.destination(), Box::from(TEMP), OpenHow::CreateNoFollow { mode: Some(0o600) }),
            Phase::Read => Op::Read {
                fd: self.source.expect("source opened"),
                buf: skein_lib::bytes::zeroed(usize::try_from(limits.read_bytes).expect("read bytes")),
                at: self.offset,
            },
            Phase::Write => Op::Write {
                fd: self.target.expect("temporary opened"),
                bytes: self.content.clone(),
                from: self.written,
                at: self.offset.checked_add(u64::from(self.written)).expect("file offset"),
            },
            Phase::CloseSource | Phase::DrainSource => Op::Close { fd: self.source.expect("source opened") },
            Phase::CloseTarget | Phase::DrainTarget => Op::Close { fd: self.target.expect("temporary opened") },
            Phase::Rename => Op::Rename {
                from_dir: self.destination(),
                from: Box::from(TEMP),
                to_dir: self.destination(),
                to: self.leaf(),
                no_replace: false,
            },
            Phase::CloseDirectory | Phase::DrainDirectory => {
                Op::Close { fd: self.directory.expect("directory opened") }
            }
            Phase::CloseParent | Phase::DrainParent => Op::Close { fd: self.parent.expect("parent opened") },
            Phase::DrainGit => Op::Close { fd: dest.expect("git opened") },
            Phase::WorkTree => open(settings.root, invocation::place(&self.place), OpenHow::DirectoryNoFollow),
            Phase::Aside => dirs.rename(self.parent.expect("work tree opened"), Box::from(b".git".as_slice())),
            Phase::CloseWorkTree => Op::Close { fd: self.parent.expect("work tree opened") },
        })
    }
    fn fail(&mut self, worker: bool, stop: bool) {
        self.ending =
            if worker || self.retry || stop || matches_broken(self.ending) { Ending::Broken } else { Ending::Repair };
    }
    pub fn complete(
        &mut self,
        complete: kernel::Complete,
        dirs: &mut Directories,
        settings: &Settings,
        limits: &Limits,
        stop: bool,
    ) -> Action {
        let result = complete.result;
        let allowed = self.phase.allowed(&result);
        let worker = self.phase.worker();
        if stop || (result.is_err() && !allowed) {
            self.fail(worker, stop);
        }
        match self.phase {
            Phase::Git => self.git = fd(result).ok(),
            Phase::OpenParent | Phase::WorkTree => self.parent = fd(result).ok(),
            Phase::OpenDirectory => self.directory = fd(result).ok(),
            Phase::Source => self.source = fd(result).ok(),
            Phase::Target => self.target = fd(result).ok(),
            Phase::CloseSource | Phase::DrainSource => self.source = None,
            Phase::CloseTarget | Phase::DrainTarget => self.target = None,
            Phase::CloseDirectory | Phase::DrainDirectory => self.directory = None,
            Phase::CloseParent | Phase::DrainParent | Phase::CloseWorkTree => self.parent = None,
            Phase::DrainGit => self.git = None,
            Phase::Read => {
                if let Ok(n) = count(result) {
                    self.content = read(complete.kind, n);
                    self.written = 0;
                }
            }
            Phase::Write => match count(result) {
                Ok(0) => self.fail(false, stop),
                Ok(n) => self.written = self.written.checked_add(n).expect("chunk size"),
                Err(_) => {}
            },
            Phase::Parent | Phase::Directory | Phase::Remove | Phase::Discard | Phase::Rename | Phase::Aside => {}
        }
        match self.ending {
            Ending::Repair | Ending::Broken => return self.drain(dirs, settings, limits),
            Ending::Success => {}
        }
        self.advance(dirs, settings, limits)
    }
    fn advance(&mut self, dirs: &mut Directories, settings: &Settings, limits: &Limits) -> Action {
        self.phase = match self.phase {
            Phase::Parent => Phase::OpenParent,
            Phase::OpenParent => Phase::Directory,
            Phase::Directory => Phase::OpenDirectory,
            Phase::OpenDirectory if self.index == 7 => {
                self.content =
                    invocation::joined(&[b"../../../../../", &invocation::git_dir(&self.place), b"/objects\n"]);
                Phase::Discard
            }
            Phase::Git | Phase::OpenDirectory => Phase::Source,
            Phase::Source if self.source.is_none() => Phase::Remove,
            Phase::Source => Phase::Discard,
            Phase::Discard => Phase::Target,
            Phase::Target if self.index == 7 => Phase::Write,
            Phase::Target => Phase::Read,
            Phase::Read if self.content.is_empty() => Phase::CloseSource,
            Phase::Read => Phase::Write,
            Phase::Write if usize::try_from(self.written).expect("chunk size") < self.content.len() => Phase::Write,
            Phase::Write if self.index == 7 => Phase::CloseTarget,
            Phase::Write => {
                self.offset = self.offset.checked_add(u64::from(self.written)).expect("file offset");
                Phase::Read
            }
            Phase::CloseSource => Phase::CloseTarget,
            Phase::CloseTarget => Phase::Rename,
            Phase::Remove | Phase::Rename => return self.next(dirs, settings, limits),
            Phase::CloseDirectory => Phase::CloseParent,
            Phase::CloseParent => Phase::Parent,
            Phase::DrainSource | Phase::DrainTarget | Phase::DrainDirectory | Phase::DrainParent | Phase::DrainGit => {
                return self.drain(dirs, settings, limits);
            }
            Phase::WorkTree if self.parent.is_none() => return self.init(),
            Phase::WorkTree => Phase::Aside,
            Phase::Aside => Phase::CloseWorkTree,
            Phase::CloseWorkTree => return self.init(),
        };
        self.first(dirs, settings, limits)
    }
    fn next(&mut self, dirs: &mut Directories, settings: &Settings, limits: &Limits) -> Action {
        self.index = self.index.checked_add(1).expect("view files");
        self.offset = 0;
        self.written = 0;
        self.content = Box::new([]);
        self.phase = match self.index {
            5 => Phase::Parent,
            7 => Phase::CloseDirectory,
            8 => return self.drain(dirs, settings, limits),
            0..=4 | 6 => Phase::Source,
            9.. => unreachable!("fixed view list"),
        };
        self.first(dirs, settings, limits)
    }
    fn init(&self) -> Action {
        Action::Operation(Next::InitView { done: self.done.clone(), format: self.format })
    }
    fn drain(&mut self, dirs: &mut Directories, settings: &Settings, limits: &Limits) -> Action {
        self.phase = if self.source.is_some() {
            Phase::DrainSource
        } else if self.target.is_some() {
            Phase::DrainTarget
        } else if self.directory.is_some() {
            Phase::DrainDirectory
        } else if self.parent.is_some() {
            Phase::DrainParent
        } else if self.git.is_some() {
            Phase::DrainGit
        } else {
            return match self.ending {
                Ending::Success => Action::Operation(Next::End(*self.done.clone())),
                Ending::Repair => {
                    self.retry = true;
                    self.ending = Ending::Success;
                    self.phase = Phase::WorkTree;
                    self.first(dirs, settings, limits)
                }
                Ending::Broken => Action::Operation(crate::sequence::failed(git::Fault::Broken, b"", 0)),
            };
        };
        self.first(dirs, settings, limits)
    }
}

fn matches_broken(ending: Ending) -> bool {
    match ending {
        Ending::Broken => true,
        Ending::Success | Ending::Repair => false,
    }
}

impl Phase {
    fn allowed(self, result: &Result<kernel::Done, Error>) -> bool {
        match self {
            Phase::Parent | Phase::Directory => result == &Err(Error::Exists),
            Phase::Source | Phase::Remove | Phase::Discard | Phase::Aside | Phase::WorkTree => {
                result == &Err(Error::NotFound)
            }
            Phase::Git
            | Phase::OpenParent
            | Phase::OpenDirectory
            | Phase::Target
            | Phase::Read
            | Phase::Write
            | Phase::CloseSource
            | Phase::CloseTarget
            | Phase::Rename
            | Phase::CloseDirectory
            | Phase::CloseParent
            | Phase::DrainSource
            | Phase::DrainTarget
            | Phase::DrainDirectory
            | Phase::DrainParent
            | Phase::DrainGit
            | Phase::CloseWorkTree => false,
        }
    }
    fn worker(self) -> bool {
        match self {
            Phase::Source | Phase::Read | Phase::CloseSource | Phase::DrainSource => true,
            Phase::Git
            | Phase::Parent
            | Phase::OpenParent
            | Phase::Directory
            | Phase::OpenDirectory
            | Phase::Remove
            | Phase::Discard
            | Phase::Target
            | Phase::Write
            | Phase::CloseTarget
            | Phase::Rename
            | Phase::CloseDirectory
            | Phase::CloseParent
            | Phase::DrainTarget
            | Phase::DrainDirectory
            | Phase::DrainParent
            | Phase::DrainGit
            | Phase::WorkTree
            | Phase::Aside
            | Phase::CloseWorkTree => false,
        }
    }
}

fn read(op: Op, count: u32) -> Box<[u8]> {
    match op {
        Op::Read { buf, .. } => {
            Box::from(buf.get(..usize::try_from(count).expect("read count")).expect("read capacity"))
        }
        Op::Open { .. }
        | Op::Write { .. }
        | Op::Close { .. }
        | Op::Rename { .. }
        | Op::Remove { .. }
        | Op::MakeDirectory { .. }
        | Op::Socket { .. }
        | Op::Bind { .. }
        | Op::Listen { .. }
        | Op::Accept { .. }
        | Op::Connect { .. }
        | Op::Recv { .. }
        | Op::Send { .. }
        | Op::Shutdown { .. }
        | Op::Append { .. }
        | Op::Truncate { .. }
        | Op::Sync { .. }
        | Op::Lock { .. }
        | Op::Stat { .. }
        | Op::HardLink { .. }
        | Op::List { .. }
        | Op::Spawn { .. }
        | Op::Wait { .. }
        | Op::Signal { .. }
        | Op::Usage
        | Op::Window { .. }
        | Op::ReadSignal { .. }
        | Op::PipeRead { .. }
        | Op::PipeWrite { .. }
        | Op::Cancel { .. } => unreachable!("read completion"),
    }
}
