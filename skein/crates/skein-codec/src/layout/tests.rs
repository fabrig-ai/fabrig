//! Hand-written wire examples, not a value decoder (data/codec/layouts.md, 6).
#![expect(clippy::disallowed_macros, reason = "small schema fixtures are formatted inline")]
use super::{Misconverted, Plan, Why, Workspace};
use crate::schema::{self, Root, Schema};
use skein_lib::Buffer;

fn schema(body: &str, version: u16) -> Schema {
    schema::parse(format!("family f {version}\n{body}").as_bytes()).expect("schema")
}
fn root(schema: &Schema, variant: Option<&[u8]>) -> Root {
    schema.root(b"R", variant).expect("root")
}
fn plan(old: &str, new: &str) -> Plan {
    let old = schema(old, 1);
    let new = schema(new, 2);
    Plan::derive(&old, root(&old, None), &[], &new, root(&new, None), &[]).expect("derived")
}
fn check(plan: &Plan, old: &[u8], new: &[u8], keeps: bool) {
    assert_eq!(plan.keeps(), keeps);
    let mut work = Workspace::new();
    assert!(work.fit(plan));
    let mut out = Buffer::with_capacity(512);
    let allocations = skein_heap::Span::start();
    let converted = plan.convert(old, &mut out, &mut work);
    assert_eq!(allocations.made(), 0, "a fitted pass allocates nothing");
    assert_eq!(converted, Ok(()));
    assert_eq!(out.as_slice(), new);
}
fn refused(old: &str, new: &str, why: Why, part: Option<&[u8]>, in_old: bool) {
    let old = schema(old, 1);
    let new = schema(new, 2);
    let error = Plan::derive(&old, root(&old, None), &[], &new, root(&new, None), &[]).expect_err("written change");
    assert_eq!(error.why, why);
    assert_eq!(error.old, in_old);
    let text = if in_old { &old } else { &new };
    assert_eq!(text.spelled(error.decl), b"R");
    match part {
        Some(expected) => assert_eq!(text.spelled(error.part.expect("part")), expected),
        None => assert!(error.part.is_none()),
    }
}

#[test]
fn added_field_uses_default_or_names_no_default() {
    let old = "record R { x: u8 }";
    check(&plan(old, "record R { x: u8 a: u32 = 7 }"), &[5], &[5, 0, 0, 0, 7], false);
    refused(old, "record R { x: u8 a: u32 }", Why::NoDefault, Some(b"a"), false);
}

#[test]
fn added_option_inserts_none_at_its_place() {
    check(&plan("record R { x: u8 }", "record R { a: option u32 x: u8 }"), &[5], &[0, 5], false);
}

#[test]
fn dropped_fields_are_scanned_and_kept_fields_reordered() {
    check(
        &plan("record R { a: u8 dropped: list 2 option u8 b: u16 }", "record R { b: u16 a: u8 }"),
        &[5, 0, 0, 0, 2, 0, 1, 9, 0, 7],
        &[0, 7, 5],
        false,
    );
}

#[test]
fn appended_variants_keep_inserted_tags_move_and_drops_are_written() {
    let old = "enum E { a b } record R { pad: u8 e: E }";
    check(&plan(old, "enum E { a b c } record R { pad: u8 e: E }"), &[9, 1], &[9, 1], true);
    check(&plan(old, "enum E { c a b } record R { pad: u8 e: E }"), &[9, 1], &[9, 2], false);
    let old = schema(old, 1);
    let new = schema("enum E { a } record R { pad: u8 e: E }", 2);
    let error = Plan::derive(&old, root(&old, None), &[], &new, root(&new, None), &[]).expect_err("variant dropped");
    assert_eq!(error.why, Why::VariantDropped);
    assert!(error.old);
    assert_eq!(old.spelled(error.decl), b"E");
    assert_eq!(old.spelled(error.part.expect("variant")), b"b");
}

#[test]
fn integers_widen_but_do_not_narrow() {
    check(&plan("record R { a: u8 }", "record R { a: u32 }"), &[5], &[0, 0, 0, 5], false);
    refused("record R { a: u32 }", "record R { a: u8 }", Why::Retyped, Some(b"a"), false);
}

