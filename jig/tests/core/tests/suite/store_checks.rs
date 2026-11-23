//! A committed snapshot must cold-start apart from the live world (domain/testing.md, 7).
use jig_core_world::{
    fixtures,
    world::{Cut, Setup, StoreChecks, World, fixture},
};
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use jig_test_domain as root;

const SEED: u64 = 2911;

#[test]
#[ignore = "selected by the isolated store-check count regression"]
fn rehearse_every_commit_in_child() {
    let mut world = World::new(Setup { seed: SEED, ..Setup::default() });
    world.store_checks(StoreChecks::every());
    for _ in 0..3 {
        world.try_seed(Box::new([])).expect("correct fixture commit");
    }
    world.finish().expect("closing oracle");
    let checked = world.store_checks.checked();
    assert!(checked > 1);
    assert_eq!(checked, world.store_checks.commits());
}

#[test]
fn every_rehearsal_writes_one_store_check_line() {
    use std::{env, fs, process::Command};

    let scratch = skein_scratch::Scratch::new("store-check-counts");
    let path = scratch.path().join("checks");
    for enabled in [false, true] {
        let mut child = Command::new(env::current_exe().expect("test binary"));
        child
            .args(["--exact", "--ignored", "--nocapture", "store_checks::rehearse_every_commit_in_child"])
            .env_remove("TIERS_CHECKS_FILE");
        if enabled {
            child.env("TIERS_CHECKS_FILE", &path);
        }
        let result = child.output().expect("run isolated store check story");
        assert!(result.status.success(), "{result:?}");
        let stderr = std::str::from_utf8(&result.stderr).expect("store check report is UTF-8");
        let report = stderr.lines().find_map(|line| line.strip_prefix("store check: ")).expect("child ran the story");
        let (checked, _) = report.split_once(" of ").expect("report names the checked count");
        let checked = checked.parse::<usize>().expect("fixture count fits usize");
        if enabled {
            let counts = fs::read_to_string(&path).expect("store check counts");
            assert_eq!(counts, "store-check 1\n".repeat(checked));
        } else {
            assert!(!path.exists(), "unset counting leaves no file");
        }
    }
}

fn missing_project(world: &World) -> Box<[root::Record]> {
    let mut row = fixtures::live_row(world, 1).clone();
    row.project = 99;
    Box::new([root::Record::Core(core::Record::Tasks(tasks::Stored::Live(Box::new(row))))])
}

fn expected(commit: u64) -> String {
    format!(
        "store check: commit {commit}: start refused at Link: Link {{ record: Tasks(Live(1)), linked: Project(99) }}"
    )
}

#[test]
fn a_refused_record_fails_at_its_fixture_commit() {
    let mut world = World::new(Setup { seed: SEED, ..Setup::default() });
    world.store_checks(StoreChecks::every());
    let commit = world.store.applied + 1;
    world.broken(); // This story's outcome is the refused store, not the closing oracle.
    assert_eq!(world.try_seed(missing_project(&world)), Err(expected(commit)));
    assert_eq!(world.store.applied, commit);
    assert_eq!(world.store_checks.checked(), 1);
    assert_eq!(world.store_checks.commits(), 1);
}

#[test]
fn every_commit_passes_without_changing_the_live_story_or_oracle() {
    let mut checked = World::new(Setup { seed: SEED, ..Setup::default() });
    let mut unchecked = World::new(Setup { seed: SEED, ..Setup::default() });
    checked.store_checks(StoreChecks::every());
    unchecked.store_checks(StoreChecks::none("paired live-world isolation control"));
    let first = checked.store.applied;
    for world in [&mut checked, &mut unchecked] {
        world.try_seed(Box::new([])).expect("correct fixture commit");
        world.turn(1, 1, None, b"durable answer");
        world.end(tasks::End::Parked, 1);
        world.finish().expect("unchanged closing oracle");
    }
    assert!(checked.store_checks.commits() > 1);
    assert_eq!(checked.store_checks.checked(), checked.store_checks.commits());
    assert_eq!(checked.store_checks.commits(), checked.store.applied - first);
    assert_eq!(unchecked.store_checks.checked(), 0);
    assert_eq!(checked.store.rows, unchecked.store.rows);
    assert_eq!(checked.commits, unchecked.commits);
    assert_eq!(checked.assignments, unchecked.assignments);
    assert_eq!(checked.systems[0].observed(), unchecked.systems[0].observed());
}

