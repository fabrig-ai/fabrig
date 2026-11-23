//! Maps refuse author mistakes before emitting a module (data/codec/maps.md, 6-7).

use std::{fs, process::Command};

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
    let scratch = skein_scratch::Scratch::new("maps-refusals");
    let schema_path = scratch.path().join("refusal-v1.schema");
    let maps_path = scratch.path().join("refusal.maps");
    let out = scratch.path().join("generated");
    fs::write(&schema_path, SCHEMA).expect("schema input");
    for (label, maps, line, reason) in cases {
        let error = translate(&schema, maps).expect_err(label);
        assert_eq!(error.line, line, "{label}: {error}");
        assert!(error.message.contains("schema refusal"), "{label}: {error}");
        assert!(error.message.contains(reason), "{label}: {error}");
        fs::write(&maps_path, maps).expect("maps input");
        let result = Command::new(env!("CARGO_BIN_EXE_skein-codegen"))
            .args([schema_path.as_os_str(), maps_path.as_os_str(), out.as_os_str()])
            .output()
            .expect("run maps command");
        assert!(!result.status.success(), "{label}: command accepted invalid maps");
        let error = String::from_utf8(result.stderr).expect("error is text");
        assert!(error.contains(&format!("line {line}: schema refusal")), "{label}: {error}");
        assert!(!out.exists(), "{label}: command wrote output");
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
    assert!(output.contains("row_to_record(value: &crate::domain::Row) -> crate::records::Row"));
    assert!(output.contains("row_to_domain(record: crate::records::Row) -> crate::domain::Row"));
    assert!(!output.contains("root_to_"));
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
            let prefix = prefix.strip_suffix("#[must_use]\n").unwrap_or(prefix);
            let mut preceding = prefix.lines().rev();
            if lines > 100 {
                assert_eq!(preceding.next(), Some(expectation), "{direction}: {declaration}");
            }
            assert_ne!(preceding.next(), Some(expectation), "{direction}: {lines} lines");
        }
        assert!(!output.contains("// Maps line"));
        let expected_count = usize::from(record_lines > 100) + usize::from(domain_lines > 100);
        assert_eq!(output.matches("#[expect(").count(), expected_count);
    }
}

#[test]
fn maps_command_uses_split_schema_order_and_the_first_stem() {
    let scratch = skein_scratch::Scratch::new("maps-command");
    let first = scratch.path().join("worker-wire-v1.schema");
    let second = scratch.path().join("worker-wire-v1-part1.schema");
    let maps = scratch.path().join("worker-wire.maps");
    let out = scratch.path().join("generated");
    let source = "family worker_wire 1\nrecord Row { value: u64 }\n";
    let tail = "record Root { row: Row }\n";
    let mapping = "records r\nrecord Row = d::Row\nrecord Root by hand\n";
    fs::write(&first, source).expect("first schema");
    fs::write(&second, tail).expect("second schema");
    fs::write(&maps, mapping).expect("maps");
    let result = Command::new(env!("CARGO_BIN_EXE_skein-codegen"))
        .args([first.as_os_str(), second.as_os_str(), maps.as_os_str(), out.as_os_str()])
        .output()
        .expect("run maps command");
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let schema = parse(&format!("{source}{tail}")).expect("split schema");
    assert_eq!(
        fs::read_to_string(out.join("worker_wire.rs")).expect("translation"),
        translate(&schema, mapping).expect("translate")
    );
    assert_eq!(fs::read_dir(out).expect("output directory").count(), 1);
}

fn mark_error(source: &str, line: usize, reason: &str) {
    let schema = parse("family marks 1\nrecord Row { value: u32 }").expect("schema");
    let error = translate(&schema, source).expect_err("invalid mark");
    assert_eq!(error.line, line);
    assert!(error.message.contains("schema marks"), "{error}");
    assert!(error.message.contains(reason), "{error}");
}

#[test]
fn refusal_mark_requires_a_call() {
    mark_error("records r\nrefuses c::unused\nrecord Row = d::Row", 2, "uncalled function c::unused");
}

