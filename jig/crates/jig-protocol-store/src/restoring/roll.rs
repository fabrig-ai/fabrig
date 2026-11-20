//! The durable intent's four matching states (protocol/store/restoring.md, 4.7 and 5).
use crate::{Arrived, CheckpointName, Root, component_v3 as wire, generations};
use alloc::boxed::Box;
use skein_codec::Record as _;
use skein_io::{file, kernel::Error};
use skein_lib::{Queue, Reader, Token};

pub(crate) const INTENT: &[u8] = b"restore";
pub(crate) const STAGED: &[u8] = b"transcripts.staged";
pub(crate) const ASIDE: &[u8] = b"transcripts.aside";
const BYTES: u32 = 256;
#[derive(Clone, Copy, Debug)]
pub(crate) struct Intent {
    pub generation: u64,
    pub active: skein_io::digest::Digest,
    pub source: CheckpointName,
    pub safety: CheckpointName,
    pub made: u64,
}
impl Intent {
    pub fn encode(self) -> Box<[u8]> {
        let row = wire::RestoreIntent {
            generation: self.generation,
            active: digest_bytes(self.active),
            source: Box::from(self.source.bytes()),
            safety: Box::from(self.safety.bytes()),
            made: self.made,
        };
        row.encode(&wire::CEILINGS).expect("checkpoint names fit the intent")
    }
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        let row = wire::RestoreIntent::decode(&wire::CEILINGS, &mut Reader::new(bytes)).ok()?;
        if row.generation <= 1 {
            return None;
        }
        Some(Self {
            generation: row.generation,
            active: bytes_digest(row.active),
            source: CheckpointName::parse(&row.source)?,
            safety: CheckpointName::parse(&row.safety)?,
            made: row.made,
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Progress {
    Pending,
    Done,
    Unmatched,
    Failed,
}
#[derive(Debug)]
enum Step {
    Intent,
    Tree(u8),
    TreeClose(u8),
    Active,
    Manifest,
    Rename(u8),
    Sync(u8),
    Replace,
    Remove,
    Aside(generations::Removing),
    Finished,
}
#[derive(Debug)]
pub(crate) struct Rolling {
    root: Root,
    owner: Token,
    intent: Option<Intent>,
    trees: u8,
    step: Step,
    path: Box<[u8]>,
    error: Option<Error>,
}
impl Rolling {
    pub fn path(&self) -> Box<[u8]> {
        self.path.clone()
    }
    pub fn error(&self) -> Option<Error> {
        self.error
    }
    pub fn begin(root: Root, owner: Token, queue: &mut Queue<file::Request>) -> Self {
        queue.push(file::Request::Load {
            owner,
            root: root.token,
            path: Box::from(INTENT),
            max: BYTES,
            no_follow: true,
        });
        Self { root, owner, intent: None, trees: 0, step: Step::Intent, path: Box::from(INTENT), error: None }
    }
    fn tree(&mut self, index: u8, queue: &mut Queue<file::Request>) {
        let name = match index {
            0 => STAGED,
            1 => generations::TRANSCRIPTS,
            2 => ASIDE,
            _ => unreachable!("three trees"),
        };
        self.path = Box::from(name);
        queue.push(file::Request::OpenDirectory {
            owner: self.owner,
            root: self.root.fd,
            name: Box::from(name),
            no_follow: true,
        });
        self.step = Step::Tree(index);
    }
    fn after_tree(&mut self, index: u8, queue: &mut Queue<file::Request>) {
        if index < 2 {
            self.tree(index.saturating_add(1), queue);
        } else {
            self.path = Box::from(generations::ACTIVE);
            queue.push(file::Request::Load {
                owner: self.owner,
                root: self.root.token,
                path: self.path.clone(),
                max: generations::ACTIVE_BYTES,
                no_follow: true,
            });
            self.step = Step::Active;
        }
    }
    fn sync(&mut self, after: u8, queue: &mut Queue<file::Request>) {
        queue.push(file::Request::SyncDirectory { owner: self.owner, root: self.root.fd });
        self.step = Step::Sync(after);
    }
    fn rename(&mut self, index: u8, queue: &mut Queue<file::Request>) {
        let (from, to) =
            if index == 1 { (generations::TRANSCRIPTS, ASIDE) } else { (STAGED, generations::TRANSCRIPTS) };
        self.path = Box::from(from);
        queue.push(file::Request::Rename {
            owner: self.owner,
            from_root: self.root.fd,
            from: Box::from(from),
            to_root: self.root.fd,
            to: Box::from(to),
        });
        self.step = Step::Rename(index);
    }
    fn replace(&mut self, queue: &mut Queue<file::Request>) {
        let intent = self.intent.expect("validated intent");
        self.path = Box::from(generations::ACTIVE);
        queue.push(file::Request::Store {
            owner: self.owner,
            root: self.root.token,
            path: self.path.clone(),
            bytes: generations::encode(intent.generation),
            expected: file::Expect::Digest(intent.active),
            no_follow: true,
        });
        self.step = Step::Replace;
    }
    fn remove(&mut self, queue: &mut Queue<file::Request>) {
        self.path = Box::from(INTENT);
        queue.push(file::Request::Remove {
            owner: self.owner,
            root: self.root.fd,
            name: self.path.clone(),
            directory: false,
        });
        self.step = Step::Remove;
    }
    #[expect(clippy::too_many_lines, reason = "one total intent validation and roll-forward transition table")]
    pub fn arrived(&mut self, arrived: Arrived, queue: &mut Queue<file::Request>) -> Progress {
        if let Some(error) = super::error(&arrived.event) {
            self.error = Some(error);
        }
        match core::mem::replace(&mut self.step, Step::Finished) {
            Step::Intent => {
                if let file::Event::Failed { error: Error::NotFound, .. } = arrived.event {
                    return Progress::Done;
                }
                let file::Event::Loaded { bytes, .. } = arrived.event else {
                    return Progress::Unmatched;
                };
                let Some(intent) = Intent::decode(&bytes) else {
                    return Progress::Unmatched;
                };
                self.intent = Some(intent);
                self.tree(0, queue);
            }
            Step::Tree(index) => {
                if let file::Event::Failed { error: Error::NotFound, .. } = arrived.event {
                    self.error = None;
                    self.after_tree(index, queue);
                } else {
                    let Some(root) = crate::directory::opened(arrived) else {
                        return Progress::Unmatched;
                    };
                    self.trees |= 1_u8.checked_shl(u32::from(index)).expect("three bits");
                    queue.push(file::Request::Close { owner: self.owner, file: root.token });
                    self.step = Step::TreeClose(index);
                }
            }
            Step::TreeClose(index) => {
                let file::Event::Closed { .. } = arrived.event else {
                    return Progress::Failed;
                };
                self.after_tree(index, queue);
            }
            Step::Active => {
                let file::Event::Loaded { bytes, .. } = arrived.event else {
                    return Progress::Unmatched;
                };
                let intent = self.intent.expect("intent read");
                let digest = skein_io::digest::digest(&bytes);
                if digest == intent.active {
                    match self.trees {
                        3 => self.rename(1, queue),
                        5 => self.rename(2, queue),
                        6 => {
                            self.path = super::join(&generations::generation(intent.generation), b"/manifest");
                            queue.push(file::Request::Load {
                                owner: self.owner,
                                root: self.root.token,
                                path: self.path.clone(),
                                max: crate::sizes::MANIFEST_BYTES,
                                no_follow: true,
                            });
                            self.step = Step::Manifest;
                        }
                        _ => return Progress::Unmatched,
                    }
                } else if self.trees == 6 && generations::parse(&bytes) == Some(intent.generation) {
                    self.remove(queue);
                } else {
                    return Progress::Unmatched;
                }
            }
            Step::Manifest => {
                let file::Event::Loaded { bytes, .. } = arrived.event else {
                    return Progress::Unmatched;
                };
                let Ok(manifest) = crate::codec::component::Manifest::decode(&bytes) else {
                    return Progress::Unmatched;
                };
                if manifest.generation != self.intent.expect("intent read").generation {
                    return Progress::Unmatched;
                }
                self.replace(queue);
            }
            Step::Rename(index) => {
                let file::Event::Renamed { .. } = arrived.event else {
                    return Progress::Failed;
                };
                self.sync(index, queue);
            }
            Step::Sync(index) => {
                let file::Event::Synced { .. } = arrived.event else {
                    return Progress::Failed;
                };
                match index {
                    1 => self.rename(2, queue),
                    2 => self.replace(queue),
                    4 => {
                        self.path = Box::from(ASIDE);
                        self.step = Step::Aside(generations::remove_checkpoint_owned(
                            self.root,
                            self.path.clone(),
                            self.owner,
                            queue,
                        ));
                    }
                    5 => return Progress::Done,
                    _ => unreachable!("sync follows a roll-forward step"),
                }
            }
            Step::Replace => {
                let file::Event::Stored { .. } = arrived.event else {
                    return Progress::Failed;
                };
                self.remove(queue);
            }
            Step::Remove => {
                let file::Event::Removed { .. } = arrived.event else {
                    return Progress::Failed;
                };
                self.sync(4, queue);
            }
            Step::Aside(mut removing) => match generations::removed(&mut removing, arrived, queue) {
                generations::Progress::Pending => self.step = Step::Aside(removing),
                generations::Progress::Failed => return Progress::Failed,
                generations::Progress::Done => self.sync(5, queue),
            },
            Step::Finished => return Progress::Failed,
        }
        Progress::Pending
    }
}

fn digest_bytes(digest: skein_io::digest::Digest) -> [u8; 32] {
    let mut bytes = [0; 32];
    for (chunk, word) in bytes.as_chunks_mut::<8>().0.iter_mut().zip(digest.0) {
        *chunk = word.to_be_bytes();
    }
    bytes
}
fn bytes_digest(bytes: [u8; 32]) -> skein_io::digest::Digest {
    let mut words = [0; 4];
    for (word, chunk) in words.iter_mut().zip(bytes.as_chunks::<8>().0) {
        *word = u64::from_be_bytes(*chunk);
    }
    skein_io::digest::Digest(words)
}
