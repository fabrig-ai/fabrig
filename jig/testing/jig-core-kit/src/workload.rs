//! Seeded tasks and independent parties (`jig/docs/design/domain/testing-workloads.md`, 3).
mod draw;
mod run;

pub use draw::{Act, Moment, Party, Task, TaskKind, Workload};
pub use run::run;

use jig_domain_tasks::{Class, Hold, HoldSource, Phase};
use skein_lib::{Duration, Time};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expect {
    /// Done, failed or cancelled within the run's bound.
    Ends,
    /// A party holds it, or does not answer it; its ancestors may wait too.
    Waits,
    /// Held past its run tries, with the reason predicted by the draw.
    Stuck,
}

/// The committed state and, for `Stuck`, the expected words and allowed run tries.
#[derive(Clone, Copy, Debug)]
pub struct Observation<'a> {
    pub phase: Option<&'a Phase>,
    pub run_tries: u32,
    pub stuck: Option<(&'a str, u32)>,
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

impl Bound {
    fn spent(self, moment: (Time, u64), last: (Time, u64)) -> bool {
        moment.0.as_nanos().saturating_sub(last.0.as_nanos()) >= self.time.as_nanos()
            || moment.1.saturating_sub(last.1) >= self.iterations
    }
}

impl Expect {
    /// Check a task at a quiet point. True means its expectation is satisfied.
    pub fn check(
        self,
        task: (&str, u64),
        seed: u64,
        moment: (Time, u64),
        last: (Time, u64),
        bound: Bound,
        observation: Observation<'_>,
    ) -> Result<bool, String> {
        assert!(bound.time > Duration::ZERO && bound.iterations > 0, "nonzero workload bound");
        let Observation { phase, run_tries, stuck } = observation;
        let ready = match self {
            Self::Ends => matches!(phase, Some(Phase::Ended(_))),
            Self::Waits => true,
            Self::Stuck => matches!(
                (phase, stuck),
                (Some(Phase::Held { why: Hold::Failures(Class::Run), source: Some(source), .. }), Some((expected, limit)))
                    if run_tries > limit
                        && matches!(source.as_ref(), HoldSource::Preparation { words } if words.as_ref() == expected.as_bytes())
            ),
        };
        let wrong_stuck = self == Self::Stuck && matches!(phase, Some(Phase::Held { .. } | Phase::Ended(_)));
        if !ready && (bound.spent(moment, last) || wrong_stuck) {
            return Err(format!(
                "seed {seed}: task {} ({}) expected {self:?} at {:?}, iteration {}: {phase:?}, run tries {run_tries}, stuck reason {stuck:?}",
                task.0, task.1, moment.0, moment.1
            ));
        }
        Ok(ready)
    }
}
