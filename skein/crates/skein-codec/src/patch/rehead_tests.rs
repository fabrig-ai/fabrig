//! Bytes kept and heads alone, from both sides (data/codec/patches.md, 3.1 and 8).
#![expect(clippy::disallowed_macros, reason = "inline schema and patch fixtures use formatting")]
use super::{Capacity, Patcher, PlanIx, Step};
use crate::schema::DeclIx;
use crate::value::Value;

const ROOM: Capacity = Capacity { versions: 2, patches: 2, text: 4096, bytes: 256, nodes: 64 };

fn planned(old: &str, new: &str, lines: &str) -> (Patcher, PlanIx, DeclIx) {
    let mut patcher = Patcher::new(ROOM);
    patcher.keep(format!("family f 1\n{old}").as_bytes()).expect("old text");
    patcher.keep(format!("family f 2\n{new}").as_bytes()).expect("new text");
    let plan = patcher
        .plan(format!("patch 1\nowner o 1\nfamily f 1 to 2\n{lines}").as_bytes())
        .expect("patch")
        .expect("family");
    let root = patcher.schema(1).expect("old").find(b"R").expect("root");
    (patcher, plan, root)
}

fn check(old: &str, new: &str, lines: &str, encoded: &[u8], keeps: bool, identity: bool) {
    let (mut patcher, plan, root) = planned(old, new, lines);
    let plan = if identity { patcher.identity(1).expect("identity") } else { plan };
    // The inline bytes encode each small record in its old text's field order.
    Value::with_room(ROOM.nodes)
        .decode(patcher.schema(1).expect("old"), root, encoded.into())
        .expect("fixture is an old value");
    let mut bytes = skein_lib::bytes::copy_of(encoded);
    let allocations = skein_heap::Span::start();
    let answer = patcher.rehead(plan, root, &mut bytes);
    assert_eq!(allocations.made(), 0, "rehead allocates nothing");
    assert_eq!(answer, keeps, "{old} -> {new}: {lines}");
    if keeps {
        patcher.start(plan, root, encoded.into()).expect("start");
        assert_eq!(patcher.next(), Ok(Step::Done));
        assert_eq!(bytes.as_ref(), patcher.finish().expect("finish"));
    } else {
        assert_eq!(bytes.as_ref(), encoded, "false writes nothing");
    }
}

