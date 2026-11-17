//! Replaying cuts while commits, drops, snapshots and rollovers race checkpoints.
use super::super::checkpoints::recovered;
use jig_protocol_store::{CheckpointName, Event, Request, SecretOwner, SecretValue, Secrets, Write};
use jig_store_world::{World, referee::Referee, root};
use skein_lib::{Rng, Token};
#[test]
fn seeded_checkpoint_races_and_crashes_recover_each_selected_cut_and_the_live_store() {
    let mut found = 0;
    for seed in 0..8 {
        let mut random = Rng::new(seed);
        let mut world = World::history_world();
        let mut limits = root::limits();
        limits.store.snapshot_after = 1;
        limits.store.segment = 1;
        world.restart_with_limits(seed, limits);
        world.reopen();
        world.mark_if_needed();
        let number = world.model.acknowledged + 1;
        assert_eq!(
            world.commit(Box::new([Write::Save(root::header(number)), Write::Save(root::person(1))])),
            Some(number)
        );
        world.secret(Secrets::Keep {
            owner: SecretOwner::Service,
            number: 1,
            value: SecretValue::Digest { expiry: u64::MAX, digest: [1; 32] },
        });
        for round in 0..4_u8 {
            let first = world.model.acknowledged + 1;
            enqueue(&mut world, first, random.below(3));
            let mut draw = [0; 16];
            draw[7] = u8::try_from(seed).expect("eight seeds");
            draw[15] = round;
            world.enqueue(Request::Checkpoint { owner: Token::new(987), draw });
            enqueue(&mut world, first + 1, random.below(3));
            let start = world.operation_count();
            let cut =
                if round == 0 { u32::MAX } else { start + u32::try_from(random.below(100)).expect("bounded cut") };
            world.cut_at(cut);
            let events = world.settle();
            let mut answered = Vec::new();
            for event in &events {
                match event {
                    Event::Committed { number, .. } => world.model.acknowledge(*number),
                    Event::Checkpointed { checkpoint, .. } => answered.push(checkpoint.name),
                    Event::Census { .. } => {}
                    other => panic!("checkpoint race, seed {seed}, round {round}: {other:?}"),
                }
            }
            world.crash(seed * 10 + u64::from(round));
            let names: Vec<_> = world
                .paths()
                .iter()
                .filter_map(|path| path.strip_prefix(b"checkpoints/").and_then(CheckpointName::parse))
                .collect();
            for name in answered {
                assert!(names.contains(&name), "answered checkpoint survives, seed {seed}, round {round}");
            }
            for name in names {
                recovered(&mut world, name);
                found += 1;
            }
            let restored = world.checkpoint_reopen();
            Referee::check(&world.model, &restored)
                .unwrap_or_else(|why| panic!("live recovery, seed {seed}, round {round}: {why:?}"));
            world.accept_recovery(restored.last);
            world.mark_if_needed();
        }
    }
    assert!(found >= 8, "each seed leaves at least one checkpoint");
}
fn enqueue(world: &mut World, number: u64, kind: u64) {
    let writes = match kind {
        0 => root::turn_commit(number, root::LIVE, u32::try_from(number).expect("small turn"), b"racing turn"),
        1 => Box::new([Write::Save(root::header(number)), Write::DropTranscript { task: root::LIVE + 1 }]),
        2 => root::history_commit(number),
        _ => unreachable!("three choices"),
    };
    world.model.stage(number, &writes);
    world.enqueue(Request::Commit { number, writes });
}
