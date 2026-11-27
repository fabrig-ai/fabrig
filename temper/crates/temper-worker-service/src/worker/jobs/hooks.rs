//! Find, check and time the job's own hook (temper/docs/design/protocol/hooks.md, 2.2, 2.3 and 4.3).
use alloc::boxed::Box;
use skein_io::kernel::{self, Complete, Done, Fd, Kind, Op, OpenHow, Spawn};
use skein_lib::{Buffer, Time, Token};
use temper_worker_domain::tooling::{CONFIG, CONFIG_BYTES, Conflict, Exit, Fault, Hooks, hook_failure, times};

#[derive(Debug)]
pub(super) struct Check {
    pub(super) op: Token,
    pub(super) cancel: Option<Token>,
    pub(super) deadline: Option<Time>,
    stopped: Option<Exit>,
    spawn: Spawn,
    hook: Box<[u8]>,
    config_path: Option<Box<[u8]>>,
    hooks: Hooks,
    phase: Phase,
}

#[derive(Debug)]
enum Phase {
    OpenDirectory { name: Box<[u8]> },
    OpenFile { dir: Fd },
    Stat { dir: Fd, file: Fd },
    CloseFile { dir: Fd, why: Option<Box<[u8]>> },
    OpenConfig { dir: Fd },
    StatConfig { dir: Fd, file: Fd },
    ReadConfig { dir: Fd, file: Fd, bytes: Buffer },
    CloseConfig { dir: Fd, outcome: Outcome },
    CloseDirectory { outcome: Outcome },
    Ended(Outcome),
}

#[derive(Debug)]
enum Outcome {
    Hook(Option<u32>),
    Fallback,
    Absent,
    Failed(Box<[u8]>),
}

pub(super) enum End {
    Run { spawn: Spawn, deadline: Time, time: CommandTime },
    NotRun { exit: Exit, tail: Box<[u8]> },
}

pub(super) enum CommandTime {
    Hook { path: Box<[u8]>, asked: Option<u32>, ceiling: u32 },
    Fallback { ceiling: u32 },
}

const CONFIG_NAME: &[u8] = b"config.toml";
const CONFIG_READ: usize = CONFIG_BYTES + 1; // Read one extra byte to detect growth after the stat.

impl Check {
    pub(super) fn start(
        op: Token,
        deadline: Time,
        spawn: Spawn,
        program: &[u8],
        hook: &[u8],
        hooks: Hooks,
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
        if name.len() > kernel::LONGEST_NAME {
            return Err(hook_failure(hook, b"name too long"));
        }
        let hook_directory = CONFIG.strip_suffix(CONFIG_NAME).expect("time file's directory");
        let config_path = if hook.strip_prefix(hook_directory) == Some(name) {
            None
        } else {
            // An older engine can name a command deeper inside the hooks directory.
            let repository = program.strip_suffix(hook).expect("repository-relative hook");
            Some(super::joined(&[repository, CONFIG]))
        };
        let open = Op::Open { root: spawn.root, path: parent.into(), how: OpenHow::DirectoryNoFollow };
        Ok((
            Self {
                op,
                cancel: None,
                deadline: Some(deadline),
                stopped: None,
                spawn,
                hook: hook.into(),
                config_path,
                hooks,
                phase: Phase::OpenDirectory { name: name.into() },
            },
            open,
        ))
    }

    /// Cancel the pending open or read; the remaining operations drain their descriptors.
    pub(super) fn stop(&mut self, exit: Exit) -> Option<Token> {
        if self.stopped.is_some() {
            return None;
        }
        self.stopped = Some(exit);
        self.deadline = None;
        match &self.phase {
            Phase::OpenDirectory { .. }
            | Phase::OpenFile { .. }
            | Phase::OpenConfig { .. }
            | Phase::ReadConfig { .. } => Some(self.op),
            Phase::Stat { .. }
            | Phase::CloseFile { .. }
            | Phase::StatConfig { .. }
            | Phase::CloseConfig { .. }
            | Phase::CloseDirectory { .. }
            | Phase::Ended(_) => None,
        }
    }

