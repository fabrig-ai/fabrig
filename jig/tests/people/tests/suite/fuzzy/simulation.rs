use jig_people_world::fixtures::{OWNER, PERSON, PROJECT};
use jig_people_world::{ENDINGS, Settings, World};
use skein_lib::length::len_u64;
use std::collections::BTreeSet;

#[test]
fn random_people_settle_and_reach_every_ending() {
    let mut endings = BTreeSet::new();
    for seed in 0..256 {
        let mut world = World::new(Settings::random(seed));
        world.run();
        endings.extend(world.stats().endings.iter().copied());
    }
    for ending in ENDINGS {
        assert!(endings.contains(ending), "{ending}: {endings:?}");
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one seeded census compares selected volatile rebuilding with ordinary projections"
)]
fn selected_projections_and_stored_diagnostics_replay_by_seed() {
    use jig_domain_people::{self as p, RestoreProjection as P};
    use jig_people_world::{LIMITS, identity};
    use skein_lib::{Env, Queue, Rng, Time, Wall};
    for seed in 0..256 {
        let mut rng = Rng::new(seed);
        let limits = p::Limits { inbox_entries: u32::try_from(rng.below(5)).expect("prefix below five"), ..LIMITS };
        let env = Env { limits, now: Time::ZERO, wall: Wall::EPOCH };
        let mut selected = p::Domain::new(&limits, Box::new([]), 1, Box::new([]));
        let mut ordinary = p::Domain::new(&limits, Box::new([]), 1, Box::new([]));
        let mut out = Queue::uncapped();
        selected.start_restore(env.wall);
        ordinary.start_restore(env.wall);
        let through = rng.between(1, 3);
        for row in [
            p::Stored::Person { number: PERSON, identity: identity(OWNER) },
            p::Stored::Roles {
                project: PROJECT,
                holdings: Box::new([p::Holding { person: PERSON, role: p::Role::Member }]),
            },
            p::Stored::ReadResults { party: PERSON, through },
            p::Stored::ReadConversation { party: PERSON, task: 30, through },
            p::Stored::ReadConversation { party: PERSON, task: 31, through },
        ] {
            p::step(&mut selected, &env, p::Event::Restore { record: row.clone() }, &mut out);
            p::step(&mut ordinary, &env, p::Event::Restore { record: row }, &mut out);
        }
        assert!(ordinary.start_link());
        ordinary.start_sweep();
        assert!(ordinary.start_complete(&env));
        assert!(out.is_empty(), "seed {seed}");
        // Live projections' arrivals are separate from the census's output.
        let mut ordinary_out = Queue::uncapped();
        let mut pending = Vec::new();
        for task in 1..=8 {
            let kind = match rng.below(4) {
                0 => p::EntryKind::PersonTask,
                1 => p::EntryKind::Question { message: task },
                2 => p::EntryKind::Proposal { number: task },
                3 => p::EntryKind::Escalation { revision: task },
                _ => unreachable!(),
            };
            pending.push(p::Entry {
                task,
                project: PROJECT,
                whom: p::Whom::Person(PERSON),
                kind,
                at: Wall::from_nanos(task),
            });
        }
        while !pending.is_empty() {
            let index = usize::try_from(rng.below(len_u64(pending.len()))).expect("index into eight items");
            let entry = pending.swap_remove(index);
            assert!(selected.start_project(&limits, P::Waiting { task: entry.task, entries: Box::new([entry]) }));
            p::step(
                &mut ordinary,
                &env,
                p::Event::Waiting { task: entry.task, entries: Box::new([entry]) },
                &mut ordinary_out,
            );
        }
        let mut positions: Vec<u64> = (1..=8).collect();
        while !positions.is_empty() {
            let index = usize::try_from(rng.below(len_u64(positions.len()))).expect("index into eight items");
            let position = positions.swap_remove(index);
            let entry = p::ResultRef {
                task: position + 10,
                project: PROJECT,
                position,
                at: Wall::from_nanos(position + 10),
                forgettable: None,
            };
            assert!(selected.start_project(&limits, P::Result { party: PERSON, entry }));
            ordinary.remember_result(&limits, PERSON, entry);
        }
        for (task, live, seen) in [(30, true, None), (31, false, Some(through))] {
            let replies: Box<[_]> = (1..=4)
                .map(|message| p::ReplyRef {
                    activity: task + message,
                    entry: p::Entry {
                        task,
                        project: PROJECT,
                        whom: p::Whom::Person(PERSON),
                        kind: p::EntryKind::ChatReply { message },
                        at: Wall::from_nanos(task + message),
                    },
                })
                .collect();
            assert!(selected.start_project(
                &limits,
                P::ChatState {
                    party: PERSON,
                    project: PROJECT,
                    task,
                    last_reply: 4,
                    live,
                    seen,
                    replies: replies.clone()
                }
            ));
            p::step(
                &mut ordinary,
                &env,
                p::Event::ChatState { party: PERSON, project: PROJECT, task, last_reply: 4, live, seen, replies },
                &mut ordinary_out,
            );
        }
        assert!(selected.start_project(&limits, P::EndedConversation { task: 31 }));
        assert!(!selected.ready());
        assert_eq!(selected.next_deadline(), None);
        assert!(out.is_empty(), "restore with an explicit ended mark writes nothing, seed {seed}");
        assert!(selected.start_link());
        assert_eq!(selected.counts(&limits, PERSON), ordinary.counts(&limits, PERSON), "seed {seed}");
        assert_eq!(selected.cached_updates(&limits, PERSON), ordinary.cached_updates(&limits, PERSON), "seed {seed}");
        selected.start_sweep();
        assert!(selected.start_complete(&env));
        assert_eq!(selected.cached_results(PERSON), ordinary.cached_results(PERSON), "seed {seed}");
        assert_eq!(selected.cached_waiting(&limits, PERSON), ordinary.cached_waiting(&limits, PERSON), "seed {seed}");
        assert_eq!(selected.restore_failure(), None);

        let mut diagnostic = p::Domain::new(&limits, Box::new([]), 1, Box::new([]));
        diagnostic.start_restore(env.wall);
        let key = p::RequestKey { person: PERSON, key: [u8::try_from(seed).expect("256 seeds"); 16] };
        let extra = rng.below(2);
        let held = u64::from(limits.words) + extra;
        p::step(
            &mut diagnostic,
            &env,
            p::Event::Restore {
                record: p::Stored::Answer {
                    key,
                    ask: Box::new(p::Ask::Say {
                        project: PROJECT,
                        task: 1,
                        words: vec![0; usize::try_from(held).expect("bounded word fixture")].into(),
                    }),
                    models: Box::new([]),
                    outcome: p::Outcome::SignedOut,
                    at: Wall::EPOCH,
                },
            },
            &mut out,
        );
        let expected = (extra != 0).then_some(p::RestoreFailure {
            key: p::Key::Answer(key),
            cause: p::RestoreCause::Bound { which: p::RestoreBound::Words, held, limit: u64::from(limits.words) },
        });
        assert_eq!(diagnostic.restore_failure(), expected, "seed {seed}");
        if extra != 0 {
            assert!(matches!(out.pop(), Some(p::Request::RestoreRefused { .. })));
            assert!(!diagnostic.start_project(&limits, P::EndedConversation { task: 31 }));
            assert!(!diagnostic.start_link());
            assert_eq!(diagnostic.restore_failure(), expected, "seed {seed}");
        }
        assert!(out.is_empty());
    }
}
