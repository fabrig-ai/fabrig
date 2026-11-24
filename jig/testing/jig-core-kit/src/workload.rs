//! Seeded tasks and independent parties (`jig/docs/design/domain/testing-workloads.md`, 3).
mod draw;
mod run;

pub use draw::{Act, Moment, Party, Task, TaskKind, Workload};
pub use run::run;

use skein_lib::{Duration, Time};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expect {
    /// Done, failed or cancelled within the run's bound.
    Ends,
    /// A party holds it, or does not answer it; its ancestors may wait too.
    Waits,
}

/// Both clocks bound progress after the last act, not after initial creation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bound {
    pub time: Duration,
    pub iterations: u64,
}

impl Default for Bound {
    fn default() -> Self {
        Self { time: Duration::from_secs(900), iterations: 10_000 }
    }
}

impl Expect {
    /// Check a retained task at a run's boundary. `ended` includes all three terminal outcomes.
    pub fn check(
        self,
        task: (&str, u64),
        seed: u64,
        moment: (Time, u64),
        last: (Time, u64),
        bound: Bound,
        ended: bool,
    ) -> Result<(), String> {
        assert!(bound.time > Duration::ZERO && bound.iterations > 0, "nonzero workload bound");
        if self == Self::Ends
            && !ended
            && (moment.0.as_nanos().saturating_sub(last.0.as_nanos()) >= bound.time.as_nanos()
                || moment.1.saturating_sub(last.1) >= bound.iterations)
        {
            return Err(format!(
                "seed {seed}: task {} ({}) expected Ends past its bound at {:?}, iteration {}",
                task.0, task.1, moment.0, moment.1
            ));
        }
        Ok(())
    }
}
