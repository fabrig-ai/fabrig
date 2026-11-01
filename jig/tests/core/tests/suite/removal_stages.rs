//! Whole project deletion and every durable restart boundary (draft 5, 8).
use jig_core_world::{
    effects::{Cut, World},
    fixtures::{ask_as, chat, delegate, member},
    removal::rows_naming,
};
use jig_domain_core as core;
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{List, Token};

const SEED: u64 = 1880;
const PROJECT: u32 = 1;
const OWNER: u64 = 1;
const REMOVE: u8 = 91;
const NOTE: u64 = 1;
const POOL: tasks::Funder = tasks::Funder::Pool { project: PROJECT, person: OWNER, period: 1 };
const PERIOD: tasks::Funder = tasks::Funder::Period { project: PROJECT, period: 1 };

fn ask(world: &mut World, key: u8, value: people::Ask) {
    ask_as(world, world.sign_in.expect("owner signed in"), key, value);
}
fn finish(world: &mut World) {
    world.end(
        tasks::End::Finished {
            result: tasks::TaskResult::Report { words: b"done".as_slice().into() },
            cancel_delegates: false,
        },
        0,
    );
}
fn fixture() -> (World, Vec<u64>) {
    let mut world = World::with_patch_sets(SEED, false, crate::fresh_windows::BEFORE);
    world.turn(1, 0, None, &crate::fresh_windows::turn(b"ended chat"));
    finish(&mut world);
    let new_chat = chat(&world, PROJECT, b"live chat".as_slice().into());
    ask(&mut world, REMOVE - 3, new_chat);
    let run = *world.assignments.last().expect("live chat assigned");
    world.turn(1, 0, None, &crate::fresh_windows::turn(b"live chat turn"));
    let mut child = member(&world, tasks::Executor::Agent { charter: 1 }, b"delegate");
    child.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let _child = delegate(&mut world, run, 1, child);
    ask(&mut world, REMOVE - 2, people::Ask::SetPool { project: PROJECT, person: OWNER, budget: 500 });
    let mut references = List::with_capacity(1);
    references.push(run.0).expect("one note reference");
    let scope = notes::Scope::Project { project: PROJECT };
    world.store.seed(Box::new([
        root::Record::Core(core::Record::Notes(notes::Record::Line(notes::Line {
            scope: scope.clone(),
            name: NOTE,
            description: b"project note".as_slice().into(),
            revision: 1,
        }))),
        root::Record::Core(core::Record::Notes(notes::Record::Entry(notes::Entry {
            name: NOTE,
            scope,
            description: b"project note".as_slice().into(),
            body: b"remember this".as_slice().into(),
            references,
            author: notes::Author::Task { task: run.0, attempt: run.1 },
            revision: 1,
            changed: world.wall_time(),
        }))),
    ]));
    restart(&mut world);
    let mut ids: Vec<u64> = world
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row)))
                if row.project == PROJECT =>
            {
                Some(row.number)
            }
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .collect();
    ids.sort_unstable();
    (world, ids)
}
fn remove(world: &mut World) {
    ask(world, REMOVE, people::Ask::RemoveProject { project: PROJECT });
}
fn complete(world: &mut World) {
    // The scripted host answers the cancelled root; its unassigned delegate
    // has no run to wait for and ends through the connector lifecycle.
    world.end(tasks::End::Parked, 0);
}
fn restart(world: &mut World) {
    let hosting = world
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            root::Record::Core(core::Record::Core(core::CoreRecord::RunProof(row))) => Some(jig_domain_fleet::Hosted {
                run: Token::new(row.task),
                attempt: Token::new(row.attempt),
                phase: jig_domain_fleet::Phase::Active,
            }),
            root::Record::Core(_) | root::Record::Connector { .. } => None,
        })
        .collect();
    world.restart(SEED, false);
    crate::fresh_windows::hello_hosting(world, hosting);
    world.grant_account(1);
}
fn removal_key() -> root::Key {
    root::Key::Core(core::Key::Core(core::CoreKey::Removal(PROJECT)))
}
fn whole(world: &mut World, ids: &[u64]) {
    assert_eq!(rows_naming(world, PROJECT, ids), Vec::<root::Key>::new());
    assert!(!world.store.rows.contains_key(&removal_key()));
    for task in ids {
        let name = format!("transcripts/{:016x}/{task:016x}", task >> 12);
        assert_eq!(world.store.file_bytes(name.as_bytes()), None, "transcript {task}");
    }
    for funder in [POOL, PERIOD] {
        assert!(matches!(
            world.store.rows.get(&root::Key::Core(core::Key::Tasks(tasks::Key::ClosedLedger(funder)))),
            Some(root::Record::Core(core::Record::Tasks(tasks::Stored::ClosedLedger(_))))
        ));
    }
}

