//! Value roots, encoding-2 bytes, held reach and reviewed layout digests (README.md, 9.4).
#![expect(clippy::disallowed_macros, reason = "layout fixtures format schema texts")]
#![expect(clippy::disallowed_types, reason = "canonical byte tables use ordinary Rust test containers")]

use super::{Fingerprint, LayoutError, MAX_HELD, Root};
use crate::schema::{DeclIx, MAX_DECLARATIONS, Schema, VariantIx, parse};
use core::fmt::Write;
use skein_lib::sha256::{Sha256, digest};

fn schema(text: &str) -> Schema {
    parse(text.as_bytes()).expect("valid layout fixture")
}

fn layout(schema: &Schema, name: &str, variant: Option<&str>, held: &[&Schema]) -> Fingerprint {
    let root = match variant {
        Some(variant) => schema.root(name.as_bytes(), Some(variant.as_bytes())),
        None => schema.root(name.as_bytes(), None),
    }
    .expect("fixture root");
    schema.layout(root, held).expect("valid fixture layout")
}

fn canonical(parts: &[&[u8]]) -> [u8; 32] {
    let mut hash = Sha256::new();
    for part in parts {
        hash.update(part);
    }
    hash.finish()
}

#[test]
fn family_versions_leave_layouts_not_declaration_fingerprints_unchanged() {
    let one = schema("family f 1 versioned record R { a: u8 } record Outer { r: R }");
    let two = schema("family f 2 versioned record R { a: u8 } record Outer { r: R }");
    let root = one.root(b"R", None).expect("root");
    assert_eq!(one.layout(root, &[]), two.layout(root, &[]));
    assert_eq!(layout(&one, "R", None, &[]).digest(), digest(&[2, 1, 1, 0, 0, 0, 1, 1]));
    assert_eq!(layout(&one, "Outer", None, &[]), layout(&two, "Outer", None, &[]));
    assert_ne!(one.fingerprints().declaration(root.decl), two.fingerprints().declaration(root.decl));
    let plain = schema("family f 1 record R { a: u8 }");
    assert_ne!(layout(&one, "R", None, &[]), layout(&plain, "R", None, &[]));
}

fn broad(a: &str, b: &str, variants: &str, models: &str) -> Schema {
    schema(&format!(
        "family f 1 record A {{ {a} }} record B {{ {b} }} enum Value {{ {variants} }}
         versioned record Row {{ value: Value models: {models} }}"
    ))
}

#[test]
fn broad_roots_keep_siblings_out_but_include_payload_tail_and_tag() {
    let base = broad("x: u8", "y: u8", "a: A, b: B", "list 4 u8");
    let cases = [
        ("sibling", broad("x: u8", "y: u8 z: u16", "a: A, b: B", "list 4 u8"), false, false),
        ("own payload", broad("x: u8 z: u16", "y: u8", "a: A, b: B", "list 4 u8"), true, true),
        ("tail", broad("x: u8", "y: u8", "a: A, b: B", "list 5 u8"), true, false),
        ("tag", broad("x: u8", "y: u8", "before, a: A, b: B", "list 4 u8"), true, true),
        ("append", broad("x: u8", "y: u8", "a: A, b: B, after", "list 4 u8"), false, false),
        ("no payload", broad("x: u8", "y: u8", "a, b: B", "list 4 u8"), true, true),
    ];
    for (label, other, row_moves, enum_moves) in cases {
        assert_eq!(layout(&base, "Row", Some("a"), &[]) != layout(&other, "Row", Some("a"), &[]), row_moves, "{label}");
        assert_eq!(
            layout(&base, "Value", Some("a"), &[]) != layout(&other, "Value", Some("a"), &[]),
            enum_moves,
            "{label}"
        );
    }
    let leaf = digest(&[2, 1, 0, 0, 0, 0, 1, 1]);
    assert_eq!(layout(&base, "Value", Some("a"), &[]).digest(), canonical(&[&[2, 4, 0, 0, 0, 0, 1], &leaf]));
    assert_eq!(
        layout(&base, "Row", Some("a"), &[]).digest(),
        canonical(&[&[2, 1, 1, 0, 0, 0, 2, 15, 4, 0, 0, 0, 0, 1], &leaf, &[11, 0, 0, 0, 4, 1]])
    );
    let empty = broad("x: u8", "y: u8", "before, a, b: B", "list 4 u8");
    assert_eq!(layout(&empty, "Value", Some("a"), &[]).digest(), digest(&[2, 4, 0, 0, 0, 1, 0]));
}

