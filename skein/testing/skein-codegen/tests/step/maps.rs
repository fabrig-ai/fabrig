//! Maps refuse author mistakes before emitting a module (data/codec/maps.md, 6-7).

use skein_codegen::{parse, translate};

const SCHEMA: &str = "family refusal 1
record Scalar { value: u64 }
record Pair { first: u32 second: u32 }
record Wrapped { value: Scalar }
record Pieces { bytes: bytes 64 pieces Scalar }
enum Choice { unit, value: Scalar }
record Root { scalar: Scalar maybe: option Scalar items: list 2 Scalar choice: Choice }
";

#[test]
fn maps_refusals_name_the_line_and_schema_without_writing() {
    let schema = parse(SCHEMA).expect("schema");
    let cases = [
        ("missing records", "record Scalar = d::Scalar", 1, "records"),
        ("unknown declaration", "records r\nrecord Absent = d::Absent", 2, "Absent"),
        ("wrong kind", "records r\nenum Scalar = d::Scalar", 2, "kind"),
        ("duplicate declaration", "records r\nrecord Scalar = d::Scalar\nrecord Scalar = d::Scalar", 3, "order"),
        ("declaration order", "records r\nrecord Pair = d::Pair\nrecord Scalar = d::Scalar", 3, "order"),
        ("unknown field", "records r\nrecord Pair = d::Pair { absent }", 2, "absent"),
        ("duplicate field", "records r\nrecord Pair = d::Pair { first, first }", 2, "order"),
        ("field order", "records r\nrecord Pair = d::Pair {\nsecond\nfirst\n}", 4, "order"),
        ("unknown variant", "records r\nenum Choice = d::Choice { absent }", 2, "absent"),
        ("duplicate variant", "records r\nenum Choice = d::Choice { unit, unit }", 2, "order"),
        ("variant order", "records r\nenum Choice = d::Choice { value tuple, unit }", 2, "order"),
        ("missing hand module", "records r\nrecord Scalar by hand\nrecord Wrapped = d::Wrapped", 3, "by hand"),
        ("missing declaration map", "records r\nrecord Wrapped = d::Wrapped", 2, "Scalar"),
        ("payload used as value", "records r\nrecord Scalar\nrecord Wrapped = d::Wrapped", 3, "domain type"),
        ("missing tuple map", "records r\nenum Choice = d::Choice { value tuple }", 2, "Scalar"),
        ("payload tuple", "records r\nrecord Scalar\nenum Choice = d::Choice { value tuple }", 3, "domain type"),
        ("missing inline map", "records r\nenum Choice = d::Choice", 2, "Scalar"),
        ("stands for many", "records r\nrecord Pair stands for first", 2, "one field"),
        ("stands for absent", "records r\nrecord Scalar stands for absent", 2, "absent"),
        ("unit tuple", "records r\nenum Choice = d::Choice { unit tuple }", 2, "unit"),
        ("unit block", "records r\nenum Choice = d::Choice { unit {} }", 2, "unit"),
        ("via declaration", "records r\nrecord Wrapped = d::Wrapped { value via c::x }", 2, "via"),
        (
            "via option declaration",
            "records r\nrecord Scalar = d::Scalar\nrecord Root = d::Root { maybe via c::x }\n",
            3,
            "via",
        ),
        (
            "via list declaration",
            "records r\nrecord Scalar = d::Scalar\nrecord Root = d::Root { items via c::x }\n",
            3,
            "via",
        ),
        ("mapped pieces", "records r\nrecord Pieces = d::Pieces", 2, "pieces"),
        ("payload pieces", "records r\nrecord Pieces", 2, "pieces"),
        ("stands for pieces", "records r\nrecord Pieces stands for bytes", 2, "pieces"),
        ("slice scalar", "records r\nrecord Scalar = d::Scalar { value slice }", 2, "slice"),
        (
            "slice declaration",
            "records r\nrecord Scalar = d::Scalar\nrecord Wrapped = d::Wrapped { value slice }",
            3,
            "slice",
        ),
        (
            "slice optional declaration",
            "records r\nrecord Scalar = d::Scalar\nrecord Root = d::Root { maybe slice }",
            3,
            "slice",
        ),
        ("boxed scalar", "records r\nrecord Scalar = d::Scalar { value boxed }", 2, "boxed"),
        (
            "boxed stands for",
            "records r\nrecord Scalar stands for value\nrecord Wrapped = d::Wrapped { value boxed }",
            3,
            "boxed",
        ),
        ("boxed list", "records r\nrecord Scalar = d::Scalar\nrecord Root = d::Root { items boxed }", 3, "boxed"),
        ("boxed inline", "records r\nrecord Scalar\nenum Choice = d::Choice { value boxed }", 3, "tuple"),
        ("boxed unit", "records r\nenum Choice = d::Choice { unit boxed }", 2, "tuple"),
    ];
    for (label, maps, line, reason) in cases {
        let error = translate(&schema, maps).expect_err(label);
        assert_eq!(error.line, line, "{label}: {error}");
        assert!(error.message.contains("schema refusal"), "{label}: {error}");
        assert!(error.message.contains(reason), "{label}: {error}");
        assert!(error.to_string().contains(&format!("line {line}: schema refusal")), "{label}: {error}");
    }
}