fn sampled() -> (Vec<u64>, String) {
    let mut world = World::new(Setup { seed: SEED, ..Setup::default() });
    world.store_checks(StoreChecks::sampled(SEED, 4));
    let mut selected = Vec::new();
    let mut policy = StoreChecks::sampled(SEED, 4);
    for _ in 0..16 {
        let before = world.store_checks.checked();
        world.try_seed(Box::new([])).expect("correct sampled commit");
        let commit = world.store.applied;
        let due = policy.due(commit);
        assert_eq!(world.store_checks.checked() != before, due, "commit {commit}");
        if due {
            selected.push(commit);
        }
    }
    let report = world.store_checks.report();
    assert_eq!(report, format!("store check: {} of 16 commits checked (one in 4, seed {SEED})", selected.len()));
    assert!(!selected.is_empty() && selected.len() < 16);
    world.finish().expect("sampled oracle");
    (selected, report)
}

#[test]
fn the_sample_replays_its_selected_commits_and_reports_its_count() {
    assert_eq!(sampled(), sampled());
}

#[test]
fn a_sweeps_first_restart_refusal_fails_its_child_and_names_the_cut() {
    let mut world = World::new(Setup { seed: SEED, ..Setup::default() });
    world.store_checks(StoreChecks::none("the sweep's children check"));
    let commit = world.store.applied + 1;
    let cut = Cut::Durable(commit);
    world.sweep(vec![cut]);
    world.broken(); // The parent deliberately keeps the row its child refuses.
    let result = world.try_seed(missing_project(&world));
    if world.forked_cut().is_some() {
        assert_eq!(world.try_restart_with(fixture(SEED, false).0), Err(expected(commit)));
        std::process::exit(23);
    }
    assert_eq!(result, Err(format!("forked cut {cut:?} failed: Code(23)")));
    assert_eq!(world.store_checks.checked(), 0, "a failed child is not counted as a successful check");
}

#[test]
fn a_sweep_checks_a_page_refusal_after_its_first_restart_waits() {
    use jig_protocol_store::restore::{Cause, Failure, RangeFailure};
    let mut world = World::new(Setup { seed: SEED, ..Setup::default() });
    world.store_checks(StoreChecks::none("the sweep's children check"));
    let commit = world.store.applied + 1;
    let cut = Cut::Durable(commit);
    world.sweep(vec![cut]);
    let result = world.try_seed(Box::new([]));
    if world.forked_cut().is_some() {
        world.hold_restore_pages = true;
        world.try_restart_with(fixture(SEED, false).0).expect("first restart waits for its page");
        assert!(!world.domain.ready());
        world.events.push_back(root::Event::StartPageRefused {
            failure: Failure { cause: Cause::Range(RangeFailure::Cursor), row: None },
        });
        assert_eq!(
            world.try_drain(),
            Err(format!(
                "store check: commit {commit}: start refused at LoadCore: Failure {{ cause: Range(Cursor), row: None }}"
            )),
        );
        std::process::exit(23);
    }
    assert_eq!(result, Err(format!("forked cut {cut:?} failed: Code(23)")));
    assert_eq!(world.store_checks.checked(), 0);
}

#[test]
fn successful_sweep_children_count_each_durable_commit_once() {
    let mut world = World::new(Setup { seed: SEED, ..Setup::default() });
    world.store_checks(StoreChecks::none("the sweep's children check"));
    let first = world.store.applied + 1;
    let cuts: Vec<_> = (first..first + 3).map(Cut::Durable).collect();
    world.sweep(cuts.clone());
    for _ in 0..3 {
        world.try_seed(Box::new([])).expect("correct swept commit");
        if let Some(Cut::Durable(commit)) = world.forked_cut() {
            assert_eq!(world.store.applied, commit);
            world.restart(SEED, false);
            assert!(world.domain.ready());
            world.cut_passed();
        }
    }
    assert_eq!(world.forked, cuts);
    assert_eq!(world.store_checks.commits(), 3);
    assert_eq!(world.store_checks.checked(), 3);
}