#[test]
fn whole_broad_roots_include_every_variant() {
    let base = broad("x: u8", "y: u8", "a: A, b: B", "list 4 u8");
    for changed in [
        broad("x: u16", "y: u8", "a: A, b: B", "list 4 u8"),
        broad("x: u8", "y: u16", "a: A, b: B", "list 4 u8"),
        broad("x: u8", "y: u8", "a: A, b: B, c", "list 4 u8"),
    ] {
        for name in ["Row", "Value"] {
            assert_ne!(layout(&base, name, None, &[]), layout(&changed, name, None, &[]), "{name}");
        }
    }
}

#[test]
fn roots_find_own_names_and_refuse_variants_without_a_leading_enumeration() {
    let schema = schema(
        "family f 1 record A { x: u8 } enum Other { z } enum Value { a: A, b }
         versioned record Row { value: Value models: list 4 u8 }
         record Optional { v: option Value } record Listed { v: list 4 Value } record Empty {}",
    );
    let row = schema.root(b"Row", Some(b"a")).expect("Row at a");
    assert_eq!(row, Root { decl: DeclIx::new(3), variant: Some(VariantIx::new(1)) });
    schema.layout(row, &[]).expect("Row at a has a layout");
    assert_eq!(schema.root(b"Missing", None), None);
    assert_eq!(schema.root(b"Row", Some(b"missing")), None);
    assert_eq!(schema.root(b"row", Some(b"a")), None);
    for name in ["A", "Optional", "Listed", "Empty"] {
        assert_eq!(schema.root(name.as_bytes(), Some(b"a")), None, "{name}");
        let root = schema.root(name.as_bytes(), None).expect("nonbroad root");
        schema.layout(root, &[]).expect("nonbroad root without variant");
        assert_eq!(schema.layout(Root { variant: row.variant, ..root }, &[]), Err(LayoutError::Root), "{name}");
    }
    for name in ["Row", "Value"] {
        let root = schema.root(name.as_bytes(), None).expect("declaration by name");
        schema.layout(root, &[]).expect("whole broad declaration");
        assert_eq!(schema.layout(Root { variant: Some(VariantIx::new(0)), ..root }, &[]), Err(LayoutError::Root));
        assert_eq!(schema.layout(Root { variant: Some(VariantIx::new(99)), ..root }, &[]), Err(LayoutError::Root));
    }
    assert_eq!(
        schema.layout(Root { decl: DeclIx::new(schema.declarations()), variant: None }, &[]),
        Err(LayoutError::Root)
    );
}

#[test]
fn held_records_resolve_by_family_and_keep_the_named_fallback() {
    let holder = schema("family f 1 record R { a: bytes 64 holds g T b: secret 64 holds g T }");
    let one = schema("family g 1 record T { x: u8 }");
    let two = schema("family g 2 record T { x: u16 }");
    let unrelated = schema("family other 1 record T { x: u64 }");
    assert_ne!(layout(&holder, "R", None, &[&one]), layout(&holder, "R", None, &[&two]));
    assert_eq!(layout(&holder, "R", None, &[]), layout(&holder, "R", None, &[&unrelated]));
    let inner = digest(&[2, 1, 0, 0, 0, 0, 1, 1]);
    assert_eq!(
        layout(&holder, "R", None, &[&unrelated, &one]).digest(),
        canonical(&[&[2, 1, 0, 0, 0, 0, 2, 10, 0, 0, 0, 64, 3], &inner, &[14, 0, 0, 0, 64, 3], &inner])
    );
    let named = &[1, 0, 0, 0, 1, b'g', 0, 0, 0, 1, b'T'];
    assert_eq!(
        layout(&holder, "R", None, &[]).digest(),
        canonical(&[&[2, 1, 0, 0, 0, 0, 2, 10, 0, 0, 0, 64], named, &[14, 0, 0, 0, 64], named])
    );
    let nested = schema("family g 1 record T { x: bytes 64 holds h U }");
    let leaf_one = schema("family h 1 record U { x: u8 }");
    let leaf_two = schema("family h 1 record U { x: u16 }");
    assert_ne!(layout(&holder, "R", None, &[&nested, &leaf_one]), layout(&holder, "R", None, &[&leaf_two, &nested]));
    assert_eq!(layout(&holder, "R", None, &[&nested]), layout(&holder, "R", None, &[&nested, &unrelated]));
}

