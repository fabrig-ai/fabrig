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
}

#[test]
fn a_point_failure_is_broken_without_forge_classification() {
    let mut scenario = fixtures::point();
    let invocation = scenario.script.invocations.front_mut().expect("update-ref");
    invocation.transcript.exit = 128;
    invocation.transcript.stderr = b"fatal: Authentication failed for 'remote'\n".as_slice().into();
    scenario.expected = Expected::fault(git::Fault::Broken);
    let outcome = world::run(&scenario, fixtures::SEED);
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert_eq!(outcome.spawns, 1);
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
fn memory_stays_within_its_heap_ceiling_at_its_limits() {
    // Measured peak: 14,427 bytes with all git operation slots capturing maximum output, including the driver; 25% headroom, rounded up.
    const GIT_OPERATIONS_HEAP_CEILING: u64 = 19_000;
    let peak = fixtures::memory_peak();
    assert!(peak > 0, "the allocator observed the component");
    assert!(peak <= GIT_OPERATIONS_HEAP_CEILING);
}

#[test]
fn a_store_already_there_is_kept_without_a_clone_after_setting_aside() {
    for aside in [None, Some(skein_lib::Token::new(8))] {
        let mut scenario =
            fixtures::operations().into_iter().find(|(name, _)| *name == "store").expect("store sequence").1;
        let git::Op::Store { aside: target, .. } = &mut scenario.op else { panic!("Store fixture") };
        *target = aside;
        scenario.files.insert(b"stores/9".to_vec(), Node::Directory);
        scenario.files.insert(b"stores/9/kept".to_vec(), Node::File(b"first clone".as_slice().into()));
        if aside.is_some() {
            scenario.files.insert(b"stores/8".to_vec(), Node::Directory);
            scenario.files.insert(b"stores/8/old".to_vec(), Node::File(b"unborrowed".as_slice().into()));
        }
        let outcome = world::run(&scenario, fixtures::SEED);
        let Some(Node::File(kept)) = outcome.tree.get(b"stores/9/kept".as_slice()) else {
            panic!("first clone remains")
        };
        assert_eq!(kept.as_ref(), b"first clone");
        assert_eq!(outcome.verdict, Verdict::Passed);
        assert_eq!(outcome.spawns, 0, "existing store needs no clone");
        assert!(!outcome.tree.keys().any(|path| path.starts_with(b"making/")));
        if aside.is_some() {
            assert!(outcome.tree.contains_key(b"trash/1003/old".as_slice()));
        }
    }
}

#[test]
fn a_restart_keeps_a_completed_store_and_runs_no_second_clone() {
    let mut scenario = fixtures::operations().into_iter().find(|(name, _)| *name == "store").expect("store sequence").1;
    let first = world::run(&scenario, fixtures::SEED);
    assert_eq!(first.verdict, Verdict::Passed);
    assert_eq!(first.spawns, 1);
    assert!(matches!(first.tree.get(b"stores/9".as_slice()), Some(Node::Directory)));
    assert!(!first.tree.keys().any(|path| path.starts_with(b"making/")));
    let mut root = first.tree;
    root.insert(b"stores/9/objects".to_vec(), Node::File(b"completed store".as_slice().into()));
    root.insert(b"making/interrupted".to_vec(), Node::Directory);
    scenario.script.invocations.clear();
    let restarted = world::run_on_tree(&scenario, fixtures::SEED, root);
    assert_eq!(restarted.verdict, Verdict::Passed);
    assert_eq!(restarted.spawns, 0);
    let Some(Node::File(objects)) = restarted.tree.get(b"stores/9/objects".as_slice()) else {
        panic!("restart keeps the store's contents")
    };
    assert_eq!(objects.as_ref(), b"completed store");
    assert!(!restarted.tree.keys().any(|path| path.starts_with(b"making/")));
}
