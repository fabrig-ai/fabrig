use jig_conformance::{Harness, store_check::StoreChecks};
use jig_conformance_world::{Config, Testing, actions, negative, replay};

#[test]
fn a_writer_leaving_a_task_without_its_project_fails_at_that_commit() {
    let mut world = negative::world(negative::Fault::UnlinkedTask);
    world.store_checks(StoreChecks::every());
    let failure = world.drain().expect_err("the committed missing project must refuse a cold start");
    assert!(
        failure.why.contains(&format!("store check: commit {}: start refused at", world.store.applied)),
        "{failure:?}"
    );
    assert!(failure.why.contains("Link") && failure.why.contains("999"), "{failure:?}");
    assert!(failure.why.contains("Live(1)"), "{failure:?}");
}

#[test]
fn every_commit_passes_without_the_writer_defect() {
    let mut world = actions::world(31, false);
    world.store_checks(StoreChecks::every());
    world.drain().expect("valid startup");
    actions::effect(&mut world, 1);
    let outcome = world.drain().expect("valid effect");
    assert!(outcome.commits > 1);
    assert_eq!(outcome.checked, outcome.commits);
    assert_eq!(
        world.store_check_report(),
        format!("store check: {} of {} commits checked (every commit)", outcome.commits, outcome.commits)
    );
}

#[test]
fn a_cut_child_reads_its_roots_refusal_even_when_the_parent_checks_none() {
    let mut world = negative::world(negative::Fault::UnlinkedTask);
    world.store_checks(StoreChecks::none("the sweep's children check"));
    world.fork_cuts(&[5]);
    let result = world.drain();
    if let Some(commit) = world.forked_at() {
        let failure = result.as_ref().expect_err("this child's restart refuses the writer's record");
        assert!(failure.why.contains(&format!("store check: commit {commit}: start refused at")), "{failure:?}");
        assert!(
            failure.why.contains("Link") && failure.why.contains("999") && failure.why.contains("Live(1)"),
            "{failure:?}"
        );
        jig_conformance::cut_passed();
    }
    // The parent still runs its ordinary referee over the deliberately bad row.
    let failure = result.expect_err("the parent also rejects the writer's invalid authority");
    assert_eq!(failure.promise, jig_conformance::referee::Promise::Authority);
    assert_eq!(world.outcome().forked, [5], "the child's refusal was checked");
}

#[test]
fn a_sample_replays_its_commit_selection_and_reports_its_count() {
    fn run() -> (jig_conformance::Outcome, String) {
        let mut world = actions::world(31, false);
        world.store_checks(StoreChecks::sampled(31, 4));
        let outcome = world.drain().expect("sampled startup");
        let mut selection = StoreChecks::sampled(31, 4);
        for commit in 1..=outcome.commits {
            selection.due(commit);
        }
        assert_eq!(world.store_check_report(), selection.report());
        assert_eq!(outcome.checked, selection.checked());
        assert!(outcome.checked > 0 && outcome.checked < outcome.commits);
        (outcome, world.store_check_report())
    }
    assert_eq!(run(), run());
}

#[test]
fn a_stalled_start_names_its_commit_and_phase() {
    let world = negative::world(negative::Fault::StalledStart);
    let failure = world.check_store().expect_err("the load never gets its answer");
    assert_eq!(failure.why, "store check: commit 0: start stalled at Some(LoadCore)");
}

#[test]
fn checking_a_store_changes_none_of_the_live_world() {
    let mut world = Harness::<Testing>::new(Config::new(31, false, 2, None), 31);
    world.drain().expect("startup");
    actions::effect(&mut world, 1);
    world.drain().expect("an effect gives the live system state unlike a fresh fake");
    let before = replay::outside(&world);
    let clock = world.clock();
    let outcome = world.outcome();
    let trace = world.trace.clone();
    world.check_store().expect("the store alone rehearses");
    assert_eq!(before.first_difference(&replay::outside(&world)), None);
    assert_eq!(world.clock().now, clock.now);
    assert_eq!(world.outcome(), outcome);
    assert_eq!(&*world.trace, &*trace);
}
