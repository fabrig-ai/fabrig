use jig_store_world::{
    World,
    referee::Referee,
    root::{self, LIVE},
};
use skein_lib::Rng;

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