    pub(super) fn ended(&self) -> bool {
        self.cancel.is_none()
            && match &self.phase {
                Phase::Ended(_) => true,
                Phase::OpenDirectory { .. }
                | Phase::OpenFile { .. }
                | Phase::Stat { .. }
                | Phase::CloseFile { .. }
                | Phase::OpenConfig { .. }
                | Phase::StatConfig { .. }
                | Phase::ReadConfig { .. }
                | Phase::CloseConfig { .. }
                | Phase::CloseDirectory { .. } => false,
            }
    }

    pub(super) fn end(mut self, worker_path: &[u8]) -> End {
        if let Some(exit) = self.stopped {
            return End::NotRun { exit, tail: Box::new([]) };
        }
        let Phase::Ended(outcome) = self.phase else { unreachable!("the hook check drained") };
        let time = match outcome {
            Outcome::Hook(asked) => CommandTime::Hook { path: self.hook, asked, ceiling: self.hooks.ceiling },
            Outcome::Fallback => {
                let program = self.hooks.fallback.take().expect("discovery has a fallback");
                self.spawn.program = super::joined(&[worker_path, b"/", &program.program]);
                CommandTime::Fallback { ceiling: self.hooks.ceiling }
            }
            Outcome::Absent => return End::NotRun { exit: Exit::Absent, tail: Box::new([]) },
            Outcome::Failed(tail) => return End::NotRun { exit: Exit::NotRun, tail },
        };
        End::Run { spawn: self.spawn, deadline: self.deadline.expect("not stopped"), time }
    }

    pub(super) fn complete(&mut self, answer: Complete) -> Option<Op> {
        // The default is to finish; an open descriptor must choose a close or a further step.
        match core::mem::replace(&mut self.phase, Phase::Ended(Outcome::Absent)) {
            Phase::OpenDirectory { name } => match answer.result {
                Ok(Done::Fd(dir)) if self.stopped.is_some() => Some(self.close_directory(dir, Outcome::Absent)),
                Ok(Done::Fd(dir)) => {
                    self.phase = Phase::OpenFile { dir };
                    Some(Op::Open { root: dir, path: name, how: OpenHow::PathSiblingFile })
                }
                Err(error) => {
                    let outcome = if self.hooks.fallback.is_some() && no_directory(error) {
                        if self.hooks.conflict == Some(Conflict::Path) {
                            Outcome::Failed(hook_failure(&self.hook, Conflict::Path.why()))
                        } else {
                            self.fallback()
                        }
                    } else {
                        Outcome::Failed(hook_failure(&self.hook, open_failed(error, true)))
                    };
                    self.phase = Phase::Ended(outcome);
                    None
                }
                Ok(_) => unreachable!("directory open answers a descriptor"),
            },
            Phase::OpenFile { dir } => match answer.result {
                Ok(Done::Fd(file)) if self.stopped.is_some() => Some(self.close_file(dir, file, None)),
                Ok(Done::Fd(file)) => {
                    self.phase = Phase::Stat { dir, file };
                    Some(Op::Stat { fd: file })
                }
                Err(kernel::Error::NotFound)
                    if self.hooks.fallback.is_some() && self.hooks.conflict != Some(Conflict::Path) =>
                {
                    Some(self.close_directory(dir, Outcome::Absent))
                }
                Err(error) => {
                    let why = if self.hooks.conflict == Some(Conflict::Path) {
                        Conflict::Path.why()
                    } else {
                        open_failed(error, false)
                    };
                    let failure = Outcome::Failed(hook_failure(&self.hook, why));
                    Some(self.close_directory(dir, failure))
                }
                Ok(_) => unreachable!("hook open answers a descriptor"),
            },
            Phase::Stat { dir, file } => {
                let why = match answer.result {
                    Ok(Done::Stat(stat)) => match stat.kind {
                        Kind::File if stat.mode & 0o111 != 0 => None,
                        Kind::File => Some(b"not executable".as_slice().into()),
                        Kind::Directory | Kind::Other | Kind::Symlink => Some(b"not a regular file".as_slice().into()),
                    },
                    Err(_) => Some(b"could not stat the file".as_slice().into()),
                    Ok(_) => unreachable!("stat answers metadata"),
                };
                Some(self.close_file(dir, file, why))
            }
            Phase::CloseFile { dir, why } => {
                if self.stopped.is_some() {
                    Some(self.close_directory(dir, Outcome::Absent))
                } else if let Some(why) = closed(why, &answer) {
                    let failure = Outcome::Failed(hook_failure(&self.hook, &why));
                    Some(self.close_directory(dir, failure))
                } else if let Some(conflict) = self.hooks.conflict {
                    let failure = Outcome::Failed(hook_failure(&self.hook, conflict.why()));
                    Some(self.close_directory(dir, failure))
                } else {
                    let (root, path) = match self.config_path.take() {
                        Some(path) => (self.spawn.root, path),
                        None => (dir, CONFIG_NAME.into()),
                    };
                    self.phase = Phase::OpenConfig { dir };
                    Some(Op::Open { root, path, how: OpenHow::ReadNoFollow })
                }
            }
            Phase::CloseDirectory { outcome } => {
                self.phase = Phase::Ended(if answer.result.is_err() && !failed(&outcome) {
                    Outcome::Failed(hook_failure(&self.hook, b"could not close the file check"))
                } else {
                    outcome
                });
                None
            }
            phase @ (Phase::OpenConfig { .. }
            | Phase::StatConfig { .. }
            | Phase::ReadConfig { .. }
            | Phase::CloseConfig { .. }) => Some(self.config_complete(phase, answer)),
            Phase::Ended(_) => unreachable!("a drained hook has no file operation"),
        }
    }

