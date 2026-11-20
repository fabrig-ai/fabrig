//! File-operation cuts over the simulated disk (domain/testing.md, 9).
use crate::fault_support::{chat, commit_saving};
use jig_core_world::world::{Cut, Setup, World};
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;

fn disk_world(seed: u64, cut: Option<Cut>) -> World {
    disk_world_sweeping(seed, cut, Vec::new())
}

fn disk_world_sweeping(seed: u64, cut: Option<Cut>, sweep: Vec<Cut>) -> World {
    let (configuration, limits) = crate::person_support::person_world_fixture(seed);
    World::new(Setup {
        seed,
        configuration: Some(configuration),
        limits,
        disk_store: true,
        require_assignment: false,
        cut,
        sweep,
        ..Setup::default()
    })
}

fn person_decision(world: &mut World) {
    if world.reached_cut.is_some() {
        return;
    }
    let mut member = jig_core_world::fixtures::member(
        world,
        tasks::Executor::Person(tasks::PersonAddress::Person(1)),
        b"resolve the caller's question",
    );
    member.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    world.send(root::Event::Core(core::Event::DelegateValidated {
        to: skein_lib::ReplyTo::new(skein_lib::Token::new(9801)),
        key: world.call_key(1),
        batch: Box::new([member]),
        stubs: Box::new([]),
    }));
}

fn person_task(row: &root::Record) -> bool {
    matches!(row, root::Record::Core(core::Record::Tasks(tasks::Stored::Live(task)))
        if task.executor == tasks::Executor::Person(tasks::PersonAddress::Person(1)))
}

fn person(row: &root::Record) -> bool {
    matches!(row, root::Record::Core(core::Record::People(people::Stored::Person { .. })))
}

fn cut_operation(cut: Cut) -> u32 {
    let Cut::Disk { operation, .. } = cut else { panic!("disk cut") };
    operation
}

fn disk_cut(commit: u64, operation: u32) -> Cut {
    Cut::Disk { from: jig_test_store_files::KillFrom::Commit(commit), operation }
}

fn disk_referee(world: &mut World, label: &str) {
    assert!(world.domain.ready(), "{label}: {:?}", world.domain.start_failure());
    assert_eq!(world.domain.start_failure(), None, "{label}");
    assert!(!world.stopped, "{label}");
    let actual = world.store.readback().expect("complete readback");
    assert_eq!(actual, world.store.rows, "{label}: complete store readback");
    assert_eq!(Some(&actual), world.store.prefixes.get(&world.store.applied), "{label}: whole decision prefix");
}

fn disk_restart(world: &mut World, seed: u64) {
    let acknowledged = world.store.acknowledged;
    // Inspect the recovered number before the start can write its own decisions.
    world.hold_restore_pages = true;
    world.restart_with(crate::person_support::person_world_fixture(seed).0);
    if !world.store.reached_kill() {
        assert!(
            (acknowledged..=acknowledged + 1).contains(&world.store.applied),
            "last acknowledged decision or one whole uncertain decision"
        );
        world.finish_restore_pages();
    }
}

/// Count the uncut recovery once for the focused and exhaustive second kills.
fn disk_restart_reference(seed: u64, first: Cut) -> (u32, Vec<u32>) {
    let mut world = disk_world(seed, Some(first));
    person_decision(&mut world);
    assert_eq!(world.reached_cut, Some(first));
    disk_restart(&mut world, seed);
    let count = world.store.open_operation_count();
    disk_referee(&mut world, "killed twice");
    (count, world.store.mutation_operations().to_vec())
}

