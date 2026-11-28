//! The worker reads hook times in the held checkout before sending an agent start
//! (temper/docs/design/protocol/hooks.md, 2.3 and 4.1). Only one directory and
//! its bounded time file are open at a time. The reader keeps no credentials.

use super::HOOK_CONFIG_READER;
use alloc::boxed::Box;
use skein_io::{file, file_layer, kernel};
use skein_lib::{Decimal, Duration, Env, Map, Queue, Time, Token, Wall, bytes};
use smith_host_domain::{self as smith, HookTime, HookTimes};
use temper_worker_domain::tooling::hooks::{self, Fault};

// Within each worker's file-operation lane, separate these from source inspection.
const TAG: u64 = 1 << 47;
const SEQUENCE: u64 = TAG - 1;

#[derive(Debug)]
pub(super) struct Reader {
    files: file_layer::FileIo,
    events: Queue<file::Event>,
    operations: Queue<kernel::Submit>,
    waiting: Map<Token, Reading>,
    active: Option<Reading>,
}

#[derive(Debug)]
struct Reading {
    ready: Ready,
    ceiling: Duration,
    next: usize,
    phase: Phase,
    abandoned: bool,
}

#[derive(Debug)]
pub(super) struct Ready {
    pub(super) owner: Token,
    pub(super) start: smith::Start,
    pub(super) window: smith::Window,
}

#[derive(Debug)]
enum Phase {
    Next,
    Opening,
    Loading { directory: Token },
    Closing,
}

impl Reader {
    pub(super) fn new(slots: u32) -> Self {
        Self {
            files: file_layer::FileIo::with_limits(&limits()),
            events: Queue::uncapped(),
            operations: Queue::uncapped(),
            waiting: Map::with_capacity(slots),
            active: None,
        }
    }

    pub(super) fn start(&mut self, ready: Ready, ceiling: u32) {
        let prior = self
            .waiting
            .insert(
                ready.owner,
                Reading {
                    ready,
                    ceiling: Duration::from_secs(u64::from(ceiling)),
                    next: 0,
                    phase: Phase::Next,
                    abandoned: false,
                },
            )
            .expect("one start read per agent slot");
        assert!(prior.is_none(), "each agent starts only once");
    }

    pub(super) fn abandon(&mut self, owner: Token) -> bool {
        let waiting = self.waiting.remove(&owner).is_some();
        if let Some(active) = &mut self.active
            && active.ready.owner == owner
        {
            return !core::mem::replace(&mut active.abandoned, true);
        }
        waiting
    }

    pub(super) fn complete(
        &mut self,
        now: Time,
        prefix: u64,
        mut complete: kernel::Complete,
    ) -> Result<(), kernel::Complete> {
        let lane = prefix | file_layer::LANE | TAG;
        if complete.op.raw() & !SEQUENCE != lane {
            return Err(complete);
        }
        complete.op = Token::new(file_layer::LANE | (complete.op.raw() & SEQUENCE));
        file_layer::step::up(&mut self.files, &env(now), complete, &mut self.events, &mut self.operations);
        Ok(())
    }

    pub(super) fn step(
        &mut self,
        now: Time,
        root: kernel::Fd,
        prefix: u64,
        submitted: &mut Queue<kernel::Submit>,
    ) -> Option<Ready> {
        let env = env(now);
        file_layer::step::expire(&mut self.files, &env, &mut self.operations);
        if self.active.is_none()
            && let Some((&owner, _)) = self.waiting.first()
        {
            self.active = self.waiting.remove(&owner);
        }
        let mut ready = None;
        if self.files.takes()
            && let Some(active) = &mut self.active
        {
            let request = match self.events.pop() {
                Some(event) => active.event(event),
                None => None,
            };
            let request = match request {
                Some(request) => Some(request),
                None if !active.abandoned => active.next(root),
                None => None,
            };
            match request {
                Some(request) => {
                    file_layer::step::down(&mut self.files, &env, request, &mut self.events, &mut self.operations);
                }
                None => {
                    let done = self.active.take().expect("the completed read owns its start");
                    if !done.abandoned {
                        ready = Some(done.ready);
                    }
                }
            }
        }
        for _ in 0..self.operations.len() {
            let mut submit = self.operations.pop().expect("file operation count");
            submit.op = tagged(submit.op, prefix);
            // Cancel completions are identified by their operation, not by the target.
            if let kernel::Op::Cancel { target } = &mut submit.kind {
                *target = tagged(*target, prefix);
            }
            submitted.push(submit);
        }
        ready
    }

