//! Identical shallow inputs need no plan, not even at a stored current id.
use super::*;
use skein_codec::{layout::Plan, schema, value::Value};
use skein_lib::Buffer;

const STABLE: KeptFamily = KeptFamily {
    name: "fixture-stable",
    current: "family fixture-stable 1\nversioned record Stable { ok: bool }\n",
    keys: &[Keys { from: b"c", to: b"d", root: "Stable" }],
    types: &[ValueType { number: 0, root: "Stable", variant: None, keys: &[KeyRange { from: b"c", to: b"d" }] }],
    ..FAMILY
};
const TWO: OwnerSets = OwnerSets { sets: &[OwnerSet { families: &[FAMILY, STABLE], ..SET }] };

#[test]
fn every_current_shallow_type_opens_without_a_plan_and_reads_its_rows() {
    use jig_test_layouts::layouts::{CHILD_CURRENT, HOLDER};
    const SETS: OwnerSets = OwnerSets {
        sets: &[OwnerSet {
            families: &[
                FAMILY,
                STABLE,
                KeptFamily {
                    name: "fixture-holder",
                    current: HOLDER,
                    keys: &[Keys { from: b"h", to: b"i", root: "Row" }],
                    types: &[ValueType {
                        number: 0,
                        root: "Row",
                        variant: Some("changed"),
                        keys: &[KeyRange { from: b"h", to: b"i" }],
                    }],
                    ..FAMILY
                },
                KeptFamily { name: "fixture-child", current: CHILD_CURRENT, keys: &[], types: &[], ..FAMILY },
            ],
            ..SET
        }],
    };
    let mut store = planning_store(SETS);
    let rows: &[(&str, &[u8], &[u8])] = &[
        ("fixture-layouts", b"\x02a/0", &[0, 2, 0, 42, 7]),
        ("fixture-stable", b"\x02c/0", &[0, 1, 1]),
        ("fixture-holder", b"\x02h/0", &[0, 1, 0, 0, 0, 0, 4, 0, 1, 42, 7]),
    ];
    let mut ops = skein_lib::List::uncapped();
    for (_, key, value) in rows {
        ops.push(skein_kv::Op::Put { key: Box::from(*key), value: Box::from(*value) }).expect("test rows");
    }
    fixtures::plant(&mut store.stores, ops.as_slice());
    assert_eq!(super::super::open(&mut store, false), Ok(false));
    assert_eq!(counting::get().plans, 0, "all stored inputs are current and shallow");
    assert_eq!(store.registry.types.len(), 3);
    let mut decoder = super::super::rehearsal::Decoder::load(&mut store).expect("stored registry");
    for (family, key, bytes) in rows {
        let mut value = Box::from(*bytes);
        let mut out = Buffer::with_capacity(1024);
        assert_eq!(store.registry.read(key, &mut value, &mut out, &mut store.layout_work.workspace), *bytes);
        decoder.decode(family, 0, &mut Value::with_room(32), Box::from(*bytes)).expect("each stored row still decodes");
        decoder.family();
    }
}

#[test]
fn changing_one_types_layout_derives_only_its_old_plan() {
    const OLD: OwnerSets =
        OwnerSets { sets: &[OwnerSet { families: &[KeptFamily { current: KEPT, ..FAMILY }, STABLE], ..SET }] };
    let mut store = planning_store(OLD);
    fixtures::plant(
        &mut store.stores,
        &[
            skein_kv::Op::Put { key: Box::from(b"\x02a/0".as_slice()), value: Box::from([0, 2, 0, 42]) },
            skein_kv::Op::Put { key: Box::from(b"\x02c/0".as_slice()), value: Box::from([0, 1, 1]) },
        ],
    );
    store.owner_sets(TWO);
    assert_eq!(super::super::open(&mut store, false), Ok(true));
    assert_eq!(counting::get().plans, 1, "only Growing's held old layout needs a plan");
    store.registry.activate();
    let mut value = [0, 2, 0, 42];
    let mut out = Buffer::with_capacity(32);
    assert_eq!(
        store.registry.read(b"\x02a/0", &mut value, &mut out, &mut store.layout_work.workspace),
        &[0, 2, 0, 42, 7]
    );
    let mut stable = [0, 1, 1];
    assert_eq!(store.registry.read(b"\x02c/0", &mut stable, &mut out, &mut store.layout_work.workspace), &[0, 1, 1]);
}

#[test]
#[expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "the depth-bound test builds a static owner set from generated fixture text"
)]
fn an_identical_layout_past_the_depth_bound_keeps_its_exact_refusal() {
    use core::fmt::Write;
    const LEVELS: u32 = skein_codec::layout::MAX_DEPTH / 4;
    let mut text = String::from("family fixture-deep 1\nrecord Leaf { n: u8 }\n");
    for at in 0..LEVELS {
        let previous = if at == 0 { "Leaf".into() } else { format!("R{}", at - 1) };
        writeln!(text, "versioned record R{at} {{ n: list 1 list 1 list 1 {previous} }}").expect("fixture text");
    }
    let root: &'static str = Box::leak(format!("R{}", LEVELS - 1).into_boxed_str());
    let schema = schema::parse(text.as_bytes()).expect("deep but bounded declarations");
    let reached = schema.root(root.as_bytes(), None).expect("deep root");
    let refusal = Plan::derive(&schema, reached, &[], &schema, reached, &[]).expect_err("derive refuses deep identity");
    assert_eq!(refusal.why, skein_codec::layout::Why::Deep);
    let sets = OwnerSets {
        sets: Box::leak(Box::new([OwnerSet {
            families: Box::leak(Box::new([KeptFamily {
                name: "fixture-deep",
                current: Box::leak(text.into_boxed_str()),
                keys: Box::leak(Box::new([Keys { from: b"a", to: b"b", root }])),
                types: Box::leak(Box::new([ValueType { number: 0, root, variant: None, keys: FAMILY.types[0].keys }])),
                ..FAMILY
            }])),
            ..SET
        }])),
    };
    let mut store = planning_store(OwnerSets::NONE);
    store.owner_sets(sets);
    // Seed a valid registry without deriving: the stored format permits this
    // depth, but the open must still apply the plan's stricter refusal.
    let mut layouts = Layouts::new();
    let mut binary = texts::Binary::new(&sets, &mut layouts).expect("deep text parses");
    super::super::rows::register(&store, &mut layouts, &mut binary).expect("deep layout fingerprints");
    super::super::rows::operations(store.limits.store.key, &mut layouts).expect("registry fits");
    *store.registry = layouts;
    persist_registry(&mut store);
    let last = store.stores.store.last();
    assert_eq!(
        super::super::open(&mut store, false),
        Err(crate::OpenRefusal::Layout {
            family: Box::from("fixture-deep"),
            value_type: 0,
            id: 1,
            values: Some(0),
            why: crate::Unplanned::Underived(refusal),
        })
    );
    assert_eq!(counting::get().plans, 1, "uncertain depth falls back to derive");
    assert_eq!(store.stores.store.last(), last, "refusal writes nothing");
}
