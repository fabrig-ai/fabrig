//! A record-driven generation and transcript tree, never writing live files (restoring.md, 4.4–4.5).
use super::{join, roll::STAGED};
use crate::{
    Arrived, Root,
    checkpoints::{
        files,
        transfer::{Linking, Transfer},
    },
    codec::component::{Checkpoint, Manifest},
    generations::{self, Progress, Removing},
};
use alloc::boxed::Box;
use skein_io::{file, kernel::Error};
use skein_lib::{Queue, Token};
#[derive(Debug)]
enum Step {
    Generation(super::removal::Generation),
    Remove(Removing),
    Make(u8),
    Private,
    PrivateClose,
    Shard,
    Transfer(Box<Transfer>),
    SyncOpen { index: u8 },
    Sync { index: u8 },
    SyncClose { index: u8 },
    Manifest,
    Drain,
    Finished,
}
#[derive(Debug)]
pub(crate) struct Staging {
    source: Root,
    root: Root,
    record: Checkpoint,
    manifest: Manifest,
    generation: u64,
    owner: Token,
    next: u32,
    tree: bool,
    open: Option<Root>,
    step: Step,
    path: Box<[u8]>,
    failure: Option<Error>,
    linked: u32,
    copied: u32,
    bytes: u64,
}
impl Staging {
    pub fn path(&self) -> Box<[u8]> {
        self.path.clone()
    }
    pub fn failure(&self) -> Option<Error> {
        self.failure
    }
    pub fn linked(&self) -> u32 {
        self.linked
    }
    pub fn copied(&self) -> u32 {
        self.copied
    }
    pub fn bytes(&self) -> u64 {
        self.bytes
    }
    pub fn begin(
        source: Root,
        root: Root,
        record: Checkpoint,
        manifest: Manifest,
        generation: u64,
        owner: Token,
        queue: &mut Queue<file::Request>,
    ) -> Self {
        let path = generations::generation(generation);
        let removing = super::removal::Generation::begin(root, generation, owner, queue);
        Self {
            source,
            root,
            record,
            manifest,
            generation,
            owner,
            next: 0,
            tree: false,
            open: None,
            step: Step::Generation(removing),
            path,
            failure: None,
            linked: 0,
            copied: 0,
            bytes: 0,
        }
    }
    fn directory(&self, index: u8) -> Box<[u8]> {
        match index {
            0 => generations::generation(self.generation),
            1 => generations::map(self.generation, generations::Map::Store),
            2 => join(&generations::map(self.generation, generations::Map::Store), b"/logs"),
            3 => generations::map(self.generation, generations::Map::Secrets),
            4 => Box::from(STAGED),
            _ => unreachable!("five staged directories"),
        }
    }
    fn make(&mut self, index: u8, queue: &mut Queue<file::Request>) {
        self.path = self.directory(index);
        queue.push(file::Request::MakeDirectory {
            owner: self.owner,
            root: self.root.token,
            path: self.path.clone(),
            mode: 0o700,
        });
        self.step = Step::Make(index);
    }
    fn shard(&self) -> Option<Box<[u8]>> {
        let row = self.record.files.get(self.next)?;
        match row.part() {
            3 => Some(join(
                &join(&generations::map(self.generation, generations::Map::Store), b"/"),
                &skein_kv::log_directory(row.number()),
            )),
            4 => Some(join(&join(STAGED, b"/"), &crate::transcripts::name(row.number() >> 12))),
            1 | 2 | 5 | 6 => None,
            _ => unreachable!("source's parts validated before safety checkpoint"),
        }
    }
    fn transfer(&mut self, queue: &mut Queue<file::Request>) {
        let row = self.record.files.get(self.next).expect("current file");
        let linking = match row.part() {
            1 | 2 => Linking::Always,
            3 => {
                if row.way() == 1 {
                    Linking::Exact
                } else {
                    Linking::Never
                }
            }
            4 => {
                if row.way() == 1 {
                    Linking::Always
                } else {
                    Linking::Never
                }
            }
            5 | 6 => Linking::Never,
            _ => unreachable!("validated part"),
        };
        let from = files::file_path(1, row.part(), row.number());
        let to = if row.part() == 4 {
            join(&join(STAGED, b"/"), &crate::transcripts::path(row.number()))
        } else {
            files::file_path(self.generation, row.part(), row.number())
        };
        self.path.clone_from(&from);
        self.step = Step::Transfer(Box::new(Transfer::begin(
            self.source,
            self.root,
            from,
            to,
            Some(row.length()),
            linking,
            false,
            self.owner,
            queue,
        )));
    }
    fn next(&mut self, queue: &mut Queue<file::Request>) {
        // Missing transcripts require no file request. The bounded record supplies this loop's bound.
        for _ in self.next..self.record.files.len() {
            let row = self.record.files.get(self.next).expect("bounded record cursor");
            if (row.part() == 4) != self.tree || row.way() == 3 {
                self.next = self.next.saturating_add(1);
                continue;
            }
            if let Some(path) = self.shard() {
                self.path = path;
                queue.push(file::Request::MakeDirectory {
                    owner: self.owner,
                    root: self.root.token,
                    path: self.path.clone(),
                    mode: 0o700,
                });
                self.step = Step::Shard;
            } else {
                self.transfer(queue);
            }
            return;
        }
        self.sync(if self.tree { 6 } else { 1 }, queue);
    }
    fn sync(&mut self, index: u8, queue: &mut Queue<file::Request>) {
        self.path = match index {
            0 => self.shard().expect("current file's shard"),
            1 => self.directory(2),
            2 => self.directory(1),
            3 => self.directory(3),
            4 => self.directory(0),
            5 => Box::from(generations::GENERATIONS),
            6 => Box::from(STAGED),
            7 => Box::from(b".".as_slice()),
            _ => unreachable!("eight sync stages"),
        };
        queue.push(file::Request::OpenDirectory {
            owner: self.owner,
            root: self.root.fd,
            name: self.path.clone(),
            no_follow: true,
        });
        self.step = Step::SyncOpen { index };
    }
    fn fail(&mut self, error: Option<Error>, queue: &mut Queue<file::Request>) -> Progress {
        self.failure = error;
        if let Some(root) = self.open.take() {
            queue.push(file::Request::Close { owner: self.owner, file: root.token });
            self.step = Step::Drain;
            Progress::Pending
        } else {
            Progress::Failed
        }
    }
    #[expect(clippy::too_many_lines, reason = "one total generation and transcript staging transition table")]
    pub fn arrived(&mut self, arrived: Arrived, written: u64, queue: &mut Queue<file::Request>) -> Progress {
        let error = files::error(&arrived.event);
        match core::mem::replace(&mut self.step, Step::Finished) {
            Step::Generation(mut removing) => match removing.arrived(arrived, queue) {
                Progress::Pending => self.step = Step::Generation(removing),
                Progress::Failed => return self.fail(Some(removing.error().unwrap_or(Error::Other(0))), queue),
                Progress::Done => self.make(0, queue),
            },
            Step::Remove(mut removing) => match generations::removed(&mut removing, arrived, queue) {
                Progress::Pending => self.step = Step::Remove(removing),
                Progress::Failed => return self.fail(Some(error.unwrap_or(Error::Other(0))), queue),
                Progress::Done => self.make(4, queue),
            },
            Step::Make(index) => {
                if !files::made(&arrived.event, false) {
                    return self.fail(Some(error.unwrap_or(Error::Other(0))), queue);
                }
                if index == 2 {
                    self.path = self.directory(3);
                    queue.push(file::Request::OpenPrivate {
                        owner: self.owner,
                        root: self.root.token,
                        path: self.path.clone(),
                    });
                    self.step = Step::Private;
                } else if index == 4 {
                    self.next(queue);
                } else {
                    self.make(index.saturating_add(1), queue);
                }
            }
            Step::Private => {
                let Some(root) = crate::directory::opened(arrived) else {
                    return self.fail(Some(error.unwrap_or(Error::Other(0))), queue);
                };
                queue.push(file::Request::Close { owner: self.owner, file: root.token });
                self.step = Step::PrivateClose;
            }
            Step::PrivateClose => {
                let file::Event::Closed { .. } = arrived.event else {
                    return self.fail(Some(error.unwrap_or(Error::Other(0))), queue);
                };
                self.next(queue);
            }
            Step::Shard => {
                if !files::made(&arrived.event, true) {
                    return self.fail(Some(error.unwrap_or(Error::Other(0))), queue);
                }
                self.transfer(queue);
            }
            Step::Transfer(mut transfer) => {
                if !transfer.arrived(arrived, queue) {
                    self.step = Step::Transfer(transfer);
                } else if let Some(error) = transfer.failure() {
                    self.path = transfer.path();
                    return self.fail(error.error(), queue);
                } else {
                    if transfer.way() == 1 {
                        self.linked = self.linked.saturating_add(1);
                    } else {
                        self.copied = self.copied.saturating_add(1);
                        self.bytes = self.bytes.saturating_add(transfer.length());
                    }
                    if self.shard().is_some() {
                        self.sync(0, queue);
                    } else {
                        self.next = self.next.saturating_add(1);
                        self.next(queue);
                    }
                }
            }
            Step::SyncOpen { index } => {
                let Some(root) = crate::directory::opened(arrived) else {
                    return self.fail(Some(error.unwrap_or(Error::Other(0))), queue);
                };
                self.open = Some(root);
                queue.push(file::Request::SyncDirectory { owner: self.owner, root: root.fd });
                self.step = Step::Sync { index };
            }
            Step::Sync { index } => {
                let file::Event::Synced { .. } = arrived.event else {
                    return self.fail(Some(error.unwrap_or(Error::Other(0))), queue);
                };
                let root = self.open.take().expect("sync directory");
                queue.push(file::Request::Close { owner: self.owner, file: root.token });
                self.step = Step::SyncClose { index };
            }
            Step::SyncClose { index } => {
                let file::Event::Closed { .. } = arrived.event else {
                    return self.fail(Some(error.unwrap_or(Error::Other(0))), queue);
                };
                if index == 0 {
                    self.next = self.next.saturating_add(1);
                    self.next(queue);
                } else if index == 7 {
                    return Progress::Done;
                } else if index == 5 {
                    self.tree = true;
                    self.next = 0;
                    self.path = Box::from(STAGED);
                    self.step = Step::Remove(generations::remove_checkpoint_owned(
                        self.root,
                        self.path.clone(),
                        self.owner,
                        queue,
                    ));
                } else if index == 3 {
                    let manifest = Manifest {
                        generation: self.generation,
                        from: None,
                        before: self.manifest.after,
                        after: self.manifest.after,
                        counts: skein_lib::List::with_capacity(0),
                        store: self.record.store,
                        secrets: Some(self.record.secrets),
                        written,
                    };
                    self.path = join(&self.directory(0), b"/manifest");
                    queue.push(file::Request::Store {
                        owner: self.owner,
                        root: self.root.token,
                        path: self.path.clone(),
                        bytes: manifest.encode(),
                        expected: file::Expect::Absent,
                        no_follow: true,
                    });
                    self.step = Step::Manifest;
                } else {
                    self.sync(index.saturating_add(1), queue);
                }
            }
            Step::Manifest => {
                let file::Event::Stored { .. } = arrived.event else {
                    return self.fail(Some(error.unwrap_or(Error::Other(0))), queue);
                };
                self.sync(4, queue);
            }
            Step::Drain | Step::Finished => return Progress::Failed,
        }
        Progress::Pending
    }
}
