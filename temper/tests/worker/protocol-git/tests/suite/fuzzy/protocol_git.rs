use skein_world::{
    streams::{Stream, Streams},
    sweep::Sweep,
};
use temper_worker_domain_checkout::git;
use temper_worker_inspect::faults::TRASH_STUCK;
use temper_worker_protocol_git_world::{fixtures, referee::Verdict, world};

#[test]
fn drawn_operations_and_faults_keep_the_referee_s_rules() {
    let mut faults = Vec::new();
    for seed in 1..=64 {
        let outcome = world::run(&fixtures::drawn(seed), seed);
        assert_eq!(outcome.verdict, Verdict::Passed, "seed {seed}");
        faults.extend(outcome.faults().into_iter().filter_map(|(_, fault)| fault));
    }
    for fault in [git::Fault::Broken, git::Fault::TimedOut, git::Fault::Cancelled] {
        assert!(faults.contains(&fault), "the injected {fault:?} fell");
    }
}

#[test]
fn seeded_trash_faults_fall_and_the_checks_hold() {
    let mut sweep = Sweep::new("worker trash faults");
    sweep.configure(TRASH_STUCK);
    for seed in 1..=8 {
        eprintln!("seed {seed}");
        let streams = Streams::new(seed);
        let mut workload = streams.rng(Stream::Workload, "worker");
        let mut world = world::World::with_trash_faults(streams);
        let mut scenarios = vec![fixtures::trash_make()];
        for _ in 0..3 {
            scenarios.push(fixtures::drawn(workload.next_u64()));
        }
        scenarios.push(fixtures::operations().remove(0).1);
        for scenario in scenarios {
            let outcome = world.run(&scenario).unwrap_or_else(|finding| panic!("{finding}"));
            assert_eq!(outcome.verdict, Verdict::Passed, "seed {seed}");
        }
        assert_eq!(world.ledger.entries().len(), 1, "one permanent stuck mark");
        assert_eq!(world.ledger.entries()[0].until, None);
        sweep.ran(&world.ledger);
        world.shutdown();
    }
    sweep.check_fell().expect("configured trash fault fell");
}

#[test]
fn drawn_sweep_runs_leave_only_the_stuck_file_until_the_retry() {
    for seed in 1..=64 {
        crate::protocol_git::sweep_past_a_stuck_file(seed);
    }
}
