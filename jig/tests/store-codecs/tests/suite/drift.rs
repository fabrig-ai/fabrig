//! The store records' drift (skein's data/codec/README.md, 6).
#[path = "../../../codec-families.rs"]
mod families;

#[test]
fn codec_families_match_their_schemas() {
    families::check();
}
