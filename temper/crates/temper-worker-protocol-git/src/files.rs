//! Kernel steps beneath the worker's root, with one submission per machine (protocol/git-operations.md, 1.1).
use crate::{Limits, Settings, invocation, output::Markers, sequence::Next};
use alloc::boxed::Box;
use skein_io::kernel::{self, Error, Fd, Op, OpenHow};
use skein_lib::{Decimal, List, Token, bytes};
use temper_worker_domain_checkout::git;
mod index;
mod kept;
mod objects;
mod reds;
mod results;
mod sweep;
mod view;
use objects::{Alternates, Look, Make, Store};
pub(crate) use sweep::SetAside;
use sweep::Sweep;

#[derive(Debug)]
pub(crate) struct Directories {
    pub keep: bool,
    pub kept: Option<Fd>,
    pub scanning: Option<Fd>,
    pub mark_file: Option<Fd>,
    pub marks: List<git::Kept>,
    pub trash: Option<Fd>,
    pub workspaces: Option<Fd>,
    pub gitdirs: Option<Fd>,
    pub stores: Option<Fd>,
    pub making: Option<Fd>,
    pub serial: u64,
}
impl Directories {
    pub fn new(serial: u64) -> Directories {
        Directories {
            keep: false,
            kept: None,
            scanning: None,
            mark_file: None,
            marks: List::uncapped(),
            trash: None,
            workspaces: None,
            gitdirs: None,
            stores: None,
            making: None,
            serial,
        }
    }
    fn rename(&mut self, from_dir: Fd, from: Box<[u8]>) -> Op {
        let to = Box::from(Decimal::of(self.serial).as_bytes());
        self.serial = self.serial.checked_add(1).expect("trash serial lifetime");
        Op::Rename { from_dir, from, to_dir: self.trash.expect("trash opened"), to, no_replace: true }
    }
}
#[derive(Debug)]
pub(crate) struct Task {
    /// The sequence's file task (temper/docs/design/protocol/git.md, section 6).
    pub word: Option<&'static [u8]>,
    pub pending: Token,
    pub phase: Phase,
}
#[derive(Debug)]
pub(crate) enum Phase {
    Reds(reds::Files),
    Results(results::File),
    Layout(Layout, Option<kept::Layout>),
    KeptLayout(kept::Layout),
    Keep(kept::File),
    Make(Make),
    Look(Look),
    Store(Store),
    Alternates(Alternates),
    View(view::View),
    Scan(Scan),
    Index(index::File),
    Sweep(Sweep),
    Closing,
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum Layout {
    Trash,
    OpenTrash,
    NewGitdirs,
    OpenGitdirs,
    Home,
    Making,
    NewMaking,
    OpenMaking,
    NewStores,
    OpenStores,
    NewWorkspaces,
    NewHome,
    OpenWorkspaces,
}
#[derive(Debug)]
pub(crate) struct Scan {
    files: Box<[Box<[u8]>]>,
    marked: List<Box<[u8]>>,
    index: usize,
    fd: Option<Fd>,
    at: u64,
    markers: Markers,
    closing: bool,
    failed: bool,
    place: Box<[u8]>,
    next: invocation::Command,
}
#[derive(Debug)]
#[expect(clippy::large_enum_variant, reason = "bounded domain terminal is passed inline")]
pub enum Action {
    Run(Op),
    Layout(Result<(), Error>),
    Operation(Next),
    Swept { changed: bool, set_aside: bool, refused_removal: bool },
    Closed,
}
fn open(root: Fd, path: Box<[u8]>, how: OpenHow) -> Op {
    Op::Open { root, path, how }
}
fn mkdir(dir: Fd, name: &[u8]) -> Op {
    Op::MakeDirectory { dir, name: Box::from(name), mode: 0o700 }
}
fn close(fd: Fd) -> Action {
    Action::Run(Op::Close { fd })
}
fn fd(result: Result<kernel::Done, Error>) -> Result<Fd, Error> {
    match result {
        Ok(kernel::Done::Fd(fd)) => Ok(fd),
        Err(error) => Err(error),
        Ok(_) => unreachable!("kernel completion shape"),
    }
}
fn count(result: Result<kernel::Done, Error>) -> Result<u32, Error> {
    match result {
        Ok(kernel::Done::Count(count)) => Ok(count),
        Err(error) => Err(error),
        Ok(_) => unreachable!("kernel completion shape"),
    }
}
impl Task {
    pub fn reds(directory: Box<[u8]>, content: Option<Box<[u8]>>, marker: bool, rerun: Option<u32>) -> Task {
        Task {
            word: Some(b"reds"),
            pending: Token::new(0),
            phase: Phase::Reds(reds::Files::new(directory, content, marker, rerun)),
        }
    }
    pub fn results(directory: Box<[u8]>) -> Task {
        Task { word: Some(b"results"), pending: Token::new(0), phase: Phase::Results(results::File::new(directory)) }
    }
    pub fn layout(limits: temper_worker_domain_checkout::Limits) -> Task {
        Task {
            word: None,
            pending: Token::new(0),
            phase: Phase::Layout(Layout::Trash, Some(kept::Layout::new(limits))),
        }
    }
    pub fn keep(workspace: Token, mark: Option<&git::Kept>) -> Task {
        Task { word: Some(b"keep"), pending: Token::new(0), phase: Phase::Keep(kept::File::new(workspace, mark)) }
    }
    pub fn make(workspace: Token, stores: Box<[Token]>) -> Task {
        Task { word: None, pending: Token::new(0), phase: Phase::Make(Make::new(workspace, stores)) }
    }
    pub fn look(directory: Box<[u8]>) -> Task {
        Task { word: Some(b"look"), pending: Token::new(0), phase: Phase::Look(Look::new(directory)) }
    }
    pub fn store(store: Token, aside: Option<Token>, remote: Box<[u8]>) -> Task {
        Task { word: Some(b"store"), pending: Token::new(0), phase: Phase::Store(Store::new(store, aside, remote)) }
    }
    pub fn install(store: Token, directory: &[u8], done: Box<git::Done>) -> Task {
        Task {
            word: Some(b"install"),
            pending: Token::new(0),
            phase: Phase::Store(Store::install(store, directory, done)),
        }
    }
    pub fn alternates(at: &git::Place, store: Token, remote: Box<[u8]>, format: invocation::Format) -> Task {
        Task {
            word: Some(b"alternates"),
            pending: Token::new(0),
            phase: Phase::Alternates(Alternates::new(at, store, remote, format)),
        }
    }
    pub fn view(
        at: &git::Place,
        done: Box<git::Done>,
        retry: bool,
        remake: bool,
        format: Option<invocation::Format>,
    ) -> Task {
        Task {
            word: Some(b"view"),
            pending: Token::new(0),
            phase: Phase::View(view::View::new(at, done, retry, remake, format)),
        }
    }
    pub fn scan(files: Box<[Box<[u8]>]>, place: Box<[u8]>, cut: bool) -> Task {
        let marked = List::with_capacity(u32::try_from(files.len()).expect("validated conflict count"));
        Task {
            word: Some(b"scan"),
            pending: Token::new(0),
            phase: Phase::Scan(Scan {
                next: if cut {
                    invocation::Command::UpdateIndex { paths: crate::output::path_input(&files) }
                } else {
                    invocation::Command::Add
                },
                files,
                marked,
                index: 0,
                fd: None,
                at: 0,
                markers: Markers::START,
                closing: false,
                failed: false,
                place,
            }),
        }
    }
    pub fn copy_index(directory: Box<[u8]>, next: Next) -> Task {
        Task { word: Some(b"scan"), pending: Token::new(0), phase: Phase::Index(index::File::copy(directory, next)) }
    }
    pub fn remove_index(directory: Box<[u8]>, next: Next, word: Option<&'static [u8]>) -> Task {
        Task { word, pending: Token::new(0), phase: Phase::Index(index::File::remove(directory, next)) }
    }
    pub fn sweep() -> Task {
        Task { word: None, pending: Token::new(0), phase: Phase::Sweep(Sweep::new()) }
    }
    /// Each sweep completion can set at most one path aside.
    pub fn take_set_aside(&mut self) -> Option<SetAside> {
        match &mut self.phase {
            Phase::Sweep(sweep) => sweep.take_set_aside(),
            Phase::Reds(_)
            | Phase::Results(_)
            | Phase::Layout(_, _)
            | Phase::KeptLayout(_)
            | Phase::Keep(_)
            | Phase::Make(_)
            | Phase::Look(_)
            | Phase::Store(_)
            | Phase::Alternates(_)
            | Phase::View(_)
            | Phase::Scan(_)
            | Phase::Index(_)
            | Phase::Closing => None,
        }
    }
    pub fn closing() -> Task {
        Task { word: None, pending: Token::new(0), phase: Phase::Closing }
    }
    pub fn first(&mut self, dirs: &mut Directories, settings: &Settings, limits: &Limits) -> Action {
        match &mut self.phase {
            Phase::Reds(files) => files.first(settings),
            Phase::Results(file) => file.first(settings),
            Phase::Layout(_, _) => Action::Run(mkdir(settings.root, b"trash")),
            Phase::KeptLayout(layout) => layout.first(dirs, settings),
            Phase::Keep(file) => file.first(dirs),
            Phase::Make(make) => make.first(dirs),
            Phase::Look(look) => look.first(settings),
            Phase::Store(store) => store.first(dirs),
            Phase::Alternates(alternates) => alternates.first(settings),
            Phase::View(view) => view.first(dirs, settings, limits),
            Phase::Scan(scan) => scan.next(settings),
            Phase::Index(file) => file.first(settings),
            Phase::Sweep(_) => Action::Run(open(settings.root, Box::from(&b"trash"[..]), OpenHow::DirectoryNoFollow)),
            Phase::Closing => closing(dirs),
        }
    }
    pub fn complete(
        &mut self,
        complete: kernel::Complete,
        dirs: &mut Directories,
        settings: &Settings,
        limits: &Limits,
        stop: bool,
    ) -> Action {
        if !stop && complete.result == Err(Error::Exists) {
            match &complete.kind {
                Op::Rename { from_dir, from, to_dir, no_replace, .. } => {
                    if *no_replace && Some(*to_dir) == dirs.trash {
                        return Action::Run(dirs.rename(*from_dir, from.clone()));
                    }
                }
                Op::HardLink { .. }
                | Op::Lock { .. }
                | Op::Socket { .. }
                | Op::Bind { .. }
                | Op::Listen { .. }
                | Op::Accept { .. }
                | Op::Connect { .. }
                | Op::Recv { .. }
                | Op::Send { .. }
                | Op::Shutdown { .. }
                | Op::Close { .. }
                | Op::Open { .. }
                | Op::Read { .. }
                | Op::Write { .. }
                | Op::Append { .. }
                | Op::Truncate { .. }
                | Op::Sync { .. }
                | Op::Stat { .. }
                | Op::Remove { .. }
                | Op::MakeDirectory { .. }
                | Op::List { .. }
                | Op::Spawn { .. }
                | Op::Wait { .. }
                | Op::Signal { .. }
                | Op::Usage
                | Op::Window { .. }
                | Op::ReadSignal { .. }
                | Op::PipeRead { .. }
                | Op::PipeWrite { .. }
                | Op::Cancel { .. } => {}
            }
        }
        match &mut self.phase {
            Phase::Reds(files) => files.complete(complete.result, settings, stop),
            Phase::Results(file) => file.complete(complete, settings, stop),
            Phase::Layout(stage, kept) => {
                let action = layout(stage, complete.result, dirs, settings);
                if stop {
                    return Action::Layout(Err(Error::Cancelled));
                }
                match action {
                    Action::Layout(Ok(())) => {
                        let kept = kept.take().expect("layout continuation");
                        self.phase = Phase::KeptLayout(kept);
                        self.first(dirs, settings, limits)
                    }
                    other @ (Action::Run(_)
                    | Action::Layout(Err(_))
                    | Action::Operation(_)
                    | Action::Swept { .. }
                    | Action::Closed) => other,
                }
            }
            Phase::KeptLayout(layout) => {
                let action = layout.complete(complete, dirs, settings);
                if stop { Action::Layout(Err(Error::Cancelled)) } else { action }
            }
            Phase::Keep(file) => file.complete(complete.result, dirs, stop),
            Phase::Make(make) => make.complete(complete.result, dirs, settings, stop),
            Phase::Look(look) => look.complete(complete.result, stop),
            Phase::Store(store) => store.complete(complete.result, dirs, stop),
            Phase::Alternates(alternates) => alternates.complete(complete.result, stop),
            Phase::View(view) => view.complete(complete, dirs, settings, limits, stop),
            Phase::Scan(scan) => scan.complete(complete, settings, limits, stop),
            Phase::Index(file) => file.complete(complete, settings, limits, stop),
            Phase::Sweep(sweep) => sweep.complete(complete, dirs, settings, limits, stop),
            Phase::Closing => closing(dirs),
        }
    }
}
fn closing(dirs: &mut Directories) -> Action {
    if let Some(fd) = dirs.mark_file.take() {
        return close(fd);
    }
    if let Some(fd) = dirs.scanning.take() {
        return close(fd);
    }
    if let Some(fd) = dirs.kept.take() {
        return close(fd);
    }
    if let Some(fd) = dirs.making.take() {
        return close(fd);
    }
    if let Some(fd) = dirs.stores.take() {
        return close(fd);
    }
    if let Some(fd) = dirs.gitdirs.take() {
        return close(fd);
    }
    if let Some(fd) = dirs.workspaces.take() {
        return close(fd);
    }
    if let Some(fd) = dirs.trash.take() {
        return close(fd);
    }
    Action::Closed
}
fn layout(
    stage: &mut Layout,
    result: Result<kernel::Done, Error>,
    dirs: &mut Directories,
    settings: &Settings,
) -> Action {
    let allowed = match stage {
        Layout::Trash | Layout::NewStores | Layout::NewWorkspaces | Layout::NewGitdirs => result == Err(Error::Exists),
        Layout::Making | Layout::Home => result == Err(Error::NotFound),
        Layout::OpenTrash
        | Layout::NewMaking
        | Layout::OpenMaking
        | Layout::NewHome
        | Layout::OpenWorkspaces
        | Layout::OpenGitdirs
        | Layout::OpenStores => false,
    };
    if let Err(error) = result
        && !allowed
    {
        return Action::Layout(Err(error));
    }
    let (next, op) = match stage {
        Layout::Trash => (Layout::OpenTrash, open(settings.root, Box::from(&b"trash"[..]), OpenHow::DirectoryNoFollow)),
        Layout::OpenTrash => {
            dirs.trash = Some(fd(result).expect("successful open"));
            (Layout::Making, dirs.rename(settings.root, Box::from(&b"making"[..])))
        }
        Layout::Making => (Layout::Home, dirs.rename(settings.root, Box::from(&b"home"[..]))),
        Layout::Home => (Layout::NewWorkspaces, mkdir(settings.root, b"workspaces")),
        Layout::NewWorkspaces => (Layout::NewGitdirs, mkdir(settings.root, b"gitdirs")),
        Layout::NewGitdirs => (Layout::NewStores, mkdir(settings.root, b"stores")),
        Layout::NewStores => (Layout::NewMaking, mkdir(settings.root, b"making")),
        Layout::NewMaking => (Layout::NewHome, mkdir(settings.root, b"home")),
        Layout::NewHome => {
            (Layout::OpenWorkspaces, open(settings.root, Box::from(&b"workspaces"[..]), OpenHow::DirectoryNoFollow))
        }
        Layout::OpenWorkspaces => {
            dirs.workspaces = Some(fd(result).expect("successful open"));
            (Layout::OpenGitdirs, open(settings.root, Box::from(&b"gitdirs"[..]), OpenHow::DirectoryNoFollow))
        }
        Layout::OpenGitdirs => {
            dirs.gitdirs = Some(fd(result).expect("successful open"));
            (Layout::OpenStores, open(settings.root, Box::from(&b"stores"[..]), OpenHow::DirectoryNoFollow))
        }
        Layout::OpenStores => {
            dirs.stores = Some(fd(result).expect("successful open"));
            (Layout::OpenMaking, open(settings.root, Box::from(&b"making"[..]), OpenHow::DirectoryNoFollow))
        }
        Layout::OpenMaking => {
            dirs.making = Some(fd(result).expect("successful open"));
            return Action::Layout(Ok(()));
        }
    };
    *stage = next;
    Action::Run(op)
}
impl Scan {
    fn next(&mut self, settings: &Settings) -> Action {
        let Some(path) = self.files.get(self.index) else {
            return Action::Operation(if self.marked.is_empty() {
                Next::Run(self.next.clone())
            } else {
                Next::End(git::Done::Conflicted {
                    files: core::mem::replace(&mut self.marked, List::with_capacity(0)).into_boxed(),
                    cut: false,
                })
            });
        };
        self.at = 0;
        self.markers = Markers::START;
        Action::Run(open(settings.root, invocation::joined(&[&self.place, b"/", path]), OpenHow::ReadNoFollow))
    }
    fn read(&self, limits: &Limits) -> Action {
        Action::Run(Op::Read {
            fd: self.fd.expect("scan open"),
            buf: bytes::zeroed(usize::try_from(limits.read_bytes).expect("read size fits")),
            at: self.at,
        })
    }
    fn complete(&mut self, complete: kernel::Complete, settings: &Settings, limits: &Limits, stop: bool) -> Action {
        if self.closing {
            self.fd = None;
            self.closing = false;
            if complete.result.is_err() {
                self.failed = true;
            }
            if self.failed || stop {
                return Action::Operation(crate::sequence::failed(git::Fault::Broken, b"", 0));
            }
            self.index = self.index.checked_add(1).expect("conflict list length");
            return self.next(settings);
        }
        if self.fd.is_none() {
            match fd(complete.result) {
                Ok(fd) => self.fd = Some(fd),
                Err(Error::NotFound | Error::TooManyLinks | Error::NotAFile) => {
                    if stop {
                        return Action::Operation(crate::sequence::failed(git::Fault::Broken, b"", 0));
                    }
                    self.index = self.index.checked_add(1).expect("conflict list length");
                    return self.next(settings);
                }
                Err(_) => return Action::Operation(crate::sequence::failed(git::Fault::Broken, b"", 0)),
            }
            if !stop {
                return self.read(limits);
            }
        } else {
            match count(complete.result) {
                Ok(n) => {
                    let buf = read_buffer(complete.kind);
                    let kept = buf.get(..usize::try_from(n).expect("read count fits")).expect("kernel read count");
                    self.markers.feed(kept);
                    self.at = self.at.checked_add(u64::from(n)).expect("file offset fits");
                    if n > 0 && !stop {
                        return self.read(limits);
                    }
                    if !stop && self.markers.finish() {
                        self.marked
                            .push(self.files.get(self.index).expect("current conflict").clone())
                            .expect("conflict count");
                    }
                }
                Err(Error::IsADirectory) if self.at == 0 => {}
                Err(_) => self.failed = true,
            }
        }
        self.closing = true;
        close(self.fd.expect("scan open"))
    }
}
fn read_buffer(op: Op) -> Box<[u8]> {
    match op {
        Op::Read { buf, .. } => buf,
        Op::Lock { .. }
        | Op::Socket { .. }
        | Op::Bind { .. }
        | Op::Listen { .. }
        | Op::Accept { .. }
        | Op::Connect { .. }
        | Op::Recv { .. }
        | Op::Send { .. }
        | Op::Shutdown { .. }
        | Op::Close { .. }
        | Op::Open { .. }
        | Op::Write { .. }
        | Op::Append { .. }
        | Op::Truncate { .. }
        | Op::Sync { .. }
        | Op::Stat { .. }
        | Op::HardLink { .. }
        | Op::Rename { .. }
        | Op::Remove { .. }
        | Op::MakeDirectory { .. }
        | Op::List { .. }
        | Op::Spawn { .. }
        | Op::Wait { .. }
        | Op::Signal { .. }
        | Op::Usage
        | Op::Window { .. }
        | Op::ReadSignal { .. }
        | Op::PipeRead { .. }
        | Op::PipeWrite { .. }
        | Op::Cancel { .. } => unreachable!("results read completion"),
    }
}

fn page(complete: kernel::Complete) -> Result<Box<[u8]>, Error> {
    let n = count(complete.result)?;
    if n == 0 {
        return Ok(Box::from(&b""[..]));
    }
    match complete.kind {
        Op::List { entries, names, .. } => {
            Ok(Box::from(entries.first().expect("nonempty listing").name(&names).expect("kernel entry name")))
        }
        Op::Lock { .. }
        | Op::Socket { .. }
        | Op::Bind { .. }
        | Op::Listen { .. }
        | Op::Accept { .. }
        | Op::Connect { .. }
        | Op::Recv { .. }
        | Op::Send { .. }
        | Op::Shutdown { .. }
        | Op::Close { .. }
        | Op::Open { .. }
        | Op::Read { .. }
        | Op::Write { .. }
        | Op::Append { .. }
        | Op::Truncate { .. }
        | Op::Sync { .. }
        | Op::Stat { .. }
        | Op::HardLink { .. }
        | Op::Rename { .. }
        | Op::Remove { .. }
        | Op::MakeDirectory { .. }
        | Op::Spawn { .. }
        | Op::Wait { .. }
        | Op::Signal { .. }
        | Op::Usage
        | Op::Window { .. }
        | Op::ReadSignal { .. }
        | Op::PipeRead { .. }
        | Op::PipeWrite { .. }
        | Op::Cancel { .. } => unreachable!("sweep listing"),
    }
}