struct Case {
    old: &'static str,
    new: &'static str,
    lines: &'static str,
    encoded: &'static [u8],
    keeps: bool,
}
const LAYOUTS: &[Case] = &[
    Case {
        old: "enum K { a b } versioned record R { k: K }",
        new: "enum K { a b c } versioned record R { k: K }",
        lines: "",
        encoded: &[0, 1, 1],
        keeps: true,
    },
    Case { old: "enum R { a b }", new: "enum R { a b c }", lines: "", encoded: &[1], keeps: true },
    Case { old: "enum R { a b }", new: "enum R { b a }", lines: "order R b a\n", encoded: &[1], keeps: false },
    Case { old: "enum R { a b }", new: "enum R { c a b }", lines: "add R.c\n", encoded: &[1], keeps: false },
    Case {
        old: "record A { x: u8 } versioned record R { a: A }",
        new: "record A { x: u16 } versioned record R { a: A }",
        lines: "widen A.x u8 u16\n",
        encoded: &[0, 1, 9],
        keeps: false,
    },
    Case {
        old: "versioned record R { x: text 8 }",
        new: "versioned record R { x: text 16 }",
        lines: "",
        encoded: &[0, 1, 0, 0, 0, 1, b'a'],
        keeps: true,
    },
    Case {
        old: "record A { x: u8 } versioned record R { a: list 1 option A }",
        new: "record A { x: u8 } versioned record R { a: list 2 option A }",
        lines: "",
        encoded: &[0, 1, 0, 0, 0, 1, 1, 9],
        keeps: true,
    },
    Case {
        old: "record A { x: u8 } versioned record R { a: list 1 option A }",
        new: "record A { x: u16 } versioned record R { a: list 1 option A }",
        lines: "widen A.x u8 u16\n",
        encoded: &[0, 1, 0, 0, 0, 1, 1, 9],
        keeps: false,
    },
    Case {
        old: "enum K { a } versioned record R { k: K n: u8 }",
        new: "enum K { a } versioned record R { k: K n: u16 }",
        lines: "widen R.n u8 u16\n",
        encoded: &[0, 1, 0, 9],
        keeps: false,
    },
    Case {
        old: "enum K { a } versioned record R { k: K }",
        new: "enum K { a } versioned record R { k: option K }",
        lines: "wrap R.k K option K\n",
        encoded: &[0, 1, 0],
        keeps: false,
    },
    Case {
        old: "enum K { a } versioned record R { k: K n: u8 }",
        new: "enum K { a } versioned record R { n: u8 k: K }",
        lines: "order R n k\n",
        encoded: &[0, 1, 0, 9],
        keeps: false,
    },
    Case {
        old: "record R { a: u8 }",
        new: "record R { a: u8 b: bool }",
        lines: "add R.b bool true\n",
        encoded: &[9],
        keeps: false,
    },
    Case {
        old: "record R { a: u8 b: bool }",
        new: "record R { a: u8 }",
        lines: "drop R.b\n",
        encoded: &[9, 1],
        keeps: false,
    },
    Case {
        old: "record R { a: u8 }",
        new: "versioned record R { a: u8 }",
        lines: "versioned R\n",
        encoded: &[9],
        keeps: false,
    },
    Case {
        old: "versioned record R { a: u8 }",
        new: "record R { a: u8 }",
        lines: "unversioned R\n",
        encoded: &[0, 1, 9],
        keeps: false,
    },
    Case { old: "record R { a: u8 }", new: "record R { a: u8 }", lines: "", encoded: &[9], keeps: true },
    Case { old: "record R { a: u8 }", new: "record R { a: u8 }", lines: "written R\n", encoded: &[9], keeps: false },
    Case {
        old: "record A { a: u8 } record R { a: A }",
        new: "record A { a: u8 } record R { a: A }",
        lines: "written A.a\n",
        encoded: &[9],
        keeps: false,
    },
    Case {
        old: "enum K { a } versioned record R { k: K }",
        new: "enum K { a } versioned record R { k: K }",
        lines: "written K\n",
        encoded: &[0, 1, 0],
        keeps: false,
    },
    Case {
        old: "enum K { a } versioned record R { k: K }",
        new: "enum K { a } versioned record R { k: K }",
        lines: "written R.k\n",
        encoded: &[0, 1, 0],
        keeps: false,
    },
    Case {
        old: "record R { a: bytes 8 }",
        new: "record R { a: bytes 8 holds other A }",
        lines: "",
        encoded: &[0, 0, 0, 1, 9],
        keeps: false,
    },
    Case {
        old: "record R { a: secret 8 holds other A }",
        new: "record R { a: secret 8 }",
        lines: "",
        encoded: &[0, 0, 0, 1, 9],
        keeps: false,
    },
    Case {
        old: "record A { x: u8 } record R { a: bytes 16 pieces A }",
        new: "record A { x: u8 } record R { a: bytes 16 pieces A }",
        lines: "",
        encoded: &[0, 0, 0, 5, 0, 0, 0, 1, 9],
        keeps: true,
    },
    Case {
        old: "record A { x: u8 } record R { a: bytes 16 pieces A }",
        new: "record A { x: u16 } record R { a: bytes 16 pieces A }",
        lines: "widen A.x u8 u16\n",
        encoded: &[0, 0, 0, 5, 0, 0, 0, 1, 9],
        keeps: false,
    },
    Case {
        old: "record A {} record B { x: u8 } enum R { a: A }",
        new: "record A {} record B { x: u8 } enum R { a: B }",
        lines: "payload R.a A B\nwritten R\n",
        encoded: &[0],
        keeps: false,
    },
];

