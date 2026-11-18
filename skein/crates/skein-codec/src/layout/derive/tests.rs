//! Discovery order and fixed-point edges of layout derivation (data/codec/layouts.md, 2).
#![expect(clippy::disallowed_macros, reason = "small schema fixtures are formatted inline")]
use super::{At, Builder, Key, Location, Place, Policy, location};
use crate::layout::{Kind, Node, Read, ReadKind};
use crate::schema::{self, DeclIx, Span, TypeIx, VariantIx};

const FIRST: Place = Place { decl: Span { start: 1, end: 2 }, part: None, old: false };
const LATER: Place = Place { decl: Span { start: 3, end: 4 }, part: None, old: true };
const ROOT: Location = Location { text: 0, at: At::Decl(DeclIx::new(0)), variant: None };
const KEY: Key = Key { old: ROOT, new: ROOT, root: true };

#[test]
fn keyed_requests_share_nodes_without_reordering_first_encounters() {
    let mut builder = Builder::new(Policy::Strict, None, false, None);
    // Each component distinguishes a reach. A destination-only change shares
    // the old reader, and a duplicate keeps its first refusal's place.
    for (key, place, node, read) in [
        (KEY, FIRST, 0, 0),
        (Key { new: Location { text: 1, ..ROOT }, ..KEY }, LATER, 1, 0),
        (Key { old: Location { text: 1, ..ROOT }, ..KEY }, LATER, 2, 1),
        (Key { old: Location { at: At::Type(TypeIx::new(0)), ..ROOT }, ..KEY }, LATER, 3, 2),
        (Key { new: Location { at: At::Type(TypeIx::new(0)), ..ROOT }, ..KEY }, LATER, 4, 0),
        (Key { old: Location { variant: Some(VariantIx::new(0)), ..ROOT }, ..KEY }, LATER, 5, 3),
        (Key { new: Location { variant: Some(VariantIx::new(0)), ..ROOT }, ..KEY }, LATER, 6, 0),
        (Key { root: false, ..KEY }, LATER, 7, 0),
        (KEY, LATER, 0, 0),
    ] {
        let found = builder.request(key, place);
        assert_eq!(found, node, "{key:?}");
        assert_eq!(builder.plan.node(found).read, read, "{key:?}");
    }
    assert_eq!(builder.places.get(0).expect("first conversion").error(super::Why::Deep), FIRST.error(super::Why::Deep));
    assert_eq!(
        builder.read_places.get(0).expect("first reader").error(super::Why::Deep),
        FIRST.error(super::Why::Deep)
    );
}

fn finished(body: &str) -> Builder {
    let schema = schema::parse(format!("family f 1\n{body}").as_bytes()).expect("fixture schema");
    let decl = schema.root(b"R", None).expect("fixture root").decl;
    let mut builder = Builder::new(Policy::Strict, None, false, None);
    let root = location(0, At::Decl(decl));
    builder.request(Key { old: root, new: root, root: true }, FIRST);
    builder.complete(&[&schema], &[&schema]).expect("fixture derives");
    builder
}

#[test]
fn a_first_pass_complete_layout_keeps_its_literal_plan() {
    let builder = finished("record R {}");
    // One settling pass, then the pass that observes no change.
    assert_eq!(builder.passes, 2);
    let plan = &builder.plan;
    assert_eq!((plan.root, plan.depth, plan.fields, plan.head, plan.old_head, plan.tag), (0, 1, 0, None, false, None));
    assert!(plan.dropped.is_empty());
    let [
        Node {
            read: 0,
            kind: Kind::Record { old_head: false, head: None, fields, in_place: true },
            keeps: true,
            empty: true,
            depth: 1,
        },
    ] = plan.nodes.as_slice()
    else {
        panic!("literal empty conversion: {:?}", plan.nodes);
    };
    assert!(fields.is_empty());
    let [Read { kind: ReadKind::Record { head: false, fields }, minimum: 0, depth: 1 }] = plan.reads.as_slice() else {
        panic!("literal empty reader: {:?}", plan.reads);
    };
    assert!(fields.is_empty());
}

#[test]
fn the_last_permitted_pass_still_settles_the_root() {
    // A record and a scalar take two levels; the remaining levels are options.
    let wrappers = super::super::MAX_DEPTH.checked_sub(2).expect("room for record and scalar");
    let builder = finished(&format!("record R {{ x: {}u8 }}", "option ".repeat(skein_lib::usize_of(wrappers))));
    assert_eq!(builder.passes, 64);
    assert_eq!(builder.plan.depth, 64);
    assert_eq!((builder.plan.read(0).depth, builder.plan.read(0).minimum), (64, 1));
    assert!(builder.plan.keeps());
}
