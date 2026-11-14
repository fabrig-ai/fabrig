//! Defaults and former names, independent of the layout tables.

use crate::schema::{Expected, FieldIx, Named, Span, VariantIx, Wrong, parse};

#[test]
fn defaults_are_decoder_encodings() {
    let cases: &[(&str, Option<&[u8]>)] = &[
        ("u32 = 7", Some(&[0, 0, 0, 7])),
        ("u8 = 255", Some(&[255])),
        ("u16 = 65535", Some(&[255, 255])),
        ("u64 = 18446744073709551615", Some(&[255; 8])),
        ("option u8", Some(&[0])),
        ("option u8 = none", Some(&[0])),
        ("option u8 = some 3", Some(&[1, 3])),
        ("option option u8 = some # comment\n some 3", Some(&[1, 1, 3])),
        ("text 8 = \"hi\"", Some(&[0, 0, 0, 2, b'h', b'i'])),
        ("text 4 = \"é#\"", Some(&[0, 0, 0, 3, 0xc3, 0xa9, b'#'])),
        ("text 0 = \"\"", Some(&[0, 0, 0, 0])),
        ("list 4 u8 = []", Some(&[0, 0, 0, 0])),
        ("list 0 u8 = [ #empty\n ]", Some(&[0, 0, 0, 0])),
        ("E = b", Some(&[1])),
        ("fixed 2 = 0x0a0b", Some(&[10, 11])),
        ("fixed 0 = 0x", Some(&[])),
        ("duration = 5", Some(&[0, 0, 0, 0, 0, 0, 0, 5])),
        ("bool = true", Some(&[1])),
        ("bool = false", Some(&[0])),
        ("bytes 2 = 0x0a0b", Some(&[0, 0, 0, 2, 10, 11])),
        ("bytes 8 pieces P = 0x", Some(&[0, 0, 0, 0])),
        ("u8", None),
    ];
    for &(ty, expected) in cases {
        let text = alloc::format!("family f 1 record P {{}} enum E {{ a b }} record R {{ a: {ty} next: u8 }}");
        let schema = parse(text.as_bytes()).expect("valid default");
        assert_eq!(schema.default(FieldIx::new(0)), expected, "{ty}");
        assert_eq!(schema.default(FieldIx::new(1)), None, "default ends before next field");
    }
}

#[test]
fn defaults_refuse_values_outside_their_type_at_the_default_span() {
    for (ty, value) in [
        ("u8", "256"),
        ("u8", "-1"),
        ("u16", "65536"),
        ("u32", "4294967296"),
        ("u64", "18446744073709551616"),
        ("u8", "some 1"),
        ("u8", "none"),
        ("u8", "[]"),
        ("bool", "1"),
        ("E", "absent"),
        ("E", "payload"),
        ("P", "0"),
        ("secret 8", "0x"),
        ("bytes 8 holds other P", "0x"),
        ("bytes 8 pieces P", "0x00"),
        ("text 1", "\"hi\""),
        ("text 8", "0x"),
        ("fixed 2", "0x00"),
        ("fixed 1", "0xAB"),
        ("bytes 1", "0x0000"),
        ("bytes 1", "0x0"),
        ("option u8", "3"),
        ("option u8", "some 256"),
    ] {
        let prefix = alloc::format!("family f 1\nrecord P {{}} enum E {{ a payload: P }}\nrecord R {{\n a: {ty} = ");
        let text = alloc::format!("{prefix}{value}\n}}");
        let error = parse(text.as_bytes()).expect_err("invalid default");
        assert_eq!(error.wrong, Wrong::Default, "{ty} = {value}");
        assert_eq!(error.line, 4);
        assert_eq!(
            error.at,
            Span {
                start: u32::try_from(prefix.len()).expect("short fixture"),
                end: u32::try_from(prefix.len().checked_add(value.len()).expect("short fixture"))
                    .expect("short fixture")
            }
        );
    }
    let text = b"family f 1\nrecord R { a: text 8 = \"\xff\" }";
    let error = parse(text).expect_err("text must be UTF-8");
    assert_eq!(error.wrong, Wrong::Default);
    assert_eq!(error.line, 2);
    assert_eq!(error.at, Span { start: 34, end: 37 });
}

