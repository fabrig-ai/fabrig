//! Check the job's own hook without following links (temper/docs/design/protocol/hooks.md, 2.2 and 4.3).
use alloc::boxed::Box;
use skein_io::kernel::{self, Complete, Done, Fd, Kind, Op, OpenHow, Spawn};
use skein_lib::{Time, Token, bytes};
use temper_worker_domain::tooling::{Exit, hook_failure, hook_link_target};

#[derive(Debug)]
pub(super) struct Check {
    pub(super) op: Token,
    pub(super) cancel: Option<Token>,
    pub(super) deadline: Option<Time>,
    stopped: Option<Exit>,
    spawn: Spawn,
    hook: Box<[u8]>,
    phase: Phase,
}

#[derive(Debug)]
enum Phase {
    OpenDirectory { name: Box<[u8]> },
    ReadLink { dir: Fd },
    OpenFile { dir: Fd, linked: bool },
    Stat { dir: Fd, file: Fd, linked: bool },
    CloseFile { dir: Fd, why: Option<Box<[u8]>> },
    CloseDirectory { why: Option<Box<[u8]>> },
    Ended { why: Option<Box<[u8]>> },
}

pub(super) enum End {
    Run { spawn: Spawn, deadline: Time },
    Failed { exit: Exit, tail: Box<[u8]> },
}

impl Check {
    pub(super) fn start(
        op: Token,
        deadline: Time,
        spawn: Spawn,
        program: &[u8],
        hook: &[u8],
    ) -> Result<(Self, Op), Box<[u8]>> {
        let mut slash = None;
        for (index, byte) in program.iter().enumerate() {
            if *byte == b'/' {
                slash = Some(index);
            }
        }
        let (parent, name) =
            program.split_at_checked(slash.expect("a hook has a parent directory")).expect("slash in path");
        let name = name.strip_prefix(b"/").expect("the last slash");
        if !kernel::is_name(name) {
            return Err(hook_failure(hook, b"not a regular file"));
        }
        let open = Op::Open { root: spawn.root, path: parent.into(), how: OpenHow::DirectoryNoFollow };
        Ok((
            Self {
                op,
                cancel: None,
                deadline: Some(deadline),
                stopped: None,
                spawn,
                hook: hook.into(),
                phase: Phase::OpenDirectory { name: name.into() },
            },
            open,
        ))
    }

    /// Only opens can stall. Everything else drains to its terminal, closing both descriptors if open.
    pub(super) fn stop(&mut self, exit: Exit) -> Option<Token> {
        if self.stopped.is_some() {
            return None;
        }
        self.stopped = Some(exit);
        self.deadline = None;
        match &self.phase {
            Phase::OpenDirectory { .. } | Phase::OpenFile { .. } => Some(self.op),
            Phase::ReadLink { .. }
            | Phase::Stat { .. }
            | Phase::CloseFile { .. }
            | Phase::CloseDirectory { .. }
            | Phase::Ended { .. } => None,
        }
    }

    pub(super) fn ended(&self) -> bool {
        self.cancel.is_none()
            && match &self.phase {
                Phase::Ended { .. } => true,
                Phase::OpenDirectory { .. }
                | Phase::ReadLink { .. }
                | Phase::OpenFile { .. }
                | Phase::Stat { .. }
                | Phase::CloseFile { .. }
                | Phase::CloseDirectory { .. } => false,
            }
    }

    pub(super) fn end(self) -> End {
        if let Some(exit) = self.stopped {
            return End::Failed { exit, tail: Box::new([]) };
        }
        match self.phase {
            Phase::Ended { why: Some(why) } => End::Failed { exit: Exit::NotRun, tail: hook_failure(&self.hook, &why) },
            Phase::Ended { why: None } => End::Run { spawn: self.spawn, deadline: self.deadline.expect("not stopped") },
            Phase::OpenDirectory { .. }
            | Phase::ReadLink { .. }
            | Phase::OpenFile { .. }
            | Phase::Stat { .. }
            | Phase::CloseFile { .. }
            | Phase::CloseDirectory { .. } => unreachable!("the hook check drained"),
        }
    }

