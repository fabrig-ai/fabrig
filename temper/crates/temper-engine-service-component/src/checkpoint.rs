//! Stopped checkpoint operations and the held child's lifecycle (protocol/checkpoints.md, 2).
use crate::{End, engine::Engine};
use alloc::boxed::Box;
use jig_protocol_store::{CheckpointName, CheckpointRefusal, Checkpointed, Event, OpenRefusal, Request};
use skein_io::{
    self as io,
    kernel::{Exit, Reason, ServiceSignal, Signal, Spawn, Target},
};
use skein_lib::{Queue, Token};

/// A terminal retained through store and child teardown for the shell's report.
#[derive(Debug)]
pub enum Outcome {
    Taken(Checkpointed),
    Removed(CheckpointName),
    Refused { name: Option<CheckpointName>, why: CheckpointRefusal },
    OpenRefused(OpenRefusal),
    Child(Exit),
    CantRun { command: Box<[u8]>, why: Option<io::Error> },
}

/// The command as typed and the shell's resolution, prepared before the loop.
pub type Child = (Box<[u8]>, Result<Spawn, ()>);

// Outside every serving component's token range.
pub(crate) const CHILD: Token = Token::new(13_u64 << 60);

#[derive(PartialEq, Eq)]
enum Phase {
    Opening,
    Spawning,
    Running(Token),
    Reaping,
    Releasing,
    Done,
}

pub(crate) struct State {
    spawn: Option<Result<Spawn, ()>>,
    name: Option<CheckpointName>,
    command: Box<[u8]>,
    phase: Phase,
    signal: Option<Signal>,
    pub(crate) outcome: Option<Outcome>,
    pub(crate) io: Queue<io::Request>,
    store: Queue<Request>,
}
impl State {
    pub(crate) fn new(name: Option<CheckpointName>, child: Option<Child>) -> Self {
        let (command, spawn) = match child {
            Some((command, spawn)) => (command, Some(spawn)),
            None => (Box::from(b"".as_slice()), None),
        };
        Self {
            spawn,
            name,
            command,
            phase: Phase::Opening,
            signal: None,
            outcome: None,
            io: Queue::uncapped(),
            store: Queue::uncapped(),
        }
    }

    pub(crate) fn pending(&self) -> bool {
        !self.io.is_empty() || !self.store.is_empty()
    }
    pub(crate) fn done(&self) -> bool {
        self.phase == Phase::Done
    }

    fn release(&mut self) {
        self.phase = Phase::Releasing;
        self.store.push(Request::Close { owner: Token::new(1) });
    }

    pub(crate) fn store(&mut self, event: Event) {
        match event {
            Event::CheckpointHeld { .. } => match self.spawn.take().expect("hold has a prepared command") {
                Ok(spawn) => {
                    self.phase = Phase::Spawning;
                    self.io.push(io::Request::Spawn { owner: CHILD, spawn });
                }
                Err(()) => {
                    self.outcome = Some(Outcome::CantRun { command: self.command.clone(), why: None });
                    self.release();
                }
            },
            Event::Closed { .. } => {
                self.phase = Phase::Done;
            }
            Event::Checkpointed { checkpoint, .. } => {
                self.outcome = Some(Outcome::Taken(checkpoint));
                self.phase = Phase::Done;
            }
            Event::CheckpointRemoved { .. } => {
                self.outcome = Some(Outcome::Removed(self.name.expect("removal has a name")));
                self.phase = Phase::Done;
            }
            Event::CheckpointRefused { why, .. } => {
                self.outcome = Some(Outcome::Refused { name: self.name, why });
                self.phase = Phase::Done;
            }
            Event::OpenRefused { why, .. } => {
                self.outcome = Some(Outcome::OpenRefused(why));
                self.phase = Phase::Done;
            }
            Event::Corrupt { .. } | Event::Failed { .. } => {
                self.outcome = Some(Outcome::OpenRefused(OpenRefusal::Failed));
                self.phase = Phase::Done;
            }
            Event::Opened { .. }
            | Event::Census { .. }
            | Event::Switched { .. }
            | Event::Job { .. }
            | Event::Ran { .. }
            | Event::Rehearsing { .. }
            | Event::Decoded { .. }
            | Event::Undecodable { .. }
            | Event::Back { .. }
            | Event::BackRefused { .. }
            | Event::Restored { .. }
            | Event::RestoreRefused { .. }
            | Event::Committed { .. }
            | Event::FrameTooLarge { .. }
            | Event::Page { .. }
            | Event::PageRefused { .. }
            | Event::Loaded { .. }
            | Event::Read { .. }
            | Event::Got { .. }
            | Event::Turns { .. }
            | Event::Unread { .. }
            | Event::SecretsNotReady { .. }
            | Event::Kept { .. }
            | Event::Stale { .. }
            | Event::Erased { .. }
            | Event::Carried { .. }
            | Event::Secret { .. }
            | Event::SecretsLoaded { .. } => {}
        }
    }

