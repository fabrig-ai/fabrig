//! Writer snapshots read back through the cold reader (domain/engine/world.md, 1).
use skein_world::{
    sweep::{Case, cases},
    this_test,
};
use std::convert::Infallible;

#[test]
fn seeded_writer_snapshots_restore_and_replay() -> Result<(), Case<Infallible>> {
    cases(this_test!("seeded_writer_snapshots_restore_and_replay"), 0..128, |seed| {
        assert_eq!(
            crate::start_restore::writer_readback(seed),
            crate::start_restore::writer_readback(seed),
            "seed {seed}"
        );
        Ok(())
    })
}