    fn fallback(&self) -> Outcome {
        match &self.hooks.fallback {
            Some(fallback) if !fallback.program.is_empty() => match fallback.conflict {
                Some(conflict) => Outcome::Failed(super::joined(&[b"fallback command: ", conflict.why()])),
                None => Outcome::Fallback,
            },
            Some(_) | None => Outcome::Absent,
        }
    }

    fn config_complete(&mut self, phase: Phase, answer: Complete) -> Op {
        match phase {
            Phase::OpenConfig { dir } => match answer.result {
                Ok(Done::Fd(file)) if self.stopped.is_some() => self.close_config(dir, file, Outcome::Absent),
                Ok(Done::Fd(file)) => {
                    self.phase = Phase::StatConfig { dir, file };
                    Op::Stat { fd: file }
                }
                Err(kernel::Error::NotFound) => self.close_directory(dir, Outcome::Hook(None)),
                Err(kernel::Error::TooManyLinks | kernel::Error::NotAFile | kernel::Error::IsADirectory) => {
                    self.close_directory(dir, config_fault(b"not a regular file"))
                }
                Err(_) => self.close_directory(dir, config_fault(b"could not open the file")),
                Ok(_) => unreachable!("time file open answers a descriptor"),
            },
            Phase::StatConfig { dir, file } => {
                let outcome = if self.stopped.is_some() {
                    Outcome::Absent
                } else {
                    match answer.result {
                        Ok(Done::Stat(stat)) if stat.kind != Kind::File => config_fault(b"not a regular file"),
                        Ok(Done::Stat(stat)) if stat.size > u64::try_from(CONFIG_BYTES).expect("time file bound") => {
                            config_fault(b"file too large")
                        }
                        Ok(Done::Stat(_)) => {
                            return self.read_config(dir, file, Buffer::with_capacity(config_read_bound()));
                        }
                        Err(_) => config_fault(b"could not stat the file"),
                        Ok(_) => unreachable!("stat answers metadata"),
                    }
                };
                self.close_config(dir, file, outcome)
            }
            Phase::ReadConfig { dir, file, mut bytes } => {
                let outcome = if self.stopped.is_some() {
                    Outcome::Absent
                } else {
                    match answer.result {
                        Ok(Done::Count(0)) => match times(bytes.as_slice()) {
                            Ok(value) => {
                                let verb = match self.spawn.args.first() {
                                    Some(verb) => verb.as_ref(),
                                    None => b"",
                                };
                                Outcome::Hook(match verb {
                                    b"merge" => value.merge,
                                    b"close" => value.close,
                                    b"deliver" => value.deliver,
                                    b"finish" => value.finish,
                                    _ => None,
                                })
                            }
                            Err(fault) => Outcome::Failed(fault.text()),
                        },
                        Ok(Done::Count(count)) => {
                            let Op::Read { buf, .. } = answer.kind else { unreachable!("time file read") };
                            let part = buf.get(..skein_lib::usize_of(count)).expect("read count");
                            bytes.put(part).expect("read within the buffer's free room");
                            if bytes.as_slice().len() > CONFIG_BYTES {
                                config_fault(b"file too large")
                            } else {
                                return self.read_config(dir, file, bytes);
                            }
                        }
                        Err(_) => config_fault(b"could not read the file"),
                        Ok(_) => unreachable!("read answers a count"),
                    }
                };
                self.close_config(dir, file, outcome)
            }
            Phase::CloseConfig { dir, outcome } => {
                let outcome = if answer.result.is_err() && !failed(&outcome) {
                    config_fault(b"could not close the file")
                } else {
                    outcome
                };
                self.close_directory(dir, outcome)
            }
            Phase::OpenDirectory { .. }
            | Phase::OpenFile { .. }
            | Phase::Stat { .. }
            | Phase::CloseFile { .. }
            | Phase::CloseDirectory { .. }
            | Phase::Ended(_) => unreachable!("not a time file operation"),
        }
    }

