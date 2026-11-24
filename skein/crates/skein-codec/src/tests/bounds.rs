use crate::bounds::{Bounds, Declaration, Node, declaration, family};
use core::mem::offset_of;

#[repr(C)]
struct Limits {
    text: u32,
    items: u32,
}

const CEILINGS: &[u32] = &[3, 2];
const TABLE: &[Declaration] = &[
    Declaration {
        name: "Choice",
        nodes: &[
            Node::Scalar(2),
            Node::Named("Row"),
            Node::Boxed { item: 1, size: 32 },
            Node::Enum { variants: &[0, 2] },
        ],
    },
    Declaration {
        name: "Pair",
        nodes: &[Node::Named("Row"), Node::Named("Row"), Node::Record { prefix: 0, fields: &[0, 1] }],
    },
    Declaration {
        name: "Row",
        nodes: &[
            Node::Scalar(2),
            Node::Bytes { limit: offset_of!(Limits, text) },
            Node::Option { item: 1 },
            Node::List { limit: offset_of!(Limits, items), item: 2, size: 16 },
            Node::Record { prefix: 2, fields: &[0, 3] },
        ],
    },
    Declaration { name: "Scalar", nodes: &[Node::Scalar(2)] },
];

#[test]
fn tables_compute_wire_heap_maxima_and_refuse_overflow() {
    const OVERFLOW: &[Declaration] = &[
        Declaration { name: "Enum", nodes: &[Node::Scalar(u64::MAX), Node::Enum { variants: &[0] }] },
        Declaration { name: "Forward", nodes: &[Node::List { item: 1, limit: 0, size: 1 }, Node::Scalar(0)] },
        Declaration { name: "List", nodes: &[Node::Scalar(u64::MAX), Node::List { item: 0, limit: 0, size: 0 }] },
        Declaration { name: "Option", nodes: &[Node::Scalar(u64::MAX), Node::Option { item: 0 }] },
        Declaration { name: "Record", nodes: &[Node::Scalar(u64::MAX), Node::Record { prefix: 1, fields: &[0] }] },
    ];
    const HEAP_OVERFLOW: &[Declaration] = &[Declaration {
        name: "Heap",
        nodes: &[
            Node::Scalar(0),
            Node::Boxed { item: 0, size: 16 },
            Node::List { item: 1, limit: 0, size: 0 },
            Node::List { item: 2, limit: 0, size: 0 },
        ],
    }];

    // text: (7,3); option: (8,3); list: (20,38); versioned row: (24,38).
    for (name, limits, expected) in [
        ("Row", CEILINGS, Bounds { encoded: 24, decoded: 38 }),
        ("Choice", CEILINGS, Bounds { encoded: 25, decoded: 70 }),
        ("Pair", CEILINGS, Bounds { encoded: 48, decoded: 76 }),
        ("Row", &[1, 1], Bounds { encoded: 14, decoded: 17 }),
    ] {
        assert_eq!(declaration(TABLE, limits, CEILINGS, name), Some(expected));
    }
    assert_eq!(family(TABLE, CEILINGS, CEILINGS), Some(Bounds { encoded: 48, decoded: 76 }));
    assert_eq!(family(&[], &[], &[]), Some(Bounds { encoded: 0, decoded: 0 }));

    for overflow in OVERFLOW {
        assert_eq!(declaration(OVERFLOW, &[2], &[2], overflow.name), None, "{}", overflow.name);
    }
    assert_eq!(declaration(HEAP_OVERFLOW, &[u32::MAX], &[u32::MAX], "Heap"), None);
}

#[test]
fn both_bounds_refuse_limits_past_even_unrelated_ceilings() {
    const TEXT_PAST_CEILING: &[u32] = &[4, 2];
    for limits in [TEXT_PAST_CEILING, &[3]] {
        assert_eq!(declaration(TABLE, limits, CEILINGS, "Scalar"), None);
        assert_eq!(family(TABLE, limits, CEILINGS), None);
    }
}

#[test]
fn a_missing_root_or_named_child_refuses_bounds() {
    const INCOMPLETE: &[Declaration] = &[Declaration { name: "Parent", nodes: &[Node::Named("Missing")] }];
    for name in ["Missing", "Parent"] {
        assert_eq!(declaration(INCOMPLETE, &[], &[], name), None);
    }
}

#[test]
fn a_cycle_of_named_children_refuses_bounds() {
    const CYCLE: &[Declaration] = &[
        Declaration { name: "First", nodes: &[Node::Named("Second")] },
        Declaration { name: "Second", nodes: &[Node::Named("First")] },
    ];
    assert_eq!(declaration(CYCLE, &[], &[], "First"), None);
    assert_eq!(family(CYCLE, &[], &[]), None);
}

#[test]
fn family_takes_the_two_maxima_across_its_declarations() {
    const SHAPES: &[Declaration] = &[
        Declaration { name: "Fixed", nodes: &[Node::Scalar(8)] },
        Declaration { name: "Text", nodes: &[Node::Bytes { limit: offset_of!(Limits, text) }] },
    ];
    // Fixed wins on encoded bytes (8 vs 7), Text on heap bytes (3 vs 0).
    assert_eq!(family(SHAPES, CEILINGS, CEILINGS), Some(Bounds { encoded: 8, decoded: 3 }));
}
