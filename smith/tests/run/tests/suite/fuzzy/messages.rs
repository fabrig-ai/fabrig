//! Seeded messages across real run callbacks. Contract: smith/docs/design/domain/run.md, section 12.
use std::{collections::BTreeMap, convert::Infallible};

use skein_world::{
    domain::Verdict,
    sweep::{Case, cases, gate},
};
use smith_run_world::{World, messages_referee::judge, noisy};

#[test]
fn seeded_message_callbacks_replay_with_read_and_unread_terminals() -> Result<(), Case<Infallible>> {
    let collect_reach = gate();
    let mut cells = BTreeMap::<&str, u32>::new();
    cases(skein_world::this_test!("seeded_message_callbacks_replay_with_read_and_unread_terminals"), 0..96, |seed| {
        let settings = noisy(seed);

        let mut world = World::new(settings);
        world.inject_messages(500);
        world.run(1_000_000);
        assert!(judge(world.message_seen()).iter().all(|verdict| *verdict == Verdict::Passed), "seed {seed}");
        if collect_reach {
            for (cell, count) in world.message_cells() {
                *cells.entry(cell).or_default() += count;
            }
        }
        let mut replay = World::new(settings);
        replay.inject_messages(500);
        replay.run(1_000_000);
        assert_eq!(world.trace(), replay.trace(), "seeded message callbacks replay exactly");
        Ok(())
    })?;
    if collect_reach {
        for cell in ["preparing", "working", "landing", "winding"] {
            assert!(cells.get(cell).copied().unwrap_or(0) > 0, "missing {cell}: {cells:?}");
        }
    }
    Ok(())
}
