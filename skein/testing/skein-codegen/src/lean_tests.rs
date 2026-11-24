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
    let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
    let path = skein_scratch::paths::tree().join("crates/skein-codec/src/tests/lean_fixture.rs");
    check(&path, &output.records, write);
}

fn check(path: &std::path::Path, text: &str, write: bool) {
    if write {
        std::fs::create_dir_all(path.parent().expect("output has a parent")).expect("output directory");
        std::fs::write(path, text).expect("write generated output");
    } else {
        assert!(
            std::fs::read_to_string(path).ok().as_deref() == Some(text),
            "{} drifted: run SKEIN_CODEGEN_WRITE=1 cargo nextest run -p skein-codegen --lib -E 'test(=lean_tests::lean_helper_fixture_has_no_drift)'",
            path.display()
        );
    }
}

#[test]
fn lean_secret_fields_need_no_box_import() {
    let schema = parse("family hidden 1 record Row { value: option list 2 secret 8 }").expect("secret schema");
    let output = generate(&schema).records;
    assert!(output.contains("pub value: Option<List<skein_lib::Secret>>"));
    assert!(!output.contains("use alloc::boxed::Box;"), "secret fields use Secret, not Box");
}

#[test]
fn lean_declarations_have_a_concrete_debug_decode_entry() {
    let schema = parse("family roots 1\nrecord Item { byte: u8 }\nrecord Root { item: Item }\n").expect("root schema");
    let output = generate(&schema).records;
    let (nested, root) = output.split_once("// Record: Root.").expect("root implementation");
    let entry = "#[cfg(debug_assertions)]\n    fn decode(";
    assert!(nested.contains(entry), "nested types keep the entry even when used by another declaration");
    assert!(root.contains(entry), "only debug builds override the default entry");
}

#[test]
fn lean_surface_is_public() {
    let schema = parse(FIXTURE).expect("helper schema");
    let output = generate(&schema).records;
    assert_eq!(output.matches("pub const ENCODED_AT_ZERO: u64").count(), schema.declarations.len());
    for required in [
        "pub words: Box<[u8]>",
        "impl Record for Words",
        "pub use skein_codec::Record;",
        "pub type Path = skein_codec::Path<Limits>;",
        "pub type Problem = skein_codec::Problem<Limits>;",
        "pub const WORDS_WORDS: Path = Path::new(\"Words.words\");",
        "pub static DECLARATIONS: &[skein_codec::bounds::Declaration]",
        "static CEILING_VALUES: &[u32]",
        "pub fn at_most(&self, most: u32) -> Limits",
        "#[repr(C)]",
        "core::mem::offset_of!(Limits, nested_records)",
        "Node::Named(\"Words\")",
        "Node::List",
        "size_of::<Words>()",
    ] {
        assert!(output.contains(required), "missing lean surface: {required}");
    }
    assert!(!output.contains("WordsParts"));
    assert!(!output.contains("pub fn new"));
    assert!(!output.contains("PATH_NAMES"));
    assert!(!output.contains("pub fn name(path:"));
    // Every bound names the tables: as constants, each mention is its own copy in the binary.
    assert!(!output.contains("const DECLARATIONS:"));
    assert!(!output.contains("const CEILING_VALUES:"));
}

#[test]
fn lean_shapes_have_one_entry_per_declaration_in_name_order() {
    let schema = parse(FIXTURE).expect("helper schema");
    let output = generate(&schema).records;
    let (_, table) = output
        .split_once("pub static DECLARATIONS: &[skein_codec::bounds::Declaration] = &[\n")
        .expect("public shapes table");
    let (rows, _) = table.split_once("];\n").expect("end of shapes table");
    let names: Vec<_> = rows
        .lines()
        .map(|line| {
            let (_, name) = line.split_once("name: \"").expect("named shape");
            name.split_once('"').expect("quoted name").0
        })
        .collect();
    assert_eq!(names.len(), schema.declarations.len());
    assert!(names.is_sorted_by(|left, right| left < right), "strictly ascending, got {names:?}");
    assert_eq!(names, ["Choice", "Empty", "Nested", "Row", "Words"]);
}

#[test]
fn inserting_a_record_only_adds_its_own_lines() {
    const FAMILY: &str = "family insertion 1\n";
    const ITEM: &str = "record Item { byte: u8 }\n";
    const REPORT: &str = "record Report { title: text 8 samples: list 2 u16 note: option bytes 7 }\n";
    const EXTRA_ENTRIES: &[&str] = &[
        "    pub extra_title: u32,",
        "    pub extra_entries: u32,",
        "    pub extra_hint_some: u32,",
        "    extra_title: 5,",
        "    extra_entries: 2,",
        "    extra_hint_some: 3,",
        "    \"extra_title\",",
        "    \"extra_entries\",",
        "    \"extra_hint_some\",",
        "            self.extra_title,",
        "            self.extra_entries,",
        "            self.extra_hint_some,",
        "            extra_title: self.extra_title.min(most),",
        "            extra_entries: self.extra_entries.min(most),",
        "            extra_hint_some: self.extra_hint_some.min(most),",
        "    paths::EXTRA_TITLE,",
        "    paths::EXTRA_ENTRIES,",
        "    paths::EXTRA_HINT,",
        "    pub const EXTRA_TITLE: Path = Path::new(\"Extra.title\");",
        "    pub const EXTRA_ENTRIES: Path = Path::new(\"Extra.entries\");",
        "    pub const EXTRA_HINT: Path = Path::new(\"Extra.hint\");",
        "    pub const EXTRA_TAIL: Path = Path::new(\"Extra (tail)\");",
    ];
    const EXTRA_BOUNDS: &[&str] = &[
        "    skein_codec::bounds::Declaration { name: \"Extra\", nodes: Extra::SHAPE },",
        "    CEILINGS.extra_title,",
        "    CEILINGS.extra_entries,",
        "    CEILINGS.extra_hint_some,",
    ];
    let base = generate(&parse(&[FAMILY, ITEM, REPORT].concat()).expect("base schema")).records;
    let body = [ITEM, REPORT].concat();
    // A first declaration cannot name a later one. In the middle, the list
    // names Item, which now has a user where it did not before.
    for (before, after, item, item_node) in
        [("", body.as_str(), "u8", "Node::Scalar(1)"), (ITEM, REPORT, "Item", "Node::Named(\"Item\")")]
    {
        let text = format!(
            "{FAMILY}{before}record Extra {{ title: text 5 entries: list 2 {item} hint: option bytes 3 }}\n{after}"
        );
        let expanded = generate(&parse(&text).expect("schema with Extra")).records;
        let mut kept = base.lines().peekable();
        let added: Vec<_> = expanded
            .lines()
            .filter(|line| {
                if kept.peek() == Some(line) {
                    kept.next();
                    false
                } else {
                    true
                }
            })
            .collect();
        assert_eq!(kept.next(), None, "inserting Extra with {item} changed a pre-existing line");
        let block = added
            .strip_prefix(EXTRA_ENTRIES)
            .and_then(|lines| lines.strip_suffix(EXTRA_BOUNDS))
            .expect("the only new family entries are Extra's");
        assert_eq!(block.first().copied(), Some("// Type: Extra."));
        assert_eq!(block.iter().filter(|line| line.starts_with("// Type: ")).count(), 1);
        let block = block.join("\n");
        for line in ["pub struct Extra {", "// Record: Extra.", "const SHAPE:", "impl Record for Extra", item_node] {
            assert!(block.contains(line), "the added Extra block is missing {line}");
        }
    }
}
