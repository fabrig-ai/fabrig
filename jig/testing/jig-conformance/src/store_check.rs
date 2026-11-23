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
    One { seed: u64, draw: u64, drawn: u64, kept: Option<u64>, keeps: bool, ended: bool },
    Sampled { seed: u64, one_in: u64, draw: u64 },
    None { reason: &'static str },
}

fn draw_seed(seed: u64) -> u64 {
    Streams::new(seed).seed_of(Stream::Scheduling, "store check")
}

impl StoreChecks {
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
    fn one_defers_the_last_kept_commit_to_end() {
        // Seed 7 keeps positions 1 and 2 of these three commits.
        const SEED: u64 = 7;
        const COMMITS: u64 = 3;
        let mut checks = StoreChecks::one(SEED);
        for (number, keeps) in [(2, true), (5, true), (9, false)] {
            assert!(!checks.due(number), "commit {number}");
            assert_eq!(checks.keeps(), keeps, "commit {number}");
        }
        assert_eq!(checks.end(), Some(5));
        assert_eq!(checks.checked(), 1);
        assert_eq!(checks.commits(), COMMITS);
        assert_eq!(checks.end(), None);
        assert_eq!(checks.checked(), 1);
        assert_eq!(checks.commits(), COMMITS);
    }

    #[test]
    fn one_replays_the_named_streams_draw_by_position() {
        // The same 41 commits as the sampled policy's stream test.
        const COMMITS: u64 = 41;
        for (seed, kept) in [(7, 13), (7, 13), (8, 11)] {
            let draw = Streams::new(seed).seed_of(Stream::Scheduling, "store check");
            let mut checks = StoreChecks::one(seed);
            for k in 1..=COMMITS {
                let _ = checks.due(k);
                assert_eq!(checks.keeps(), Rng::new(draw ^ k).below(k) == 0, "seed {seed}, position {k}");
            }
            assert_eq!(checks.end(), Some(kept), "seed {seed}");
        }
    }

    #[test]
    fn one_reports_the_kept_commit_only_after_end() {
        // The design's 22-commit report example keeps commit 17.
        const SEED: u64 = 45;
        const COMMITS: u64 = 22;
        let mut checks = StoreChecks::one(SEED);
        for number in 1..=COMMITS {
            let _ = checks.due(number);
        }
        assert_eq!(checks.report(), "store check: 0 of 22 commits checked (one per run, seed 45)");
        assert_eq!(checks.end(), Some(17));
        assert_eq!(checks.report(), "store check: 1 of 22 commits checked (one per run, seed 45, commit 17)");
    }

    #[test]
    fn one_ends_an_empty_run_without_a_check() {
        let mut checks = StoreChecks::one(45);
        assert_eq!(checks.end(), None);
        assert_eq!(checks.report(), "store check: 0 of 0 commits checked (one per run, seed 45)");
    }

    #[test]
    fn restarted_counts_but_does_not_enter_the_draw() {
        // Seed 8 keeps positions 1 and 4, not positions 2 or 3.
        const SEED: u64 = 8;
        const COMMITS_WITH_RESTART: u64 = 4;
        let mut checks = StoreChecks::one(SEED);
        let _ = checks.due(2);
        let _ = checks.due(5);
        checks.restarted(9);
        let _ = checks.due(12);
        assert_eq!(checks.checked(), 1);
        assert_eq!(checks.commits(), COMMITS_WITH_RESTART);
        assert_eq!(checks.end(), Some(2));
        assert_eq!(checks.checked(), 2);
    }

    #[test]
    fn other_policies_keep_nothing_for_the_runs_end() {
        for mut checks in
            [StoreChecks::every(), StoreChecks::sampled(7, 1), StoreChecks::none("missing project fixture")]
        {
            let _ = checks.due(2);
            assert!(!checks.keeps(), "{}", checks.report());
            assert_eq!(checks.end(), None, "{}", checks.report());
        }
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