    fn read_config(&mut self, dir: Fd, file: Fd, bytes: Buffer) -> Op {
        let at = bytes.as_slice().len();
        let left = CONFIG_READ.checked_sub(at).expect("bounded read");
        self.phase = Phase::ReadConfig { dir, file, bytes };
        Op::Read { fd: file, buf: skein_lib::bytes::zeroed(left), at: u64::try_from(at).expect("time file offset") }
    }

    fn close_config(&mut self, dir: Fd, file: Fd, outcome: Outcome) -> Op {
        self.phase = Phase::CloseConfig { dir, outcome };
        Op::Close { fd: file }
    }

    fn close_file(&mut self, dir: Fd, file: Fd, why: Option<Box<[u8]>>) -> Op {
        self.phase = Phase::CloseFile { dir, why };
        Op::Close { fd: file }
    }

    fn close_directory(&mut self, dir: Fd, outcome: Outcome) -> Op {
        self.phase = Phase::CloseDirectory { outcome };
        Op::Close { fd: dir }
    }
}

fn config_read_bound() -> u32 {
    u32::try_from(CONFIG_READ).expect("time file bound")
}

fn config_fault(why: &[u8]) -> Outcome {
    Outcome::Failed(Fault { line: 0, why: why.into() }.text())
}

fn failed(outcome: &Outcome) -> bool {
    match outcome {
        Outcome::Failed(_) => true,
        Outcome::Hook(_) | Outcome::Fallback | Outcome::Absent => false,
    }
}

fn no_directory(error: kernel::Error) -> bool {
    error == kernel::Error::NotFound || error == kernel::Error::NotADirectory || error == kernel::Error::TooManyLinks
}

fn closed(why: Option<Box<[u8]>>, answer: &Complete) -> Option<Box<[u8]>> {
    if why.is_none() && answer.result.is_err() {
        Some(b"could not close the file check".as_slice().into())
    } else {
        why
    }
}

fn open_failed(error: kernel::Error, parent: bool) -> &'static [u8] {
    match error {
        kernel::Error::NotFound => b"missing",
        kernel::Error::NotADirectory => b"parent is not a directory",
        kernel::Error::TooManyLinks if parent => b"parent is a symbolic link",
        kernel::Error::TooManyLinks => b"a link that names no regular file beside it",
        kernel::Error::NotAFile | kernel::Error::IsADirectory => b"not a regular file",
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
        | kernel::Error::NotEmpty
        | kernel::Error::NoSpace
        | kernel::Error::ReadOnly
        | kernel::Error::Busy
        | kernel::Error::InvalidArgument
        | kernel::Error::Reach(_)
        | kernel::Error::Other(_) => b"could not inspect the file",
    }
}
