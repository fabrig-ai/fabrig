//! Tombstones and droppable marks are schema metadata, not layout (skein/docs/design/data/codec/run-time.md, 1).

use crate::schema::{Expected, FieldIx, Held, Named, Schema, Shape, Type, Wrong, parse, usize_of};

const ANNOTATED: &[u8] = b"family f 1
record First { x: u8 y: bytes 3 }
enum Choice { no yes: First }
record R {
    a: u32
    dropped b was c: u16 = 7
}
record S {
    dropped early: Choice = no
    z: Choice
    droppable hint: text 8
    b: u8
    dropped late: bytes 8 pieces First = 0x
}
record Empty { dropped obsolete: list 4294967295 list 4294967295 u64 = [] }
";

fn annotated() -> Schema {
    parse(ANNOTATED).expect("annotated schema")
}

#[test]
fn tombstones_follow_live_fields_and_keep_their_metadata() {
    let schema = annotated();
    let record = schema.find(b"R").expect("record R");
    assert_eq!(schema.declaration(record).shape, Shape::Record { versioned: false, first: FieldIx::new(2), count: 1 });
    let (b, count) = schema.tombstones(record);
    assert_eq!((b, count), (FieldIx::new(6), 1)); // First: 2, R: 1, S: 3, Empty: 0 live fields.
    let field = schema.field(b);
    assert_eq!(schema.spelled(field.name), b"b");
    assert_eq!(field.line, 6);
    assert_eq!(schema.ty(field.ty), Type::U16);
    assert_eq!(schema.spelled(schema.default_span(b).expect("explicit default")), b"7");
    assert_eq!(schema.default(b), Some([0, 7].as_slice()));
    let former = schema.was(Named::Field(b));
    assert_eq!(former.len(), 1);
    assert_eq!(schema.spelled(*former.first().expect("former name")), b"c");
}

#[test]
fn tombstone_ranges_preserve_text_order_and_resolve_types() {
    let schema = annotated();
    for name in [b"First".as_slice(), b"Choice"] {
        assert_eq!(schema.tombstones(schema.find(name).expect("declaration")).1, 0);
    }
    let (first, count) = schema.tombstones(schema.find(b"S").expect("record S"));
    assert_eq!((first, count), (FieldIx::new(7), 2));
    let first_decl = schema.find(b"First").expect("record First");
    let choice_decl = schema.find(b"Choice").expect("enum Choice");
    let cases = [
        (first, b"early".as_slice(), Type::Named(choice_decl), [0].as_slice()),
        (FieldIx::new(8), b"late", Type::Bytes { bound: 8, held: Held::Pieces(first_decl) }, [0; 4].as_slice()),
    ];
    for (at, name, ty, bytes) in cases {
        let field = schema.field(at);
        assert_eq!(schema.spelled(field.name), name);
        assert_eq!(schema.ty(field.ty), ty);
        assert_eq!(schema.default(at), Some(bytes));
    }
}

#[test]
fn only_a_marked_live_field_is_droppable() {
    let schema = annotated();
    for (index, name, droppable) in [(4, b"hint".as_slice(), true), (5, b"b", false), (6, b"b", false)] {
        let at = FieldIx::new(index);
        assert_eq!(schema.spelled(schema.field(at).name), name);
        assert_eq!(schema.droppable(at), droppable, "{index}");
    }
}

#[test]
fn dropped_and_droppable_remain_available_as_field_names() {
    let schema = parse(b"family f 1 record R { dropped: u64 droppable was old: u8 }").expect("keyword fields");
    let record = schema.find(b"R").expect("record R");
    assert_eq!(schema.declaration(record).shape, Shape::Record { versioned: false, first: FieldIx::new(0), count: 2 });
    for (index, name) in [(0, b"dropped".as_slice()), (1, b"droppable")] {
        let at = FieldIx::new(index);
        assert_eq!(schema.spelled(schema.field(at).name), name);
        assert!(!schema.droppable(at));
    }
    let former = schema.was(Named::Field(FieldIx::new(1)));
    assert_eq!(schema.spelled(*former.first().expect("former name")), b"old");
}