#[test]
fn context_mark_requires_a_by_hand_call() {
    mark_error(
        "records r\ncontext C\nwith context c::unused\nrecord Row = d::Row",
        3,
        "uncalled by-hand function c::unused",
    );
}

#[test]
fn refusal_mark_cannot_repeat_through_an_alias() {
    mark_error(
        "records r\nuse c = crate::conv\nrefuses c::x_to_record\nrefuses crate::conv::x_to_record\nrecord Row = d::Row { value via c::x }",
        4,
        "marked twice",
    );
}

#[test]
fn context_mark_cannot_repeat() {
    mark_error(
        "records r\ncontext C\nwith context h::row_to_record\nwith context h::row_to_record\nrecord Row = d::Row",
        4,
        "marked twice",
    );
}

#[test]
fn context_mark_requires_a_type() {
    mark_error("records r\nwith context h::row_to_record\nrecord Row = d::Row", 2, "requires a context line");
}

#[test]
fn context_type_requires_a_mark() {
    mark_error("records r\ncontext C\nrecord Row = d::Row", 2, "requires a function marked with context");
}

#[test]
fn signatures_follow_reach_in_each_direction() {
    let schema = parse(
        "family reach 1
record Hand { value: u32 }
record Converted { value: u32 }
record Nested { rows: list 2 Converted }
record Slice { values: option list 2 u32 }
record Contextual { item: Hand }
enum Outer { idle, held: Contextual }",
    )
    .expect("schema");
    let output = translate(
        &schema,
        "records r
use h = crate::hand
use d = crate::domain
by hand h
context core::option::Option<d::Context>
with context h::hand_to_record
refuses c::v_to_domain
refuses h::hand_to_record
record Hand by hand
record Converted = d::Converted { value via c::v }
record Nested = d::Nested
record Slice = d::Slice { values slice }
record Contextual = d::Contextual
enum Outer = d::Outer { held tuple }",
    )
    .expect("maps");
    for signature in [
        "converted_to_record(value: &crate::domain::Converted) -> r::Converted",
        "converted_to_domain(record: r::Converted) -> Result<crate::domain::Converted, r::Problem>",
        "nested_to_record(value: &crate::domain::Nested) -> r::Nested",
        "nested_to_domain(record: r::Nested) -> Result<crate::domain::Nested, r::Problem>",
        "slice_to_record(value: &crate::domain::Slice) -> Result<r::Slice, r::Problem>",
        "slice_to_domain(record: r::Slice) -> crate::domain::Slice",
        "contextual_to_record(value: &crate::domain::Contextual, context: &core::option::Option<crate::domain::Context>) -> Result<r::Contextual, r::Problem>",
        "contextual_to_domain(record: r::Contextual) -> crate::domain::Contextual",
        "outer_to_record(value: &crate::domain::Outer, context: &core::option::Option<crate::domain::Context>) -> Result<r::Outer, r::Problem>",
        "outer_to_domain(record: r::Outer) -> crate::domain::Outer",
    ] {
        assert!(output.contains(&format!("pub fn {signature} {{")), "missing {signature}");
    }
    assert!(output.contains("crate::hand::hand_to_record(field_000000, context)?"));
    assert!(output.contains("crate::hand::hand_to_domain(field_000000)"));
    assert!(output.contains("contextual_to_record(held, context)?"));
}

#[test]
fn inserting_a_declaration_leaves_existing_translation_lines_in_order() {
    let source = "family stable 1\nrecord Leaf { value: u32 }\nrecord Root { leaf: Leaf }";
    let maps = "records r\nrecord Leaf = d::Leaf\nrecord Root = d::Root";
    let before = translate(&parse(source).expect("schema"), maps).expect("maps");
    let schema = parse(&source.replace("record Root", "record Added { value: bool }\nrecord Root")).expect("schema");
    let after = translate(&schema, &maps.replace("record Root", "record Added = d::Added\nrecord Root")).expect("maps");
    assert!(after.contains("pub fn added_to_record("));
    let mut later = after.lines();
    for line in before.lines() {
        assert!(later.any(|candidate| candidate == line), "missing line in order: {line}");
    }
}
