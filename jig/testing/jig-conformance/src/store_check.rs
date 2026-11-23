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
    /// Record a rehearsal or a successful commit-cut restart in the test runner's count.
    #[inline]
    pub fn record_rehearsal() {
        skein_world::checks::count("store-check", 1);
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
mod tests {
    use super::*;

    #[test]
    fn every_checks_each_counted_commit() {
        let mut checks = StoreChecks::every();
        for number in [2, 5, 9] {
            assert!(checks.due(number));
        }
        assert_eq!(checks.commits(), 3);
        assert_eq!(checks.checked(), 3);
        assert_eq!(checks.report(), "store check: 3 of 3 commits checked (every commit)");
    }

    #[test]
    fn none_counts_commits_and_names_its_reason() {
        let mut checks = StoreChecks::none("missing project fixture");
        for number in [2, 5, 9] {
            assert!(!checks.due(number));
        }
        assert_eq!(checks.commits(), 3);
        assert_eq!(checks.checked(), 0);
        assert_eq!(checks.report(), "store check: 0 of 3 commits checked (none: missing project fixture)");
    }

    #[test]
    fn sampled_uses_the_named_stream_and_commit_number() {
        let seed = 7;
        let one_in = 16;
        let draw = Streams::new(seed).seed_of(Stream::Scheduling, "store check");
        let mut checks = StoreChecks::sampled(seed, one_in);
        for number in (1..=41).rev() {
            assert_eq!(checks.due(number), Rng::new(draw ^ number).below(one_in) == 0, "commit {number}");
        }
        assert_eq!(checks.commits(), 41);
        assert_eq!(checks.checked(), 6);
        let example = StoreChecks { commits: 41, checked: 3, ..StoreChecks::sampled(seed, one_in) };
        assert_eq!(example.report(), "store check: 3 of 41 commits checked (one in 16, seed 7)");
    }

    #[test]
    fn the_same_seed_replays_and_one_in_one_checks_every_commit() {
        let mut first = StoreChecks::sampled(19, 8);
        let mut replay = StoreChecks::sampled(19, 8);
        let mut every = StoreChecks::sampled(19, 1);
        for number in 1..=128 {
            assert_eq!(first.due(number), replay.due(number));
            assert!(every.due(number));
        }
        assert_eq!(first.report(), replay.report());
        assert_eq!(every.checked(), 128);
        assert_eq!(every.commits(), 128);
    }

    #[test]
    fn restarted_counts_the_commit_and_its_check_even_when_none_are_due() {
        let mut checks = StoreChecks::none("refusal fixture");
        assert!(!checks.due(9));
        checks.restarted(12);
        assert_eq!(checks.checked(), 1);
        assert_eq!(checks.commits(), 2);
    }

    #[test]
    fn finding_names_the_commit_phase_and_refusal() {
        assert_eq!(finding(12, "Link", "Link { .. }"), "store check: commit 12: start refused at Link: Link { .. }");
    }
}
