//! Lean surface and helper fixture drift (skein/docs/design/data/codec/README.md, 4).

use crate::{generate, parse};

const FIXTURE: &str = "family helper-fixture 7
record Empty {}
record Words { words: text 3 }
enum Choice { unit, words: Words, boxed_words: boxed Words }
versioned record Row { choice: Choice }
record Nested {
    words: list 2 option text 3
    maybe: option list 2 u16
    records: list 2 Words
}
";

#[test]
#[expect(clippy::disallowed_methods, reason = "reads the fixture rewrite switch SKEIN_CODEGEN_WRITE")]
fn lean_helper_fixture_has_no_drift() {
    let schema = parse(FIXTURE).expect("helper schema");
    let output = generate(&schema);
    let path = skein_scratch::paths::tree().join("crates/skein-codec/src/tests/lean_fixture.rs");
    if std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1") {
        std::fs::write(&path, &output.records).expect("write helper fixture");
    }
    let committed = std::fs::read_to_string(path).expect("helper fixture committed");
    assert!(committed == output.records, "helper fixture drifted: run this test with SKEIN_CODEGEN_WRITE=1");
}

#[test]
fn lean_secret_fields_need_no_box_import() {
    let schema = parse("family hidden 1 record Row { value: option list 2 secret 8 }").expect("secret schema");
    let output = generate(&schema).records;
    assert!(output.contains("pub value: Option<List<skein_lib::Secret>>"));
    assert!(!output.contains("use alloc::boxed::Box;"), "secret fields use Secret, not Box");
}

#[test]
fn lean_roots_have_a_concrete_debug_decode_entry() {
    let schema = parse("family roots 1\nrecord Item { byte: u8 }\nrecord Root { item: Item }\n").expect("root schema");
    let output = generate(&schema).records;
    let (nested, root) = output.split_once("// Record: Root.").expect("root implementation");
    assert!(!nested.contains("fn decode("), "nested reads use the family's read entry");
    assert!(root.contains("#[cfg(debug_assertions)]\n    fn decode("), "only debug builds override the default entry");
}

#[test]
fn lean_surface_is_public() {
    let schema = parse(FIXTURE).expect("helper schema");
    let output = generate(&schema).records;
    for required in [
        "pub words: Box<[u8]>",
        "impl Record for Words",
        "pub use skein_codec::Record;",
        "pub type Path = skein_codec::Path<Limits>;",
        "pub type Problem = skein_codec::Problem<Limits>;",
        "pub const WORDS_WORDS: Path = Path::new(1, 0);",
        "static TABLE: &[skein_codec::bounds::Node]",
        "static CEILING_VALUES: &[u32]",
        "pub fn name(path: Path) -> Option<&'static str>",
        "Node::List",
        "size_of::<Words>()",
    ] {
        assert!(output.contains(required), "missing lean surface: {required}");
    }
    assert!(!output.contains("WordsParts"));
    assert!(!output.contains("pub fn new"));
    // Every bound names the tables: as constants, each mention is its own copy in the binary.
    assert!(!output.contains("const TABLE:"));
    assert!(!output.contains("const CEILING_VALUES:"));
}