#[test]
fn held_depth_is_bounded_on_new_memoized_and_cyclic_paths() {
    assert_eq!(MAX_HELD, 4);
    let mut chain = Vec::new();
    for at in 0_u32..=5 {
        let field = if at == 5 { "x: u8".into() } else { format!("x: bytes 64 holds f{} T", at + 1) };
        chain.push(schema(&format!("family f{at} 1 record T {{ {field} }}")));
    }
    let primary = chain.first().expect("primary");
    let root = primary.root(b"T", None).expect("chain root");
    let mut at_bound = Vec::new();
    for text in chain.iter().skip(1).take(4) {
        at_bound.push(text);
    }
    primary.layout(root, &at_bound).expect("four resolved holds");
    let mut past_bound = Vec::new();
    for text in chain.iter().skip(1) {
        past_bound.push(text);
    }
    assert_eq!(primary.layout(root, &past_bound), Err(LayoutError::Deep));

    // d is first cached at depth one, then reached at depth four along a/b/c.
    let primary = schema("family f 1 record R { short: bytes 64 holds d T long: bytes 64 holds a T }");
    let cached = schema("family d 1 record T { x: bytes 64 holds e T }");
    let leaf = schema("family e 1 record T { x: u8 }");
    let first = schema("family a 1 record T { x: bytes 64 holds b T }");
    let second = schema("family b 1 record T { x: bytes 64 holds c T }");
    let third = schema("family c 1 record T { x: bytes 64 holds d T }");
    let root = primary.root(b"R", None).expect("memoized root");
    assert_eq!(primary.layout(root, &[&first, &second, &third, &cached, &leaf]), Err(LayoutError::Deep));
    primary.layout(root, &[&first, &second, &cached, &leaf]).expect("incomplete long path stops by name");

    let cycle = schema("family e 1 record T { x: bytes 64 holds d T }");
    assert_eq!(primary.layout(root, &[&cached, &cycle]), Err(LayoutError::Deep));
    let unreachable = schema("family f 1 record Unused { x: bytes 64 holds d T } record R { x: u8 }");
    unreachable
        .layout(unreachable.root(b"R", None).expect("unrelated root"), &[&cached, &cycle])
        .expect("unreached holds ignored");
    let sibling = schema(
        "family f 1 record A { x: u8 } record B { x: bytes 64 holds d T } enum E { a: A, b: B } record R { x: E }",
    );
    sibling
        .layout(sibling.root(b"R", Some(b"a")).expect("restricted root"), &[&cached, &cycle])
        .expect("unselected payload ignored");
}

