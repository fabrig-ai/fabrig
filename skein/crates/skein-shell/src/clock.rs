//! The clock (runtime/shell.md, 5; programming-model.md, section 9).

use std::cell::Cell;

use skein_lib::{Env, FixedClock, FixedStart, Time, Wall};

use crate::system;

/// Reads the host's times or a fixed run's times for each iteration
/// (skein/docs/design/runtime/shell.md, 5).
#[derive(Debug)]
pub struct Clock {
    fixed: Option<Cell<FixedClock>>,
}

/// The times of one iteration, read together.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Now {
    /// Monotonic time for step deadlines. [`Clock::until`] translates them
    /// to the native clock for [`crate::Wait::Until`].
    pub now: Time,
    /// Wall time for things about the world, never a deadline.
    pub wall: Wall,
}

impl Now {
    /// These times, for a step with `limits`.
    #[must_use]
    pub fn with<L>(self, limits: L) -> Env<L> {
        Env { now: self.now, wall: self.wall, limits }
    }
}

impl Clock {
    #[must_use]
    pub fn new() -> Clock {
        Clock { fixed: None }
    }

    /// Starts a clock that advances only to the run's requested deadlines.
    #[must_use]
    pub fn fixed(start: &FixedStart) -> Clock {
        let native = monotonic();
        Clock { fixed: Some(Cell::new(FixedClock::new(start, native))) }
    }

    #[must_use]
    pub fn now(&self) -> Now {
        let native = monotonic();
        match &self.fixed {
            None => {
                // A wall clock set before 1970 reads as the epoch: it is never a
                // deadline, and nothing about the world happened before then.
                let wall = system::clock_nanos(libc::CLOCK_REALTIME).unwrap_or(0);
                Now { now: native, wall: Wall::from_nanos(wall) }
            }
            Some(cell) => {
                let mut fixed = cell.get();
                let (now, wall) = fixed.read(native);
                cell.set(fixed);
                Now { now, wall }
            }
        }
    }

    /// Asks for the step's deadline and returns its native monotonic instant.
    /// The host's clock passes deadlines through unchanged.
    #[must_use]
    pub fn until(&self, deadline: Option<Time>) -> Option<Time> {
        match &self.fixed {
            None => deadline,
            Some(cell) => {
                let native = monotonic();
                let mut fixed = cell.get();
                let instant = fixed.until(deadline, native);
                cell.set(fixed);
                instant
            }
        }
    }
}

fn monotonic() -> Time {
    let nanos = system::clock_nanos(libc::CLOCK_MONOTONIC).expect("the monotonic clock is past its origin");
    Time::from_nanos(nanos)
}

impl Default for Clock {
    fn default() -> Clock {
        Clock::new()
    }
}
