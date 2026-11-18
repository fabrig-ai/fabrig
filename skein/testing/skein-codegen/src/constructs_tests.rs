//! Construct classification regressions (data/codec/README.md, section 6).

use super::{CONSTRUCTS, Construct, Kind, constructs, unpinned, without};

fn found(declarations: &str) -> Vec<Construct> {
    constructs(&format!("family f 1\n{declarations}")).expect("valid construct fixture")
}

#[test]
fn each_kind_is_found_in_each_immediate_place() {
    let kinds = [
        ("u8", Kind::U8),
        ("u16", Kind::U16),
        ("u32", Kind::U32),
        ("u64", Kind::U64),
        ("bool", Kind::Bool),
        ("duration", Kind::Duration),
        ("fixed 8", Kind::Fixed),
        ("bytes 8", Kind::Bytes),
        ("secret 8", Kind::Secret),
        ("text 8", Kind::Text),
        ("list 2 u8", Kind::List),
        ("option u8", Kind::Option),
        ("R", Kind::Record),
        ("E", Kind::Enum),
    ];
    let mut expected = Vec::new();
    for (text, kind) in kinds {
        for (prefix, construct) in [
            ("", Construct::Field(kind)),
            ("list 2 ", Construct::ListItem(kind)),
            ("option ", Construct::OptionValue(kind)),
        ] {
            let source =
                format!("record R {{}} enum E {{ empty }} record Test {{ a: {prefix}{text} b: {prefix}{text} }}");
            let found = found(&source);
            assert_eq!(found.iter().filter(|item| **item == construct).count(), 1, "{source}: {found:?}");
            assert!(found.iter().all(|item| CONSTRUCTS.contains(item)), "{source}: {found:?}");
            expected.push(construct);
        }
    }
    let found = found("record R { a: u8 }");
    assert_eq!(found, [Construct::Field(Kind::U8)]);
    assert_eq!(expected.len(), 42);
    assert!(expected.iter().all(|item| CONSTRUCTS.contains(item)));
}

#[test]
fn declaration_forms_and_held_clauses_are_distinct() {
    for (source, expected) in [
        ("record R {}", vec![Construct::EmptyRecord]),
        ("versioned record R { a: u8 }", vec![Construct::VersionedRecord, Construct::Field(Kind::U8)]),
        ("enum E { empty }", vec![Construct::UnitVariant]),
        ("record R {} enum E { value: R }", vec![Construct::EmptyRecord, Construct::RecordVariant]),
        (
            "record R {} enum E { value: boxed R }",
            vec![Construct::EmptyRecord, Construct::RecordVariant, Construct::BoxedVariant],
        ),
        ("record R { a: bytes 8 holds other T }", vec![Construct::Field(Kind::Bytes), Construct::BytesHolds]),
        ("record R { a: secret 8 holds other T }", vec![Construct::Field(Kind::Secret), Construct::SecretHolds]),
        (
            "record R {} record S { a: bytes 8 pieces R }",
            vec![Construct::EmptyRecord, Construct::Field(Kind::Bytes), Construct::BytesPieces],
        ),
        (
            "record R {} record S { a: secret 8 pieces R }",
            vec![Construct::EmptyRecord, Construct::Field(Kind::Secret), Construct::SecretPieces],
        ),
        (
            "record R { a: list 2 option secret 8 holds other T }",
            vec![
                Construct::Field(Kind::List),
                Construct::ListItem(Kind::Option),
                Construct::OptionValue(Kind::Secret),
                Construct::SecretHolds,
            ],
        ),
    ] {
        assert_eq!(found(source), expected, "{source}");
        assert!(expected.iter().all(|item| CONSTRUCTS.contains(item)));
    }
}

#[test]
fn defaults_are_explicit_and_former_names_cover_each_named_part() {
    for source in ["record R { a: u8 = 0 }", "record R { a: option u8 = none }", "record R { a: text 8 = \"abc\" }"] {
        assert!(found(source).contains(&Construct::Default), "{source}");
    }
    assert!(!found("record R { a: option u8 }").contains(&Construct::Default));
    for source in
        ["record R was Q was P {}", "enum E was F { empty }", "record R { a was b: u8 }", "enum E { a was b }"]
    {
        assert_eq!(found(source).iter().filter(|item| **item == Construct::FormerName).count(), 1, "{source}");
    }
    assert!(!found("record R { a: u8 }").contains(&Construct::FormerName));
}

#[test]
fn output_shapes_keep_named_booleans_and_field_helpers() {
    assert!(!found("record R { a: bool b: bool c: bool }").contains(&Construct::NamedBooleans));
    let booleans = "record R { a: bool b: bool c: bool d: bool width: fixed 4 }";
    assert!(found(booleans).contains(&Construct::NamedBooleans));
    let output = crate::generate(&crate::parse(&format!("family f 1\n{booleans}")).expect("boolean record")).records;
    assert!(!output.contains("skein_bools"));
    assert!(!output.contains("[bool;"));
    assert!(output.contains("pub width: [u8; 4],"));
    assert_eq!(output.matches("#[expect(clippy::struct_excessive_bools,").count(), 2);
    for name in ["a", "b", "c", "d"] {
        assert!(output.contains(&format!("    {name}: bool,")));
        assert!(output.contains(&format!("    pub {name}: bool,")));
    }
    assert!(!found("record R { a: list 4 bool }").contains(&Construct::NamedBooleans));
    assert!(!found("record R { a: bytes 8 }").contains(&Construct::SplitFunction));
    let mut source = String::from("record R {");
    for at in 0_u32..40 {
        use core::fmt::Write;
        write!(source, " a{at}: bytes 8").expect("schema string");
    }
    source.push('}');
    assert!(found(&source).contains(&Construct::SplitFunction));
    let output = crate::generate(&crate::parse(&format!("family f 1\n{source}")).expect("long record")).records;
    assert!(output.contains("fn encode_a0_field("));
    assert!(output.contains("fn measure_a39_field("));
    assert!(!output.contains("Decoded"));
    crate::tests::check_item_names(&output);
    crate::tests::check_function_lengths(&output);
}

#[test]
fn pin_is_unique_and_difference_keeps_encounter_order() {
    assert_eq!(CONSTRUCTS.len(), 55);
    for (at, construct) in CONSTRUCTS.iter().enumerate() {
        assert!(!CONSTRUCTS.iter().skip(at.checked_add(1).expect("small list")).any(|item| item == construct));
    }
    let text = "family f 1 record R { a: text 8 b: u8 c: text 8 d: u8 = 0 }";
    let found = constructs(text).expect("valid schema");
    assert_eq!(found, [Construct::Field(Kind::Text), Construct::Field(Kind::U8), Construct::Default]);
    let pinned = CONSTRUCTS.iter().copied().filter(|item| *item != Construct::Field(Kind::Text)).collect::<Vec<_>>();
    assert_eq!(without(found.clone(), &pinned), [Construct::Field(Kind::Text)]);
    assert_eq!(without(found.clone(), &[]), found);
    assert_eq!(unpinned(text).expect("valid schema"), []);
}

#[test]
fn refusals_are_the_parsers_errors() {
    for text in [
        "not a schema",
        "family f 1 record R { a: ? }",
        "family f 1 record R { a: Missing }",
        "family f 1 record lowercase {}",
        "family f 1 record R { a: u8 = none }",
        "family f 1 record R was R {}",
    ] {
        let error = crate::parse(text).expect_err("invalid schema");
        assert_eq!(constructs(text), Err(error.clone()), "{text}");
        assert_eq!(unpinned(text), Err(error), "{text}");
    }
}