#[test]
fn rehead_judges_kept_kinds_and_each_layout_rule() {
    const OLD: &str =
        "record A { x: u8 } record B { x: u8 } enum K { a: A b: B } versioned record R { kind: K n: u32 }";
    const NEW: &str =
        "record A { x: u8 } record B { x: u8 y: bool } enum K { a: A b: B } versioned record R { kind: K n: u32 }";
    check(OLD, NEW, "add B.y bool true\n", &[0, 1, 0, 7, 0, 0, 0, 9], true, false);
    check(OLD, NEW, "add B.y bool true\n", &[0, 1, 1, 7, 0, 0, 0, 9], false, false);
    for case in LAYOUTS {
        check(case.old, case.new, case.lines, case.encoded, case.keeps, false);
    }
    for identity in [true, false] {
        const NESTED: &str = "versioned record A { x: u8 } versioned record R { a: A }";
        const HELD: &str = "record A { a: bytes 8 holds other A } versioned record R { a: A }";
        const SECRET: &str = "versioned record R { a: secret 8 holds other A }";
        check(NESTED, NESTED, "", &[0, 1, 0, 1, 9], identity, identity);
        check(HELD, HELD, "", &[0, 1, 0, 0, 0, 1, 9], false, identity);
        check(SECRET, SECRET, "", &[0, 1, 0, 0, 0, 1, 9], false, identity);
    }
}

#[test]
fn rehead_reads_only_the_head_and_one_valid_old_tag() {
    const TEXT: &str = "enum K { a b } versioned record R { k: K n: u32 }";
    const NEW: &str = "enum K { a b c } versioned record R { k: K n: u32 }";
    let (patcher, plan, root) = planned(TEXT, NEW, "");
    for original in [&[0, 9, 0][..], &[1, 1, 0][..], &[0, 1, 2][..], &[0][..], &[0, 1][..]] {
        let mut bytes = skein_lib::bytes::copy_of(original);
        assert!(!patcher.rehead(plan, root, &mut bytes));
        assert_eq!(bytes.as_ref(), original);
    }
    // Truncated body and trailing garbage are not decoded, even when a tag is needed.
    for original in [&[0, 1, 0][..], &[0, 1, 1, 0, 0, 0, 9, 99][..]] {
        let mut bytes = skein_lib::bytes::copy_of(original);
        assert!(patcher.rehead(plan, root, &mut bytes));
        assert_eq!(bytes.get(..2), Some(&[0, 2][..]));
        assert_eq!(bytes.get(2..), original.get(2..));
    }
    let enumeration = patcher.schema(1).expect("old").find(b"K").expect("enum root");
    assert!(!patcher.rehead(plan, enumeration, &mut []));
    let mut wide = Patcher::new(ROOM);
    wide.keep(b"family f 255\nversioned record R {}").expect("old");
    wide.keep(b"family f 256\nversioned record R {}").expect("new");
    let plan = wide.plan(b"patch 1\nowner o 1\nfamily f 255 to 256\n").expect("patch").expect("family");
    let mut bytes = [0, 255];
    assert!(wide.rehead(plan, DeclIx::new(0), &mut bytes));
    assert_eq!(bytes, [1, 0], "head is big-endian across both bytes");
}

#[test]
fn rehead_leaves_a_stopped_run_alone() {
    const TEXT: &str = "record A { x: u8 } versioned record R { a: u8 }";
    let (mut patcher, plan, root) = planned(TEXT, TEXT, "written A.x\n");
    let a = patcher.schema(1).expect("old").find(b"A").expect("A");
    patcher.start(plan, a, Box::from([9_u8])).expect("start A");
    let Step::Written { at, .. } = patcher.next().expect("site") else { panic!("written A.x") };
    let mut other = [0, 1, 7];
    assert!(patcher.rehead(plan, root, &mut other));
    assert_eq!(other, [0, 2, 7]);
    let (old, new) = patcher.parts();
    assert_eq!(old.uint(old.field(at, crate::schema::FieldIx::new(0)).expect("A.x")), Ok(9));
    new.put_uint(10).expect("answer original stop");
    assert_eq!(patcher.next(), Ok(Step::Done));
    assert_eq!(patcher.finish(), Ok(&[10][..]));
}