#[test]
fn a_project_with_ended_and_live_chats_is_deleted_whole() {
    let (mut world, ids) = fixture();
    assert_eq!(ids.len(), 3); // Ended chat, live chat and delegate.
    assert!(world.store.rows.contains_key(&root::Key::Core(core::Key::Notes(notes::Key::Entry { name: NOTE }))));
    for task in &ids[..2] {
        let name = format!("transcripts/{:016x}/{task:016x}", task >> 12);
        assert!(world.store.file_bytes(name.as_bytes()).is_some());
    }
    remove(&mut world);
    complete(&mut world);
    whole(&mut world, &ids);
}

fn stage_commit(world: &World, expected: core::RemovalStage) -> u64 {
    let writes =
        world.commits.iter().find(|writes| writes.iter().any(|write| stage(write, expected))).expect("stage committed");
    writes
        .iter()
        .find_map(|write| match write {
            root::Write::Save(root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(row)))) => {
                Some(row.commits)
            }
            _ => None,
        })
        .expect("commit's deployment header")
}

fn stage(write: &root::Write, expected: core::RemovalStage) -> bool {
    matches!(write, root::Write::Save(root::Record::Core(core::Record::Core(core::CoreRecord::Removal(row)))) if row.project == PROJECT && row.stage == expected)
}

#[test]
fn a_deletion_cut_by_a_crash_after_each_stage_ends_whole() {
    let (mut reference, ids) = fixture();
    let before = reference.store.applied;
    remove(&mut reference);
    complete(&mut reference);
    let ending = stage_commit(&reference, core::RemovalStage::Ending);
    let erasing = stage_commit(&reference, core::RemovalStage::Erasing);
    let releasing = stage_commit(&reference, core::RemovalStage::Releasing);
    assert!(before < ending && ending < erasing && erasing < releasing);
    // Every durable decision of deletion is a cut, including partial task pages
    // and Releasing before its outputs can leave their commit barrier.
    for number in ending..=releasing {
        let (mut world, same_ids) = fixture();
        assert_eq!(same_ids, ids);
        let cut = Cut::Durable(number);
        world.cut = Some(cut);
        remove(&mut world);
        if world.reached_cut.is_none() {
            complete(&mut world);
        }
        assert_eq!(world.reached_cut, Some(cut));
        restart(&mut world);
        // The adopted host still owes the cancellation's terminal answer.
        complete(&mut world);
        assert!(
            rows_naming(&world, PROJECT, &ids).is_empty(),
            "cut {number}: {:?}",
            rows_naming(&world, PROJECT, &ids)
        );
        whole(&mut world, &ids);
    }
}

#[test]
fn a_retried_remove_after_the_deletion_answers_unknown() {
    let (mut world, ids) = fixture();
    remove(&mut world);
    complete(&mut world);
    whole(&mut world, &ids);
    ask(&mut world, REMOVE, people::Ask::RemoveProject { project: PROJECT });
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Unknown)))
    );
}

#[test]
fn the_deleted_projects_name_makes_a_new_project() {
    let (mut config, limits) = jig_core_world::effects::fixture(SEED, false);
    let name = config.core.seed_projects[0].name.clone();
    config.core.defaults.policies = skein_lib::Map::with_capacity(1);
    config.core.defaults.policies.insert(1, config.core.seed_projects[0].policy.clone()).expect("one offered policy");
    let mut world = World::configured(SEED, false, config, limits);
    remove(&mut world);
    complete(&mut world);
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: people::DEPLOYMENT,
        holdings: Box::new([people::Holding { person: OWNER, role: people::Role::Owner }]),
    })));
    ask(&mut world, REMOVE + 1, people::Ask::MakeProject { name, policy: 1 });
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::ProjectMade { project: PROJECT + 1 }))
    );
}
