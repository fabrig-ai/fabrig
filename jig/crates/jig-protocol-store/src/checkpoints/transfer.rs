//! One immutable link or bounded prefix copy, shared by taking and restoring (checkpoints.md, 4.5; restoring.md, 4.4–4.5).
use super::files::{CHUNK, error, join};
use crate::{Arrived, Root};
use alloc::boxed::Box;
use skein_io::{file, kernel::Error};
use skein_lib::{List, Queue, Token};

#[derive(Clone, Copy, Debug)]
pub(crate) enum Fault {
    Short,
    Missing,
    File(Error),
}
impl Fault {
    pub fn error(self) -> Option<Error> {
        match self {
            Self::Short => None,
            Self::Missing => Some(Error::NotFound),
            Self::File(error) => Some(error),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum Linking {
    Never,
    Always,
    Exact,
}
#[derive(Debug)]
enum Step {
    Open,
    Stat { reader: Token },
    Source { reader: Token },
    Destination { reader: Token, source: Root },
    Link { reader: Token, source: Root, destination: Root },
    DestinationClose { reader: Token, source: Root, linked: bool },
    SourceClose { reader: Token, linked: bool },
    Create { reader: Token },
    Read { reader: Token, writer: Token, offset: u64 },
    Write { reader: Token, writer: Token, offset: u64, count: u64 },
    Sync,
    Drain,
}
#[derive(Debug)]
pub(crate) struct Transfer {
    source: Root,
    destination: Root,
    from: Box<[u8]>,
    to: Box<[u8]>,
    owner: Token,
    length: Option<u64>,
    linking: Linking,
    missing: bool,
    handles: List<Token>,
    step: Step,
    path: Box<[u8]>,
    failure: Option<Fault>,
    way: u8,
}
impl Transfer {
    pub fn path(&self) -> Box<[u8]> {
        self.path.clone()
    }
    pub fn failure(&self) -> Option<Fault> {
        self.failure
    }
    pub fn way(&self) -> u8 {
        self.way
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "the transfer's two roots and paths, extent and policy are independent inputs"
    )]
    pub fn begin(
        source: Root,
        destination: Root,
        from: Box<[u8]>,
        to: Box<[u8]>,
        length: Option<u64>,
        linking: Linking,
        missing: bool,
        owner: Token,
        queue: &mut Queue<file::Request>,
    ) -> Self {
        queue.push(file::Request::OpenReadNoFollow { owner, root: source.fd, name: from.clone() });
        Self {
            source,
            destination,
            path: from.clone(),
            from,
            to,
            owner,
            length,
            linking,
            missing,
            handles: List::with_capacity(4),
            step: Step::Open,
            failure: None,
            way: 2,
        }
    }
    pub fn length(&self) -> u64 {
        self.length.unwrap_or(0)
    }
    fn opened(&mut self, arrived: &Arrived) -> Option<Token> {
        let file::Event::Opened { file, .. } = &arrived.event else {
            return None;
        };
        let file = *file;
        self.handles.push(file).expect("at most four transfer handles");
        Some(file)
    }
    fn directory(&mut self, arrived: Arrived) -> Option<Root> {
        let token = self.opened(&arrived)?;
        Some(Root { token, fd: arrived.fd.expect("opened directory descriptor") })
    }
    fn close(&mut self, file: Token, queue: &mut Queue<file::Request>) {
        let index = self.handles.len().checked_sub(1).expect("owned handle");
        assert_eq!(self.handles.swap_remove(index), Some(file), "reverse transfer closure");
        queue.push(file::Request::Close { owner: self.owner, file });
    }
    fn drain(&mut self, queue: &mut Queue<file::Request>) -> bool {
        self.step = Step::Drain;
        if let Some(file) = self.handles.iter().last().copied() {
            self.close(file, queue);
            false
        } else {
            true
        }
    }
    fn create(&mut self, reader: Token, queue: &mut Queue<file::Request>) {
        self.path.clone_from(&self.to);
        queue.push(file::Request::CreateNoFollow {
            owner: self.owner,
            root: self.destination.fd,
            name: self.to.clone(),
            mode: 0o600,
        });
        self.step = Step::Create { reader };
    }
    fn read(&mut self, reader: Token, writer: Token, offset: u64, queue: &mut Queue<file::Request>) {
        if offset == self.length() {
            queue.push(file::Request::Sync { owner: self.owner, file: writer });
            self.step = Step::Sync;
        } else {
            self.path.clone_from(&self.from);
            let max = u32::try_from(self.length().saturating_sub(offset).min(u64::from(CHUNK))).expect("chunk bound");
            queue.push(file::Request::ReadAt { owner: self.owner, file: reader, offset, max });
            self.step = Step::Read { reader, writer, offset };
        }
    }
    fn fail(&mut self, error: Option<Error>, queue: &mut Queue<file::Request>) -> bool {
        self.failure = Some(Fault::File(error.unwrap_or(Error::Other(0))));
        self.drain(queue)
    }
    fn short(&mut self, queue: &mut Queue<file::Request>) -> bool {
        self.failure = Some(Fault::Short);
        self.drain(queue)
    }
    fn selected(&mut self, reader: Token, size: u64, queue: &mut Queue<file::Request>) -> bool {
        let length = *self.length.get_or_insert(size);
        if size < length {
            return self.short(queue);
        }
        let link = match self.linking {
            Linking::Never => false,
            Linking::Always => true,
            Linking::Exact => size == length,
        };
        if link {
            let (name, _) = split(&self.from);
            self.path.clone_from(&name);
            queue.push(file::Request::OpenDirectory { owner: self.owner, root: self.source.fd, name, no_follow: true });
            self.step = Step::Source { reader };
        } else {
            self.create(reader, queue);
        }
        false
    }
    #[expect(clippy::too_many_lines, reason = "one total single-flight transfer transition table")]
    pub fn arrived(&mut self, arrived: Arrived, queue: &mut Queue<file::Request>) -> bool {
        let error = error(&arrived.event);
        match core::mem::replace(&mut self.step, Step::Drain) {
            Step::Open => {
                if error == Some(Error::NotFound) && self.missing {
                    self.way = 3;
                    return true;
                }
                if error == Some(Error::NotFound) {
                    self.failure = Some(Fault::Missing);
                    return self.drain(queue);
                }
                let file::Event::Opened { len, .. } = &arrived.event else {
                    return self.fail(error, queue);
                };
                let length = *len;
                let file = self.opened(&arrived).expect("opened event has its file token");
                match self.linking {
                    Linking::Exact => {
                        queue.push(file::Request::Stat { owner: self.owner, file });
                        self.step = Step::Stat { reader: file };
                    }
                    Linking::Always | Linking::Never => return self.selected(file, length, queue),
                }
            }
            Step::Stat { reader } => {
                let file::Event::Stated { stat, .. } = arrived.event else {
                    return self.fail(error, queue);
                };
                return self.selected(reader, stat.size, queue);
            }

            Step::Source { reader } => {
                let Some(source) = self.directory(arrived) else {
                    return self.fail(error, queue);
                };
                let (name, _) = split(&self.to);
                self.path.clone_from(&name);
                queue.push(file::Request::OpenDirectory {
                    owner: self.owner,
                    root: self.destination.fd,
                    name,
                    no_follow: true,
                });
                self.step = Step::Destination { reader, source };
            }
            Step::Destination { reader, source } => {
                let Some(destination) = self.directory(arrived) else {
                    return self.fail(error, queue);
                };
                let (_, from) = split(&self.from);
                let (_, to) = split(&self.to);
                self.path.clone_from(&self.from);
                queue.push(file::Request::HardLink {
                    owner: self.owner,
                    from_root: source.fd,
                    from,
                    to_root: destination.fd,
                    to,
                });
                self.step = Step::Link { reader, source, destination };
            }
            Step::Link { reader, source, destination } => {
                let linked = if let file::Event::Linked { .. } = arrived.event {
                    true
                } else if error == Some(Error::NotFound) && self.missing {
                    self.way = 3;
                    true
                } else {
                    if error == Some(Error::NotFound) {
                        self.failure = Some(Fault::Missing);
                        return self.drain(queue);
                    }
                    match error {
                        Some(Error::Permission | Error::TooManyLinks | Error::Reach(_) | Error::Other(_)) => false,
                        Some(
                            Error::Busy
                            | Error::Refused
                            | Error::Reset
                            | Error::BrokenPipe
                            | Error::NotConnected
                            | Error::AddressInUse
                            | Error::AddressNotAvailable
                            | Error::Unreachable
                            | Error::TimedOut
                            | Error::TooManyOpenFiles
                            | Error::NoBufferSpace
                            | Error::Cancelled
                            | Error::TooLate
                            | Error::Exists
                            | Error::NotADirectory
                            | Error::IsADirectory
                            | Error::NotEmpty
                            | Error::ReadOnly
                            | Error::NoSpace
                            | Error::NameTooLong
                            | Error::Escape
                            | Error::NotAFile
                            | Error::InvalidArgument
                            | Error::NotFound,
                        )
                        | None => return self.fail(error, queue),
                    }
                };
                self.close(destination.token, queue);
                self.step = Step::DestinationClose { reader, source, linked };
            }
            Step::DestinationClose { reader, source, linked } => {
                let file::Event::Closed { .. } = arrived.event else {
                    return self.fail(error, queue);
                };
                self.close(source.token, queue);
                self.step = Step::SourceClose { reader, linked };
            }
            Step::SourceClose { reader, linked } => {
                let file::Event::Closed { .. } = arrived.event else {
                    return self.fail(error, queue);
                };
                if linked {
                    if self.way != 3 {
                        self.way = 1;
                    }
                    return self.drain(queue);
                }
                self.create(reader, queue);
            }
            Step::Create { reader } => {
                let Some(writer) = self.opened(&arrived) else {
                    return self.fail(error, queue);
                };
                self.read(reader, writer, 0, queue);
            }
            Step::Read { reader, writer, offset } => {
                let file::Event::Read { bytes, .. } = arrived.event else {
                    return self.fail(error, queue);
                };
                let count = skein_lib::length::len_u64(bytes.len());
                if count == 0 {
                    return self.short(queue);
                }
                self.path.clone_from(&self.to);
                queue.push(file::Request::WriteAt { owner: self.owner, file: writer, offset, bytes });
                self.step = Step::Write { reader, writer, offset, count };
            }
            Step::Write { reader, writer, offset, count } => {
                let file::Event::Written { .. } = arrived.event else {
                    return self.fail(error, queue);
                };
                self.read(reader, writer, offset.checked_add(count).expect("bounded prefix"), queue);
            }
            Step::Sync => {
                let file::Event::Synced { .. } = arrived.event else {
                    return self.fail(error, queue);
                };
                return self.drain(queue);
            }
            Step::Drain => {
                if self.failure.is_none() {
                    let file::Event::Closed { .. } = arrived.event else {
                        self.failure = Some(Fault::File(error.unwrap_or(Error::Other(0))));
                        return self.drain(queue);
                    };
                }
                return self.drain(queue);
            }
        }
        false
    }
}
fn split(path: &[u8]) -> (Box<[u8]>, Box<[u8]>) {
    let mut split = 0;
    for (index, byte) in path.iter().enumerate() {
        if *byte == b'/' {
            split = index;
        }
    }
    if split == 0 {
        return (Box::from(b".".as_slice()), Box::from(path));
    }
    let (parent, leaf) = path.split_at_checked(split).expect("slash found within path");
    (Box::from(parent), join(b"", leaf.strip_prefix(b"/").expect("split slash")))
}