#[test]
fn maps_defaults_are_concrete_and_unused_hand_maps_need_no_module() {
    let schema = parse("family defaults 1\nrecord Row { nums: list 2 u32 }\nrecord Root { row: Row }").expect("schema");
    let output = translate(
        &schema,
        "records crate::records\nuse base = crate::domain\nuse d = base\nrecord Row = d::Row\nrecord Root by hand",
    )
    .expect("maps");
    assert!(output.starts_with("// Generated by skein-codegen from defaults 1 and its maps. Do not edit.\n"));
    assert!(output.contains("row_to_record(value: &crate::domain::Row, _context: &())"));
    assert!(output.contains("row_to_domain(record: crate::records::Row, _context: &())"));
    assert!(!output.contains("root_to_"));
    let with_context = translate(&schema, "records r\nuse d = crate::domain\ncontext core::option::Option<d::Row>\nrecord Row = d::Row\nrecord Root by hand").expect("context type aliases");
    assert!(with_context.contains("_context: &core::option::Option<crate::domain::Row>"));
    assert!(output.contains("nums: field_000000.clone()"));
    assert!(output.contains("nums: field_000000\n"));
    for forbidden in ["From::from", "unwrap(", "expect(", " as ", "=> _", "..", "matches!", "fn helper"] {
        assert!(!output.contains(forbidden), "generated {forbidden}");
    }
}

#[test]
fn long_translation_pairs_expect_only_the_direction_over_the_line_bound() {
    let unit_variants = |count: usize| (0..count).map(|index| format!("v{index}")).collect::<Vec<_>>().join(", ");
    let fields = |count: usize| (0..count).map(|index| format!("f{index}: u32")).collect::<Vec<_>>().join(", ");
    let payload_variants =
        |count: usize| (0..count).map(|index| format!("v{index}: Field")).collect::<Vec<_>>().join(", ");
    let cases = [
        (format!("enum Wide {{ {} }}", unit_variants(3)), "enum Wide = d::Wide", 5, 5),
        (format!("enum Wide {{ {} }}", unit_variants(90)), "enum Wide = d::Wide", 92, 92),
        (format!("enum Wide {{ {} }}", unit_variants(98)), "enum Wide = d::Wide", 100, 100),
        (format!("enum Wide {{ {} }}", unit_variants(99)), "enum Wide = d::Wide", 101, 101),
        (format!("enum Wide {{ {} }}", unit_variants(120)), "enum Wide = d::Wide", 122, 122),
        (format!("record Wide {{ {} }}", fields(97)), "record Wide = d::Wide", 100, 100),
        (format!("record Wide {{ {} }}", fields(98)), "record Wide = d::Wide", 101, 101),
        (
            format!("record Field {{ value: u32 }} enum Wide {{ {} }}", payload_variants(16)),
            "record Field\nenum Wide = d::Wide",
            50,
            98,
        ),
        (
            format!("record Field {{ value: u32 }} enum Wide {{ {} }}", payload_variants(17)),
            "record Field\nenum Wide = d::Wide",
            53,
            104,
        ),
    ];
    let expectation = "#[expect(clippy::too_many_lines, reason = \"one translation step per field or variant (data/codec/maps.md, 4.1)\")]";
    for (declaration, mapping, record_lines, domain_lines) in cases {
        let schema = parse(&format!("family line_bound 1\n{declaration}")).expect("schema");
        let output = translate(&schema, &format!("records r\n{mapping}")).expect("maps");
        for (direction, lines) in [("record", record_lines), ("domain", domain_lines)] {
            let signature = format!("pub fn wide_to_{direction}(");
            let (prefix, body) = output.split_once(&signature).expect("pair emitted");
            let body = body.split_once('\n').expect("body starts").1.split_once("\n}\n").expect("body ends").0;
            assert_eq!(body.lines().filter(|line| !line.trim().is_empty()).count(), lines);
            let mut preceding = prefix.lines().rev();
            if lines > 100 {
                assert_eq!(preceding.next(), Some(expectation), "{direction}: {declaration}");
            }
            assert!(preceding.next().expect("maps comment").starts_with("// Maps line "), "{direction}: {lines} lines");
        }
        let expected_count = usize::from(record_lines > 100) + usize::from(domain_lines > 100);
        assert_eq!(output.matches("#[expect(").count(), expected_count);
    }
}
