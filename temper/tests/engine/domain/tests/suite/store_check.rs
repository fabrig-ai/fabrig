//! A writer's malformed commit is found without changing its live world.
use jig_conformance::store_check::StoreChecks;
use temper_engine_domain::{Record, Write};
use temper_engine_domain_world::{
    direct::Driver,
    replay,
    store::{Fault, Store},
    walking::{config, limits},
};

fn starting() -> Driver {
    starting_with_seed(91)
}

fn starting_with_seed(seed: u64) -> Driver {
    let mut driver = Driver::configured(Store::new(), [config(seed), config(seed)], &limits());
    for _ in 0..200 {
        driver.advance(false);
        if !driver.store.pending.is_empty() {
            return driver;
        }
    }
    panic!("start asks for a commit");
}

fn malformed(driver: &mut Driver) -> u64 {
    let (cold, _) = temper_engine_forge_world::changes::change_row().split();
    let pending = driver.store.pending.front_mut().expect("start transaction");
    let mut writes = pending.1.to_vec();
    writes.push(Write::Save(Record::Forge {
        row: Box::new(temper_engine_domain_forge::Stored::ChangeCold(Box::new(cold))),
    }));
    pending.1 = writes.into_boxed_slice();
    pending.0
}

#[test]
fn a_half_change_fails_at_its_commit() {
    let mut driver = starting();
    driver.store_checks(StoreChecks::every());
    let commit = malformed(&mut driver);
    driver.advance(true);
    driver.broken("the writer defect's ForgeRow finding is read below");
    let error = driver.check_store().expect_err("the writer saved only the cold part");
    assert_eq!(error, format!("store check: commit {commit}: start refused at Link: ForgeRow(ChangeCold(51))"));
    assert_eq!(driver.store_check_counts().checked(), 1);
}

#[test]
fn a_correct_store_checks_every_commit() {
    let mut driver = starting();
    driver.store_checks(StoreChecks::every());
    driver.pump();
    driver.close();
    let checks = driver.store_check_counts();
    assert!(checks.commits() > 0);
    assert_eq!(checks.checked(), checks.commits());
    assert!(checks.report().ends_with("(every commit)"));
}

#[test]
fn a_held_commit_is_checked_before_its_acknowledgement() {
    let mut driver = starting();
    driver.store_checks(StoreChecks::every());
    driver.store.fault(Fault::Hold { commits: 1 });
    let commit = malformed(&mut driver);
    driver.advance(true);
    assert_eq!(driver.store.applied, commit);
    assert_eq!(driver.store.held.front(), Some(&commit));
    assert_eq!(driver.store_check_counts().commits(), 1);
    assert!(driver.check_store().expect_err("held durable defect").contains("ForgeRow"));
    driver.advance(false);
    assert_eq!(driver.store_check_counts().commits(), 1, "holding the reply does not count again");
    driver.broken("the held writer defect is the ForgeRow finding consumed above");
}

#[test]
fn a_sample_replays_the_same_commits_and_reports_its_count() {
    let mut first = starting();
    let mut second = starting();
    first.store_checks(StoreChecks::sampled(19, 2));
    second.store_checks(StoreChecks::sampled(19, 2));
    let mut expected = StoreChecks::sampled(19, 2);
    for _ in 0..80 {
        let previous = first.store.applied;
        let checked = first.store_check_counts().checked();
        first.advance(true);
        second.advance(true);
        if first.store.applied != previous {
            let due = expected.due(first.store.applied);
            assert_eq!(first.store_check_counts().checked() != checked, due);
        }
        assert_eq!(first.store_check_counts().report(), second.store_check_counts().report());
    }
    assert!(expected.checked() > 0);
    assert_eq!(first.store_check_counts().report(), expected.report());
}

#[test]
fn rehearsals_leave_the_live_replay_digest_untouched() {
    let mut first = starting();
    let mut second = starting();
    first.store_checks(StoreChecks::every());
    second.store_checks(StoreChecks::none("replay control: compare the live world without rehearsal"));
    for step in 0..80 {
        first.advance(true);
        second.advance(true);
        replay::same_digest(
            &(&first.root, &first.store, &first.delivered),
            &(&second.root, &second.store, &second.delivered),
            &step,
        );
    }
    assert!(first.store_check_counts().checked() > 0);
    assert_eq!(second.store_check_counts().checked(), 0);
    assert!(first.root.ready());
    assert_eq!(first.switches, second.switches);
    assert_eq!(first.host_requests, second.host_requests);
}

fn default_report(seed: u64) -> String {
    let mut driver = starting_with_seed(seed);
    let before = driver.store.applied;
    driver.pump();
    assert_eq!(driver.store_check_counts().checked(), 0, "even a settled live story hasn't checked yet");
    driver.close_erasing(None).expect("the kept store starts");
    let commits = driver.store.applied - before;
    assert!(commits > 1, "the story exercises the reservoir after bootstrap");
    let checks = driver.store_check_counts();
    assert_eq!(checks.checked(), 1);
    assert_eq!(checks.commits(), commits, "include the closing oracle's commits, not bootstrap's");
    let report = checks.report();
    driver.close_erasing(None).expect("the kept image is opened once");
    assert_eq!(driver.store_check_counts().report(), report);
    report
}

#[test]
fn the_default_keeps_one_commit_and_its_seed_replays_it() {
    let first = default_report(4);
    assert!(first.ends_with("(one per run, seed 4, commit 4)"), "{first}");
    assert_eq!(default_report(4), first);
    let second = default_report(26);
    assert!(second.ends_with("(one per run, seed 26, commit 5)"), "{second}");
}

#[test]
fn a_kept_half_change_fails_only_as_the_story_ends() {
    // Seed 4 keeps the first counted commit. The live world is repaired later.
    let mut driver = starting_with_seed(4);
    let commit = malformed(&mut driver);
    driver.pump();
    assert_eq!(driver.check_store(), Ok(()));
    driver.store.seed(
        Box::new([]),
        Box::new([temper_engine_domain::Key::Forge(temper_engine_domain_forge::Key::ChangeCold(51))]),
    );
    driver.broken("the fixture repaired the store directly, outside the live root's commit sequence");
    let error = driver.close_erasing(None).expect_err("the kept store saved only the cold part");
    assert_eq!(error, format!("store check: commit {commit}: start refused at Link: ForgeRow(ChangeCold(51))"));
    assert_eq!(driver.store_check_counts().checked(), 1);
    driver.close_erasing(None).expect("the finding is not emitted again at Drop");
}

#[test]
fn transferring_a_store_does_not_check_the_departing_run() {
    let mut driver = starting_with_seed(4);
    let commit = malformed(&mut driver);
    driver.advance(true);
    let store = driver.take_store();
    drop(driver);
    assert_eq!(store.applied, commit);
}
