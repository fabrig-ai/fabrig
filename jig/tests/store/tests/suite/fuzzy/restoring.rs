//! Seeded restore cuts after commits, drops, rollovers and snapshots on both sides of C.
use super::super::restoring::{oracle, pair, scene};
use jig_protocol_store::{Event, Write};
use skein_lib::Rng;
use skein_world::{
    sweep::{Case, cases},
    this_test,
};
use std::convert::Infallible;

#[test]
fn seeded_restore_cuts_keep_each_maps_and_transcripts_at_the_same_prefix() -> Result<(), Case<Infallible>> {
    cases(this_test!("seeded_restore_cuts_keep_each_maps_and_transcripts_at_the_same_prefix"), 0..12, |seed| {
        let mut random = Rng::new(seed);
        let (mut reference, name, _) = scene(seed);
        assert!(matches!(reference.restore_checkpoint(name, [2; 16]).as_slice(), [Event::Restored { .. }]));
        let end = reference.operation_count();
        let (mut world, name, mut later) = scene(seed);
        if random.below(2) == 0 {
            world.reopen();
            world.mark_if_needed();
            let number = world.model.acknowledged + 1;
            world
                .commit(Box::new([
                    Write::Save(jig_store_world::root::header(number)),
                    Write::DropTranscript { task: jig_store_world::root::LIVE + 3 },
                ]))
                .expect("seeded drop");
            later = world.model.acknowledged;
            world.crash(seed);
        }
        let cut = u32::try_from(random.below(u64::from(end))).expect("bounded cut");
        let expected = oracle(&mut world, name);
        world.cut_at(cut);
        world.restore_checkpoint(name, [2; 16]);
        world.crash(seed.wrapping_add(100));
        pair(&mut world, name.store, later, &expected);
        Ok(())
    })
}
