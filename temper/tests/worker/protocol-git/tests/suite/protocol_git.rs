use skein_lib::Token;
use std::collections::BTreeMap;
use temper_worker_domain_checkout::git;
use temper_worker_protocol_git::invocation::{self, Command};
use temper_worker_protocol_git_world::{
    fixtures,
    io::{Fault, IndexOverride, Invocation, Node, is_command},
    referee::{Expected, Verdict},
    world::{self, Stop},
};

mod sweep;
pub(crate) use sweep::sweep_past_a_stuck_file;

#[test]
fn a_mark_keeps_only_its_workspace_and_git_directories_across_start() {
    let mut scenario = fixtures::operations().remove(0).1;
    let mark = git::Kept {
        workspace: Token::new(7),
        key: b"stream".as_slice().into(),
        when: 42,
        repositories: Box::new([git::KeptRepository {
            name: b"app".as_slice().into(),
            remote: b"remote".as_slice().into(),
            commit: git::Commit::new([3; 32]),
        }]),
    };
    scenario.op = git::Op::Keep { workspace: mark.workspace, mark: Some(mark.clone()) };
    for directory in [
        "workspaces/7",
        "workspaces/7/app",
        "workspaces/7/app/.git",
        "workspaces/8",
        "gitdirs/7",
        "gitdirs/7/app",
        "gitdirs/8",
    ] {
        scenario.files.insert(directory.as_bytes().to_vec(), Node::Directory);
    }
    for file in [
        "workspaces/7/app/output",
        "workspaces/7/app/.git/passed",
        "gitdirs/7/app/HEAD",
        "workspaces/8/lost",
        "gitdirs/8/lost",
    ] {
        scenario.files.insert(file.as_bytes().to_vec(), Node::File(b"content".as_slice().into()));
    }
    let written = world::run(&scenario, fixtures::SEED);
    assert_eq!(written.verdict, Verdict::Passed);
    assert!(written.tree.contains_key(b"kept/7".as_slice()));
    scenario.files.clear();
    scenario.op = git::Op::Keep { workspace: Token::new(999), mark: None };
    let restarted = world::run_on_tree(&scenario, fixtures::SEED, written.tree);
    assert_eq!(restarted.verdict, Verdict::Passed);
    assert_eq!(restarted.kept.as_ref(), &[mark]);
    for file in ["workspaces/7/app/output", "workspaces/7/app/.git/passed", "gitdirs/7/app/HEAD"] {
        assert!(restarted.tree.contains_key(file.as_bytes()), "{file} survives");
    }
    assert!(!restarted.tree.keys().any(|path| path.starts_with(b"workspaces/8") || path.starts_with(b"gitdirs/8")));
}

