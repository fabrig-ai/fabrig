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
