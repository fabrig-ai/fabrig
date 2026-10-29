use skein_forge_connection::limits::Forges;
use skein_forge_machine_world::memory;

#[test]
fn full_forgejo_holders_stay_under_the_ceiling() {
    // Component peak while filling every Forgejo fixture holder.
    const CEILING: u64 = 320_000;
    check(Forges { forgejo: 2, github: 0 }, [2, 2, 2, 2, 8, 4, 0, 0, 2, 8, 8, 0, 0, 65408, 4], CEILING);
}
#[test]
fn full_github_holders_stay_under_the_ceiling() {
    // Component peak while filling every GitHub fixture holder.
    const CEILING: u64 = 400_000;
    check(Forges { forgejo: 0, github: 2 }, [2, 2, 4, 2, 8, 4, 6, 12, 2, 8, 1, 8, 1, 65408, 0], CEILING);
}
#[test]
fn full_mixed_holders_stay_under_the_ceiling() {
    // Component peak while filling every mixed fixture holder.
    const CEILING: u64 = 370_000;
    check(Forges { forgejo: 1, github: 1 }, [2, 2, 3, 4, 16, 4, 3, 6, 2, 8, 1, 8, 1, 65408, 4], CEILING);
}
fn check(forges: Forges, saturation: [u32; 15], ceiling: u64) {
    let full = memory::full_heap(forges, ceiling);
    assert_eq!(full.saturation(), saturation);
    assert_eq!((full.op_bytes, full.body_bytes, full.rows, full.inner), (512, 1042, 8, 8));
    assert_eq!(full.answer_bytes, 65_536);
    if forges.github > 0 {
        assert_eq!(full.key_bytes, 1192);
        assert!(full.jwt_bytes > 0);
    }
    assert!(full.owned > 0 && full.owned <= ceiling);
    println!(
        "{forges:?}: owned={}, ceiling={ceiling}, margin={}, saturation={:?}, key_bytes={}, jwt_bytes={}",
        full.owned,
        ceiling - full.owned,
        full.saturation(),
        full.key_bytes,
        full.jwt_bytes
    );
}
