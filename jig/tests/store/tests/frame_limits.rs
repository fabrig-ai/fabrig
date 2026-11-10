//! The world profiles admit fixture frames (protocol/store/transcripts.md, 6).
#[test]
fn world_read_room_admits_fixture_frames() {
    const FIXTURE_FRAME: u64 = 130;
    const MAX_FRAME: u64 = 33_554_496;
    for limits in [jig_store_world::root::limits(), jig_store_world::patches::limits()] {
        assert_eq!(limits.core.resume_bytes, 256);
        let room = u64::from(limits.core.resume_bytes).min(u64::from(limits.store.page.bytes));
        assert!(room >= FIXTURE_FRAME);
        assert!(room < MAX_FRAME);
    }
}