    pub(crate) fn up(&mut self, event: io::Event) {
        match event {
            io::Event::Spawned { child, pipes, .. } => {
                assert!(pipes.is_empty(), "hold inherits descriptors, never pipes");
                self.phase = Phase::Running(child);
                if let Some(signal) = self.signal.take() {
                    self.forward(child, signal);
                }
            }
            io::Event::Exited { exit, .. } => {
                let Phase::Running(child) = self.phase else { unreachable!("held child exits after spawning") };
                self.outcome = Some(Outcome::Child(exit));
                self.phase = Phase::Reaping;
                self.io.push(io::Request::Close { entity: child });
            }
            io::Event::Failed { error, .. } => {
                if let Phase::Running(child) = self.phase {
                    self.io.push(io::Request::Close { entity: child });
                }
                self.outcome = Some(Outcome::CantRun { command: self.command.clone(), why: Some(error) });
                self.phase = Phase::Reaping;
            }
            io::Event::Closed { .. } => self.release(),
            io::Event::Shutdown { signal, .. } => self.shutdown(signal),
            io::Event::Listening { .. }
            | io::Event::Accepted { .. }
            | io::Event::Connecting { .. }
            | io::Event::Connected { .. }
            | io::Event::Stream { .. }
            | io::Event::Output { .. }
            | io::Event::Usage { .. }
            | io::Event::Window { .. }
            | io::Event::Resized => {}
        }
    }
    fn forward(&mut self, child: Token, signal: Signal) {
        self.io.push(io::Request::Signal { child, signal, to: Target::Group, reason: Reason::Asked });
    }
    pub(crate) fn shutdown(&mut self, signal: ServiceSignal) {
        let signal = match signal {
            ServiceSignal::Interrupt => Signal::Interrupt,
            ServiceSignal::Terminate | ServiceSignal::Hangup => Signal::Terminate,
        };
        match self.phase {
            Phase::Running(child) => self.forward(child, signal),
            Phase::Opening | Phase::Spawning => self.signal = Some(signal),
            Phase::Reaping | Phase::Releasing | Phase::Done => {}
        }
    }
}

pub(crate) fn advance(engine: &mut Engine) -> bool {
    let Some(state) = &mut engine.checkpoint else { return false };
    for _ in 0..state.store.len() {
        engine.store.host.ask(state.store.pop().expect("checkpoint store request"));
    }
    if state.done() {
        engine.end = Some(End::Aborted);
        engine.abort_if_ended();
    }
    true
}

impl Engine {
    /// Adopt only the configured store. The shell prepares a child only for `HoldCheckpoint`.
    pub fn checkpoint_store(
        &mut self,
        directory: io::kernel::Fd,
        user: u32,
        adoption: jig_protocol_store::host::Adoption,
        child: Option<Child>,
    ) -> Result<(), io::kernel::Fd> {
        crate::upgrade::prepare_stopped(self);
        let name = match adoption {
            jig_protocol_store::host::Adoption::HoldCheckpoint { name }
            | jig_protocol_store::host::Adoption::RemoveCheckpoint { name } => Some(name),
            jig_protocol_store::host::Adoption::Checkpoint { .. } => None,
            jig_protocol_store::host::Adoption::Open
            | jig_protocol_store::host::Adoption::Back
            | jig_protocol_store::host::Adoption::Restore { .. }
            | jig_protocol_store::host::Adoption::Rehearse { .. } => unreachable!("checkpoint adoption"),
        };
        self.checkpoint = Some(State::new(name, child));
        self.store.host.adopt_mode(directory, user, self.store.installation, adoption)
    }
    #[must_use]
    pub fn checkpoint_outcome(&self) -> Option<&Outcome> {
        self.checkpoint.as_ref()?.outcome.as_ref()
    }
}

#[cfg(test)]
mod tests;
