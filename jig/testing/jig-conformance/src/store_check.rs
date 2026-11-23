//! Commit selection and findings for cold store checks
//! (`jig/docs/design/domain/testing.md`, 7).

use skein_lib::Rng;
use skein_world::streams::{Stream, Streams};

/// A run's store-check policy and the commits it has counted.
#[derive(Clone, Debug)]
pub struct StoreChecks {
    policy: Policy,
    commits: u64,
    checked: u64,
}

#[derive(Clone, Debug)]
enum Policy {
    Every,
    Sampled { seed: u64, one_in: u64, draw: u64 },
    None { reason: &'static str },
}

impl StoreChecks {
    fn record_rehearsal() {
        skein_checks::count("store-check", 1);
    }

    /// Run a cold rehearsal and record its verdict, including a refused start.
    #[inline]
    pub fn rehearse<T, E>(run: impl FnOnce() -> Result<T, E>) -> Result<T, E> {
        let verdict = run();
        Self::record_rehearsal();
        verdict
    }

    /// Check every counted commit.
    #[must_use]
    pub fn every() -> Self {
        Self { policy: Policy::Every, commits: 0, checked: 0 }
    }

    /// Check each commit with one chance in `one_in`, independently of order.
    #[must_use]
    pub fn sampled(seed: u64, one_in: u64) -> Self {
        assert!(one_in > 0, "store check sampling denominator is positive");
        let draw = Streams::new(seed).seed_of(Stream::Scheduling, "store check");
        Self { policy: Policy::Sampled { seed, one_in, draw }, commits: 0, checked: 0 }
    }

    /// Check none, recording why the story deliberately admits a refused store.
    #[must_use]
    pub fn none(reason: &'static str) -> Self {
        Self { policy: Policy::None { reason }, commits: 0, checked: 0 }
    }

    /// Count this commit and, when selected, its check.
    pub fn due(&mut self, number: u64) -> bool {
        self.commits = self.commits.checked_add(1).expect("run commit count fits u64");
        let due = match self.policy {
            Policy::Every => true,
            Policy::Sampled { one_in, draw, .. } => Rng::new(draw ^ number).below(one_in) == 0,
            Policy::None { .. } => false,
        };
        if due {
            self.checked = self.checked.checked_add(1).expect("run check count fits u64");
        }
        due
    }

    /// Count a commit checked by the run's own cold restart.
    pub fn restarted(&mut self, _number: u64) {
        self.commits = self.commits.checked_add(1).expect("run commit count fits u64");
        self.checked = self.checked.checked_add(1).expect("run check count fits u64");
    }

    /// Record the successful outcome of a commit-cut child in the parent.
    pub fn fork_restarted(&mut self, number: u64) {
        self.restarted(number);
        Self::record_rehearsal();
    }

    #[must_use]
    pub const fn checked(&self) -> u64 {
        self.checked
    }

    #[must_use]
    pub const fn commits(&self) -> u64 {
        self.commits
    }

    /// The checked count and the policy that chose it.
    #[must_use]
    pub fn report(&self) -> String {
        let policy = match self.policy {
            Policy::Every => "every commit".to_owned(),
            Policy::Sampled { seed, one_in, .. } => format!("one in {one_in}, seed {seed}"),
            Policy::None { reason } => format!("none: {reason}"),
        };
        format!("store check: {} of {} commits checked ({policy})", self.checked, self.commits)
    }
}

/// A cold start's refusal, naming the commit whose store it checked.
#[must_use]
pub fn finding(commit: u64, phase: &str, refusal: &str) -> String {
    format!("store check: commit {commit}: start refused at {phase}: {refusal}")
}

#[cfg(test)]
mod tests;