#[test]
fn bounds_and_bytes_clauses_widen_but_not_to_another_held_family() {
    let old = "record A { a: u8 } record R { text: text 2 raw: bytes 4 items: bytes 8 pieces A }";
    let new = "record A { a: u8 } record R { text: bytes 4 raw: secret 8 items: bytes 16 }";
    let wire = &[0, 0, 0, 2, b'h', b'i', 0, 0, 0, 1, 9, 0, 0, 0, 5, 0, 0, 0, 1, 5];
    check(&plan(old, new), wire, wire, true);
    refused("record R { text: text 4 }", "record R { text: text 2 }", Why::Retyped, Some(b"text"), false);
    refused(
        "record R { held: bytes 8 holds g T }",
        "record R { held: secret 8 holds h T }",
        Why::Retyped,
        Some(b"held"),
        false,
    );
}

#[test]
fn wrappers_convert_level_by_level_but_retypes_are_written() {
    check(
        &plan("record R { a: list 2 option u8 }", "record R { a: option list 4 option option u16 }"),
        &[0, 0, 0, 2, 0, 1, 5],
        &[1, 0, 0, 0, 2, 0, 1, 1, 0, 5],
        false,
    );
    refused("record R { a: option u8 }", "record R { a: u8 }", Why::Retyped, Some(b"a"), false);
    refused("record R { a: u8 }", "record R { a: bool }", Why::Retyped, Some(b"a"), false);
}

#[test]
fn root_heads_are_replaced_inserted_and_removed() {
    check(&plan("versioned record R { a: u8 }", "versioned record R { a: u8 }"), &[99, 42, 5], &[0, 2, 5], true);
    check(&plan("record R { a: u8 }", "versioned record R { a: u8 }"), &[5], &[0, 2, 5], false);
    check(&plan("versioned record R { a: u8 }", "record R { a: u8 }"), &[99, 42, 5], &[5], false);
    let s = schema("versioned record R { a: u8 }", 1);
    let identity = Plan::derive(&s, root(&s, None), &[], &s, root(&s, None), &[]).expect("identity");
    check(&identity, &[99, 42, 5], &[0, 1, 5], true);
}

#[test]
fn nested_heads_prevent_keeps_until_their_versions_are_equal() {
    let body = "versioned record A { a: u8 } record R { a: A }";
    check(&plan(body, body), &[0, 1, 5], &[0, 2, 5], false);
    let s = schema(body, 1);
    let identity = Plan::derive(&s, root(&s, None), &[], &s, root(&s, None), &[]).expect("identity");
    check(&identity, &[0, 1, 5], &[0, 1, 5], true);
}

#[test]
fn broad_root_ignores_siblings_but_not_its_tag_or_tail() {
    let old = schema(
        "record A { x: u8 } record B { x: u8 } enum E { a: A b: B c } versioned record R { value: E models: list 2 u8 }",
        1,
    );
    let new = schema(
        "record A { x: u8 } record B { x: bool } enum E { a: A b: B } versioned record R { value: E models: list 2 u8 }",
        2,
    );
    let p = Plan::derive(&old, root(&old, Some(b"a")), &[], &new, root(&new, Some(b"a")), &[])
        .expect("sibling not reached");
    check(&p, &[0, 1, 0, 5, 0, 0, 0, 1, 8], &[0, 2, 0, 5, 0, 0, 0, 1, 8], true);
    let moved = schema("record A { x: u8 } enum E { c a: A } versioned record R { value: E models: list 2 u8 }", 2);
    let p =
        Plan::derive(&old, root(&old, Some(b"a")), &[], &moved, root(&moved, Some(b"a")), &[]).expect("tag derived");
    check(&p, &[0, 1, 0, 5, 0, 0, 0, 0], &[0, 2, 1, 5, 0, 0, 0, 0], false);
    let mut work = Workspace::new();
    assert!(work.fit(&p));
    assert_eq!(p.convert(&[0, 1, 1, 5, 0, 0, 0, 0], &mut Buffer::with_capacity(64), &mut work), Err(Misconverted::Tag));
}