    pub(super) fn work_pending(&self, now: Time) -> bool {
        self.files.is_due(now)
            || !self.operations.is_empty()
            || (self.files.takes() && (!self.events.is_empty() || !self.waiting.is_empty()))
    }

    pub(super) fn next_deadline(&self) -> Option<Time> {
        self.files.next_deadline()
    }

    pub(super) fn settled(&self) -> bool {
        self.active.is_none()
            && self.waiting.is_empty()
            && self.files.takes()
            && self.files.open_files() == 0
            && self.events.is_empty()
            && self.operations.is_empty()
    }
}

impl Reading {
    fn next(&mut self, root: kernel::Fd) -> Option<file::Request> {
        match self.phase {
            Phase::Next => {}
            Phase::Opening | Phase::Loading { .. } | Phase::Closing => {
                unreachable!("the previous read has not settled")
            }
        }
        let pending = self.ready.start.directories.get(self.next..).expect("hook read stays within the mounts");
        for directory in pending {
            if directory.writable {
                self.phase = Phase::Opening;
                let token = Decimal::of(directory.root.raw());
                let name = bytes::concat(&[b"workspaces/", token.as_bytes(), b"/", &directory.name, b"/.temper/hooks"]);
                return Some(file::Request::OpenDirectory { owner: self.ready.owner, root, name, no_follow: true });
            }
            self.next = self.next.checked_add(1).expect("bounded mounts");
        }
        None
    }

    fn event(&mut self, event: file::Event) -> Option<file::Request> {
        assert_eq!(self.ready.owner, event.owner(), "the held start owns the file read");
        let observed = observed(event);
        match self.phase {
            Phase::Opening => match observed {
                Observed::Opened(file) if self.abandoned => Some(self.close(file)),
                Observed::Opened(file) => {
                    self.phase = Phase::Loading { directory: file };
                    Some(file::Request::Load {
                        owner: self.ready.owner,
                        root: file,
                        path: b"config.toml".as_slice().into(),
                        max: config_bytes(),
                        no_follow: true,
                    })
                }
                Observed::Failed(
                    kernel::Error::NotFound | kernel::Error::NotADirectory | kernel::Error::TooManyLinks,
                ) => {
                    self.record(None);
                    None
                }
                Observed::Failed(error) => {
                    let result = load_result(Observed::Failed(error), self.ceiling);
                    self.record(Some(result));
                    None
                }
                Observed::Loaded(_) | Observed::TooLarge | Observed::Closed => unreachable!("directory open terminal"),
            },
            Phase::Loading { directory } => {
                if !self.abandoned {
                    let result = load_result(observed, self.ceiling);
                    self.record(Some(result));
                }
                Some(self.close(directory))
            }
            Phase::Closing => match observed {
                Observed::Closed | Observed::Failed(_) => {
                    self.phase = Phase::Next;
                    None
                }
                Observed::Opened(_) | Observed::Loaded(_) | Observed::TooLarge => {
                    unreachable!("directory close terminal")
                }
            },
            Phase::Next => unreachable!("only a file request has a terminal"),
        }
    }

    fn close(&mut self, file: Token) -> file::Request {
        self.phase = Phase::Closing;
        file::Request::Close { owner: self.ready.owner, file }
    }

