use temper_worker_domain_checkout::git;
use temper_worker_protocol_git_world::{
    fixtures,
    io::{Fault, Node},
    referee::{Expected, Verdict},
    world::{self, Stop},
};

#[test]
fn each_operation_ends_once_with_its_transcript_s_outcome() {
    for (name, mut scenario) in fixtures::operations() {
        for exit_first in [false, true] {
            scenario.script.exit_first = exit_first;
            let outcome = world::run(&scenario, fixtures::SEED);
            assert_eq!(outcome.verdict, Verdict::Passed, "{name}, exit first {exit_first}");
        }
    }
    for fault in [Fault::Spawn, Fault::Signal] {
        let mut scenario = fixtures::fetch();
        scenario.script.invocations.front_mut().expect("fetch").fault = fault;
        scenario.expected = Expected::fault(git::Fault::Broken);
        assert_eq!(world::run(&scenario, fixtures::SEED).verdict, Verdict::Passed);
    }
    let mut missing = fixtures::fetch();
    missing.identity = false;
    missing.expected = Expected::fault(git::Fault::Broken);
    assert_eq!(world::run(&missing, fixtures::SEED).spawns, 0);
}

#[test]
fn a_deadline_at_each_invocation_ends_timed_out_and_starts_no_more() {
    for (name, mut scenario) in fixtures::operations() {
        for invocation in 1..=u32::try_from(scenario.script.invocations.len()).expect("sequence") {
            scenario.stop = Stop::Deadline(invocation);
            scenario.expected = Expected::fault(git::Fault::TimedOut);
            let outcome = world::run(&scenario, fixtures::SEED);
            assert_eq!(outcome.verdict, Verdict::Passed, "{name}, invocation {invocation}");
            assert_eq!(outcome.spawns, invocation);
            assert_eq!(outcome.signals, 1);
        }
    }
}

#[test]
fn a_cancel_racing_the_last_exit_ends_with_whichever_won() {
    for (race, fault) in [(fixtures::Race::ExitFirst, None), (fixtures::Race::CancelFirst, Some(git::Fault::Cancelled))]
    {
        let outcome = world::run(&fixtures::cancel_race(race), fixtures::SEED);
        assert_eq!(outcome.verdict, Verdict::Passed, "{race:?}");
        assert_eq!(outcome.faults(), [(fixtures::OWNER, fault)]);
    }
    for (name, mut scenario) in fixtures::operations() {
        for invocation in 1..=u32::try_from(scenario.script.invocations.len()).expect("sequence") {
            scenario.stop = Stop::Cancel(invocation);
            scenario.expected = Expected::fault(git::Fault::Cancelled);
            assert_eq!(world::run(&scenario, fixtures::SEED).spawns, invocation, "{name}");
        }
    }
}

#[test]
fn outputs_cut_at_random_and_past_each_cap_end_as_git_md_says() {
    for seed in 1..=8 {
        assert_eq!(world::run(&fixtures::fetch(), seed).verdict, Verdict::Passed);
    }
    for change in 0..3 {
        let mut scenario = fixtures::fetch();
        let invocation = scenario.script.invocations.front_mut().expect("fetch");
        match change {
            0 => invocation.transcript.stdout = vec![b'x'; 513].into_boxed_slice(),
            1 => {
                invocation.transcript.stdout =
                    Box::from(&invocation.transcript.stdout[..invocation.transcript.stdout.len() - 1]);
            }
            2 => {
                invocation.transcript.exit = 1;
                invocation.transcript.stderr = vec![b'x'; 513].into_boxed_slice();
            }
            _ => unreachable!("three cases"),
        }
        scenario.expected = Expected::fault(git::Fault::Broken);
        let outcome = world::run(&scenario, fixtures::SEED);
        if change == 2 {
            let git::Done::FailedWithOutput { diagnostic, .. } = &outcome.terminals[0].1 else {
                panic!("output diagnostic")
            };
            assert_eq!(diagnostic.output(), &[b'x'; 512]);
            assert_eq!(diagnostic.cut(), 1);
        }
    }
}

#[test]
fn the_sweep_empties_deep_and_wide_trees_in_pages_of_two() {
    let outcome = world::run(&fixtures::sweep(), fixtures::SEED);
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert!(!outcome.tree.keys().any(|path| path.starts_with(b"trash/")));
    assert!(matches!(outcome.tree.get(b"workspaces/7".as_slice()), Some(Node::Directory)));
    let mut failed = fixtures::sweep();
    failed.script.file_failure = Some((1, skein_io::kernel::Error::Permission));
    failed.expected = Expected::fault(git::Fault::Broken);
    assert_eq!(world::run(&failed, fixtures::SEED).verdict, Verdict::Passed);
}

#[test]
fn memory_stays_within_the_worst_case_at_its_limits() {
    let peak = fixtures::memory_peak();
    assert!(peak > 0, "the allocator observed the component");
    assert!(peak <= temper_worker_protocol_git::worst_case(&fixtures::limits(), &fixtures::checkout()).expect("bound"));
}