#[test]
fn held_records_convert_in_their_length_and_enforce_the_new_bound() {
    let old = schema("record R { held: bytes 4 holds g T }", 1);
    let new = schema("record R { held: bytes 4 holds g T }", 2);
    let a = schema::parse(b"family g 1 versioned record T { a: u8 }").expect("held old");
    let b = schema::parse(b"family g 9 versioned record T { a: u16 }").expect("held new");
    let p = Plan::derive(&old, root(&old, None), &[&a], &new, root(&new, None), &[&b]).expect("held plan");
    check(&p, &[0, 0, 0, 3, 0, 1, 5], &[0, 0, 0, 4, 0, 9, 0, 5], false);
    let large = schema::parse(b"family g 9 versioned record T { a: u32 }").expect("large held");
    let p = Plan::derive(&old, root(&old, None), &[&a], &new, root(&new, None), &[&large])
        .expect("bound is checked per value");
    let mut work = Workspace::new();
    assert!(work.fit(&p));
    assert_eq!(p.convert(&[0, 0, 0, 3, 0, 1, 5], &mut Buffer::with_capacity(64), &mut work), Err(Misconverted::Bound));
    let error =
        Plan::derive(&old, root(&old, None), &[&a], &new, root(&new, None), &[]).expect_err("one held text missing");
    assert_eq!(error.why, Why::Held);
    assert_eq!(new.spelled(error.part.expect("field")), b"held");
    let p = Plan::derive(&old, root(&old, None), &[], &new, root(&new, None), &[]).expect("both opaque");
    check(&p, &[0, 0, 0, 3, 0, 1, 5], &[0, 0, 0, 3, 0, 1, 5], true);
}

#[test]
fn pieces_copy_kept_items_and_open_changed_items() {
    let old = "record A { a: u8 } record R { items: bytes 32 pieces A }";
    let wire = &[0, 0, 0, 10, 0, 0, 0, 1, 5, 0, 0, 0, 1, 7];
    check(&plan(old, old), wire, wire, true);
    check(
        &plan(old, "record A { a: u32 } record R { items: bytes 32 pieces A }"),
        wire,
        &[0, 0, 0, 16, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0, 4, 0, 0, 0, 7],
        false,
    );
    refused(old, "record R { items: bytes 32 }", Why::Pieces, Some(b"items"), false);
}

#[test]
fn former_declaration_field_and_variant_names_match() {
    let old = schema("record A { a: u8 } enum R { a: A }", 1);
    let new = schema("record B was A { b was a: u8 } enum R { b was a: B }", 2);
    let p = Plan::derive(&old, root(&old, Some(b"a")), &[], &new, root(&new, Some(b"b")), &[]).expect("former names");
    check(&p, &[0, 5], &[0, 5], true);
}

#[test]
fn unmatched_roots_reshapes_and_changed_payloads_are_underived() {
    let old = schema("record R {}", 1);
    let new = schema("record S {}", 2);
    let error = Plan::derive(&old, root(&old, None), &[], &new, new.root(b"S", None).expect("S"), &[])
        .expect_err("different roots");
    assert_eq!(error.why, Why::Root);
    let old = schema("record R {}", 1);
    let new = schema("enum R { a }", 2);
    let error = Plan::derive(&old, root(&old, None), &[], &new, root(&new, Some(b"a")), &[]).expect_err("root shape");
    assert_eq!(error.why, Why::Root);
    let old = schema("record A {} record R { a: A }", 1);
    let new = schema("enum A { a } record R { a: A }", 2);
    let error = Plan::derive(&old, root(&old, None), &[], &new, root(&new, Some(b"a")), &[]).expect_err("broad shape");
    assert_eq!(error.why, Why::Root);
    let old = schema("record A {} record R { pad: u8 a: A }", 1);
    let new = schema("enum A { a } record R { pad: u8 a: A }", 2);
    let error = Plan::derive(&old, root(&old, None), &[], &new, root(&new, None), &[]).expect_err("reshape reached");
    assert_eq!(error.why, Why::Reshaped);
    let old = schema("record A {} record B {} enum E { a: A } record R { pad: u8 e: E }", 1);
    let new = schema("record A {} record B {} enum E { a: B } record R { pad: u8 e: E }", 2);
    let error = Plan::derive(&old, root(&old, None), &[], &new, root(&new, None), &[]).expect_err("payload changed");
    assert_eq!(error.why, Why::Payload);
    assert_eq!(new.spelled(error.decl), b"E");
    assert_eq!(new.spelled(error.part.expect("variant")), b"a");
}

