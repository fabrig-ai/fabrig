//! File I/O for source inspection, shared by worker services (domain/hosts/README.md, 5.1).
//! Each worker tags the file lane with its own prefix. The stage owns no policy.
use super::{Component, sources};
use skein_io::{self as io, file, file_layer, kernel};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};

const FILE_LIMITS: file_layer::Limits =
    file_layer::Limits { files: 1, read: 1, entries: 1, whole: 1, timeout: Duration::from_secs(1) };

/// A worker's serialized source file operations and their kernel handoffs.
#[derive(Debug)]
pub struct FileStage {
    files: file_layer::FileIo,
    prefix: u64,
    requests: Queue<file::Request>,
    events: Queue<file::Event>,
    submitted: Queue<kernel::Submit>,
}
impl FileStage {
    /// Keep room for one inspection descriptor and its cancellation handoff.
    #[must_use]
    pub fn new(prefix: u64) -> Self {
        Self {
            files: file_layer::FileIo::with_limits(&FILE_LIMITS),
            prefix,
            requests: Queue::uncapped(),
            events: Queue::uncapped(),
            submitted: Queue::uncapped(),
        }
    }
    /// Inspection requests, drained while the file driver has credit.
    pub fn requests(&mut self) -> &mut Queue<file::Request> {
        &mut self.requests
    }
    /// Consume this worker's tagged file completion, returning other owners intact.
    pub fn complete(&mut self, mut complete: kernel::Complete) -> Result<(), kernel::Complete> {
        if complete.op.raw() & !((1_u64 << 48).checked_sub(1).expect("mask") | file_layer::LANE) != self.prefix
            || !file_layer::owns(complete.op)
        {
            return Err(complete);
        }
        complete.op = Token::new(complete.op.raw() & !self.prefix);
        cancel_target(&mut complete.kind, self.prefix, false);
        file_layer::step::up(&mut self.files, &file_env(Time::ZERO), complete, &mut self.events, &mut self.submitted);
        Ok(())
    }
    /// Route inspection terminals by owner and flush the file driver's requests.
    pub fn step(
        &mut self,
        now: Time,
        sources: &mut Component,
        below: &mut Queue<io::Request>,
        submitted: &mut Queue<kernel::Submit>,
    ) -> Option<(Token, sources::Answer)> {
        file_layer::step::expire(&mut self.files, &file_env(now), &mut self.submitted);
        let mut answer = None;
        for _ in 0..self.events.len() {
            let event = self.events.pop().expect("file event count");
            if event.owner() == Token::new(self.prefix) {
                continue;
            }
            assert!(sources.owns(event.owner()), "source owns inspection events");
            let settled = sources.from_file(event, below, &mut self.requests);
            if settled.is_some() {
                assert!(answer.is_none(), "one serialized source answer");
                answer = settled;
            }
        }
        if self.files.takes()
            && let Some(request) = self.requests.pop()
        {
            self.down(now, sources.next_deadline(), request);
        }
        for _ in 0..self.submitted.len() {
            let mut submit = self.submitted.pop().expect("file submission count");
            submit.op = Token::new(self.prefix | submit.op.raw());
            cancel_target(&mut submit.kind, self.prefix, true);
            submitted.push(submit);
        }
        answer
    }
    fn down(&mut self, now: Time, deadline: Option<Time>, request: file::Request) {
        match &request {
            file::Request::Close { .. } => {
                file_layer::step::down(&mut self.files, &file_env(now), request, &mut self.events, &mut self.submitted);
            }
            file::Request::OpenDirectory { .. } | file::Request::Stat { .. } => {
                let deadline = deadline.unwrap_or(now);
                file_layer::step::down_until(
                    &mut self.files,
                    &file_env(now),
                    deadline,
                    request,
                    &mut self.events,
                    &mut self.submitted,
                );
            }
            file::Request::Lock { .. }
            | file::Request::MakeDirectory { .. }
            | file::Request::OpenPrivate { .. }
            | file::Request::Create { .. }
            | file::Request::CreateNoFollow { .. }
            | file::Request::OpenRead { .. }
            | file::Request::OpenReadNoFollow { .. }
            | file::Request::OpenWrite { .. }
            | file::Request::OpenWriteNoFollow { .. }
            | file::Request::Load { .. }
            | file::Request::Scan { .. }
            | file::Request::Store { .. }
            | file::Request::WriteAt { .. }
            | file::Request::ReadAt { .. }
            | file::Request::Cut { .. }
            | file::Request::Sync { .. }
            | file::Request::SyncDirectory { .. }
            | file::Request::HardLink { .. }
            | file::Request::Rename { .. }
            | file::Request::Remove { .. }
            | file::Request::List { .. }
            | file::Request::ListPage { .. } => unreachable!("source files inspect and close only"),
        }
    }
    /// Close the shell-opened programs parent through the same file driver.
    pub fn close_parent(&mut self, parent: kernel::Fd) {
        let file = self.files.adopt_root(parent).expect("settled inspection leaves one file slot");
        self.requests.push(file::Request::Close { owner: Token::new(self.prefix), file });
    }
    /// Whether there are no file requests, completions or open descriptors left.
    #[must_use]
    pub fn settled(&self) -> bool {
        self.files.takes()
            && self.files.open_files() == 0
            && self.requests.is_empty()
            && self.events.is_empty()
            && self.submitted.is_empty()
    }
    /// Whether file work can advance in the next worker iteration.
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        !self.events.is_empty()
            || !self.submitted.is_empty()
            || (!self.requests.is_empty() && self.files.takes())
            || self.files.is_due(now)
    }
    /// The outstanding file operation's mechanism deadline.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        self.files.next_deadline()
    }
}

fn file_env(now: Time) -> Env<file_layer::Limits> {
    Env { now, wall: Wall::EPOCH, limits: FILE_LIMITS }
}

fn cancel_target(kind: &mut kernel::Op, prefix: u64, tagging: bool) {
    match kind {
        kernel::Op::Cancel { target } => {
            *target = Token::new(if tagging { prefix | target.raw() } else { target.raw() & !prefix });
        }
        kernel::Op::Socket { .. }
        | kernel::Op::Bind { .. }
        | kernel::Op::Listen { .. }
        | kernel::Op::Accept { .. }
        | kernel::Op::Connect { .. }
        | kernel::Op::Recv { .. }
        | kernel::Op::Send { .. }
        | kernel::Op::Shutdown { .. }
        | kernel::Op::Close { .. }
        | kernel::Op::Open { .. }
        | kernel::Op::Read { .. }
        | kernel::Op::Write { .. }
        | kernel::Op::Append { .. }
        | kernel::Op::Truncate { .. }
        | kernel::Op::Sync { .. }
        | kernel::Op::Lock { .. }
        | kernel::Op::Stat { .. }
        | kernel::Op::HardLink { .. }
        | kernel::Op::Rename { .. }
        | kernel::Op::Remove { .. }
        | kernel::Op::MakeDirectory { .. }
        | kernel::Op::List { .. }
        | kernel::Op::Spawn { .. }
        | kernel::Op::Wait { .. }
        | kernel::Op::Signal { .. }
        | kernel::Op::Usage
        | kernel::Op::Window { .. }
        | kernel::Op::ReadSignal { .. }
        | kernel::Op::PipeRead { .. }
        | kernel::Op::PipeWrite { .. } => {}
    }
}
