//! Writer snapshots read back through the opted-in cold reader (engine.md, 15).
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