#[test]
fn nesting_is_refused_past_max_depth() {
    let body = format!(
        "record R {{ a: {}u8 }}",
        "option ".repeat(skein_lib::usize_of(super::MAX_DEPTH.checked_sub(1).expect("depth exceeds two")))
    );
    let old = schema(&body, 1);
    let new = schema(&body, 2);
    assert_eq!(
        Plan::derive(&old, root(&old, None), &[], &new, root(&new, None), &[]).expect_err("deep").why,
        Why::Deep
    );
    let at_bound = format!(
        "record R {{ a: {}u8 }}",
        "option ".repeat(skein_lib::usize_of(super::MAX_DEPTH.checked_sub(2).expect("depth exceeds two")))
    );
    let p = plan(&at_bound, &at_bound);
    check(&p, &[0], &[0], true);
}

#[test]
fn zero_byte_items_convert_without_walking_their_count() {
    let p = plan(
        "record E { a: fixed 0 } record R { items: list 4294967295 E }",
        "record E {} record R { items: list 4294967295 E }",
    );
    check(&p, &[255; 4], &[255; 4], false);
}

#[test]
fn malformed_input_and_output_room_have_specific_refusals() {
    let p = plan(
        "enum E { a } record R { a: bool e: E x: bytes 2 }",
        "enum E { a } record R { a: bool e: E x: bytes 2 more: option u8 }",
    );
    let mut work = Workspace::new();
    assert!(work.fit(&p));
    let mut out = Buffer::with_capacity(64);
    for (bytes, reason) in [
        (&[][..], Misconverted::Short),
        (&[2][..], Misconverted::Bool),
        (&[0, 1][..], Misconverted::Tag),
        (&[0, 0, 0, 0, 0, 3][..], Misconverted::Bound),
        (&[0, 0, 0, 0, 0, 1][..], Misconverted::Short),
        (&[0, 0, 0, 0, 0, 0, 9][..], Misconverted::Trailing),
    ] {
        assert_eq!(p.convert(bytes, &mut out, &mut work), Err(reason));
    }
    assert_eq!(p.convert(&[1, 0, 0, 0, 0, 0], &mut out, &mut work), Ok(()));
    assert_eq!(out.as_slice(), &[1, 0, 0, 0, 0, 0, 0]);
    assert_eq!(p.convert(&[1, 0, 0, 0, 0, 0], &mut Buffer::with_capacity(6), &mut work), Err(Misconverted::Room));
    let p = plan("record R { a: u8 }", "record R { a: u8 }");
    assert!(work.fit(&p));
    assert_eq!(p.convert(&[5], &mut Buffer::with_capacity(0), &mut work), Err(Misconverted::Room));
}

#[test]
fn unfitted_workspace_is_room_and_shared_workspace_reuses_without_allocating() {
    let a = plan("record R { a: list 2 u8 }", "record R { a: list 2 u32 }");
    let b = plan("record R { a: u8 b: u16 }", "record R { b: u16 a: u8 c: option u8 }");
    let mut work = Workspace::default();
    let mut out = Buffer::with_capacity(64);
    out.put(&[9]).expect("room");
    assert!(!work.fits(&a));
    assert_eq!(a.convert(&[], &mut out, &mut work), Err(Misconverted::Room));
    assert!(out.is_empty());
    assert!(work.fit(&a));
    assert!(!work.fits(&b));
    assert_eq!(b.convert(&[], &mut out, &mut work), Err(Misconverted::Room));
    assert!(work.fit(&b));
    assert!(work.fits(&a));
    let allocations = skein_heap::Span::start();
    for _ in 0_u32..3 {
        assert_eq!(a.convert(&[0, 0, 0, 2, 5, 7], &mut out, &mut work), Ok(()));
        assert_eq!(out.as_slice(), &[0, 0, 0, 2, 0, 0, 0, 5, 0, 0, 0, 7]);
        assert_eq!(b.convert(&[5, 0, 7], &mut out, &mut work), Ok(()));
        assert_eq!(out.as_slice(), &[0, 7, 5, 0]);
        assert_eq!(a.convert(&[0, 0, 0, 2], &mut out, &mut work), Err(Misconverted::Short));
    }
    assert_eq!(allocations.made(), 0);
    let _clone = work.clone();
    assert!(a.workspace_bound().is_some());
}