    pub(super) fn complete(&mut self, answer: Complete) -> Option<Op> {
        // The default is to finish; an open descriptor must choose a close or a further step.
        match core::mem::replace(&mut self.phase, Phase::Ended { why: None }) {
            Phase::OpenDirectory { name } => match answer.result {
                Ok(Done::Fd(dir)) if self.stopped.is_some() => Some(self.close_directory(dir, None)),
                Ok(Done::Fd(dir)) => {
                    self.phase = Phase::ReadLink { dir };
                    Some(Op::ReadLink { dir, name, buf: bytes::zeroed(kernel::LINK_BYTES) })
                }
                Err(error) => {
                    self.phase = Phase::Ended { why: Some(open_failed(error).into()) };
                    None
                }
                Ok(_) => unreachable!("directory open answers a descriptor"),
            },
            Phase::ReadLink { dir } => {
                if self.stopped.is_some() {
                    return Some(self.close_directory(dir, None));
                }
                let (name, buf) = read_link_buffers(answer.kind);
                match answer.result {
                    Ok(Done::Count(count)) => {
                        let target = buf.get(..skein_lib::usize_of(count)).expect("readlink count fits");
                        if !kernel::is_name(target) || target.len() > kernel::LONGEST_NAME {
                            let why = hook_link_target(target);
                            return Some(self.close_directory(dir, Some(why)));
                        }
                        Some(self.open_file(dir, target.into(), true))
                    }
                    Err(kernel::Error::NotALink) => Some(self.open_file(dir, name, false)),
                    Err(error) => Some(self.close_directory(dir, Some(open_failed(error).into()))),
                    Ok(_) => unreachable!("readlink answers a count"),
                }
            }
            Phase::OpenFile { dir, linked } => match answer.result {
                Ok(Done::Fd(file)) if self.stopped.is_some() => Some(self.close_file(dir, file, None)),
                Ok(Done::Fd(file)) => {
                    self.phase = Phase::Stat { dir, file, linked };
                    Some(Op::Stat { fd: file })
                }
                Err(kernel::Error::NotFound) if linked => {
                    Some(self.close_directory(dir, Some(b"link target is missing".as_slice().into())))
                }
                Err(error) => Some(self.close_directory(dir, Some(open_failed(error).into()))),
                Ok(_) => unreachable!("hook open answers a descriptor"),
            },
            Phase::Stat { dir, file, linked } => {
                let why = match answer.result {
                    Ok(Done::Stat(stat)) => match stat.kind {
                        Kind::File if stat.mode & 0o111 != 0 => None,
                        Kind::File if linked => Some(b"link target is not executable".as_slice().into()),
                        Kind::File => Some(b"not executable".as_slice().into()),
                        Kind::Symlink if linked => Some(b"link target is another link".as_slice().into()),
                        Kind::Directory | Kind::Other | Kind::Symlink => Some(b"not a regular file".as_slice().into()),
                    },
                    Err(_) => Some(b"could not stat the file".as_slice().into()),
                    Ok(_) => unreachable!("stat answers metadata"),
                };
                Some(self.close_file(dir, file, why))
            }
            Phase::CloseFile { dir, why } => Some(self.close_directory(dir, closed(why, &answer))),
            Phase::CloseDirectory { why } => {
                self.phase = Phase::Ended { why: closed(why, &answer) };
                None
            }
            Phase::Ended { .. } => unreachable!("a drained hook has no file operation"),
        }
    }

    fn open_file(&mut self, dir: Fd, name: Box<[u8]>, linked: bool) -> Op {
        self.phase = Phase::OpenFile { dir, linked };
        Op::Open { root: dir, path: name, how: OpenHow::PathNoFollow }
    }

    fn close_file(&mut self, dir: Fd, file: Fd, why: Option<Box<[u8]>>) -> Op {
        self.phase = Phase::CloseFile { dir, why };
        Op::Close { fd: file }
    }

    fn close_directory(&mut self, dir: Fd, why: Option<Box<[u8]>>) -> Op {
        self.phase = Phase::CloseDirectory { why };
        Op::Close { fd: dir }
    }
}

fn closed(why: Option<Box<[u8]>>, answer: &Complete) -> Option<Box<[u8]>> {
    if why.is_none() && answer.result.is_err() {
        Some(b"could not close the file check".as_slice().into())
    } else {
        why
    }
}

fn open_failed(error: kernel::Error) -> &'static [u8] {
    match error {
        kernel::Error::NotFound => b"missing",
        kernel::Error::NotADirectory => b"parent is not a directory",
        kernel::Error::TooManyLinks => b"parent is a symbolic link",
        kernel::Error::Permission => b"permission denied",
        kernel::Error::NameTooLong => b"name too long",
        kernel::Error::Escape
        | kernel::Error::Refused
        | kernel::Error::Reset
        | kernel::Error::BrokenPipe
        | kernel::Error::NotConnected
        | kernel::Error::AddressInUse
        | kernel::Error::AddressNotAvailable
        | kernel::Error::Unreachable
        | kernel::Error::TimedOut
        | kernel::Error::TooManyOpenFiles
        | kernel::Error::NoBufferSpace
        | kernel::Error::Cancelled
        | kernel::Error::TooLate
        | kernel::Error::Exists
        | kernel::Error::IsADirectory
        | kernel::Error::NotEmpty
        | kernel::Error::NoSpace
        | kernel::Error::ReadOnly
        | kernel::Error::Busy
        | kernel::Error::NotAFile
        | kernel::Error::NotALink
        | kernel::Error::InvalidArgument
        | kernel::Error::Reach(_)
        | kernel::Error::Other(_) => b"could not inspect the file",
    }
}

fn read_link_buffers(op: Op) -> (Box<[u8]>, Box<[u8]>) {
    match op {
        Op::ReadLink { name, buf, .. } => (name, buf),
        Op::Socket { .. }
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
        | Op::Lock { .. }
        | Op::Stat { .. }
        | Op::Rename { .. }
        | Op::HardLink { .. }
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
        | Op::Cancel { .. } => unreachable!("readlink returns its operation"),
    }
}
