use super::blocks::{BLOCK, LOWEST};
use super::*;

#[test]
fn the_range_is_read_from_the_kernels_file() {
    assert_eq!(first_of("32768\t60999\n"), Some(32768));
    assert_eq!(first_of("20000\t60999\n"), Some(20000));
    assert_eq!(first_of("garbage"), None);
}

#[test]
fn a_range_too_narrow_says_so() {
    let refusal = Blocks::new(LOWEST, 0).err().expect("no room for two blocks");
    assert_eq!(refusal, format!("{RANGE_FILE}: first port {LOWEST} leaves fewer than two blocks above {LOWEST}"));
    assert!(Blocks::new(LOWEST + 2 * BLOCK, 0).is_ok(), "exactly two blocks fit");
}
