//! The view's recovery and file-operation lifecycle (protocol/git-operations.md, 1.13).
use temper_worker_domain_checkout::git;
use temper_worker_protocol_git_world::{
    fixtures,
    io::{Invocation, Node},
    referee::{Expected, Verdict},
    world::{self, Stop},
};

fn repair_script(scenario: &mut world::Scenario) {
    scenario.script.invocations.extend([Invocation::captured("object-format"), Invocation::captured("init")]);
}
fn view_broken(outcome: &world::Outcome) {
    assert_eq!(outcome.verdict, Verdict::Passed);
    let [(_, git::Done::FailedWithOutput { fault, diagnostic })] = outcome.terminals.as_slice() else {
        panic!("one view failure: {:?}", outcome.terminals)
    };
    assert_eq!(*fault, git::Fault::Broken);
    assert_eq!(diagnostic.command(), Some(b"view".as_slice()));
}

#[test]
fn every_run_directory_step_failing_once_remakes_the_view_and_twice_is_broken() {
    let baseline = world::run(&fixtures::view(), fixtures::SEED);
    assert_eq!(baseline.verdict, Verdict::Passed);
    assert!(baseline.view_steps > 30, "copies and directory operations were exercised");
    for step in 1..=baseline.view_steps {
        for attempts in [1, 2] {
            let mut scenario = fixtures::view();
            repair_script(&mut scenario);
            scenario.script.view_failure = Some((step, attempts));
            if attempts == 2 {
                scenario.expected = Expected::fault(git::Fault::Broken);
            }
            let outcome = world::run(&scenario, fixtures::SEED);
            assert_eq!(outcome.verdict, Verdict::Passed, "step {step}, attempts {attempts}");
            if attempts == 2 {
                view_broken(&outcome);
            } else {
                assert_eq!(outcome.terminals, [(fixtures::OWNER, git::Done::Succeeded)], "step {step}");
            }
            assert_eq!(outcome.spawns, 3, "format and init after step {step}");
        }
    }
}

#[test]
fn a_deadline_or_cancel_at_each_view_step_closes_files_without_remaking() {
    let baseline = world::run(&fixtures::view(), fixtures::SEED);
    for step in 1..=baseline.view_steps {
        for (stop, fault) in
            [(Stop::FileDeadline(step), git::Fault::TimedOut), (Stop::FileCancel(step), git::Fault::Cancelled)]
        {
            let mut scenario = fixtures::view();
            scenario.stop = stop;
            scenario.expected = Expected::fault(fault);
            let outcome = world::run(&scenario, fixtures::SEED);
            assert_eq!(outcome.verdict, Verdict::Passed, "step {step}, {stop:?}");
            let [(_, git::Done::FailedWithOutput { diagnostic, .. })] = outcome.terminals.as_slice() else {
                panic!("view stopped")
            };
            assert_eq!(diagnostic.command(), Some(b"view".as_slice()));
            assert_eq!(outcome.spawns, 1, "no remake after stop");
        }
    }
}

#[test]
fn a_missing_file_or_link_in_place_of_a_directory_is_set_aside_before_init() {
    for obstruction in 0..4 {
        let mut scenario = fixtures::view();
        repair_script(&mut scenario);
        let directory = b"workspaces/7/specimen/.git".to_vec();
        match obstruction {
            0 => {
                scenario.files.remove(&directory);
            }
            1 => {
                scenario.files.insert(directory, Node::File(b"gitdir: elsewhere\n".as_slice().into()));
            }
            2 => {
                scenario.files.insert(directory, Node::Link);
            }
            3 => {
                scenario.files.insert(
                    b"workspaces/7/specimen/.git/refs".to_vec(),
                    Node::File(b"not a directory".as_slice().into()),
                );
            }
            _ => unreachable!(),
        }
        let outcome = world::run(&scenario, fixtures::SEED);
        assert_eq!(outcome.verdict, Verdict::Passed, "obstruction {obstruction}");
        assert!(matches!(outcome.tree.get(b"workspaces/7/specimen/.git".as_slice()), Some(Node::Directory)));
        assert_eq!(outcome.spawns, 3);
    }
}

#[test]
fn the_view_replaces_target_and_temporary_links_without_following_them() {
    let mut scenario = fixtures::view();
    repair_script(&mut scenario);
    scenario.files.insert(b"workspaces/7/specimen/.git/HEAD".to_vec(), Node::Link);
    scenario.files.insert(b"workspaces/7/specimen/.git/temper-view.tmp".to_vec(), Node::Link);
    let outcome = world::run(&scenario, fixtures::SEED);
    assert_eq!(outcome.verdict, Verdict::Passed);
    for (path, node) in &scenario.files {
        if let Some(name) = path.strip_prefix(b"gitdirs/7/specimen/") {
            let Node::File(expected) = node else { panic!("worker file") };
            let run_path = [b"workspaces/7/specimen/.git/".as_slice(), name].concat();
            let Some(Node::File(bytes)) = outcome.tree.get(&run_path) else { panic!("view file") };
            assert_eq!(bytes, expected, "{}", String::from_utf8_lossy(name));
        }
    }
    let Some(Node::File(alternates)) =
        outcome.tree.get(b"workspaces/7/specimen/.git/objects/info/alternates".as_slice())
    else {
        panic!("view borrows objects")
    };
    assert_eq!(alternates.as_ref(), b"../../../../../gitdirs/7/specimen/objects\n");
    assert_eq!(outcome.spawns, 1, "target links need no remake");
}

#[test]
fn nonwriting_operations_and_failed_writers_start_no_view() {
    for (name, mut scenario) in fixtures::operations() {
        if let git::Op::Look { at, .. } = &scenario.op {
            scenario.files.insert(
                [temper_worker_protocol_git::invocation::place(at).as_ref(), b"/.git"].concat(),
                Node::Directory,
            );
        }
        if matches!(
            scenario.op,
            git::Op::Clone { .. }
                | git::Op::Fetch { .. }
                | git::Op::Point { .. }
                | git::Op::CheckOut { .. }
                | git::Op::Merge { .. }
                | git::Op::Commit { .. }
                | git::Op::Reset { .. }
        ) {
            scenario.script.invocations.front_mut().expect("writer invocation").fault =
                temper_worker_protocol_git_world::io::Fault::Spawn;
            scenario.expected = Expected::fault(git::Fault::Broken);
        } else if matches!(
            scenario.op,
            git::Op::Make { .. }
                | git::Op::Store { .. }
                | git::Op::Create { .. }
                | git::Op::Push { .. }
                | git::Op::Look { .. }
                | git::Op::IsAncestor { .. }
        ) {
        } else {
            panic!("unclassified fixture");
        }
        let outcome = world::run(&scenario, fixtures::SEED);
        assert_eq!(outcome.verdict, Verdict::Passed, "{name}");
        assert_eq!(outcome.view_steps, 0, "{name}");
    }
    // A source read is a worker failure, not a reason to remake the run's directory.
    let mut scenario = fixtures::view();
    scenario.files.insert(b"gitdirs/7/specimen/HEAD".to_vec(), Node::Link);
    scenario.expected = Expected::fault(git::Fault::Broken);
    let outcome = world::run(&scenario, fixtures::SEED);
    view_broken(&outcome);
    assert_eq!(outcome.spawns, 1);
}
