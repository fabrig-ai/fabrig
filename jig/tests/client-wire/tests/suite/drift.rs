//! The committed client codecs and bytes against their schemas (skein's codec.md, 6).
#[path = "../../../codec-families.rs"]
mod families;

#[test]
fn codec_families_match_their_schemas_and_goldens() {
    families::check();
}