    fn record(&mut self, hooks: Option<HookTimes>) {
        self.ready.start.directories.get_mut(self.next).expect("the directory being read").hooks = hooks;
        self.next = self.next.checked_add(1).expect("bounded mounts");
        self.phase = Phase::Next;
    }
}

enum Observed {
    Opened(Token),
    Loaded(Box<[u8]>),
    TooLarge,
    Closed,
    Failed(kernel::Error),
}

fn observed(event: file::Event) -> Observed {
    match event {
        file::Event::Opened { file, .. } => Observed::Opened(file),
        file::Event::Loaded { bytes, .. } => Observed::Loaded(bytes),
        file::Event::TooLarge { .. } => Observed::TooLarge,
        file::Event::Closed { .. } => Observed::Closed,
        file::Event::Failed { error, .. } => Observed::Failed(error),
        file::Event::Cancelled { .. } => Observed::Failed(kernel::Error::Cancelled),
        file::Event::Locked { .. }
        | file::Event::Busy { .. }
        | file::Event::Made { .. }
        | file::Event::Refused { .. }
        | file::Event::Scanned { .. }
        | file::Event::Stored { .. }
        | file::Event::Conflict { .. }
        | file::Event::Stated { .. }
        | file::Event::Written { .. }
        | file::Event::Read { .. }
        | file::Event::Cut { .. }
        | file::Event::Synced { .. }
        | file::Event::Linked { .. }
        | file::Event::Renamed { .. }
        | file::Event::Removed { .. }
        | file::Event::Listed { .. } => unreachable!("the reader only opens, loads and closes"),
    }
}

fn load_result(event: Observed, ceiling: Duration) -> HookTimes {
    let result = match event {
        Observed::Loaded(bytes) => hooks::times(&bytes),
        Observed::Failed(kernel::Error::NotFound) => hooks::times(b""),
        Observed::TooLarge => Err(fault(b"file too large")),
        Observed::Failed(kernel::Error::TooManyLinks | kernel::Error::IsADirectory | kernel::Error::NotADirectory) => {
            Err(fault(b"not a regular file"))
        }
        Observed::Failed(_) => Err(fault(b"could not read the file")),
        Observed::Opened(_) | Observed::Closed => unreachable!("only a file load or open failure has time evidence"),
    };
    match result {
        Ok(times) => HookTimes {
            deliver: hook_time(times.deliver, ceiling),
            finish: hook_time(times.finish, ceiling),
            ceiling,
            fault: Box::default(),
        },
        Err(fault) => {
            let text = fault.text(HOOK_CONFIG_READER);
            HookTimes {
                deliver: None,
                finish: None,
                ceiling,
                fault: bytes::copy_of(bytes::utf8_prefix(&text, skein_lib::usize_of(smith::HOOK_FAULT_BYTES))),
            }
        }
    }
}

fn hook_time(seconds: Option<u32>, ceiling: Duration) -> Option<HookTime> {
    let asked = Duration::from_secs(u64::from(seconds?));
    Some(HookTime { time: asked.min(ceiling), asked })
}

fn fault(why: &[u8]) -> Fault {
    Fault::Line { line: 0, why: why.into() }
}

fn config_bytes() -> u32 {
    u32::try_from(hooks::CONFIG_BYTES).expect("the time file bound fits u32")
}

fn limits() -> file_layer::Limits {
    let bytes = config_bytes();
    file_layer::Limits { files: 1, read: bytes, entries: 1, whole: bytes, timeout: Duration::from_secs(1) }
}

fn env(now: Time) -> Env<file_layer::Limits> {
    Env { now, wall: Wall::EPOCH, limits: limits() }
}

fn tagged(op: Token, prefix: u64) -> Token {
    assert_eq!(op.raw() & !SEQUENCE, file_layer::LANE, "hook read within the worker's file lane");
    Token::new(prefix | TAG | op.raw())
}

#[cfg(test)]
mod tests;
