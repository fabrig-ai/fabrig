//! The store records' drift and pinned constructs (skein's data/codec/README.md, 6).
#[path = "../../../codec-families.rs"]
mod families;

#[test]
fn codec_families_match_their_schemas_and_are_pinned() {
    families::check();
}
