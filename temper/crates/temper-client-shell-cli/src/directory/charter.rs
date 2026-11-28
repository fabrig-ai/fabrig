//! Local charter copies (temper/docs/design/client/ctl/tools-reads.md, 1).
use super::{Directory, OWNER, Stage};
use skein_io::{file, kernel::Error};
use skein_json::writer::{Encoder, Limits as JsonLimits};
use skein_lib::Token;
use temper_client_tools::jig::{FileEnd, Printed, runner::FileCopy};

const FOLDER: &[u8] = b".temper/charters/";
const PATH_BYTES: usize = 4096; // The file layer's path bound.

pub(crate) struct Destination {
    pub(crate) anchor: Box<[u8]>,
    // Reversed so the next directory is the last element.
    components: Vec<Box<[u8]>>,
}
impl Destination {
    pub(crate) fn new(into: Option<&[u8]>) -> Option<Self> {
        let path = into.unwrap_or(b".");
        if path.len() >= PATH_BYTES || path.contains(&0) {
            return None;
        }
        let absolute = path.starts_with(b"/");
        let mut anchor = if absolute { b"/".to_vec() } else { b".".to_vec() };
        let mut components = Vec::<Box<[u8]>>::new();
        for name in path.split(|byte| *byte == b'/') {
            match name {
                b"" | b"." => {}
                b".." => {
                    if components.pop().is_none() && !absolute {
                        anchor.extend_from_slice(b"/..");
                    }
                }
                name => components.push(name.into()),
            }
        }
        components.push(b".temper".as_slice().into());
        components.push(b"charters".as_slice().into());
        components.reverse();
        Some(Self { anchor: anchor.into_boxed_slice(), components })
    }

    fn path(&self, leaf: &[u8]) -> Vec<u8> {
        // Print the normalized path, not a spelling whose cancelled components may not exist.
        let mut path = if self.anchor.as_ref() == b"/" {
            b"/".to_vec()
        } else if let Some(parents) = self.anchor.strip_prefix(b"./") {
            let mut path = parents.to_vec();
            path.push(b'/');
            path
        } else {
            Vec::new()
        };
        for name in self.components.iter().rev() {
            path.extend_from_slice(name);
            path.push(b'/');
        }
        path.extend_from_slice(leaf);
        path
    }
}

#[derive(Clone, Copy)]
enum Step {
    Open,
    Make,
    Reopen,
    CloseParent,
    Store,
}

pub(super) struct Writing {
    components: Vec<Box<[u8]>>,
    current: Option<Token>,
    previous: Option<Token>,
    step: Step,
    leaf: Box<[u8]>,
    bytes: Box<[u8]>,
    written: Option<Written>,
}

pub(crate) struct Written {
    path: Box<[u8]>,
    len: u64,
    json: bool,
}
impl Written {
    pub(crate) fn printed(self) -> Printed {
        let stdout = if self.json {
            let limits = JsonLimits { depth: 2, length: 32_768 };
            let mut measure = Encoder::measure(&limits);
            self.encode(&mut measure);
            let mut write = Encoder::write(measure.measured().expect("bounded copy output"), &limits);
            self.encode(&mut write);
            write.finish()
        } else {
            let mut line = self.path.into_vec();
            line.extend_from_slice(format!(" ({} bytes)\n", self.len).as_bytes());
            line.into_boxed_slice()
        };
        Printed { file: FileEnd::Keep, status: 0, stdout, stderr: Box::default() }
    }
    fn encode(&self, out: &mut Encoder) {
        out.object_start();
        out.key(b"path");
        out.string(&self.path);
        out.key(b"bytes");
        out.number(skein_lib::Decimal::of(self.len).as_bytes());
        out.object_end();
    }
}

impl Directory {
    pub(crate) fn write_charter(&mut self, copy: FileCopy) {
        let mut leaf = copy.name.into_vec();
        leaf.extend_from_slice(b".md");
        let Some(destination) = Destination::new(copy.into.as_deref()) else {
            self.fail(b"charter copy: invalid destination path");
            return;
        };
        if self.file_root.is_none() {
            self.fail(b"charter copy: destination has no directory root");
            return;
        }
        let path = destination.path(&leaf);
        if path.len() >= PATH_BYTES || copy.json && !skein_lib::bytes::is_utf8(&path) {
            self.fail(b"charter copy: invalid destination path");
            return;
        }
        let written =
            Written { path: path.into_boxed_slice(), len: skein_lib::len_u64(copy.bytes.len()), json: copy.json };
        self.charter = Some(Writing {
            components: destination.components,
            current: None,
            previous: None,
            step: Step::Open,
            leaf: leaf.into_boxed_slice(),
            bytes: copy.bytes,
            written: Some(written),
        });
        self.stage = Stage::Charter;
    }

    pub(super) fn charter_request(&mut self) -> file::Request {
        let writing = self.charter.as_mut().expect("charter in progress");
        let root = writing.current.or(self.file_root).expect("charter directory");
        match writing.step {
            Step::Open | Step::Reopen => file::Request::OpenDirectory {
                owner: OWNER,
                root: self.io.descriptor(root).expect("open directory"),
                name: writing.components.last().expect("next directory").clone(),
                no_follow: true,
            },
            Step::Make => file::Request::MakeDirectory {
                owner: OWNER,
                root,
                path: writing.components.last().expect("directory to make").clone(),
                mode: 0o777,
            },
            Step::CloseParent => {
                file::Request::Close { owner: OWNER, file: writing.previous.take().expect("previous directory") }
            }
            Step::Store => file::Request::Store {
                owner: OWNER,
                root,
                path: writing.leaf.clone(),
                bytes: core::mem::take(&mut writing.bytes),
                expected: file::Expect::Absent,
                no_follow: true,
            },
        }
    }

    pub(super) fn charter_answer(&mut self, event: file::Event) {
        let writing = self.charter.as_mut().expect("charter answer");
        if let file::Event::Opened { file, .. } = event {
            writing.previous = writing.current.replace(file);
            writing.components.pop();
            writing.step = if writing.previous.is_some() { Step::CloseParent } else { writing.next() };
        } else if !self.closing {
            match (writing.step, event) {
                (Step::Open, file::Event::Failed { error: Error::NotFound, .. }) => writing.step = Step::Make,
                (Step::Make, file::Event::Made { .. } | file::Event::Failed { error: Error::Exists, .. }) => {
                    writing.step = Step::Reopen;
                }
                (Step::CloseParent, file::Event::Closed { .. }) => writing.step = writing.next(),
                (Step::Store, file::Event::Stored { .. }) => {
                    self.stage = Stage::Ready;
                    self.charter_written = writing.written.take();
                }
                (Step::Store, file::Event::Conflict { .. }) => {
                    let words =
                        [FOLDER, &writing.leaf, b" exists; charter copy never writes over a charter's file"].concat();
                    self.fail(&words);
                }
                (_, event) => {
                    let reason = super::answer_words(&event);
                    let written = writing.written.as_ref().expect("copy has not finished");
                    let words = [written.path.as_ref(), b": charter copy: ", reason.as_bytes()].concat();
                    self.fail(&words);
                }
            }
        }
    }

    pub(super) fn close_charter(&mut self) -> Option<Token> {
        let writing = self.charter.as_mut()?;
        writing.current.take().or_else(|| writing.previous.take())
    }

    pub(crate) fn writing_charter(&self) -> bool {
        self.stage == Stage::Charter
    }
}
impl Writing {
    fn next(&self) -> Step {
        if self.components.is_empty() { Step::Store } else { Step::Open }
    }
}