#[test]
fn a_mark_for_a_missing_workspace_is_removed_at_start() {
    let mut scenario = fixtures::operations().remove(0).1;
    scenario.op = git::Op::Keep {
        workspace: Token::new(7),
        mark: Some(git::Kept {
            workspace: Token::new(7),
            key: b"stream".as_slice().into(),
            when: 42,
            repositories: Box::new([git::KeptRepository {
                name: b"app".as_slice().into(),
                remote: b"remote".as_slice().into(),
                commit: git::Commit::new([3; 32]),
            }]),
        }),
    };
    let written = world::run(&scenario, fixtures::SEED);
    assert_eq!(written.verdict, Verdict::Passed);
    scenario.op = git::Op::Keep { workspace: Token::new(999), mark: None };
    let restarted = world::run_on_tree(&scenario, fixtures::SEED, written.tree);
    assert!(restarted.kept.is_empty());
    assert!(!restarted.tree.contains_key(b"kept/7".as_slice()));
}

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
    invocation.exit = 128;
    invocation.stderr = b"fatal: Authentication failed for 'remote'\n".as_slice().into();
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
fn a_cancel_racing_the_last_git_exit_still_preempts_the_view() {
    for race in [fixtures::Race::ExitFirst, fixtures::Race::CancelFirst] {
        let outcome = world::run(&fixtures::cancel_race(race), fixtures::SEED);
        assert_eq!(outcome.verdict, Verdict::Passed, "{race:?}");
        assert_eq!(outcome.faults(), [(fixtures::OWNER, Some(git::Fault::Cancelled))]);
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
            0 => invocation.stdout = vec![b'x'; 513].into_boxed_slice(),
            1 => {
                invocation.stdout = Box::from(&invocation.stdout[..invocation.stdout.len() - 1]);
            }
            2 => {
                invocation.exit = 1;
                invocation.stderr = vec![b'x'; 513].into_boxed_slice();
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

fn reply(op: &git::Op, command: &Command, stdout: &[u8]) -> Invocation {
    let args = invocation::arguments_for(op, command);
    let word = args.iter().skip(26).map(AsRef::as_ref).find(|word| is_command(word)).expect("scripted command");
    Invocation::scripted(word, 0, stdout, b"")
}

#[test]
fn delivery_refuses_a_marker_on_the_last_private_index_page() {
    // The fixture permits four paths per page; the run has staged its first two.
    const FIRST_PAGE: &[u8] = b"file0\0file1\0file2\0file3\0";
    const LAST_PAGE: &[u8] = b"file4\0file5\0";
    const RUN_INDEX: &[u8] = b"workspaces/7/specimen/.git/index";
    const WORKER_INDEX: &[u8] = b"gitdirs/7/specimen/index";
    const SCAN_INDEX: &[u8] = b"gitdirs/7/specimen/temper-scan";
    const PRIVATE_LISTING: u32 = 7;
    const PAGE_UPDATE: u32 = PRIVATE_LISTING - 1;

    let mut scenario = fixtures::commit(true);
    let remote: Box<[u8]> = b"https://forge.example.test/fixture/specimen.git".as_slice().into();
    let git::Op::Commit { snapshot, .. } = &mut scenario.op else { panic!("merge commit fixture") };
    *snapshot = Some(git::Snapshot { remote: remote.clone(), account: None });
    let (head, second) = (fixtures::ONE, fixtures::TWO);
    let head_line = [invocation::hex(head).as_ref(), b" \n"].concat();
    let second_line = [invocation::hex(second).as_ref(), b"\n"].concat();
    let unmerged = [FIRST_PAGE, LAST_PAGE].concat();
    scenario.script.read_stdin_to_end = true;
    scenario.script.invocations = [
        reply(&scenario.op, &Command::SnapshotHead, &head_line),
        reply(&scenario.op, &Command::SnapshotMerge { head, second: None }, &second_line),
        reply(&scenario.op, &Command::SnapshotFetch { remote, head, merging: Some(second), second }, b""),
        reply(&scenario.op, &Command::SnapshotReachable { head, merging: Some(second), second }, &second_line),
        reply(&scenario.op, &Command::Unmerged, &unmerged),
        reply(&scenario.op, &Command::UpdateIndex { paths: FIRST_PAGE.into() }, b""),
        reply(&scenario.op, &Command::Unmerged, LAST_PAGE),
    ]
    .into();
    let last = b"file5";
    let run_index = b"file2\0file3\0file4\0file5\0";
    scenario.files.insert(b"gitdirs/7/specimen".to_vec(), Node::Directory);
    scenario.files.insert(WORKER_INDEX.to_vec(), Node::File(unmerged.clone().into_boxed_slice()));
    scenario.files.insert(RUN_INDEX.to_vec(), Node::File(run_index.as_slice().into()));
    for file in unmerged.strip_suffix(b"\0").expect("terminated listing").split(|byte| *byte == 0) {
        let path = [b"workspaces/7/specimen/", file].concat();
        let contents = if file == last { b"unresolved:\n<<<<<<< ours\n".as_slice() } else { b"resolved\n" };
        scenario.files.insert(path, Node::File(contents.into()));
    }
    scenario.expected = Expected::done(git::Done::Conflicted { files: Box::new([last.as_slice().into()]), cut: false })
        .after_remove(SCAN_INDEX);

    let mut world = world::World::new(fixtures::SEED, BTreeMap::new());
    let layout_files = world.below.open_files();
    let outcome = world.run(&scenario).expect("delivery settles");
    assert_eq!(world.below.open_files(), layout_files, "no operation file remains open");
    assert_eq!(outcome.verdict, Verdict::Passed);
    assert_eq!(outcome.spawns, PRIVATE_LISTING, "no staging, commit, reset or further page");
    assert!(world.below.script.invocations.is_empty(), "all scripted commands ran");
    let listing = [
        b"--git-dir=gitdirs/7/specimen".as_slice(),
        b"--work-tree=workspaces/7/specimen",
        b"diff-files",
        b"--name-only",
        b"-z",
        b"--diff-filter=U",
    ];
    let argv = |spawn: u32| {
        world.below.arguments[usize::try_from(spawn - 1).expect("small script")]
            .iter()
            .map(AsRef::as_ref)
            .collect::<Vec<_>>()
    };
    assert!(argv(PAGE_UPDATE - 1).ends_with(&listing));
    assert!(argv(PRIVATE_LISTING).ends_with(&listing));
    assert!(argv(PAGE_UPDATE).ends_with(&[b"update-index", b"--add", b"--remove", b"-z", b"--stdin"]));
    assert_eq!(world.below.messages[usize::try_from(PAGE_UPDATE - 1).expect("small script")], FIRST_PAGE);
    let index_override = |spawn| IndexOverride {
        spawn,
        path: [b"/worker/", SCAN_INDEX].concat().into_boxed_slice(),
        contents: Some(unmerged.clone().into_boxed_slice()),
    };
    assert_eq!(world.below.index_overrides, [index_override(PAGE_UPDATE), index_override(PRIVATE_LISTING)]);
    let Some(Node::File(bytes)) = outcome.tree.get(WORKER_INDEX) else { panic!("worker index remains") };
    assert_eq!(bytes.as_ref(), unmerged, "the worker's six unmerged entries are unchanged");
    world.shutdown();
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
fn the_sweep_skips_a_stuck_file_and_removes_it_after_the_fault_is_lifted() {
    sweep_past_a_stuck_file(fixtures::SEED);
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
            assert!(outcome.tree.contains_key(b"trash/1002/old".as_slice()));
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
