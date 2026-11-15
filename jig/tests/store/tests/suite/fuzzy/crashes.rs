use jig_store_world::{
    World,
    referee::Referee,
    root::{self, LIVE},
};
use skein_lib::Rng;

fn history_world() -> World {
    // This crash sweep holds log atomicity, not registry planning. Keep the
    // binary's job and key ranges, without registering unrelated layouts.
    const SETS: jig_protocol_store::PatchSets = jig_protocol_store::PatchSets {
        sets: &[jig_protocol_store::PatchSet {
            patches: &[],
            jobs: &[*jig_protocol_store::JIG_STORE_PATCH_SET.jobs.last().expect("task-logs is last")],
            families: &[
                jig_protocol_store::KeptFamily { types: &[], texts: &[], ..jig_protocol_store::STORE_FAMILIES[3] },
                jig_protocol_store::KeptFamily { types: &[], texts: &[], ..jig_protocol_store::STORE_FAMILIES[4] },
            ],
            ..jig_protocol_store::JIG_STORE_PATCH_SET
        }],
    };
    let mut limits = root::limits();
    limits.store.record = limits.store.page.bytes;
    let mut world = World::unopened_observed(0, limits, None);
    world.patch_sets(SETS);
    assert_eq!(world.open(), Some(0));
    assert_eq!(world.commit(root::first()), Some(1));
    world
}
#[test]
fn history_commits_crashed_at_every_file_operation_recover_whole() {
    for number in [2, 3] {
        let mut reference = history_world();
        if number == 3 {
            assert_eq!(reference.commit(root::history_commit(2)), Some(2));
        }
        assert_eq!(reference.commit(root::history_commit(number)), Some(number));
        let operations = reference.operation_count();
        for cut in 0..=operations {
            let mut world = history_world();
            if number == 3 {
                assert_eq!(world.commit(root::history_commit(2)), Some(2));
            }
            world.cut_at(cut);
            let acknowledged = world.commit(root::history_commit(number));
            assert!(world.reached_cut(), "cut {cut} of {operations}");
            world.crash(u64::from(cut));
            let restored = world.reopen();
            Referee::check(&world.model, &restored)
                .unwrap_or_else(|error| panic!("history commit {number}, cut {cut}: {error:?}"));
            if acknowledged.is_some() {
                assert_eq!(restored.last, number);
            }
            let inventory = world.retained();
            assert!(inventory.keys().all(|key| jig_protocol_store::keys::logged(key).is_none()));
        }
    }
}

#[test]
fn random_commits_and_crashes_keep_the_referees_promises() {
    for (linked, seed) in [false, true].into_iter().flat_map(|linked| (0..64).map(move |seed| (linked, seed))) {
        let mut random = Rng::new(seed);
        let mut world = World::new(seed);
        let mut first = root::first().into_vec();
        if linked {
            first.push(jig_protocol_store::Write::Save(root::person(1)));
        }
        assert_eq!(world.commit(first.into_boxed_slice()), Some(1), "initial prefix, seed {seed}");
        let mut next = 2;
        let mut turn = 1;
        for round in 0..8 {
            let has_turn = !linked && random.below(2) == 0;
            let writes = if linked {
                let person_missing = !world.model.prefixes[&world.model.acknowledged]
                    .rows
                    .contains_key(&root::person(root::LINKED_PERSON).key());
                root::linked_commit(next, person_missing || round % 2 == 0)
            } else if has_turn {
                root::turn_commit(next, LIVE, turn, b"seeded turn")
            } else {
                root::plain_commit(next)
            };
            let cut = u32::try_from(random.below(13)).expect("twelve file operation cuts");
            world.cut_at(cut);
            let acknowledged = world.commit(writes);
            world.crash(random.next_u64());
            if linked {
                let second = u32::try_from(random.below(8)).expect("early restart operation");
                world.cut_at(second);
                assert_eq!(world.open(), None, "second kill, seed {seed}, operation {second}");
                assert!(world.reached_cut(), "second cut reached, seed {seed}");
                world.crash(random.next_u64());
            }
            let restored = world.reopen();
            Referee::check(&world.model, &restored)
                .unwrap_or_else(|error| panic!("whole random prefix, seed {seed}, cut {cut}: {error:?}, {restored:?}"));
            if linked {
                let jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Live(task)) =
                    &restored.rows[&root::live_task(LIVE).key()]
                else {
                    panic!("linked live task")
                };
                let jig_domain_tasks::Party::Person(person) = task.requester else { panic!("person requester") };
                assert!(restored.rows.contains_key(&root::person(person).key()), "linked prefix, seed {seed}");
            }
            if acknowledged.is_some() {
                assert_eq!(restored.last, next, "acknowledged prefix, seed {seed}, cut {cut}");
            }
            if has_turn && restored.last == next {
                turn += 1;
            }
            next = restored.last + 1;
            world.accept_recovery(restored.last);
        }
    }
}
