//! The background trash walk (temper/docs/design/protocol/git-sweep.md, 2–3).
use super::{Action, Directories, close, count, fd, open};
use crate::{Limits, Settings, invocation};
use alloc::boxed::Box;
use skein_io::kernel::{self, Error, Fd, Op, OpenHow};
use skein_lib::{List, bytes};

#[derive(Debug)]
pub(crate) struct SetAside {
    pub path: Box<[u8]>,
    pub error: Error,
    pub report: bool,
}

#[derive(Debug)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "inspection records removal refusals independently of scheduling and child reporting"
)]
pub(crate) struct Sweep {
    phase: Phase,
    trash: Option<Fd>,
    directory: Option<Fd>,
    parent: Box<[u8]>,
    page: Option<Page>,
    child_stuck: bool,
    changed: bool,
    stuck: bool,
    refused_removal: bool,
    set_aside: Option<SetAside>,
}

#[derive(Clone, Copy, Debug)]
enum Phase {
    OpenTrash,
    ListTrash,
    // When not at EOF, the listing already reported why it stopped.
    CloseTrash { eof: bool },
    RemoveEntry,
    OpenParent,
    ListParent,
    CloseParent { eof: bool },
    RemoveChild,
    MoveChild,
    RemoveParent,
}

#[derive(Debug)]
struct Page {
    entries: Box<[kernel::Entry]>,
    names: Box<[u8]>,
    count: u32,
    at: u32,
}

