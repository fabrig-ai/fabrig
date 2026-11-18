//! Bounded retention, after publication and before held measurement (checkpoints.md, 4.7).
use super::{
    CheckpointName, files,
    taking::{Cut, OWNER},
};
use crate::{Arrived, Below, CheckpointRefusal, Root, codec::component::Manifest, generations};
use alloc::boxed::Box;
use skein_io::{file, kernel::Kind};
use skein_lib::Map;

const CHECKPOINTS: u32 = 256;
#[derive(Debug)]
enum Step {
    Page,
    Manifest { name: CheckpointName },
    Open,
    Lock { root: Root },
    Remove { root: Root, removing: generations::Removing },
    Sync { root: Root },
    Close,
}
#[derive(Debug)]
pub(crate) struct Retention {
    parent: Root,
    entries: Box<[file::Entry]>,
    next: usize,
    read: u32,
    keep: u32,
    names: Map<(u64, CheckpointName), ()>,
    failure: Option<CheckpointRefusal>,
    step: Step,
}
impl Retention {
    pub fn begin(parent: Root, keep: u32, below: &mut Below) -> Self {
        below.checkpoints.push(file::Request::ListPage {
            owner: OWNER,
            directory: parent.token,
            max: generations::PAGE,
        });
        Self {
            parent,
            entries: Box::new([]),
            next: 0,
            read: 0,
            keep: keep.saturating_sub(1),
            names: Map::with_capacity(CHECKPOINTS),
            failure: None,
            step: Step::Page,
        }
    }
    fn next(&mut self, cut: &mut Cut, below: &mut Below) -> bool {
        for _ in 0..generations::PAGE {
            if self.read == CHECKPOINTS {
                break;
            }
            let Some(entry) = self.entries.get(self.next) else { break };
            self.next = self.next.saturating_add(1);
            let Some(name) = CheckpointName::parse(&entry.name) else { continue };
            if name == cut.name {
                continue;
            }
            self.read = self.read.saturating_add(1);
            if entry.kind != Kind::Directory {
                cut.skipped = cut.skipped.saturating_add(1);
                continue;
            }
            let path = files::join(&files::join(b"checkpoints/", &name.bytes()), b"/generations/1/manifest");
            cut.session.path.clone_from(&path);
            below.checkpoints.push(file::Request::Load {
                owner: OWNER,
                root: cut.session.root.token,
                path,
                max: crate::sizes::MANIFEST_BYTES,
                no_follow: true,
            });
            self.step = Step::Manifest { name };
            return false;
        }
        if self.read < CHECKPOINTS && !self.entries.is_empty() {
            self.entries = Box::new([]);
            below.checkpoints.push(file::Request::ListPage {
                owner: OWNER,
                directory: self.parent.token,
                max: generations::PAGE,
            });
            self.step = Step::Page;
            false
        } else {
            self.remove(cut, below)
        }
    }
    fn remove(&mut self, cut: &mut Cut, below: &mut Below) -> bool {
        if self.names.len() <= self.keep {
            return true;
        }
        let (key, ()) = self.names.iter().next().expect("excess checkpoint");
        let key = *key;
        self.names.remove(&key);
        cut.session.path = files::join(b"checkpoints/", &key.1.bytes());
        below.checkpoints.push(file::Request::OpenDirectory {
            owner: OWNER,
            root: self.parent.fd,
            name: Box::from(key.1.bytes()),
            no_follow: true,
        });
        self.step = Step::Open;
        false
    }
    pub fn up(&mut self, cut: &mut Cut, arrived: Arrived, below: &mut Below) -> Result<bool, CheckpointRefusal> {
        let why = files::failure(&cut.session.path, &arrived.event);
        match core::mem::replace(&mut self.step, Step::Page) {
            Step::Page => {
                let file::Event::Listed { entries, .. } = arrived.event else { return Err(why) };
                self.entries = entries;
                self.next = 0;
                Ok(self.next(cut, below))
            }
            Step::Manifest { name } => {
                let manifest = if let file::Event::Loaded { bytes, .. } = arrived.event {
                    Manifest::decode(&bytes).ok()
                } else {
                    None
                };
                match manifest {
                    Some(manifest) => {
                        self.names.insert((manifest.written, name), ()).expect("CHECKPOINTS read bound");
                    }
                    None => cut.skipped = cut.skipped.saturating_add(1),
                }
                Ok(self.next(cut, below))
            }
            Step::Open => {
                if let file::Event::Failed { error: skein_io::kernel::Error::NotFound, .. } = arrived.event {
                    return Ok(self.remove(cut, below));
                }
                let root = files::directory(cut, arrived)?;
                below.checkpoints.push(file::Request::Lock { owner: OWNER, root: root.token });
                self.step = Step::Lock { root };
                Ok(false)
            }
            Step::Lock { root } => {
                if let file::Event::Busy { .. } = arrived.event {
                    cut.skipped = cut.skipped.saturating_add(1);
                    files::close(cut, root.token, below);
                    self.step = Step::Close;
                } else {
                    let file::Event::Locked { .. } = arrived.event else { return Err(why) };
                    let name = Box::from(cut.session.path.strip_prefix(b"checkpoints/").expect("checkpoint path"));
                    let removing =
                        generations::remove_checkpoint_owned(self.parent, name, OWNER, &mut below.checkpoints);
                    self.step = Step::Remove { root, removing };
                }
                Ok(false)
            }
            Step::Remove { root, mut removing } => {
                if let file::Event::Failed { .. } = &arrived.event
                    && self.failure.is_none()
                {
                    self.failure = Some(why.clone());
                }
                match generations::removed(&mut removing, arrived, &mut below.checkpoints) {
                    generations::Progress::Pending => self.step = Step::Remove { root, removing },
                    generations::Progress::Done => {
                        cut.removed = cut.removed.saturating_add(1);
                        below.checkpoints.push(file::Request::SyncDirectory { owner: OWNER, root: self.parent.fd });
                        self.step = Step::Sync { root };
                    }
                    generations::Progress::Failed => return Err(self.failure.take().unwrap_or(why)),
                }
                Ok(false)
            }
            Step::Sync { root } => {
                let file::Event::Synced { .. } = arrived.event else { return Err(why) };
                files::close(cut, root.token, below);
                self.step = Step::Close;
                Ok(false)
            }
            Step::Close => {
                let file::Event::Closed { .. } = arrived.event else { return Err(why) };
                Ok(self.remove(cut, below))
            }
        }
    }
}
