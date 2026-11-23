//! Generator regressions for data/codec/README.md, section 3, and data/codec/golden.md, section 1.

use crate::{FixedValue, fixed_values, generate, parse};

#[test]
fn fixed_rule_values_keep_names_order_and_bytes() {
    let schema = parse("family f 1 record Note { title: text 8 }").expect("valid note");
    assert_eq!(
        fixed_values(&schema),
        [
            FixedValue { name: "record_note_smallest.bin".into(), bytes: vec![0, 0, 0, 0] },
            FixedValue { name: "record_note_full.bin".into(), bytes: vec![0, 0, 0, 3, b'a', b'b', b'c'] },
        ]
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
        let names = output.records.split_once("pub const LIMIT_NAMES: &[&str] = &[\n").expect("names slice").1;
        let names = names
            .split_once("];\n")
            .expect("names end")
            .0
            .lines()
            .map(|line| line.trim().trim_end_matches(',').trim_matches('"'))
            .collect::<Vec<_>>();
        assert_eq!(names, fields);
        assert!(output.records.contains("pub fn values(&self) -> List<u32>"));
        let values = output
            .records
            .lines()
            .filter_map(|line| line.trim().strip_prefix("values.push(self."))
            .map(|line| line.split_once(')').expect("pushed field").0)
            .collect::<Vec<_>>();
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

#[test]
fn a_boxed_scalar_record_imports_its_box() {
    let schema = parse("family scalar 1\nrecord Value { n: u8 }\nenum Choice { value: boxed Value }")
        .expect("valid boxed variant");
    let output = generate(&schema);
    assert!(output.records.contains("use alloc::boxed::Box;"));
    assert!(output.records.contains("Value(Box<Value>)"));
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