#[test]
fn a_kill_at_each_file_operation_of_a_decision_starts_from_a_whole_one() {
    let seed = 18_301;
    let mut reference = disk_world(seed, None);
    person_decision(&mut reference);
    let task_commit = commit_saving(&reference, person_task);
    let task_writes = &reference.commits[usize::try_from(task_commit - 1).expect("commit index")];
    assert!(
        task_writes
            .iter()
            .filter(|write| matches!(
                write,
                root::Write::Save(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(_))))
            ))
            .count()
            >= 2,
        "the decision saves both the caller and its person task"
    );
    let person_commit = commit_saving(&reference, person);
    let caller_commit = commit_saving(&reference, chat);
    for (index, (name, scenario, commit, cuts)) in [
        (
            "task and linked person task",
            person_decision as fn(&mut World),
            task_commit,
            0..=reference.store.operation_count(task_commit),
        ),
        (
            "caller person",
            (|_: &mut World| {}) as fn(&mut World),
            person_commit,
            0..=reference.store.operation_count(person_commit),
        ),
        (
            "task requested by caller",
            (|_: &mut World| {}) as fn(&mut World),
            caller_commit,
            0..=reference.store.operation_count(caller_commit),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let row_seed = seed + u64::try_from(index).expect("three rows");
        let count = *cuts.end();
        assert!(count > 0, "{name} writes the disk");
        eprintln!("{name}: commit {commit}, operations {count}");
        let cuts: Vec<_> = cuts.map(|operation| disk_cut(commit, operation)).collect();
        let mut world = disk_world_sweeping(row_seed, None, cuts.clone());
        scenario(&mut world);
        if let Some(cut) = world.forked_cut() {
            let operation = cut_operation(cut);
            // At the complete boundary no later file operation need be asked.
            assert!(world.reached_cut == Some(cut) || operation == count, "{name}, {cut:?}");
            assert_eq!(world.store.operation_count(commit), operation, "{name}: exact operation cut");
            disk_restart(&mut world, row_seed);
            disk_referee(&mut world, name);
        }
        world.cut_passed();
        assert_eq!(world.forked, cuts, "{name}: every operation passed");
    }
    let first = disk_cut(task_commit, reference.store.operation_count(task_commit) / 2);
    let (restart_count, mut cuts) = disk_restart_reference(seed, first);
    cuts.extend([0, restart_count / 2, restart_count - 1]);
    cuts.sort_unstable();
    cuts.dedup();
    eprintln!("restart after mid-frame kill: {restart_count} operations, chosen {cuts:?}");
    double_disk_sweep(seed, first, cuts);
    let mut restarted = disk_world(seed, Some(first));
    person_decision(&mut restarted);
    disk_restart(&mut restarted, seed);
    let next = restarted.store.applied + 1;
    restarted.probe();
    restarted.drain();
    let count = restarted.store.operation_count(next);
    let mut world = disk_world(seed, Some(first));
    person_decision(&mut world);
    disk_restart(&mut world, seed);
    let cuts: Vec<_> = (0..=count).map(|operation| disk_cut(world.store.applied + 1, operation)).collect();
    world.sweep(cuts.clone());
    world.probe();
    world.drain();
    if let Some(cut) = world.forked_cut() {
        let operation = cut_operation(cut);
        assert!(world.reached_cut.is_some() || operation == count, "next decision cut {operation}");
        disk_restart(&mut world, seed);
        disk_referee(&mut world, "twice, then working");
    }
    world.cut_passed();
    assert_eq!(world.forked, cuts, "every following decision boundary passed");
}

fn double_disk_sweep(seed: u64, first: Cut, operations: Vec<u32>) {
    let mut world = disk_world(seed, Some(first));
    person_decision(&mut world);
    assert_eq!(world.reached_cut, Some(first));
    let cuts: Vec<_> = operations
        .into_iter()
        .map(|operation| Cut::Disk { from: jig_test_store_files::KillFrom::Open, operation })
        .collect();
    world.sweep(cuts.clone());
    disk_restart(&mut world, seed);
    if let Some(cut) = world.forked_cut() {
        assert_eq!(world.reached_cut, Some(cut), "restart operation reached");
        assert_eq!(world.store.open_operation_count(), cut_operation(cut));
        disk_restart(&mut world, seed);
    }
    disk_referee(&mut world, "killed twice");
    world.cut_passed();
    assert_eq!(world.forked, cuts, "every second kill passed");
}

pub(crate) fn restart_operation_sweep() {
    let seed = 18_302;
    let mut reference = disk_world(seed, None);
    person_decision(&mut reference);
    let commit = commit_saving(&reference, person_task);
    let first = disk_cut(commit, reference.store.operation_count(commit) / 2);
    let (count, _) = disk_restart_reference(seed, first);
    double_disk_sweep(seed, first, (0..count).collect());
}
