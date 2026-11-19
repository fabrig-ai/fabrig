use crate::bounds::{Bounds, Node, declaration, family};

const OVERFLOW: &[Node] = &[
    Node::Scalar(u64::MAX),
    Node::Option { item: 0 },
    Node::List { item: 0, limit: 0, size: 0 },
    Node::Record { prefix: 1, fields: &[0] },
    Node::Enum { variants: &[0] },
    Node::List { item: 6, limit: 0, size: 1 },
    Node::Scalar(0),
];

const HEAP_OVERFLOW: &[Node] = &[
    Node::Scalar(0),
    Node::Boxed { item: 0, size: 16 },
    Node::List { item: 1, limit: 0, size: 0 },
    Node::List { item: 2, limit: 0, size: 0 },
];

#[test]
fn tables_compute_wire_heap_maxima_and_refuse_overflow() {
    const TABLE: &[Node] = &[
        Node::Scalar(2),
        Node::Bytes { limit: 0 },
        Node::Option { item: 1 },
        Node::List { limit: 1, item: 2, size: 16 },
        Node::Record { prefix: 2, fields: &[0, 3] },
        Node::Boxed { item: 4, size: 32 },
        Node::Enum { variants: &[0, 5] },
        Node::Record { prefix: 0, fields: &[4, 4] },
    ];
    let limits = [3, 2];
    // text: (7,3); option: (8,3); list: (20,38); versioned row: (24,38).
    assert_eq!(declaration(TABLE, &limits, &limits, 4), Some(Bounds { encoded: 24, decoded: 38 }));
    assert_eq!(declaration(TABLE, &limits, &limits, 6), Some(Bounds { encoded: 25, decoded: 70 }));
    assert_eq!(declaration(TABLE, &limits, &limits, 7), Some(Bounds { encoded: 48, decoded: 76 }));
    assert_eq!(family(TABLE, &limits, &limits, &[4, 6, 7]), Some(Bounds { encoded: 48, decoded: 76 }));
    assert_eq!(declaration(TABLE, &[1, 1], &limits, 4), Some(Bounds { encoded: 14, decoded: 17 }));
    assert_eq!(declaration(TABLE, &[4, 2], &limits, 0), None, "even an unrelated ceiling is checked");
    assert_eq!(declaration(TABLE, &[3], &limits, 0), None);
    assert_eq!(declaration(TABLE, &limits, &limits, 100), None);
    for at in [1, 2, 3, 4, 5] {
        assert_eq!(declaration(OVERFLOW, &[2], &[2], at), None);
    }
    assert_eq!(declaration(HEAP_OVERFLOW, &[u32::MAX], &[u32::MAX], 3), None);
    assert_eq!(family(&[], &[], &[], &[]), Some(Bounds { encoded: 0, decoded: 0 }));
}
