#[cfg(test)]
#[path = "../support/layout.rs"]
mod layout;

#[cfg(test)]
mod tests {
    use super::layout::split_layout;
    use core::fmt::Write;

    use skein_codegen::{Declaration, Error, Type, generate, parse};

    #[test]
    fn all_types_and_declarations_parse() {
        let schema = parse("family sample 1\nrecord Earlier {}\nenum Choice { absent, present: Earlier }\nversioned record All {\n    a: u8\n    b: u16\n    c: u32\n    d: u64\n    e: bool\n    f: duration\n    g: fixed 7\n    h: bytes 8\n    i: text 9\n    j: list 10 option Choice\n}\n").expect("valid schema");
        assert_eq!(schema.family, "sample");
        assert_eq!(schema.version, 1);
        assert_eq!(schema.declarations.len(), 3);
        let Declaration::Record(record) = schema.declarations.get(2).expect("third declaration") else {
            unreachable!("the third declaration is a record");
        };
        assert!(record.versioned);
        assert_eq!(record.fields.len(), 10);
        assert_eq!(
            record.fields.get(9).expect("tenth field").ty,
            Type::List(10, Box::new(Type::Option(Box::new(Type::Named("Choice".into())))))
        );
    }

    #[test]
    fn each_refusal_names_its_line_and_exact_message() {
        let variants = (0_u16..257).map(|index| format!("v{index}")).collect::<Vec<_>>().join(", ");
        let too_many_variants = format!("family f 1\nenum A {{ {variants} }}");
        let cases = [
            ("family f 1\nrecord R { a_b: bytes 1 a__b: bytes 1 }", 2, "generated path RAB is used twice"),
            ("family f 1\nrecord R {\n x:\n", 3, "unexpected end of schema"),
            ("family f 1\nrecord A { b: B }\nrecord B {}", 2, "type B used before declaration"),
            ("family f 1\nrecord A {}\nenum A { x }", 3, "duplicate type A"),
            ("family f 1\nrecord A { x: u8 x: u16 }", 2, "duplicate field x"),
            ("family f 1\nenum A { x, x }", 2, "duplicate variant x"),
            ("family f 1\nenum A { x: B }\nrecord B {}", 2, "type B used before declaration"),
            ("family f 1\nenum A { x }\nenum B { x: A }", 3, "variant x must hold one record"),
            ("family f 1\nenum A { x: { a: u8 } }", 2, "invalid Rust name {"),
            ("family f 1\nrecord A { b: list 4294967295 u64 }", 2, "encoded worst case for A exceeds u32"),
            ("family f 1\nrecord A { b: bytes 4294967295 }", 2, "encoded worst case for A exceeds u32"),
            ("family f 1\nrecord A {\n a: ?\n}", 3, "unexpected character '?'"),
            ("family f 1\nrecord A {\n a text 3\n}", 3, "expected :, found text"),
            ("family f 1\nrecord type {}", 2, "invalid generated type name type"),
            ("family f 1\nrecord lowercase {}", 2, "invalid generated type name lowercase"),
            ("family f 1\nrecord Box {}", 2, "generated type name Box is used twice"),
            ("family f 1\nrecord A {}\nrecord AParts {}", 3, "generated type name AParts is used twice"),
            ("family f 1\nrecord A { type: u8 }", 2, "invalid generated field name type"),
            ("family f 1\nrecord A { new: u8 }", 2, "invalid generated field name new"),
            ("family f 1\nrecord A { skein_bools: bool }", 2, "invalid generated field name skein_bools"),
            ("family f 1\nrecord A { Title: u8 }", 2, "invalid generated field name Title"),
            ("family f 1\nenum A { Read }", 2, "invalid generated variant name Read"),
            ("family f 1\nenum A { foo_bar, foo__bar }", 2, "generated variant FooBar is used twice"),
            ("family f 1\nrecord A { tail: u8 }", 2, "generated path ATail is used twice"),
            ("family f 1\nversioned record A { version: u8 }", 2, "generated path AVersion is used twice"),
            (
                "family f 1\nrecord A { b_item: bytes 1 b: list 1 bytes 1 }",
                2,
                "generated limit name a_b_item is used twice",
            ),
            ("family f 1\nrecord R { a: é }", 2, "unexpected character 'é'"),
            ("family f 1\nrecord R { a: ? }", 2, "unexpected character '?'"),
            ("family f\n", 1, "unexpected end of schema"),
            ("record f 1", 1, "expected family, found record"),
            ("family f 1\nrecord R :", 2, "expected {, found :"),
            ("family f 1\nrecord R { a u8 }", 2, "expected :, found u8"),
            ("family f 1\nversioned enum R {}", 2, "expected record, found enum"),
            ("family f 1\nrecord R { a: }", 2, "expected a type"),
            ("family f 1\nR {}", 2, "expected record, versioned record or enum, found R"),
            ("family f 1\nrecord 1R {}", 2, "invalid Rust name 1R"),
            ("family f 1\nrecord R { a: bytes nope }", 2, "invalid bound nope"),
            ("family {\n1", 2, "invalid family name"),
            ("family {", 1, "invalid family name"),
            ("family f 65536", 1, "invalid family version 65536"),
            ("family f 1\nrecord R { a:\nMissing }", 2, "type Missing used before declaration"),
            ("family f 1\nenum E { x }\nenum F { x: E }", 3, "variant x must hold one record"),
            ("family f 1\nenum E {}", 2, "enum has no variants"),
            ("family f 1\nrecord R {}\nrecord R {}", 3, "duplicate type R"),
            ("family f 1\nrecord R { a: u8 a: u16 }", 2, "duplicate field a"),
            ("family f 1\nenum E { x x }", 2, "duplicate variant x"),
            ("family f 1\nrecord R { a: list 4294967295 list 4294967295 u64 }", 2, "encoded worst case overflows u64"),
            ("family f 1\nrecord R { a: bytes 4294967295 }", 2, "encoded worst case for R exceeds u32"),
            (too_many_variants.as_str(), 2, "enum has more than 256 variants"),
        ];
        for (source, line, message) in cases {
            assert_eq!(
                parse(source).expect_err("schema should be refused"),
                Error { line, message: message.into() },
                "{source}"
            );
        }
    }

