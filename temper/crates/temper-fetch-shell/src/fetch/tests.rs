use super::{Failed, claude_present};

const FILE: &[u8] = b"/cache/claude";
const PINNED_BYTES: u64 = 8; // A synthetic build size, independent of the host's pinned build.

#[test]
fn a_regular_cached_build_at_the_pin_is_present() {
    assert!(claude_present(FILE, PINNED_BYTES, Ok((true, PINNED_BYTES))).expect("matching build"));
}

#[test]
fn a_cached_build_one_byte_short_is_refused() {
    let refusal = claude_present(FILE, PINNED_BYTES, Ok((true, PINNED_BYTES - 1))).expect_err("wrong size");
    assert!(matches!(refusal, Failed::Refused(temper_fetch::fetch::Failure::Present { .. })));
    assert_eq!(refusal.words(), "/cache/claude: does not match the pinned digest");
}
