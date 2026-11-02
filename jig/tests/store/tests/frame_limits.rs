//! The world profiles admit fixture frames (protocol/store.md, 8.6).
#[test]
fn world_read_room_admits_fixture_frames() {
    const FIXTURE_FRAME: u64 = 130;
    const MAX_FRAME: u64 = 8_388_672;
    for (limits, page) in [(jig_store_world::root::limits(), 330_692), (jig_store_world::patches::limits(), 2_048)] {
        assert_eq!(limits.core.resume_bytes, 256);
        assert_eq!(limits.store.page.bytes, page);
        let room = u64::from(limits.core.resume_bytes).min(u64::from(page));
        assert!(room >= FIXTURE_FRAME);
        assert!(room < MAX_FRAME);
    }
}
