//! CI execution time (temper/docs/design/domain/forge/change/stalls.md, 2).
use crate::boundary::Commit;
use skein_lib::{Duration, Wall};

/// What a reading saw of pending CI at a head (stalls.md, 2.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Execution {
    Unknown,
    Waiting,
    Running,
}

/// One reading: execution, newest run and highest attempt read (0 for none).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Reading {
    pub execution: Execution,
    pub run: u64,
    pub attempt: u32,
}

/// The execution clock, counted through `at` under `since` and `head` (stalls.md, 2.2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Clock {
    pub since: Wall,
    pub head: Commit,
    pub run: u64,
    pub attempt: u32,
    pub ran: Duration,
    pub waited: Duration,
    pub unknown: Duration,
    pub at: Wall,
    pub seen: Execution,
}

impl Clock {
    /// Count the interval since the last reading to what it saw; start afresh on
    /// a new wait, head, run or attempt (stalls.md, 2.2).
    #[must_use]
    pub fn observe(clock: Option<Clock>, since: Wall, head: Commit, reading: Reading, now: Wall) -> Clock {
        let mut next = Clock {
            since,
            head,
            run: reading.run,
            attempt: reading.attempt,
            ran: Duration::ZERO,
            waited: Duration::ZERO,
            unknown: Duration::ZERO,
            at: now,
            seen: reading.execution,
        };
        if let Some(clock) = clock
            && clock.since == since
            && clock.head == head
        {
            next.run = clock.run.max(reading.run);
            next.attempt = clock.attempt.max(reading.attempt);
            if reading.run <= clock.run && reading.attempt <= clock.attempt {
                next.ran = clock.ran(now);
                next.waited = clock.waited(now);
                next.unknown = clock.unknown(now);
            }
        }
        next
    }

    /// Running time at `now`, including only the open interval if it was running.
    #[must_use]
    pub fn ran(&self, now: Wall) -> Duration {
        match self.seen {
            Execution::Running => self.ran.saturating_add(self.elapsed(now)),
            Execution::Unknown | Execution::Waiting => self.ran,
        }
    }

    /// Waiting time at `now`, including only the open interval if it was waiting.
    #[must_use]
    pub fn waited(&self, now: Wall) -> Duration {
        match self.seen {
            Execution::Waiting => self.waited.saturating_add(self.elapsed(now)),
            Execution::Unknown | Execution::Running => self.waited,
        }
    }

    /// Unknown time at `now`, including only the open interval if it was unknown.
    #[must_use]
    pub fn unknown(&self, now: Wall) -> Duration {
        match self.seen {
            Execution::Unknown => self.unknown.saturating_add(self.elapsed(now)),
            Execution::Waiting | Execution::Running => self.unknown,
        }
    }

    /// Time to the execution or runner bound if the last reading continues (stalls.md, 2.3).
    #[must_use]
    pub fn remaining(&self, now: Wall, limits: &crate::Limits) -> Duration {
        let unknown = self.unknown(now).as_nanos();
        let grace = limits.unknown_grace.as_nanos();
        let counted = self.ran(now).as_nanos().saturating_add(unknown.saturating_sub(grace));
        let waited = self.waited(now).as_nanos();
        let stall = limits.stall.as_nanos();
        let runner = limits.runner_wait.as_nanos();
        if counted >= stall || waited >= runner {
            return Duration::ZERO;
        }
        let remaining = match self.seen {
            Execution::Running => stall.saturating_sub(counted),
            Execution::Unknown => grace.saturating_sub(unknown).saturating_add(stall.saturating_sub(counted)),
            Execution::Waiting => runner.saturating_sub(waited),
        };
        Duration::from_nanos(remaining)
    }

    fn elapsed(&self, now: Wall) -> Duration {
        Duration::from_nanos(now.as_nanos().saturating_sub(self.at.as_nanos()))
    }
}
