//! Largest-value bytes written independently of the generator (data/codec/README.md, 6).

#[cfg(test)]
mod tests {
    use skein_codegen::{Schema, largest, largest_variants, parse, roots};

    #[test]
    fn roots_follow_wire_references_but_not_held_contents() {
        let schema = parse(
            "family f 1
             record Direct { x: u8 }
             record Listed { x: u8 }
             record Optional { x: u8 }
             record Payload { x: u8 }
             record Opaque { x: u8 }
             record A { x: Direct }
             record B { x: list 2 option Listed }
             record C { x: option Optional }
             enum Choice { empty, value: boxed Payload }
             record Held { x: bytes 9 pieces Opaque y: secret 9 holds f Opaque }
             record Last { x: u8 }",
        )
        .expect("reference and opaque types");
        assert_eq!(roots(&schema), ["Opaque", "A", "B", "C", "Choice", "Held", "Last"]);
    }

    #[test]
    fn largest_scalars_and_fixed_bytes_are_full() {
        let schema = parse(
            "family f 1 record Scalars {
                 a: u8 b: u16 c: u32 d: u64 e: bool f: duration g: fixed 3
             }",
        )
        .expect("scalar types");
        assert_eq!(
            largest(&schema, "Scalars", &[]).expect("largest scalars"),
            b"\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\x01\xff\xff\xff\xff\xff\xff\xff\xff\xa5\xa5\xa5"
        );
    }

    #[test]
    fn largest_variable_fields_use_each_supplied_length() {
        let schema = parse("family f 1 record Strings { a: bytes 9 b: secret 8 c: text 7 }").expect("strings");
        assert_eq!(
            largest(&schema, "Strings", &[2, 1, 3]).expect("bounded strings"),
            b"\0\0\0\x02\xa5\xa5\0\0\0\x01\xa5\0\0\0\x03aaa"
        );
        assert_eq!(largest(&schema, "Strings", &[0, 0, 0]).expect("zero lengths"), b"\0\0\0\0\0\0\0\0\0\0\0\0");
    }

    #[test]
    fn largest_lists_repeat_options_of_full_text() {
        let schema = parse("family f 1 record Rows { rows: list 5 option text 9 }").expect("optional text items");
        assert_eq!(
            largest(&schema, "Rows", &[2, 3]).expect("two full items"),
            b"\0\0\0\x02\x01\0\0\0\x03aaa\x01\0\0\0\x03aaa"
        );
        assert_eq!(largest(&schema, "Rows", &[0, 3]).expect("empty list"), b"\0\0\0\0");
        assert_eq!(
            largest(&schema, "Rows", &[2, 0]).expect("two present empty texts"),
            b"\0\0\0\x02\x01\0\0\0\0\x01\0\0\0\0"
        );
    }

    #[test]
    fn largest_named_records_repeat_with_their_own_limits() {
        let schema = parse(
            "family f 1
             record Empty {}
             record Cell { x: u16 label: text 8 }
             record Rows { first: Cell rows: list 3 Cell empty: list 3 Empty }",
        )
        .expect("named list items");
        assert_eq!(
            largest(&schema, "Rows", &[1, 2, 3]).expect("shared record limit"),
            b"\xff\xff\0\0\0\x01a\0\0\0\x02\xff\xff\0\0\0\x01a\xff\xff\0\0\0\x01a\0\0\0\x03"
        );
    }

    #[test]
    fn largest_enum_uses_longest_payload_and_first_equal_variant() {
        let schema = parse(
            "family f 1
             record Short { x: u16 }
             record Variable { x: text 9 }
             record Equal { x: bytes 9 }
             enum Choice { empty, short: Short, variable: boxed Variable, equal: Equal }
             enum Unit { first, second }",
        )
        .expect("unequal and equal payloads");
        assert_eq!(largest(&schema, "Choice", &[1, 2]).expect("later longest payload"), b"\x03\0\0\0\x02\xa5\xa5");
        assert_eq!(largest(&schema, "Choice", &[2, 2]).expect("first equal payload"), b"\x02\0\0\0\x02aa");
        assert_eq!(largest(&schema, "Choice", &[3, 1]).expect("limits change longest"), b"\x02\0\0\0\x03aaa");
        assert_eq!(largest(&schema, "Unit", &[1, 2]).expect("first unit variant"), b"\0");
    }

    fn variant_schema() -> Schema {
        parse(
            "family f 7
             record Short { text: text 9 }
             record Long { items: list 9 u16 }
             enum Choice { unit, short: Short, long: boxed Long }
             versioned record Root { choice: Choice }",
        )
        .expect("unit, text and list variants")
    }

    #[test]
    fn largest_variants_keeps_each_variant_in_schema_order() {
        let schema = variant_schema();
        assert_eq!(
            largest_variants(&schema, "Choice", &[2, 3]).expect("each clamped variant"),
            [b"\0".as_slice(), b"\x01\0\0\0\x02aa", b"\x02\0\0\0\x03\xff\xff\xff\xff\xff\xff",]
        );
    }

    #[test]
    fn largest_variants_returns_one_record_with_longest_nested_enum() {
        let schema = variant_schema();
        let variants = largest_variants(&schema, "Root", &[2, 3]).expect("one record");
        assert_eq!(variants, [b"\0\x07\x02\0\0\0\x03\xff\xff\xff\xff\xff\xff"]);
        assert_eq!(variants, [largest(&schema, "Root", &[2, 3]).expect("largest record")]);
    }

    #[test]
    fn largest_versioned_records_include_the_schema_version() {
        let schema =
            parse("family f 4097 versioned record Blank {} versioned record Row { x: u8 }").expect("versioned records");
        assert_eq!(largest(&schema, "Blank", &[]).expect("blank version"), b"\x10\x01");
        assert_eq!(largest(&schema, "Row", &[]).expect("version and field"), b"\x10\x01\xff");
    }

    #[test]
    fn largest_refuses_unknown_names_and_invalid_limits() {
        let schema = parse("family f 1 record Row { x: text 3 }").expect("one bound");
        assert!(largest(&schema, "Absent", &[1]).expect_err("unknown declaration").message.contains("Absent"));
        assert!(largest_variants(&schema, "Absent", &[1]).expect_err("unknown declaration").message.contains("Absent"));
        for limits in [&[][..], &[1, 2][..]] {
            assert!(largest(&schema, "Row", limits).expect_err("limit count").message.contains("expected 1 limits"));
            assert!(
                largest_variants(&schema, "Row", limits)
                    .expect_err("limit count")
                    .message
                    .contains("expected 1 limits")
            );
        }
        assert!(largest(&schema, "Row", &[4]).expect_err("limit above ceiling").message.contains("ceiling 3"));
        assert!(largest_variants(&schema, "Row", &[4]).expect_err("limit above ceiling").message.contains("ceiling 3"));
    }
}