    #[test]
    fn a_family_without_declarations_parses() {
        let schema = parse("family large 1\n").expect("empty family");
        assert_eq!((schema.family.as_str(), schema.version, schema.declarations.len()), ("large", 1, 0));
    }

    #[test]
    fn an_unversioned_record_can_have_a_version_field() {
        parse("family f 1\nrecord A { version: u8 }").expect("unversioned record has no version path");
    }

    #[test]
    fn new_ceiling_messages_name_their_limits() {
        let source = format!("family f 1 #{}", "x".repeat(1_048_576));
        assert_eq!(
            parse(&source).expect_err("text past ceiling"),
            Error { line: 1, message: "schema longer than 1048576 bytes".into() }
        );
        let mut source = String::from("family f 1\n");
        for index in 0_u32..4097 {
            writeln!(&mut source, "record R{index} {{}}").expect("schema fixture");
        }
        assert_eq!(
            parse(&source).expect_err("declarations past ceiling"),
            Error { line: 4098, message: "more than 4096 declarations".into() }
        );
    }

    #[test]
    fn held_clauses_leave_generated_output_identical() {
        let with = parse("family f 1\nrecord Item { value: u8 }\nrecord R { a: bytes 8 holds other-family Value b: bytes 9 pieces Item }").expect("held clauses");
        let without =
            parse("family f 1\nrecord Item { value: u8 }\nrecord R { a: bytes 8 b: bytes 9 }").expect("opaque bytes");
        assert_ne!(with, without, "the owned model retains pieces for translation validation");
        let with = generate(&with).records;
        let without = generate(&without).records;
        let (with_layout, with_code) = split_layout(&with);
        let (without_layout, without_code) = split_layout(&without);
        assert_eq!(with_code, without_code, "only LAYOUT changes");
        assert_ne!(with_layout, without_layout, "held clauses enter the family fingerprint");
    }

    #[test]
    fn held_secret_clauses_leave_generated_output_identical() {
        let with = parse("family f 1\nrecord Item { value: u8 }\nrecord R { a: secret 8 holds other-family Value b: secret 9 pieces Item }").expect("held clauses");
        let without =
            parse("family f 1\nrecord Item { value: u8 }\nrecord R { a: secret 8 b: secret 9 }").expect("opaque bytes");
        assert_ne!(with, without, "the owned model retains pieces for translation validation");
        let with = generate(&with).records;
        let without = generate(&without).records;
        let (with_layout, with_code) = split_layout(&with);
        let (without_layout, without_code) = split_layout(&without);
        assert_eq!(with_code, without_code, "only LAYOUT changes");
        assert_ne!(with_layout, without_layout, "held clauses enter the family fingerprint");
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
        let (base_layout, (base_before, base_after)) = split_layout(&base);
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
            let (expanded_layout, (expanded_before, expanded_after)) = split_layout(&expanded);
            assert_ne!(base_layout, expanded_layout, "adding a declaration changes the family fingerprint");
            let mut kept = base_before.lines().chain(base_after.lines()).peekable();
            let added: Vec<_> = expanded_before
                .lines()
                .chain(expanded_after.lines())
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
            for line in ["pub struct Extra {", "// Record: Extra.", "const SHAPE:", "impl Record for Extra", item_node]
            {
                assert!(block.contains(line), "the added Extra block is missing {line}");
            }
        }
    }

    #[test]
    fn defaults_and_former_names_leave_generated_output_identical() {
        let with = parse("family f 1\nrecord Item was Before { value was old: u32 = 7 }\nenum State was Phase { paused was suspended, ready }\nrecord R { state: State = ready text: text 8 = \"hi #é\" list: list 4 u8 = [] opt: option u8 = some 3 }").expect("metadata");
        let without = parse("family f 1\nrecord Item { value: u32 }\nenum State { paused, ready }\nrecord R { state: State text: text 8 list: list 4 u8 opt: option u8 }").expect("layout alone");
        assert_eq!(with, without);
        assert_eq!(generate(&with), generate(&without));
    }

    #[test]
    fn layout_refusals_precede_the_rust_name_checks() {
        assert_eq!(
            parse("family f 1\nrecord lowercase {}\nrecord R { a: Missing }").expect_err("layout precedes Rust names"),
            Error { line: 3, message: "type Missing used before declaration".into() }
        );
    }
}
