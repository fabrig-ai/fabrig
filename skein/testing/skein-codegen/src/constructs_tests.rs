//! Construct classification regressions (data/codec/golden.md, section 1).

use super::{CONSTRUCTS, Construct, Kind, constructs, missing, without};

// Every's owner is a private world whose library no other package may name.
const EVERY: &str = include_str!("../../../tests/codec/schema/every-v3.schema");

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
fn difference_keeps_encounter_order() {
    let text = "family f 1 record R { a: text 8 b: u8 c: text 8 d: u8 = 0 }";
    let found = constructs(text).expect("valid schema");
    assert_eq!(found, [Construct::Field(Kind::Text), Construct::Field(Kind::U8), Construct::Default]);
    let pinned = CONSTRUCTS.iter().copied().filter(|item| *item != Construct::Field(Kind::Text)).collect::<Vec<_>>();
    assert_eq!(without(found.clone(), &pinned), [Construct::Field(Kind::Text)]);
    assert_eq!(without(found.clone(), &[]), found);
}

// These exhaustive matches make a new construct or kind require a pin slot.
fn kind_slot(kind: Kind) -> usize {
    match kind {
        Kind::U8 => 0,
        Kind::U16 => 1,
        Kind::U32 => 2,
        Kind::U64 => 3,
        Kind::Bool => 4,
        Kind::Duration => 5,
        Kind::Fixed => 6,
        Kind::Bytes => 7,
        Kind::Secret => 8,
        Kind::Text => 9,
        Kind::List => 10,
        Kind::Option => 11,
        Kind::Record => 12,
        Kind::Enum => 13,
    }
}

fn construct_slot(construct: Construct) -> usize {
    match construct {
        Construct::Field(kind) => kind_slot(kind),
        Construct::ListItem(kind) => kind_slot(kind).checked_add(14).expect("kind slot"),
        Construct::OptionValue(kind) => kind_slot(kind).checked_add(28).expect("kind slot"),
        Construct::EmptyRecord => 42,
        Construct::VersionedRecord => 43,
        Construct::UnitVariant => 44,
        Construct::RecordVariant => 45,
        Construct::BoxedVariant => 46,
        Construct::BytesHolds => 47,
        Construct::SecretHolds => 48,
        Construct::BytesPieces => 49,
        Construct::SecretPieces => 50,
        Construct::Default => 51,
        Construct::FormerName => 52,
    }
}

#[test]
fn pin_lists_each_construct_exactly_once() {
    let mut counts = [0_u32; 53];
    for construct in CONSTRUCTS {
        let count = counts.get_mut(construct_slot(*construct)).expect("construct slot");
        *count = count.checked_add(1).expect("small pin");
        assert_eq!(*count, 1, "duplicate construct: {construct:?}");
    }
    assert_eq!(counts, [1; 53], "pin lacks a construct at its exhaustive slot");
}

fn edited(text: &str, edits: &[(&str, &str)]) -> String {
    let mut text = text.to_owned();
    for (old, new) in edits {
        assert!(text.contains(old), "fixture lacks edit: {old:?}");
        text = text.replace(old, new);
    }
    text
}

fn without_declaration(text: &str, header: &str) -> String {
    let (before, body) = text.split_once(header).expect("declaration header");
    let (_, after) = body.split_once("\n}").expect("declaration end");
    format!("{before}{after}")
}

#[test]
fn missing_reports_removed_field_kinds() {
    for (kind, old, new) in [
        (Kind::U8, ": u8\n", ": u16\n"),
        (Kind::U16, ": u16\n", ": u8\n"),
        (Kind::U32, ": u32\n", ": u8\n"),
        (Kind::U64, ": u64\n", ": u8\n"),
        (Kind::Bool, ": bool", ": u8"),
        (Kind::Duration, ": duration\n", ": u8\n"),
        (Kind::Fixed, ": fixed 3\n", ": u8\n"),
        (Kind::Bytes, ": bytes ", ": option bytes "),
        (Kind::Secret, ": secret ", ": option secret "),
        (Kind::Text, ": text ", ": option text "),
        (Kind::List, ": list ", ": option list "),
        (Kind::Option, ": option ", ": list 1 option "),
    ] {
        let text = edited(EVERY, &[(old, new)]);
        let text = if kind == Kind::Bool { edited(&text, &[(" = false", " = 0")]) } else { text };
        assert_eq!(missing(&text).expect("edited every-v3 valid"), [Construct::Field(kind)], "{kind:?}");
    }
    let text = without_declaration(EVERY, "versioned record Every {");
    assert_eq!(missing(&text).expect("edited every-v3 valid"), [Construct::Field(Kind::Record)]);
    let text = edited(EVERY, &[("    shape: Shape\n", ""), ("    light: Light\n", "")]);
    assert_eq!(missing(&text).expect("edited every-v3 valid"), [Construct::Field(Kind::Enum)]);
}

