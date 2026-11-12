//! One process's mechanical turn (temper/docs/design/protocol/services.md, 3).
use skein_io::{self as io, Io, kernel::{Complete, Fd, Submit}};
use skein_lib::{Env, Queue, Time, Token, Wall};

#[expect(missing_debug_implementations, reason = "io owns kernel buffers")]
pub struct Process {
    io: Io,
    env: Env<io::Limits>,
    kernel: io::kernel::Routes,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
    raw_submissions: Queue<Submit>,
    told: Queue<io::Event>,
    signals: Option<Token>,
    signals_closing: bool,
}
impl Process {
    /// The shell queues retain their own bound; staged operations share kernel credit.
    #[must_use]
    pub fn new(limits: io::Limits, operations: u32, queue: u32) -> Self {
        Self {
            io: Io::new(&limits),
            env: Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            kernel: io::kernel::Routes::new(operations),
            completions: Queue::with_capacity(queue),
            submissions: Queue::with_capacity(queue),
            raw_submissions: Queue::with_capacity(operations),
            told: Queue::uncapped(),
            signals: None,
            signals_closing: false,
        }
    }
    pub fn clock(&mut self, now: Time, wall: Wall) {
        self.env.now = now;
        self.env.wall = wall;
    }
    #[must_use]
    pub const fn io(&self) -> &Io { &self.io }
    pub const fn io_mut(&mut self) -> &mut Io { &mut self.io }
    #[must_use]
    pub const fn env(&self) -> &Env<io::Limits> { &self.env }
    pub const fn env_mut(&mut self) -> &mut Env<io::Limits> { &mut self.env }
    #[must_use]
    pub fn next_event(&self) -> Option<&io::Event> { self.told.iter().next() }
    pub const fn completions(&mut self) -> &mut Queue<Complete> { &mut self.completions }
    pub const fn submissions(&mut self) -> &mut Queue<Submit> { &mut self.submissions }
    pub const fn raw_submissions(&mut self) -> &mut Queue<Submit> { &mut self.raw_submissions }
    pub const fn told(&mut self) -> &mut Queue<io::Event> { &mut self.told }
    #[must_use]
    pub fn completion_count(&self) -> u32 { self.completions.len() }
    #[must_use]
    pub fn submission_count(&self) -> u32 { self.submissions.len() }
    #[must_use]
    pub fn raw_submission_count(&self) -> u32 { self.raw_submissions.len() }
    #[must_use]
    pub fn told_count(&self) -> u32 { self.told.len() }
    #[must_use]
    pub fn room(&self) -> u32 { self.raw_submissions.room() }
    #[must_use]
    pub fn operations(&self) -> u32 { self.raw_submissions.capacity() }
    #[must_use]
    pub fn has_operations(&self) -> bool { self.kernel.has_operations() }
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        self.io.is_ready() || self.io.is_due(now) || !self.completions.is_empty()
            || !self.told.is_empty() || !self.raw_submissions.is_empty()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.io.is_empty() && self.completions.is_empty() && self.submissions.is_empty()
            && self.raw_submissions.is_empty() && self.told.is_empty() && self.kernel.is_empty()
    }
    pub fn resume(&mut self) {
        for _ in 0..self.env.limits.sockets.checked_add(self.env.limits.refusals).expect("io entries") {
            if !self.io.is_ready() || self.room() < io::MAX_OUT_RESUME.submissions { break; }
            io::resume(&mut self.io, &self.env, &mut self.told, &mut self.raw_submissions);
        }
    }
    /// After the ready list drains, hand each credited completion to its owner to route.
    pub fn take_completion(&mut self) -> Option<Complete> {
        if self.room() < io::MAX_OUT_UP.submissions { return None; }
        let complete = self.completions.pop()?;
        Some(self.complete(complete))
    }
    /// Resolve a kernel route without deciding which component owns it.
    pub fn complete(&mut self, complete: Complete) -> Complete {
        self.kernel.complete(complete, &self.raw_submissions)
    }
    pub fn up(&mut self, complete: Complete) {
        io::up(&mut self.io, &self.env, complete, &mut self.told, &mut self.raw_submissions);
    }
    pub fn fire(&mut self) {
        if !self.completions.is_empty() { return; }
        for _ in 0..self.env.limits.sockets.checked_mul(2).expect("io timers") {
            if !self.io.is_due(self.env.now) || self.room() < io::MAX_OUT_FIRE.submissions { break; }
            io::fire(&mut self.io, &self.env, &mut self.told, &mut self.raw_submissions);
        }
    }
    pub fn down(&mut self, request: io::Request) {
        io::down(&mut self.io, &self.env, request, &mut self.raw_submissions);
    }
    /// Publish staged submissions before reclaiming io's completed entities.
    pub fn publish(&mut self) {
        self.kernel.publish(&mut self.raw_submissions, &mut self.submissions);
        self.io.reclaim();
    }
    pub fn adopt_signals(&mut self, descriptor: Fd) -> Result<(), Fd> {
        if self.signals.is_some() { return Err(descriptor); }
        match self.io.adopt_signals_for(descriptor, Token::new(crate::tokens::SIGNALS)) {
            Ok(source) => { self.signals = Some(source); Ok(()) }
            Err(descriptor) => Err(descriptor),
        }
    }
    #[must_use]
    pub const fn has_signals(&self) -> bool { self.signals.is_some() }
    #[must_use]
    pub const fn signals_closing(&self) -> bool { self.signals_closing }
    #[must_use]
    pub const fn signals_pending(&self) -> bool { self.signals.is_some() && !self.signals_closing }
    /// The owner routes the signal source's terminal, just as it routes Shutdown.
    pub const fn signals_closed(&mut self) { self.signals = None; }
    /// Called at the owner's end; credit may defer the single close to a later turn.
    pub fn close_signals(&mut self) {
        if !self.signals_closing && self.room() >= io::MAX_OUT_DOWN.submissions
            && let Some(entity) = self.signals {
            self.signals_closing = true;
            self.down(io::Request::Close { entity });
        }
    }
}