#[test]
fn layout_names_comments_text_layout_and_boxing_move_nothing() {
    let base = schema("family f 3 record A { x: u8 } enum E { a: A, b } record R { x: E y: bytes 8 pieces A }");
    let cases = [
        "family renamed 3 record A { x: u8 } enum E { a: A, b } record R { x: E y: bytes 8 pieces A }",
        "family f 3 record B { x: u8 } enum E { a: B, b } record R { x: E y: bytes 8 pieces B }",
        "family f 3 record A { z: u8 } enum E { a: A, b } record R { z: E y: bytes 8 pieces A }",
        "family f 3 record A { x: u8 } enum E { z: A, b } record R { x: E y: bytes 8 pieces A }",
        "family f 3 # comment\nrecord A {\nx: u8,\n}\nenum E {\na: A,\nb,\n}\nrecord R {\nx: E,\ny: bytes 8 pieces A,\n}",
        "family f 3 record A { x: u8 } enum E { a: boxed A, b } record R { x: E y: bytes 8 pieces A }",
    ];
    for other in cases {
        let other = schema(other);
        for at in 0..base.declarations() {
            let root = Root { decl: DeclIx::new(at), variant: if at == 0 { None } else { Some(VariantIx::new(0)) } };
            let expected = base.layout(root, &[]).expect("base root");
            assert_eq!(expected, other.layout(root, &[]).expect("renamed root"));
        }
    }
}

#[test]
fn layout_types_and_unrestricted_inner_enumerations_have_canonical_bytes() {
    let leaf = digest(&[2, 1, 0, 0, 0, 0, 1, 2]);
    let enumeration = canonical(&[&[2, 2, 0, 0, 0, 2, 0, 1], &leaf]);
    let cases: &[(&str, &[&[u8]])] = &[
        ("u8", &[&[1]]),
        ("u16", &[&[2]]),
        ("u32", &[&[3]]),
        ("u64", &[&[4]]),
        ("bool", &[&[5]]),
        ("duration", &[&[6]]),
        ("fixed 258", &[&[7, 0, 0, 1, 2]]),
        ("secret 258", &[&[8, 0, 0, 1, 2]]),
        ("text 258", &[&[9, 0, 0, 1, 2]]),
        ("bytes 258", &[&[10, 0, 0, 1, 2, 0]]),
        ("list 258 option u16", &[&[11, 0, 0, 1, 2, 12, 2]]),
        ("option list 258 u16", &[&[12, 11, 0, 0, 1, 2, 2]]),
        ("Leaf", &[&[13], &leaf]),
        ("option E", &[&[12, 13], &enumeration]),
        ("bytes 258 pieces Leaf", &[&[10, 0, 0, 1, 2, 2], &leaf]),
        ("secret 258 pieces Leaf", &[&[14, 0, 0, 1, 2, 2], &leaf]),
    ];
    for (ty, encoded) in cases {
        let schema =
            schema(&format!("family f 258 record Leaf {{ n: u16 }} enum E {{ a, b: Leaf }} record R {{ x: {ty} }}"));
        let mut hash = Sha256::new();
        hash.update(&[2, 1, 0, 0, 0, 0, 1]);
        for part in *encoded {
            hash.update(part);
        }
        assert_eq!(layout(&schema, "R", None, &[]).digest(), hash.finish(), "{ty}");
    }
}

#[test]
fn layout_full_named_chains_are_hashed_without_recursion() {
    let mut text = String::from("family f 1 record R0 {}\n");
    let mut expected = digest(&[2, 1, 0, 0, 0, 0, 0]);
    for at in 1..MAX_DECLARATIONS {
        writeln!(text, "record R{at} {{ x: option R{} }}", at - 1).expect("string write");
        expected = canonical(&[&[2, 1, 0, 0, 0, 0, 1, 12, 13], &expected]);
    }
    let schema = schema(&text);
    assert_eq!(layout(&schema, &format!("R{}", MAX_DECLARATIONS - 1), None, &[]).digest(), expected);
}

#[test]
fn every_fixture_root_layout_digest_is_golden() {
    let schema =
        parse(include_str!("../../../../../tests/codec/schema/every-v1.schema").as_bytes()).expect("every fixture");
    let mut hex = String::new();
    for byte in layout(&schema, "Every", None, &[]).digest() {
        write!(hex, "{byte:02x}").expect("string write");
    }
    assert_eq!(hex, "af4da6fb6a57fb1390a72872f460677da61b608b9720f58adf1be7f2ec5b668e");
}
