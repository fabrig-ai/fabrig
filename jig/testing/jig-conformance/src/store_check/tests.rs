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
    for mut checks in [StoreChecks::every(), StoreChecks::sampled(7, 1), StoreChecks::none("missing project fixture")] {
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
