//! The store records' drift (skein's data/codec/golden.md, 1).
#[path = "../../../codec-families.rs"]
mod families;

#[test]
fn codec_families_match_their_schemas() {
    families::check();
}
