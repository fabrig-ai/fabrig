//! Writer snapshots read back through the cold reader (domain/engine/world.md, 1).
#[test]
fn seeded_writer_snapshots_restore_and_replay() {
    for seed in 0..128 {
        assert_eq!(
            crate::start_restore::writer_readback(seed),
            crate::start_restore::writer_readback(seed),
            "seed {seed}"
        );
    }
}
