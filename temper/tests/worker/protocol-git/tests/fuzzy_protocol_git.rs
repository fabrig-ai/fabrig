use temper_worker_domain_checkout::git;
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
