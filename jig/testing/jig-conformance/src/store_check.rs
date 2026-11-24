//! Commit selection and findings for cold store checks
//! (`jig/docs/design/domain/testing-fakes.md`, 1).

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
    One { seed: u64, draw: u64, drawn: u64, kept: Option<u64>, keeps: bool, ended: bool },
    Sampled { seed: u64, one_in: u64, draw: u64 },
    None { reason: &'static str },
}

fn draw_seed(seed: u64) -> u64 {
    Streams::new(seed).seed_of(Stream::Scheduling, "store check")
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

    /// One of the run's commits, drawn from its seed, checked as the run
    /// ends: the worlds' default.
    #[must_use]
    pub fn one(seed: u64) -> Self {
        let policy = Policy::One { seed, draw: draw_seed(seed), drawn: 0, kept: None, keeps: false, ended: false };
        Self { policy, commits: 0, checked: 0 }
    }

    /// Check each commit with one chance in `one_in`, independently of order.
    #[must_use]
    pub fn sampled(seed: u64, one_in: u64) -> Self {
        assert!(one_in > 0, "store check sampling denominator is positive");
        Self { policy: Policy::Sampled { seed, one_in, draw: draw_seed(seed) }, commits: 0, checked: 0 }
    }

    /// Check none, recording why the story deliberately admits a refused store.
    #[must_use]
    pub fn none(reason: &'static str) -> Self {
        Self { policy: Policy::None { reason }, commits: 0, checked: 0 }
    }

    /// Counts commit `number`; whether the world checks it now.
    pub fn due(&mut self, number: u64) -> bool {
        self.commits = self.commits.checked_add(1).expect("run commit count fits u64");
        let due = match &mut self.policy {
            Policy::Every => true,
            Policy::One { draw, drawn, kept, keeps, .. } => {
                *drawn = drawn.checked_add(1).expect("run draw count fits u64");
                *keeps = Rng::new(*draw ^ *drawn).below(*drawn) == 0;
                if *keeps {
                    *kept = Some(number);
                }
                false
            }
            Policy::Sampled { one_in, draw, .. } => Rng::new(*draw ^ number).below(*one_in) == 0,
            Policy::None { .. } => false,
        };
        if due {
            self.checked = self.checked.checked_add(1).expect("run check count fits u64");
        }
        due
    }

    /// After `due`: whether the world keeps that commit's store, in place
    /// of any it kept, for the run's check as it ends.
    #[must_use]
    pub const fn keeps(&self) -> bool {
        matches!(self.policy, Policy::One { keeps: true, .. })
    }

    /// As the run ends: the kept commit, now counted checked, which the
    /// world rehearses over the store it kept; `None` when it kept none,
    /// under any other policy, and at every later call.
    pub fn end(&mut self) -> Option<u64> {
        let Policy::One { kept, ended, .. } = &mut self.policy else {
            return None;
        };
        if *ended {
            return None;
        }
        *ended = true;
        let commit = (*kept)?;
        self.checked = self.checked.checked_add(1).expect("run check count fits u64");
        Some(commit)
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
            Policy::One { seed, kept: Some(commit), ended: true, .. } => {
                format!("one per run, seed {seed}, commit {commit}")
            }
            Policy::One { seed, .. } => format!("one per run, seed {seed}"),
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
