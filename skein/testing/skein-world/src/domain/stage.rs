//! Domain entrances driven with uncapped output queues (programming-model.md,
//! section 2; testing-strategy.md, section 2.2). Worlds supply time and call
//! the real step; this helper owns only input and output queues.

use alloc::collections::VecDeque;

use skein_lib::{Env, Queue, Time, Wall};

/// A domain as the shell drives it (programming-model.md, section 2): its
/// environment, the events on their way to it, and the queue its steps and
/// alarms emit into. The output queue is uncapped, so the stage holds no
/// events or alarms back.
///
/// The world calls the domain itself, with `env` and `out`: for each event
/// [`Stage::next_event`] hands it, then for each alarm due while
/// [`Stage::has_room`]; and takes what the steps emitted from `out` at the end
/// of the iteration, before the domain's reclaim point.
#[derive(Debug)]
pub struct Stage<L, E, R> {
    /// Time and immutable domain limits supplied by the world for each step.
    pub env: Env<L>,
    /// FIFO events awaiting delivery; the world bounds incoming work.
    pub inbox: VecDeque<E>,
    /// Owned requests emitted by steps into an uncapped queue.
    pub out: Queue<R>,
}

impl<L, E, R> Stage<L, E, R> {
    /// A stage for a domain whose steps and alarms emit into an uncapped
    /// queue: nothing is reserved, so every event is handed on and every
    /// alarm runs (draft dynamic-memory 4).
    #[must_use]
    pub fn uncapped(limits: L) -> Stage<L, E, R> {
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
        Stage { env, inbox: VecDeque::new(), out: Queue::uncapped() }
    }

    /// Starts an iteration at `now`: every step in it sees the same time.
    pub fn tick(&mut self, now: Time) {
        self.env.now = now;
    }

    /// Sends `event` to the domain, after those on their way already.
    pub fn push(&mut self, event: E) {
        self.inbox.push_back(event);
    }

    /// The next event for the domain, if there is one.
    pub fn next_event(&mut self) -> Option<E> {
        self.inbox.pop_front()
    }

    /// Always room for another step or alarm: the output queue is uncapped.
    #[must_use]
    pub fn has_room(&self) -> bool {
        true
    }

    /// Whether events are waiting for the domain.
    #[must_use]
    pub fn has_events(&self) -> bool {
        !self.inbox.is_empty()
    }
}
