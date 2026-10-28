//! Kernel steps beneath the worker's root, with one submission per machine.
use crate::{Limits, Settings, invocation, output::Markers, sequence::Next};
use alloc::boxed::Box;
use skein_io::kernel::{self, Error, Fd, Op, OpenHow};
use skein_lib::{Decimal, List, Token, bytes};
use temper_worker_domain_checkout::git;

#[derive(Debug)]
pub(crate) struct Directories {
    pub trash: Option<Fd>,
    pub workspaces: Option<Fd>,
    pub serial: u64,
    pub failures: u64,
}
impl Directories {
    pub fn new(serial: u64) -> Directories {
        Directories { trash: None, workspaces: None, serial, failures: 0 }
    }
    fn rename(&mut self, from_dir: Fd, from: Box<[u8]>) -> Op {
        let to = Box::from(Decimal::of(self.serial).as_bytes());
        self.serial = self.serial.checked_add(1).expect("trash serial lifetime");
        Op::Rename { from_dir, from, to_dir: self.trash.expect("trash opened"), to, no_replace: false }
    }
}
#[derive(Debug)]
pub(crate) struct Task {
    pub pending: Token,
    pub phase: Phase,
}
#[derive(Debug)]
pub(crate) enum Phase {
    Layout(Layout),
    Make { workspace: Box<[u8]>, creating: bool },
    Scan(Scan),
    Sweep(Sweep),
    Closing,
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum Layout {
    Trash,
    OpenTrash,
    Workspaces,
    Home,
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
}
#[derive(Debug)]
pub(crate) struct Sweep {
    phase: SweepPhase,
    fd: Option<Fd>,
    parent: Box<[u8]>,
    child: Box<[u8]>,
    failed: bool,
}
#[derive(Clone, Copy, Debug)]
enum SweepPhase {
    OpenTrash,
    ListTrash,
    CloseTrash,
    Remove,
    OpenParent,
    ListParent,
    CloseParent,
    Child,
    Move,
    Directory,
    Restart,
    End,
}
#[derive(Debug)]
#[expect(clippy::large_enum_variant, reason = "bounded domain terminal is passed inline")]
pub(crate) enum Action {
    Run(Op),
    Layout(Result<(), Error>),
    Operation(Next),
    Swept(bool),
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
    pub fn layout() -> Task {
        Task { pending: Token::new(0), phase: Phase::Layout(Layout::Trash) }
    }
    pub fn make(workspace: Token) -> Task {
        Task {
            pending: Token::new(0),
            phase: Phase::Make { workspace: Box::from(Decimal::of(workspace.raw()).as_bytes()), creating: false },
        }
    }
    pub fn scan(files: Box<[Box<[u8]>]>, place: Box<[u8]>) -> Task {
        let marked = List::with_capacity(u32::try_from(files.len()).expect("validated conflict count"));
        Task {
            pending: Token::new(0),
            phase: Phase::Scan(Scan {
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
    pub fn sweep() -> Task {
        Task {
            pending: Token::new(0),
            phase: Phase::Sweep(Sweep {
                phase: SweepPhase::OpenTrash,
                fd: None,
                parent: Box::from(&b""[..]),
                child: Box::from(&b""[..]),
                failed: false,
            }),
        }
    }
    pub fn closing() -> Task {
        Task { pending: Token::new(0), phase: Phase::Closing }
    }
    pub fn first(&mut self, dirs: &mut Directories, settings: &Settings) -> Action {
        match &mut self.phase {
            Phase::Layout(_) => Action::Run(mkdir(settings.root, b"trash")),
            Phase::Make { workspace, .. } => {
                Action::Run(dirs.rename(dirs.workspaces.expect("layout"), workspace.clone()))
            }
            Phase::Scan(scan) => scan.next(settings),
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
        match &mut self.phase {
            Phase::Layout(stage) => {
                let action = layout(stage, complete.result, dirs, settings);
                if stop { Action::Layout(Err(Error::Cancelled)) } else { action }
            }
            Phase::Make { workspace, creating } => {
                let accepted = complete.result.is_ok() || (!*creating && complete.result == Err(Error::NotFound));
                if !accepted {
                    return Action::Operation(Next::End(git::Done::Failed { fault: git::Fault::Broken }));
                }
                if *creating || stop {
                    return Action::Operation(Next::End(git::Done::Succeeded));
                }
                *creating = true;
                Action::Run(mkdir(dirs.workspaces.expect("layout"), workspace))
            }
            Phase::Scan(scan) => scan.complete(complete, settings, limits, stop),
            Phase::Sweep(sweep) => sweep.complete(complete, dirs, settings, limits, stop),
            Phase::Closing => closing(dirs),
        }
    }
}
fn closing(dirs: &mut Directories) -> Action {
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
        Layout::Trash => result == Err(Error::Exists),
        Layout::Workspaces | Layout::Home => result == Err(Error::NotFound),
        Layout::OpenTrash | Layout::NewWorkspaces | Layout::NewHome | Layout::OpenWorkspaces => false,
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
            (Layout::Workspaces, dirs.rename(settings.root, Box::from(&b"workspaces"[..])))
        }
        Layout::Workspaces => (Layout::Home, dirs.rename(settings.root, Box::from(&b"home"[..]))),
        Layout::Home => (Layout::NewWorkspaces, mkdir(settings.root, b"workspaces")),
        Layout::NewWorkspaces => (Layout::NewHome, mkdir(settings.root, b"home")),
        Layout::NewHome => {
            (Layout::OpenWorkspaces, open(settings.root, Box::from(&b"workspaces"[..]), OpenHow::DirectoryNoFollow))
        }
        Layout::OpenWorkspaces => {
            dirs.workspaces = Some(fd(result).expect("successful open"));
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
                Next::Run(invocation::Command::Add)
            } else {
                Next::End(git::Done::Conflicted {
                    files: core::mem::replace(&mut self.marked, List::with_capacity(0)).into_boxed(),
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
                    let buf = match complete.kind {
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
                        | Op::Cancel { .. } => unreachable!("scan read"),
                    };
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
impl Sweep {
    fn list(&self, limits: &Limits) -> Action {
        let mut entries = List::with_capacity(limits.sweep_entries);
        for _ in 0..limits.sweep_entries {
            entries.push(kernel::Entry::BLANK).expect("page capacity");
        }
        let entries = entries.into_boxed();
        Action::Run(Op::List {
            fd: self.fd.expect("directory open"),
            entries,
            names: bytes::zeroed(usize::try_from(limits.sweep_bytes).expect("page size fits")),
        })
    }
    fn end(&mut self, failed: bool) -> Action {
        self.failed = self.failed || failed;
        if let Some(fd) = self.fd.take() {
            self.phase = SweepPhase::End;
            return close(fd);
        }
        Action::Swept(self.failed)
    }
    fn complete(
        &mut self,
        complete: kernel::Complete,
        dirs: &mut Directories,
        settings: &Settings,
        limits: &Limits,
        stop: bool,
    ) -> Action {
        match self.phase {
            SweepPhase::OpenTrash => self.opened(complete.result, limits, stop, false),
            SweepPhase::OpenParent => self.opened(complete.result, limits, stop, true),
            SweepPhase::ListTrash => self.listed(complete, false),
            SweepPhase::ListParent => self.listed(complete, true),
            SweepPhase::CloseTrash => {
                if complete.result.is_err() {
                    return self.end(true);
                }
                if stop || self.parent.is_empty() {
                    return self.end(false);
                }
                self.phase = SweepPhase::Remove;
                Action::Run(Op::Remove {
                    dir: dirs.trash.expect("layout"),
                    name: self.parent.clone(),
                    directory: false,
                })
            }
            SweepPhase::CloseParent => {
                if complete.result.is_err() {
                    return self.end(true);
                }
                if stop {
                    return self.end(false);
                }
                if self.child.is_empty() {
                    self.phase = SweepPhase::Directory;
                    Action::Run(Op::Remove {
                        dir: dirs.trash.expect("layout"),
                        name: self.parent.clone(),
                        directory: true,
                    })
                } else {
                    self.phase = SweepPhase::OpenParent;
                    Action::Run(open(
                        settings.root,
                        invocation::joined(&[b"trash/", &self.parent]),
                        OpenHow::DirectoryNoFollow,
                    ))
                }
            }
            SweepPhase::Remove => {
                if stop {
                    return self.end(complete.result.is_err());
                }
                if complete.result == Err(Error::IsADirectory) {
                    self.phase = SweepPhase::OpenParent;
                    self.child = Box::from(&b""[..]);
                    return Action::Run(open(
                        settings.root,
                        invocation::joined(&[b"trash/", &self.parent]),
                        OpenHow::DirectoryNoFollow,
                    ));
                }
                self.restart(complete.result, settings)
            }
            SweepPhase::Child => {
                if complete.result == Err(Error::IsADirectory) && !stop {
                    self.phase = SweepPhase::Move;
                    return Action::Run(dirs.rename(self.fd.expect("parent open"), self.child.clone()));
                }
                self.after_child(complete.result, stop)
            }
            SweepPhase::Move => self.after_child(complete.result, stop),
            SweepPhase::Directory | SweepPhase::Restart => {
                if stop {
                    self.end(complete.result.is_err())
                } else {
                    self.restart(complete.result, settings)
                }
            }
            SweepPhase::End => self.end(complete.result.is_err()),
        }
    }
    fn opened(&mut self, result: Result<kernel::Done, Error>, limits: &Limits, stop: bool, parent: bool) -> Action {
        match fd(result) {
            Ok(fd) => self.fd = Some(fd),
            Err(_) => return self.end(true),
        }
        if stop {
            return self.end(false);
        }
        if parent && !self.child.is_empty() {
            self.phase = SweepPhase::Child;
            return Action::Run(Op::Remove {
                dir: self.fd.expect("parent open"),
                name: self.child.clone(),
                directory: false,
            });
        }
        self.phase = if parent { SweepPhase::ListParent } else { SweepPhase::ListTrash };
        self.list(limits)
    }
    fn listed(&mut self, complete: kernel::Complete, parent: bool) -> Action {
        match page(complete) {
            Ok(name) => {
                if parent {
                    self.child = name;
                } else {
                    self.parent = name;
                }
            }
            Err(_) => return self.end(true),
        }
        self.phase = if parent { SweepPhase::CloseParent } else { SweepPhase::CloseTrash };
        close(self.fd.take().expect("listed directory"))
    }
    fn after_child(&mut self, result: Result<kernel::Done, Error>, stop: bool) -> Action {
        if result.is_err() || stop {
            return self.end(result.is_err());
        }
        self.phase = SweepPhase::Restart;
        close(self.fd.take().expect("parent open"))
    }
    fn restart(&mut self, result: Result<kernel::Done, Error>, settings: &Settings) -> Action {
        if result.is_err() {
            return self.end(true);
        }
        self.phase = SweepPhase::OpenTrash;
        Action::Run(open(settings.root, Box::from(&b"trash"[..]), OpenHow::DirectoryNoFollow))
    }
}
