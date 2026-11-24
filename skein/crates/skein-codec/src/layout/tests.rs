//! Hand-written wire examples, not a value decoder (data/codec/layouts.md, 6).
#![expect(clippy::disallowed_macros, reason = "small schema fixtures are formatted inline")]
use super::{Misconverted, Mode, Plan, Refused, Underived, Why, Workspace};
use crate::schema::{self, Root, Schema};
use skein_lib::Buffer;

fn schema(body: &str, version: u16) -> Schema {
    schema::parse(format!("family f {version}\n{body}").as_bytes()).expect("schema")
}
fn root(schema: &Schema, variant: Option<&[u8]>) -> Root {
    schema.root(b"R", variant).expect("root")
}
fn strict(
    old: &Schema,
    old_root: Root,
    old_held: &[&Schema],
    new: &Schema,
    new_root: Root,
    new_held: &[&Schema],
) -> Result<Plan, Underived> {
    let plan = Plan::derive(old, old_root, old_held, new, new_root, new_held)?;
    let read = Plan::between(Mode::Read, old, old_root, old_held, new, new_root, new_held).expect("Read derives");
    assert_eq!(read, plan, "the store uses the same plan as Read");
    assert!(!read.refuses());
    Ok(plan)
}
fn between(mode: Mode, old: &str, new: &str) -> Result<Plan, Underived> {
    let old = schema(old, 1);
    let new = schema(new, 2);
    Plan::between(mode, &old, root(&old, None), &[], &new, root(&new, None), &[])
}
fn plan(old: &str, new: &str) -> Plan {
    let old = schema(old, 1);
    let new = schema(new, 2);
    strict(&old, root(&old, None), &[], &new, root(&new, None), &[]).expect("derived")
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
    let error = strict(&old, root(&old, None), &[], &new, root(&new, None), &[]).expect_err("written change");
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
fn repeated_named_reaches_share_readers_and_conversions() {
    let plan = plan("record L { n: u8 } record R { z: L a: L }", "record L { n: u16 } record R { z: L a: L }");
    assert_eq!(plan.nodes.len(), 5, "one conversion per reached location pair");
    assert_eq!(plan.reads.len(), 5, "one reader per reached old location");
    check(&plan, &[5, 7], &[0, 5, 0, 7], false);
}

#[test]
fn added_field_uses_default_or_names_no_default() {
    let old = "record R { x: u8 }";
    let with = "record R { x: u8 a: u32 = 7 }";
    let without = "record R { x: u8 a: u32 }";
    check(&plan(old, with), &[5], &[5, 0, 0, 0, 7], false);
    refused(old, without, Why::NoDefault, Some(b"a"), false);
    let read = between(Mode::Read, old, with).expect("Read inserts the default");
    let keep = between(Mode::Keep, old, with).expect("Keep inserts the default");
    assert_eq!(read, keep, "a target-only field is handled alike in both modes");
    assert!(!keep.refuses());
    check(&keep, &[5], &[5, 0, 0, 0, 7], false);
    for mode in [Mode::Read, Mode::Keep] {
        let error = between(mode, old, without).expect_err("a default is required");
        assert_eq!(error.why, Why::NoDefault);
        assert!(!error.old);
        assert_eq!(schema(without, 2).spelled(error.part.expect("added field")), b"a");
    }
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
fn read_skips_a_source_only_field_while_keep_compares_its_default() {
    let old = "versioned record R { x: u8 f: u16 = 7 z: u8 }";
    let new = "versioned record R { z: u8 x: u8 }";
    let read = plan(old, new);
    let keep = between(Mode::Keep, old, new).expect("drop at the default");
    assert!(keep.refuses());
    assert_eq!(keep.refusal(1), None);
    check(&read, &[0, 1, 9, 0, 7, 8], &[0, 2, 8, 9], false);
    check(&read, &[0, 1, 9, 0, 6, 8], &[0, 2, 8, 9], false);
    check(&keep, &[0, 1, 9, 0, 7, 8], &[0, 2, 8, 9], false);
    // Even the record head waits for all drop checks.
    check_refusal(&keep, &[0, 1, 9, 0, 6, 8], &[], (Refused::Field, "R", "f"));
}

#[test]
fn tombstone_supplies_a_missing_field_unless_it_has_its_own_default() {
    let old = "record R { a: u8 dropped b: u16 = 9 }";
    for (new, wire) in [("record R { a: u8 b: u16 }", [1, 0, 9]), ("record R { a: u8 b: u16 = 4 }", [1, 0, 4])] {
        check(&plan(old, new), &[1], &wire, false);
        for mode in [Mode::Read, Mode::Keep] {
            let forward = between(mode, old, new).expect("the missing field has a default");
            check(&forward, &[1], &wire, false);
        }
        let back = between(Mode::Read, new, old).expect("Read drops the restored field");
        check(&back, &wire, &[1], false);
    }
}

#[test]
fn tombstone_supplies_the_compared_default_unless_the_field_has_its_own() {
    let new = "record R { a: u8 dropped b: u16 = 9 }";
    for (old, accepted, refused) in
        [("record R { a: u8 b: u16 }", [1, 0, 9], [1, 0, 8]), ("record R { a: u8 b: u16 = 4 }", [1, 0, 4], [1, 0, 9])]
    {
        let keep = between(Mode::Keep, old, new).expect("Keep checks the dropped field");
        assert!(keep.refuses());
        check(&keep, &accepted, &[1], false);
        check_refusal(&keep, &refused, &[], (Refused::Field, "R", "b"));
        let read = plan(old, new);
        check(&read, &accepted, &[1], false);
        check(&read, &refused, &[1], false);
        let back = between(Mode::Read, new, old).expect("Read restores the compared default");
        check(&back, &[1], &accepted, false);
    }
}

#[test]
fn tombstone_does_not_require_a_droppable_field_to_equal_the_default() {
    let old = "record R { a: u8 droppable b: u16 }";
    let new = "record R { a: u8 dropped b: u16 = 9 }";
    let read = plan(old, new);
    let keep = between(Mode::Keep, old, new).expect("Keep drops the marked field without a check");
    assert!(!keep.refuses());
    for wire in [[1, 0, 0], [1, 0, 9], [1, 255, 255]] {
        check(&keep, &wire, &[1], false);
        check(&read, &wire, &[1], false);
    }
    let back = between(Mode::Read, new, old).expect("Read restores the tombstone default");
    check(&back, &[1], &[1, 0, 9], false);

    let bare = between(Mode::Keep, old, "record R { a: u8 }").expect("a mark alone permits a drop");
    assert!(!bare.refuses());
    check(&bare, &[1, 0, 8], &[1], false);
}

#[test]
fn tombstone_matches_former_names_on_either_side() {
    for (without, with) in [
        ("record R { a: u8 dropped old: u16 = 9 }", "record R { a: u8 b was old: u16 }"),
        ("record R { a: u8 dropped b was old: u16 = 9 }", "record R { a: u8 old: u16 }"),
    ] {
        let read_back = plan(without, with);
        let keep = between(Mode::Keep, with, without).expect("the tombstone names the dropped field");
        check(&keep, &[1, 0, 9], &[1], false);
        check(&read_back, &[1], &[1, 0, 9], false);
    }
}

#[test]
fn tombstone_of_another_type_refuses_the_plan_and_names_the_live_field() {
    for (live, dead) in [
        ("record R { a: u8 droppable b was old: u16 = 4 }", "record R { a: u8 dropped old: u32 = 9 }"),
        ("record R { a: u8 b: list 1 option u8 }", "record R { a: u8 dropped b: list 2 option u8 = [] }"),
    ] {
        let live = schema(live, 1);
        let dead = schema(dead, 2);
        for (old, new, in_old) in [(&live, &dead, true), (&dead, &live, false)] {
            let ar = root(old, None);
            let br = root(new, None);
            let error = Plan::derive(old, ar, &[], new, br, &[]).expect_err("tombstone type differs");
            assert_eq!(error.why, Why::Retyped);
            assert_eq!(error.old, in_old);
            assert_eq!(live.spelled(error.decl), b"R");
            assert_eq!(live.spelled(error.part.expect("live field")), b"b");
            for mode in [Mode::Read, Mode::Keep] {
                let between = Plan::between(mode, old, ar, &[], new, br, &[]).expect_err("tombstone type differs");
                assert_eq!(between, error);
            }
        }
    }
}

#[test]
fn a_source_only_field_without_default_is_planned_but_always_refuses_in_keep() {
    let old = "record R { f: u8 }";
    let new = "record R {}";
    let read = plan(old, new);
    let keep = between(Mode::Keep, old, new).expect("no default still makes a Keep plan");
    assert!(keep.refuses());
    for byte in [u8::MIN, 7, u8::MAX] {
        check(&read, &[byte], &[], false);
        check_refusal(&keep, &[byte], &[], (Refused::Field, "R", "f"));
    }
}

#[test]
fn zero_byte_list_items_still_refuse_source_only_fields_without_defaults() {
    let old = "record E { f: fixed 0 } record R { items: list 4294967295 E }";
    let new = "record E {} record R { items: list 4294967295 E }";
    let keep = between(Mode::Keep, old, new).expect("the empty field still needs a default");
    check(&keep, &[0, 0, 0, 0], &[0, 0, 0, 0], false);
    check_refusal(&keep, &[255; 4], &[255; 4], (Refused::Field, "E", "f"));
}

#[test]
fn appended_variants_keep_inserted_tags_move_and_drops_are_written() {
    let old = "enum E { a b } record R { pad: u8 e: E }";
    check(&plan(old, "enum E { a b c } record R { pad: u8 e: E }"), &[9, 1], &[9, 1], true);
    let inserted = "enum E { c a b } record R { pad: u8 e: E }";
    check(&plan(old, inserted), &[9, 1], &[9, 2], false);
    for mode in [Mode::Read, Mode::Keep] {
        let p = between(mode, old, inserted).expect("the inserted tag moves the old tags");
        assert!(!p.refuses());
        check(&p, &[9, 0], &[9, 1], false);
        check(&p, &[9, 1], &[9, 2], false);
    }
    let old = schema(old, 1);
    let new = schema("enum E { a } record R { pad: u8 e: E }", 2);
    let error = strict(&old, root(&old, None), &[], &new, root(&new, None), &[]).expect_err("variant dropped");
    assert_eq!(error.why, Why::VariantDropped);
    assert!(error.old);
    assert_eq!(old.spelled(error.decl), b"E");
    assert_eq!(old.spelled(error.part.expect("variant")), b"b");
}

#[test]
fn dropping_whole_enum_refuses_before_between_and_after_kept_variants() {
    let old = schema("record A { x: u8 } enum V { a: A b: A c: A }", 1);
    for (variants, dropped, kept) in [
        ("b: A c: A", "a", &[(1, 0), (2, 1)][..]),
        ("a: A c: A", "b", &[(0, 0), (2, 1)][..]),
        ("a: A b: A", "c", &[(0, 0), (1, 1)][..]),
    ] {
        let new = schema(&format!("record A {{ x: u8 }} enum W was V {{ {variants} }}"), 2);
        let old_root = old.root(b"V", None).expect("whole old enum");
        let new_root = new.root(b"W", None).expect("whole new enum");
        let strict = strict(&old, old_root, &[], &new, new_root, &[]).expect_err("variant dropped");
        assert_eq!(strict.why, Why::VariantDropped);
        assert!(strict.old);
        assert_eq!(old.spelled(strict.decl), b"V");
        assert_eq!(old.spelled(strict.part.expect("variant")), dropped.as_bytes());
        for mode in [Mode::Read, Mode::Keep] {
            let p = Plan::between(mode, &old, old_root, &[], &new, new_root, &[]).expect("drops refused per value");
            assert!(p.refuses());
            assert!(!p.keeps());
            for &(from, to) in kept {
                check(&p, &[from, 7], &[to, 7], false);
            }
            let tag = match dropped {
                "a" => 0,
                "b" => 1,
                "c" => 2,
                _ => unreachable!("fixture name"),
            };
            check_refusal(&p, &[tag, 7], &[], (Refused::Variant, "V", dropped));
            // Dispatch refuses the tag before reading or writing the dropped payload.
            check_refusal(&p, &[tag], &[], (Refused::Variant, "V", dropped));
            assert_eq!(p.refusal(1), None);
            assert_eq!(p.refusal(u16::MAX), None);
            let mut work = Workspace::new();
            assert!(work.fit(&p));
            assert_eq!(p.convert(&[3, 7], &mut Buffer::with_capacity(16), &mut work), Err(Misconverted::Tag));
        }
    }
}

fn check_refusal(p: &Plan, wire: &[u8], prefix: &[u8], why: (Refused, &str, &str)) {
    let mut work = Workspace::new();
    assert!(work.fit(p));
    let mut out = Buffer::with_capacity(64);
    let allocations = skein_heap::Span::start();
    let converted = p.convert(wire, &mut out, &mut work);
    assert_eq!(allocations.made(), 0, "a refused pass allocates nothing");
    let index = match converted {
        Err(Misconverted::Refused(index)) => index,
        other => panic!("expected a value refusal, got {other:?}"),
    };
    assert_eq!(p.refusal(index), Some(why));
    assert_eq!(out.as_slice(), prefix, "nothing of the refused part was written");
}

#[test]
fn nested_drops_prevent_copying_otherwise_kept_record_spans() {
    let old = schema("enum V { a b c } record Inner { v: V } record R { pad: u8 inner: Inner }", 1);
    let new = schema("enum V { a } record Inner { v: V } record R { pad: u8 inner: Inner }", 2);
    let strict = strict(&old, root(&old, None), &[], &new, root(&new, None), &[]).expect_err("nested variant dropped");
    assert_eq!(strict.why, Why::VariantDropped);
    let plans = [Mode::Read, Mode::Keep].map(|mode| {
        Plan::between(mode, &old, root(&old, None), &[], &new, root(&new, None), &[])
            .expect("nested drop refused per value")
    });
    // The plans own their names, not spans borrowed from the schemas.
    drop(old);
    drop(new);
    for p in plans {
        assert!(p.refuses());
        check(&p, &[9, 0], &[9, 0], false);
        check_refusal(&p, &[9, 1], &[9], (Refused::Variant, "V", "b"));
        check_refusal(&p, &[9, 2], &[9], (Refused::Variant, "V", "c"));
        assert_eq!(p.refusal(2), None);
    }
}

#[test]
fn dropping_still_refuses_fields_without_defaults() {
    let old = schema("enum V { a b } record R { v: V }", 1);
    let new = schema("enum V { a } record R { v: V added: u8 }", 2);
    let strict = strict(&old, root(&old, None), &[], &new, root(&new, None), &[]).expect_err("missing default");
    let dropping = Plan::between(Mode::Read, &old, root(&old, None), &[], &new, root(&new, None), &[])
        .expect_err("missing default even when drops allowed");
    assert_eq!(dropping, strict);
    assert_eq!(dropping.why, Why::NoDefault);
}

#[test]
fn dropping_without_drops_matches_strict_keeps_and_conversion() {
    let old = schema("versioned record R { x: u8 }", 1);
    for (body, expected) in
        [("versioned record R { x: u8 }", &[0, 2, 7][..]), ("versioned record R { x: u16 }", &[0, 2, 0, 7][..])]
    {
        let new = schema(body, 2);
        let strict = strict(&old, root(&old, None), &[], &new, root(&new, None), &[]).expect("strict");
        let dropping =
            Plan::between(Mode::Read, &old, root(&old, None), &[], &new, root(&new, None), &[]).expect("no drops");
        assert!(!dropping.refuses());
        assert_eq!(dropping.refusal(0), None);
        assert_eq!(dropping.workspace_bound(), strict.workspace_bound());
        check(&strict, &[0, 1, 7], expected, strict.keeps());
        check(&dropping, &[0, 1, 7], expected, strict.keeps());
    }
}

#[test]
fn integers_widen_but_do_not_narrow() {
    check(&plan("record R { a: u8 }", "record R { a: u32 }"), &[5], &[0, 0, 0, 5], false);
    refused("record R { a: u32 }", "record R { a: u8 }", Why::Retyped, Some(b"a"), false);
}

#[test]
fn narrowing_integers_and_bounds_is_checked_per_value_in_both_modes() {
    for (from, to, fits, writes, exceeds) in [
        ("u32", "u16", &[0, 0, 0, 7][..], &[0, 7][..], &[0, 1, 17, 112][..]), // 70000
        ("list 4 u8", "list 2 u8", &[0, 0, 0, 2, 1, 2], &[0, 0, 0, 2, 1, 2], &[0, 0, 0, 3, 1, 2, 3]),
        ("text 8", "text 4", b"\x00\x00\x00\x03hey", b"\x00\x00\x00\x03hey", b"\x00\x00\x00\x05hello"),
        ("bytes 4", "bytes 2", &[0, 0, 0, 2, 8, 9], &[0, 0, 0, 2, 8, 9], &[0, 0, 0, 3, 8, 9, 10]),
        ("secret 4", "secret 2", &[0, 0, 0, 2, 8, 9], &[0, 0, 0, 2, 8, 9], &[0, 0, 0, 3, 8, 9, 10]),
    ] {
        let old = format!("record R {{ f: {from} }}");
        let new = format!("record R {{ f: {to} }}");
        refused(&old, &new, Why::Retyped, Some(b"f"), false);
        let read = between(Mode::Read, &old, &new).expect("Read checks narrowing per value");
        let keep = between(Mode::Keep, &old, &new).expect("Keep checks narrowing per value");
        assert_eq!(read, keep);
        for p in [read, keep] {
            assert!(p.refuses());
            assert_eq!(p.refusal(1), None);
            check(&p, fits, writes, false);
            check_refusal(&p, exceeds, &[], (Refused::Narrowed, "R", "f"));
        }
    }
}

#[test]
fn a_nested_narrowed_bound_prevents_an_enclosing_record_from_keeping() {
    let old = "record Leaf { n: option bytes 3 } record R { pad: u8 leaf: Leaf }";
    let new = "record Inner was Leaf { size was n: option bytes 2 } record R { pad: u8 leaf: Inner }";
    for mode in [Mode::Read, Mode::Keep] {
        let p = between(mode, old, new).expect("former names match");
        check(&p, &[9, 1, 0, 0, 0, 2, 5, 6], &[9, 1, 0, 0, 0, 2, 5, 6], false);
        check_refusal(&p, &[9, 1, 0, 0, 0, 3, 5, 6, 7], &[9, 1], (Refused::Narrowed, "Leaf", "n"));
    }
}

#[test]
fn read_unwraps_with_the_outer_field_default_but_keep_refuses_absence() {
    let old = "record R { f: option u8 }";
    for (new, default) in [("record R { f: u8 = 4 }", Some(4)), ("record R { f: u8 }", None)] {
        refused(old, new, Why::Retyped, Some(b"f"), false);
        let read = between(Mode::Read, old, new).expect("Read unwraps");
        let keep = between(Mode::Keep, old, new).expect("Keep unwraps");
        assert!(keep.refuses());
        assert_eq!(read.refuses(), default.is_none());
        assert_eq!(read == keep, default.is_none());
        for p in [&read, &keep] {
            check(p, &[1, 7], &[7], false);
        }
        if let Some(default) = default {
            assert_eq!(read.refusal(0), None);
            check(&read, &[0], &[default], false);
        } else {
            check_refusal(&read, &[0], &[], (Refused::Absent, "R", "f"));
        }
        check_refusal(&keep, &[0], &[], (Refused::Absent, "R", "f"));
    }
}

#[test]
fn an_option_inside_a_list_cannot_use_the_fields_default() {
    let old = "record R { f: list 2 option u8 }";
    let new = "record R { f: list 2 u8 = [] }";
    refused(old, new, Why::Retyped, Some(b"f"), false);
    for mode in [Mode::Read, Mode::Keep] {
        let p = between(mode, old, new).expect("list item unwraps");
        check(&p, &[0, 0, 0, 2, 1, 7, 1, 8], &[0, 0, 0, 2, 7, 8], false);
        check_refusal(&p, &[0, 0, 0, 2, 1, 7, 0], &[0, 0, 0, 2, 7], (Refused::Absent, "R", "f"));
    }
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
    let old = "record R { a: u8 }";
    let new = "record R { a: bool }";
    refused(old, new, Why::Retyped, Some(b"a"), false);
    let read = between(Mode::Read, old, new).expect_err("Read cannot retype");
    let keep = between(Mode::Keep, old, new).expect_err("Keep cannot retype");
    assert_eq!(read, keep);
    assert_eq!(read.why, Why::Retyped);
}

#[test]
fn root_heads_are_replaced_inserted_and_removed() {
    check(&plan("versioned record R { a: u8 }", "versioned record R { a: u8 }"), &[99, 42, 5], &[0, 2, 5], true);
    check(&plan("record R { a: u8 }", "versioned record R { a: u8 }"), &[5], &[0, 2, 5], false);
    check(&plan("versioned record R { a: u8 }", "record R { a: u8 }"), &[99, 42, 5], &[5], false);
    let s = schema("versioned record R { a: u8 }", 1);
    let identity = strict(&s, root(&s, None), &[], &s, root(&s, None), &[]).expect("identity");
    check(&identity, &[99, 42, 5], &[0, 1, 5], true);
}

#[test]
fn nested_heads_prevent_keeps_until_their_versions_are_equal() {
    let body = "versioned record A { a: u8 } record R { a: A }";
    check(&plan(body, body), &[0, 1, 5], &[0, 2, 5], false);
    let s = schema(body, 1);
    let identity = strict(&s, root(&s, None), &[], &s, root(&s, None), &[]).expect("identity");
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
    let p = strict(&old, root(&old, Some(b"a")), &[], &new, root(&new, Some(b"a")), &[]).expect("sibling not reached");
    check(&p, &[0, 1, 0, 5, 0, 0, 0, 1, 8], &[0, 2, 0, 5, 0, 0, 0, 1, 8], true);
    let moved = schema("record A { x: u8 } enum E { c a: A } versioned record R { value: E models: list 2 u8 }", 2);
    let p = strict(&old, root(&old, Some(b"a")), &[], &moved, root(&moved, Some(b"a")), &[]).expect("tag derived");
    check(&p, &[0, 1, 0, 5, 0, 0, 0, 0], &[0, 2, 1, 5, 0, 0, 0, 0], false);
    let p = Plan::between(Mode::Read, &old, root(&old, Some(b"a")), &[], &moved, root(&moved, Some(b"a")), &[])
        .expect("siblings outside reach are not drops");
    assert!(!p.refuses());
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
    let p = strict(&old, root(&old, None), &[&a], &new, root(&new, None), &[&b]).expect("held plan");
    check(&p, &[0, 0, 0, 3, 0, 1, 5], &[0, 0, 0, 4, 0, 9, 0, 5], false);
    let large = schema::parse(b"family g 9 versioned record T { a: u32 }").expect("large held");
    let p =
        strict(&old, root(&old, None), &[&a], &new, root(&new, None), &[&large]).expect("bound is checked per value");
    let mut work = Workspace::new();
    assert!(work.fit(&p));
    assert_eq!(p.convert(&[0, 0, 0, 3, 0, 1, 5], &mut Buffer::with_capacity(64), &mut work), Err(Misconverted::Bound));
    let error = strict(&old, root(&old, None), &[&a], &new, root(&new, None), &[]).expect_err("one held text missing");
    assert_eq!(error.why, Why::Held);
    assert_eq!(new.spelled(error.part.expect("field")), b"held");
    let p = strict(&old, root(&old, None), &[], &new, root(&new, None), &[]).expect("both opaque");
    check(&p, &[0, 0, 0, 3, 0, 1, 5], &[0, 0, 0, 3, 0, 1, 5], true);
}

#[test]
fn held_enumerations_keep_append_and_convert_whole_without_a_head() {
    let old = schema("record R { held: bytes 8 holds g E }", 1);
    let new = schema("record R { held: bytes 8 holds g E }", 2);
    let a = schema::parse(b"family g 1 enum E { a b }").expect("held enum");
    let same = strict(&old, root(&old, None), &[&a], &new, root(&new, None), &[&a]).expect("held identity");
    check(&same, &[0, 0, 0, 1, 1], &[0, 0, 0, 1, 1], true);
    let initial = old.layout(root(&old, None), &[&a]).expect("held enum fingerprint");
    for (text, tag, keeps) in
        [(b"family g 9 enum E { a b c }".as_slice(), 1, true), (b"family g 9 enum E { c a b }".as_slice(), 2, false)]
    {
        let b = schema::parse(text).expect("changed held enum");
        let plan = strict(&old, root(&old, None), &[&a], &new, root(&new, None), &[&b]).expect("held enum plan");
        check(&plan, &[0, 0, 0, 1, 1], &[0, 0, 0, 1, tag], keeps);
        assert_ne!(initial, new.layout(root(&new, None), &[&b]).expect("changed enum fingerprint"));
    }
    let absent = schema::parse(b"family g 9 record Other { n: u8 }").expect("not the named held declaration");
    let refusal = strict(&old, root(&old, None), &[&a], &new, root(&new, None), &[&absent])
        .expect_err("held type is not a record or enumeration");
    assert_eq!(refusal.why, Why::Held);
    assert_eq!(new.layout(root(&new, None), &[&absent]), Err(schema::LayoutError::Root));
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
    let p = strict(&old, root(&old, Some(b"a")), &[], &new, root(&new, Some(b"b")), &[]).expect("former names");
    check(&p, &[0, 5], &[0, 5], true);
}

#[test]
fn unmatched_roots_reshapes_and_changed_payloads_are_underived() {
    let old = schema("record R {}", 1);
    let new = schema("record S {}", 2);
    let error =
        strict(&old, root(&old, None), &[], &new, new.root(b"S", None).expect("S"), &[]).expect_err("different roots");
    assert_eq!(error.why, Why::Root);
    let old = schema("record R {}", 1);
    let new = schema("enum R { a }", 2);
    let error = strict(&old, root(&old, None), &[], &new, root(&new, Some(b"a")), &[]).expect_err("root shape");
    assert_eq!(error.why, Why::Root);
    let old = schema("record A {} record R { a: A }", 1);
    let new = schema("enum A { a } record R { a: A }", 2);
    let error = strict(&old, root(&old, None), &[], &new, root(&new, Some(b"a")), &[]).expect_err("broad shape");
    assert_eq!(error.why, Why::Root);
    let old = schema("record A {} record R { pad: u8 a: A }", 1);
    let new = schema("enum A { a } record R { pad: u8 a: A }", 2);
    let error = strict(&old, root(&old, None), &[], &new, root(&new, None), &[]).expect_err("reshape reached");
    assert_eq!(error.why, Why::Reshaped);
    for mode in [Mode::Read, Mode::Keep] {
        assert_eq!(Plan::between(mode, &old, root(&old, None), &[], &new, root(&new, None), &[]), Err(error.clone()));
    }
    let old = schema("record A {} record B {} enum E { a: A } record R { pad: u8 e: E }", 1);
    let new = schema("record A {} record B {} enum E { a: B } record R { pad: u8 e: E }", 2);
    let error = strict(&old, root(&old, None), &[], &new, root(&new, None), &[]).expect_err("payload changed");
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
    assert_eq!(strict(&old, root(&old, None), &[], &new, root(&new, None), &[]).expect_err("deep").why, Why::Deep);
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