#[test]
fn former_names_keep_text_order_and_keyword_names() {
    let schema = parse(b"family f 1 record Task was Job { was: u8 assignee was owner: u32 } enum E { paused was suspended was halted next, was } record More { raw: bytes 1 was: option u8 }").expect("former names and keyword names");
    assert_eq!(schema.spelled(schema.field(FieldIx::new(3)).name), b"was");
    assert_eq!(schema.was(Named::Declaration(schema.find(b"E").expect("enum"))), []);
    let task = schema.find(b"Task").expect("task");
    assert_eq!(schema.spelled(*schema.was(Named::Declaration(task)).first().expect("former declaration")), b"Job");
    assert_eq!(schema.was(Named::Field(FieldIx::new(0))), []);
    assert_eq!(schema.spelled(schema.field(FieldIx::new(0)).name), b"was");
    assert_eq!(schema.spelled(*schema.was(Named::Field(FieldIx::new(1))).first().expect("former field")), b"owner");
    let former = schema.was(Named::Variant(VariantIx::new(0)));
    assert_eq!(former.len(), 2);
    assert_eq!(schema.spelled(*former.first().expect("first former")), b"suspended");
    assert_eq!(schema.spelled(*former.last().expect("last former")), b"halted");
    assert_eq!(schema.spelled(schema.variant(VariantIx::new(1)).name), b"next");
    assert_eq!(schema.spelled(schema.variant(VariantIx::new(2)).name), b"was");
    assert_eq!(schema.was(Named::Variant(VariantIx::new(2))), []);
}

#[test]
fn former_names_are_unique_across_each_whole_scope() {
    for text in [
        "record R was R {}",
        "record R was S {} record S {}",
        "record R was X {} record S was X {}",
        "record R was X was X {}",
        "record R { a was a: u8 }",
        "record R { a was b: u8 b: u8 }",
        "record R { a was x: u8 b was x: u8 }",
        "enum E { a was a }",
        "enum E { a was b b }",
        "enum E { a was x b was x }",
    ] {
        let source = alloc::format!("family f 1\n{text}");
        let error = parse(source.as_bytes()).expect_err("former name conflict");
        assert_eq!(error.wrong, Wrong::Former, "{text}");
        assert_eq!(error.line, 2);
        let name = source
            .get(
                usize::try_from(error.at.start).expect("short fixture")
                    ..usize::try_from(error.at.end).expect("short fixture"),
            )
            .expect("error span");
        assert!(text.contains(&alloc::format!("was {name}")), "{text}: {error:?}");
    }
    parse(b"family f 1 record R was Before { a was old: u8 } record S { b was old: u8 } enum E { a was old }")
        .expect("separate scopes");
    // Former-name checks precede default checks even when the default is earlier.
    assert_eq!(
        parse(b"family f 1 record R { a: u8 = 256 } record S was R {}").expect_err("names first").wrong,
        Wrong::Former
    );
}

#[test]
fn missing_metadata_names_the_expected_token() {
    for (text, expected, token) in [
        ("record R was {", Expected::Name, "{"),
        ("record R { a was : u8 }", Expected::Name, ":"),
        ("enum E { a was }", Expected::Name, "}"),
        ("record R { a: u8 = }", Expected::Default, "}"),
        ("record R { a: option u8 = some }", Expected::Default, "}"),
        ("record R was", Expected::Name, ""),
        ("record R { a: u8 =", Expected::Default, ""),
    ] {
        let source = alloc::format!("family f 1\n{text}");
        let error = parse(source.as_bytes()).expect_err("metadata incomplete");
        assert_eq!(error.wrong, Wrong::Expected(expected), "{text}");
        assert_eq!(error.line, 2);
        assert_eq!(
            source
                .get(
                    usize::try_from(error.at.start).expect("short fixture")
                        ..usize::try_from(error.at.end).expect("short fixture")
                )
                .expect("error span"),
            token
        );
    }
}