fn rejected(source: &str, line: u32, token: &str, wrong: Wrong) {
    let error = parse(source.as_bytes()).expect_err("invalid tombstone");
    assert_eq!(error.wrong, wrong, "{source}");
    assert_eq!(error.line, line, "{source}");
    assert_eq!(source.get(usize_of(error.at.start)..usize_of(error.at.end)), Some(token), "{source}");
}

#[test]
fn tombstone_without_equals_reports_the_token_after_its_type() {
    for (source, line, token) in [
        ("family f 1\nrecord R { dropped a: u16\n}", 3, "}"),
        ("family f 1\nrecord R { dropped a: option u16, }", 2, ","),
        ("family f 1\nrecord R { dropped a: option u16\n", 2, ""),
    ] {
        rejected(source, line, token, Wrong::Expected(Expected::Default));
    }
}

#[test]
fn tombstone_defaults_must_be_values_of_their_types() {
    for (ty, value) in [("Old", "0"), ("secret 8", "0x"), ("bytes 8 holds other Old", "0x"), ("u8", "256")] {
        let source = alloc::format!("family f 1\nrecord Old {{}}\nrecord R {{\n dropped a: {ty} = {value}\n}}");
        rejected(&source, 4, value, Wrong::Default);
    }
}

#[test]
fn tombstones_cannot_repeat_current_names_in_the_record() {
    for earlier in ["a: u8", "dropped a: u8 = 0"] {
        let source = alloc::format!("family f 1\nrecord R {{ {earlier}\n dropped a: u16 = 1 }}");
        rejected(&source, 3, "a", Wrong::DuplicateField);
    }
}

#[test]
fn tombstone_former_names_share_the_whole_record_scope() {
    for other in ["c: u8", "a was c: u8", "dropped c: u8 = 0", "dropped a was c: u8 = 0"] {
        let source = alloc::format!("family f 1\nrecord R {{ {other}\n dropped b was c: u16 = 7 }}");
        rejected(&source, 3, "c", Wrong::Former);
    }
}

#[test]
fn tombstone_types_can_only_reference_earlier_declarations() {
    for (ty, name) in [("Later", "Later"), ("option R", "R"), ("bytes 8 pieces Later", "Later")] {
        let source = alloc::format!("family f 1\nrecord R {{\n dropped b: {ty} = 0\n}}\nrecord Later {{}}");
        rejected(&source, 3, name, Wrong::Undeclared);
    }
}

#[test]
fn metadata_does_not_change_fingerprints_or_encoded_bounds() {
    type RootCase = (&'static [u8], &'static [Option<&'static [u8]>]);
    let marked = annotated();
    let plain = parse(
        b"family f 1
record First { x: u8 y: bytes 3 }
enum Choice { no yes: First }
record R { a: u32 }
record S { z: Choice hint: text 8 b: u8 }
record Empty {}
",
    )
    .expect("same schema without metadata");
    let marked_fingerprints = marked.fingerprints();
    let plain_fingerprints = plain.fingerprints();
    assert_eq!(marked_fingerprints.family(), plain_fingerprints.family());
    let roots: &[RootCase] = &[
        (b"First", &[None]),
        (b"Choice", &[None, Some(b"no"), Some(b"yes")]),
        (b"R", &[None]),
        (b"S", &[None, Some(b"no"), Some(b"yes")]),
        (b"Empty", &[None]),
    ];
    for &(name, variants) in roots {
        let marked_decl = marked.find(name).expect("annotated declaration");
        let plain_decl = plain.find(name).expect("plain declaration");
        assert_eq!(marked_fingerprints.declaration(marked_decl), plain_fingerprints.declaration(plain_decl));
        assert_eq!(marked.most_bytes(marked_decl), plain.most_bytes(plain_decl));
        for &variant in variants {
            let marked_root = marked.root(name, variant).expect("annotated root");
            let plain_root = plain.root(name, variant).expect("plain root");
            assert_eq!(
                marked.layout(marked_root, &[]).expect("root layout"),
                plain.layout(plain_root, &[]).expect("root layout")
            );
        }
    }
}