impl Page {
    fn read(complete: kernel::Complete) -> Result<Page, Error> {
        let count = count(complete.result)?;
        match complete.kind {
            Op::List { entries, names, .. } => Ok(Page { entries, names, count, at: 0 }),
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
            | Op::ReadLink { .. }
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
    fn next(&mut self) -> Option<&[u8]> {
        if self.at == self.count {
            return None;
        }
        let at = self.at;
        self.at = self.at.checked_add(1).expect("more entries in page");
        Some(self.name(at))
    }
    fn current(&self) -> &[u8] {
        self.name(self.at.checked_sub(1).expect("a child is in flight"))
    }
    fn name(&self, at: u32) -> &[u8] {
        let at = usize::try_from(at).expect("listing index fits");
        let entry = self.entries.get(at).expect("count fits listing buffer");
        entry.name(&self.names).expect("kernel entry name")
    }
}

fn list(fd: Fd, size: u32, limits: &Limits) -> Action {
    let mut entries = List::allocated(size);
    for _ in 0..size {
        entries.push(kernel::Entry::BLANK).expect("page capacity");
    }
    Action::Run(Op::List {
        fd,
        entries: entries.into_boxed(),
        names: bytes::zeroed(usize::try_from(limits.sweep_bytes).expect("page size fits")),
    })
}

impl Sweep {
    pub(super) fn new() -> Sweep {
        Sweep {
            phase: Phase::OpenTrash,
            trash: None,
            directory: None,
            parent: Box::default(),
            page: None,
            child_stuck: false,
            changed: false,
            stuck: false,
            refused_removal: false,
            set_aside: None,
        }
    }
    pub(super) fn take_set_aside(&mut self) -> Option<SetAside> {
        self.set_aside.take()
    }
    fn record(&mut self, path: Box<[u8]>, error: Error, report: bool) {
        self.stuck = true;
        self.set_aside = Some(SetAside { path, error, report });
    }
    fn parent_path(&self) -> Box<[u8]> {
        invocation::joined(&[b"trash/", &self.parent])
    }
    fn parent_stuck(&mut self, error: Error) {
        self.record(self.parent_path(), error, !self.child_stuck);
    }
    fn list_trash(&mut self, limits: &Limits) -> Action {
        self.phase = Phase::ListTrash;
        list(self.trash.expect("trash cursor open"), 1, limits)
    }
    fn list_parent(&mut self, limits: &Limits) -> Action {
        self.phase = Phase::ListParent;
        list(self.directory.expect("parent cursor open"), limits.sweep_entries, limits)
    }
    fn close_trash(&mut self, eof: bool) -> Action {
        self.phase = Phase::CloseTrash { eof };
        close(self.trash.take().expect("trash cursor open"))
    }
    fn close_parent(&mut self, eof: bool) -> Action {
        self.phase = Phase::CloseParent { eof };
        close(self.directory.take().expect("parent cursor open"))
    }
    fn remove_entry(&self, dirs: &Directories, directory: bool) -> Action {
        Action::Run(Op::Remove { dir: dirs.trash.expect("layout"), name: self.parent.clone(), directory })
    }
    fn stop(&mut self, result: Result<kernel::Done, Error>) -> Action {
        // An Open already in flight can still give us a descriptor to close.
        match self.phase {
            Phase::OpenTrash => self.trash = fd(result).ok(),
            Phase::OpenParent => self.directory = fd(result).ok(),
            Phase::ListTrash
            | Phase::CloseTrash { .. }
            | Phase::RemoveEntry
            | Phase::ListParent
            | Phase::CloseParent { .. }
            | Phase::RemoveChild
            | Phase::MoveChild
            | Phase::RemoveParent => {}
        }
        if self.directory.is_some() {
            return self.close_parent(false);
        }
        if self.trash.is_some() {
            return self.close_trash(false);
        }
        Action::Swept { changed: false, set_aside: false, refused_removal: self.refused_removal }
    }
    pub(super) fn complete(
        &mut self,
        complete: kernel::Complete,
        dirs: &mut Directories,
        settings: &Settings,
        limits: &Limits,
        stop: bool,
    ) -> Action {
        if stop {
            return self.stop(complete.result);
        }
        match self.phase {
            Phase::RemoveEntry | Phase::RemoveChild | Phase::RemoveParent => {
                if complete.result.is_err()
                    && complete.result != Err(Error::IsADirectory)
                    && complete.result != Err(Error::NotFound)
                {
                    self.refused_removal = true;
                }
            }
            Phase::OpenTrash
            | Phase::ListTrash
            | Phase::CloseTrash { .. }
            | Phase::OpenParent
            | Phase::ListParent
            | Phase::CloseParent { .. }
            | Phase::MoveChild => {}
        }
        match self.phase {
            Phase::OpenTrash => match fd(complete.result) {
                Ok(fd) => {
                    self.trash = Some(fd);
                    self.list_trash(limits)
                }
                Err(error) => {
                    self.record(Box::from(b"trash".as_slice()), error, true);
                    self.end()
                }
            },
            Phase::ListTrash => self.listed_trash(complete, dirs),
            Phase::CloseTrash { eof } => {
                if eof && let Err(error) = complete.result {
                    self.record(Box::from(b"trash".as_slice()), error, true);
                }
                self.end()
            }
            Phase::RemoveEntry => {
                if complete.result == Err(Error::IsADirectory) {
                    self.phase = Phase::OpenParent;
                    return Action::Run(open(settings.root, self.parent_path(), OpenHow::DirectoryNoFollow));
                }
                self.removed_entry(complete.result, limits, false)
            }
            Phase::OpenParent => match fd(complete.result) {
                Ok(fd) => {
                    self.directory = Some(fd);
                    self.list_parent(limits)
                }
                Err(error) => {
                    self.parent_stuck(error);
                    self.list_trash(limits)
                }
            },
            Phase::ListParent => self.listed_parent(complete, limits),
            Phase::CloseParent { eof } => {
                if eof {
                    match complete.result {
                        Ok(_) => {
                            self.phase = Phase::RemoveParent;
                            return self.remove_entry(dirs, true);
                        }
                        Err(error) => self.parent_stuck(error),
                    }
                }
                self.list_trash(limits)
            }
            Phase::RemoveParent => self.removed_entry(complete.result, limits, true),
            Phase::RemoveChild => {
                if complete.result == Err(Error::IsADirectory) {
                    self.phase = Phase::MoveChild;
                    return Action::Run(
                        dirs.rename(self.directory.expect("parent open"), Box::from(self.current_child())),
                    );
                }
                self.after_child(complete.result, limits)
            }
            Phase::MoveChild => self.after_child(complete.result, limits),
        }
    }
    fn end(&self) -> Action {
        Action::Swept { changed: self.changed, set_aside: self.stuck, refused_removal: self.refused_removal }
    }
    fn listed_trash(&mut self, complete: kernel::Complete, dirs: &Directories) -> Action {
        match Page::read(complete) {
            Ok(mut page) => {
                let Some(name) = page.next() else {
                    return self.close_trash(true);
                };
                self.parent = Box::from(name);
                self.child_stuck = false;
                self.phase = Phase::RemoveEntry;
                self.remove_entry(dirs, false)
            }
            Err(error) => {
                self.record(Box::from(b"trash".as_slice()), error, true);
                self.close_trash(false)
            }
        }
    }
    fn removed_entry(&mut self, result: Result<kernel::Done, Error>, limits: &Limits, directory: bool) -> Action {
        match result {
            Ok(_) => self.changed = true,
            Err(Error::NotFound) if !directory => {}
            Err(error) => self.parent_stuck(error),
        }
        self.list_trash(limits)
    }
    fn listed_parent(&mut self, complete: kernel::Complete, limits: &Limits) -> Action {
        match Page::read(complete) {
            Ok(page) if page.count == 0 => self.close_parent(true),
            Ok(page) => {
                self.page = Some(page);
                self.next_child(limits)
            }
            Err(error) => {
                self.parent_stuck(error);
                self.close_parent(false)
            }
        }
    }
    fn next_child(&mut self, limits: &Limits) -> Action {
        let Some(name) = self.page.as_mut().expect("child page").next() else {
            self.page = None;
            return self.list_parent(limits);
        };
        self.phase = Phase::RemoveChild;
        Action::Run(Op::Remove { dir: self.directory.expect("parent open"), name: Box::from(name), directory: false })
    }
    fn current_child(&self) -> &[u8] {
        self.page.as_ref().expect("child page").current()
    }
    fn after_child(&mut self, result: Result<kernel::Done, Error>, limits: &Limits) -> Action {
        match result {
            Ok(_) => self.changed = true,
            Err(Error::NotFound) => {}
            Err(error) => {
                self.child_stuck = true;
                let path = invocation::joined(&[b"trash/", &self.parent, b"/", self.current_child()]);
                self.record(path, error, true);
            }
        }
        self.next_child(limits)
    }
}