#[test]
fn missing_reports_removed_list_item_kinds() {
    for (kind, old, new) in [
        (Kind::U8, "list 2 u8", "list 2 u16"),
        (Kind::U16, "list 2 u16", "list 2 u8"),
        (Kind::U32, "list 2 u32", "list 2 u8"),
        (Kind::U64, "list 2 u64", "list 2 u8"),
        (Kind::Bool, "list 2 bool", "list 2 u8"),
        (Kind::Duration, "list 2 duration", "list 2 u8"),
        (Kind::Fixed, "list 2 fixed 3", "list 2 u8"),
        (Kind::Bytes, "list 2 bytes 4", "list 2 u8"),
        (Kind::Secret, "list 2 secret", "list 2 bytes"),
        (Kind::Text, "list 2 text 4", "list 2 u8"),
        (Kind::List, "list 2 list 2", "list 2"),
        (Kind::Option, "list 2 option", "list 2"),
        (Kind::Record, "list 2 Numbers", "list 2 u8"),
        (Kind::Enum, "list 2 Light", "list 2 u8"),
    ] {
        let mut edits = vec![(old, new)];
        match kind {
            Kind::U8 => edits.push(("list 3 u8", "list 3 u16")),
            Kind::Record => {
                edits.push(("list 3 Blank", "list 3 u8"));
                edits.push(("list 2 Stamp", "list 2 u8"));
            }
            Kind::Enum => {
                edits.push(("list 3 Light", "list 3 u8"));
                edits.push(("list 2 Shape", "list 2 u8"));
            }
            Kind::U16
            | Kind::U32
            | Kind::U64
            | Kind::Bool
            | Kind::Duration
            | Kind::Fixed
            | Kind::Bytes
            | Kind::Secret
            | Kind::Text
            | Kind::List
            | Kind::Option => (),
        }
        let text = edited(EVERY, &edits);
        assert_eq!(missing(&text).expect("edited every-v3 valid"), [Construct::ListItem(kind)], "{kind:?}");
    }
}

#[test]
fn missing_reports_removed_option_value_kinds() {
    for (kind, old, new) in [
        (Kind::U8, "option u8", "option u16"),
        (Kind::U16, "option u16", "option u8"),
        (Kind::U32, "option u32", "option u8"),
        (Kind::U64, "option u64", "option u8"),
        (Kind::Bool, "option bool", "option u8"),
        (Kind::Duration, "option duration", "option u8"),
        (Kind::Fixed, "option fixed 3", "option u8"),
        (Kind::Bytes, "option bytes 4", "option u8"),
        (Kind::Secret, "option secret 4", "option u8"),
        (Kind::Text, "option text", "option bytes"),
        (Kind::List, "option list", "list"),
        (Kind::Option, "option option", "option"),
        (Kind::Record, "option Numbers", "option u8"),
        (Kind::Enum, "option Light", "option u8"),
    ] {
        let text = edited(EVERY, &[(old, new)]);
        assert_eq!(missing(&text).expect("edited every-v3 valid"), [Construct::OptionValue(kind)], "{kind:?}");
    }
}

#[test]
fn missing_reports_removed_modifiers_and_shapes() {
    for (construct, edits) in [
        (Construct::EmptyRecord, vec![("Blank {}", "Blank { byte: u8 }")]),
        (Construct::VersionedRecord, vec![("versioned ", "")]),
        (Construct::UnitVariant, vec![("enum Light { off, red, amber, green }", "enum Light { off: Blank }")]),
        (Construct::BoxedVariant, vec![("boxed ", "")]),
        (
            Construct::BytesHolds,
            vec![("bytes 8 holds every-other Thing", "bytes 8"), ("bytes 8 holds every-other Choice", "bytes 8")],
        ),
        (
            Construct::SecretHolds,
            vec![("secret 8 holds every-other Thing", "secret 8"), ("secret 8 holds every-other Choice", "secret 8")],
        ),
        (Construct::BytesPieces, vec![("bytes 64 pieces Numbers", "bytes 64")]),
        (Construct::SecretPieces, vec![("secret 64 pieces Numbers", "secret 64")]),
        (Construct::Default, vec![(" = false", "")]),
        (Construct::FormerName, vec![(" was Flags", "")]),
    ] {
        let text = edited(EVERY, &edits);
        assert_eq!(missing(&text).expect("edited every-v3 valid"), [construct], "{construct:?}");
    }
    // RecordVariant cannot be absent while BoxedVariant is present.
}

#[test]
fn missing_keeps_pin_order_and_empty_text_lacks_the_whole_pin() {
    assert_eq!(missing("family empty 1").expect("empty schema valid"), CONSTRUCTS);
    let text = edited(EVERY, &[(" was Flags", ""), (" = false", "")]);
    assert_eq!(missing(&text).expect("edited every-v3 valid"), [Construct::Default, Construct::FormerName]);
}

#[test]
fn holds_count_per_carrier_not_foreign_type() {
    for edits in [vec![(" holds every-other Choice", "")], vec![(" holds every-other Thing", "")]] {
        let text = edited(EVERY, &edits);
        assert_eq!(missing(&text).expect("edited every-v3 valid"), []);
    }
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
        assert_eq!(missing(text), Err(error), "{text}");
    }
}
