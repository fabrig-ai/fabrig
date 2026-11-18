//! Generator regressions for data/codec/README.md, sections 3 and 6.

use core::fmt::Write;

use crate::{Golden, generate, goldens, parse};

#[test]
fn fixed_rule_values_keep_names_order_and_bytes() {
    let schema = parse("family f 1 record Note { title: text 8 }").expect("valid note");
    assert_eq!(
        goldens(&schema),
        [
            Golden { name: "record_note_smallest.bin".into(), bytes: vec![0, 0, 0, 0] },
            Golden { name: "record_note_full.bin".into(), bytes: vec![0, 0, 0, 3, b'a', b'b', b'c'] },
        ]
    );
}

#[test]
fn path_names_keep_schema_spelling_and_variant_order() {
    let schema = parse(
        "family paths 1\nversioned record Record { some_field: u8 }\nrecord Plain { value: u8 }\nenum Enum { empty }",
    )
    .expect("versioned record, plain record and enumeration");
    let output = generate(&schema);
    let implementation = output.records.split_once("impl Path {\n").expect("path names").1;
    let implementation = implementation.split_once("\n}\n").expect("end of path implementation").0;
    assert_eq!(
        implementation,
        "    /// The path as the schema spells it (skein/docs/design/data/codec/README.md, 4).\n    #[must_use]\n    pub const fn name(self) -> &'static str {\n        match self {\n            Self::RecordSomeField => \"Record.some_field\",\n            Self::RecordVersion => \"Record (version)\",\n            Self::RecordTail => \"Record (tail)\",\n            Self::PlainValue => \"Plain.value\",\n            Self::PlainTail => \"Plain (tail)\",\n            Self::EnumTag => \"Enum (tag)\",\n        }\n    }"
    );
}

#[test]
fn limit_names_and_values_follow_field_order() {
    for text in [
        "family f 1 record HTTPRecord { raw: secret 64 maybe: option text 32 rows: list 3 list 2 option bytes 8 }",
        "family f 1 record Empty {}",
    ] {
        let schema = parse(text).expect("fixture family");
        let output = generate(&schema);
        let fields = output
            .records
            .split_once("pub struct Limits {\n")
            .expect("limits struct")
            .1
            .split_once("}\n")
            .expect("limits end")
            .0
            .lines()
            .map(|line| line.trim().strip_prefix("pub ").expect("public limit").split_once(':').expect("field type").0)
            .collect::<Vec<_>>();
        let names = output.records.split_once("pub const LIMIT_NAMES: [&str; ").expect("names constant").1;
        let (count, names) = names.split_once("] = [\n").expect("array length");
        let names = names
            .split_once("];\n")
            .expect("names end")
            .0
            .lines()
            .map(|line| line.trim().trim_end_matches(',').trim_matches('"'))
            .collect::<Vec<_>>();
        assert_eq!(count.parse::<usize>().expect("length"), fields.len());
        assert_eq!(names, fields);
        let values = output.records.split_once("pub fn values(&self) -> [u32; ").expect("values method").1;
        let (count, values) = values.split_once("] {\n        [\n").expect("returned array");
        let values = values
            .split_once("        ]\n")
            .expect("values end")
            .0
            .split_whitespace()
            .map(|field| field.trim_end_matches(',').strip_prefix("self.").expect("limit value"))
            .collect::<Vec<_>>();
        assert_eq!(count.parse::<usize>().expect("length"), fields.len());
        assert_eq!(values, fields);
        if !fields.is_empty() {
            use skein_codec::schema::{LimitLevel, limit_child, limit_name};
            let raw = limit_name(b"HTTPRecord", b"raw");
            let maybe = limit_child(&limit_name(b"HTTPRecord", b"maybe"), LimitLevel::Some);
            let rows = limit_name(b"HTTPRecord", b"rows");
            let item = limit_child(&rows, LimitLevel::Item);
            let nested = limit_child(&limit_child(&item, LimitLevel::Item), LimitLevel::Some);
            let expected = [&raw, &maybe, &rows, &item, &nested].map(|name| core::str::from_utf8(name).expect("ASCII"));
            assert_eq!(names, expected);
            assert_eq!(
                names,
                [
                    "h_t_t_p_record_raw",
                    "h_t_t_p_record_maybe_some",
                    "h_t_t_p_record_rows",
                    "h_t_t_p_record_rows_item",
                    "h_t_t_p_record_rows_item_item_some"
                ]
            );
        }
    }
}

#[test]
fn the_records_module_has_no_tests() {
    let schema = parse("family sample 1\nrecord Note { body: text 8 }\nenum Entry { note: Note, empty }")
        .expect("valid bounded record and enum");
    let crate::Output { records } = generate(&schema);
    assert!(!records.contains("#[cfg(test)]"));
    assert!(!records.contains("#[test]"));
}

fn list_measure(declarations: &str) -> String {
    // One admits both an empty list and a nonempty list.
    const ITEMS: u32 = 1;
    let source = format!("family measured 1\n{declarations}\nrecord Rows {{ values: list {ITEMS} Cell }}");
    let output = generate(&parse(&source).expect("valid measurement family"));
    let (_, measure) = output
        .records
        .split_once("impl Rows {\n    /// Measures this record's wire encoding.")
        .expect("Rows measurement");
    measure.split_once("\n    /// Writes").expect("encoding follows measurement").0.to_owned()
}

#[test]
fn fixed_record_lists_measure_by_count() {
    // Twelve is 2 + 2 + 8; six is the version, unit tag and three fixed bytes.
    let cases = [
        ("scalars", "record Cell { first: u16 second: u16 at: u64 }", "field_values.len().checked_mul(12)"),
        (
            "nested versioned",
            "enum Tag { one, two }\nrecord Inner { tag: Tag bytes: fixed 3 }\nversioned record Cell { value: Inner }",
            "field_values.len().checked_mul(6)",
        ),
        ("empty", "record Cell {}", "field_values.len().checked_mul(0)"),
    ];
    for (label, declarations, expected) in cases {
        let measure = list_measure(declarations);
        assert!(measure.contains(expected), "{label}: {measure}");
    }
}

#[test]
fn variable_record_lists_measure_each_value() {
    let cases = [
        ("bytes", "record Cell { value: bytes 1 }"),
        ("option", "record Cell { value: option u8 }"),
        (
            "unequal variants",
            "record Small { n: u8 }\nrecord Large { n: u16 }\nenum Choice { small: Small, large: Large }\nrecord Cell { value: Choice }",
        ),
    ];
    for (label, declarations) in cases {
        let measure = list_measure(declarations);
        assert!(
            measure.contains("for item_0 in field_values.as_slice()") && measure.contains("item_0.measure()"),
            "{label}: {measure}",
        );
    }
}

#[test]
fn a_family_of_120_records_has_no_function_past_100_lines() {
    let mut records = String::new();
    for index in 0_u32..120 {
        writeln!(&mut records, "record Record{index} {{}}").expect("schema string");
    }
    let schema = parse(&format!("family many 1\n{records}")).expect("valid family");
    let output = generate(&schema);
    assert!(output.records.contains("fn encoded_bound_0("));
    assert!(output.records.contains("fn decoded_bound_0("));
    for name in ["worst_case_bytes", "worst_case_heap", "worst_case_bytes_within", "worst_case_heap_within"] {
        assert!(!output.records.contains(name), "old name {name}");
    }
    check_function_lengths(&output.records);
}

fn check_function_lengths(source: &str) {
    let mut function = None;
    for (index, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("pub fn ") || trimmed.starts_with("pub(crate) fn ") || trimmed.starts_with("fn ") {
            let indent = line.len().checked_sub(trimmed.len()).expect("indent length");
            function = Some((index, format!("{}}}", " ".repeat(indent))));
        }
        if let Some((start, closing)) = &function
            && line == closing
        {
            assert!(index.checked_sub(*start).expect("function length") < 100, "function at line {start}");
            function = None;
        }
    }
}

#[test]
fn a_boxed_scalar_record_imports_its_box() {
    let schema = parse("family scalar 1\nrecord Value { n: u8 }\nenum Choice { value: boxed Value }")
        .expect("valid boxed variant");
    let output = generate(&schema);
    assert!(output.records.contains("use alloc::boxed::Box;"));
    assert!(output.records.contains("Value(Box<Value>)"));
}

#[test]
fn a_family_of_400_bounded_declarations_generates_in_half_a_second() {
    let mut source = String::from("family large 1\n");
    for index in 0_u32..400 {
        writeln!(&mut source, "record Record{index} {{ payload: bytes 64 }}").expect("schema string");
    }
    let schema = parse(&source).expect("valid large family");
    let output = generate(&schema);
    assert_eq!(
        output.records.matches("check_limits(limits)?;").count(),
        0,
        "constructors and decoders check only their values"
    );
}

#[test]
fn an_optional_unbounded_record_needs_no_bound_check() {
    let schema =
        parse("family tiny 1\nrecord Number { n: u64 }\nrecord Value { payload: bytes 3 maybe: option Number }")
            .expect("valid family");
    let output = generate(&schema);
    assert!(!output.records.contains("if let Some(some_0) = self.maybe.as_ref()"));
}

#[test]
fn a_bound_checks_its_familys_limits_once() {
    let schema = parse("family tiny 1\nrecord Value { payload: bytes 64 }").expect("valid family");
    let output = generate(&schema);
    let mut functions = 0_usize;
    for name in ["encoded_bound", "decoded_bound"] {
        for part in output.records.split(&format!("fn {name}")).skip(1) {
            let (signature, body) = part.split_once(" {").expect("function body");
            let (body, _) = body.split_once('}').expect("function closes");
            let checks = body.matches("check_limits(limits)").count();
            assert_eq!(checks, usize::from(!signature.contains("_within(")), "{signature}");
            functions += 1;
        }
    }
    assert_eq!(functions, 6, "two public type, two inner and two family functions");
}

#[test]
fn every_bound_name_is_reserved() {
    for name in ["worst_case_bytes", "worst_case_heap", "encoded_bound", "decoded_bound"] {
        for suffix in ["", "_within"] {
            let error = parse(&format!("family tiny 1\nrecord Value {{ {name}{suffix}: u8 }}"))
                .expect_err("generated bound names are reserved");
            assert!(error.message.contains("invalid generated field name"));
        }
    }
}

#[test]
fn a_limits_field_does_not_shadow_constructor_bounds() {
    let schema = parse(
        "family endpoint 1\nrecord LimitsValue { window: u32 }\nrecord Endpoint { name: text 64 limits: LimitsValue }",
    )
    .unwrap();
    let output = generate(&schema);
    let constructor = output.records.split_once("impl Endpoint {").unwrap().1.split_once("/// Reads").unwrap().0;
    assert!(constructor.contains("new(skein_limits: &Limits"), "{constructor}");
    assert!(constructor.contains("value.check(skein_limits)?"), "{constructor}");
}
