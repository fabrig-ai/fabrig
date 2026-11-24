//! Level-by-level comparison cases (skein/docs/design/data/codec/diff.md, 3).

#![expect(clippy::disallowed_macros, reason = "table rows assemble schema fixtures")]

use super::{Comparison, compare};
use crate::schema::{DeclIx, FieldIx};
use skein_lib::List;

#[test]
fn comparison_walks_wrappers_and_clauses_level_by_level() {
    let rows = [
        ("equal", "option list 2 u8", "option list 2 u8", Comparison::Equal),
        ("integer", "option list 2 u8", "option list 3 u16", Comparison::Widen { integer: true }),
        ("bound", "list 2 text 8", "list 3 text 9", Comparison::Widen { integer: false }),
        ("inner wrap", "list 2 text 8", "list 3 option text 9", Comparison::Wrap),
        ("option outermost", "option u8", "option option u8", Comparison::Wrap),
        ("option removed", "option u8", "u8", Comparison::Retype),
        ("bound narrowed", "list 2 u8", "list 1 u8", Comparison::Retype),
        ("duration", "u64", "duration", Comparison::Retype),
        ("fixed", "fixed 2", "fixed 3", Comparison::Retype),
        ("bytes text", "bytes 2", "text 2", Comparison::Retype),
        ("text bytes", "text 2", "bytes 2", Comparison::Widen { integer: false }),
        ("text secret", "text 2", "secret 2", Comparison::Widen { integer: false }),
        ("secret bytes", "secret 2", "bytes 2", Comparison::Widen { integer: false }),
        ("add holds", "bytes 2", "bytes 2 holds other R", Comparison::Widen { integer: false }),
        ("drop holds", "bytes 2 holds other R", "bytes 2", Comparison::Widen { integer: false }),
        ("holds family", "bytes 2 holds other R", "bytes 2 holds third R", Comparison::Retype),
        ("holds name", "bytes 2 holds other R", "bytes 2 holds other S", Comparison::Retype),
        ("pieces same", "bytes 2 pieces R", "bytes 2 pieces R", Comparison::Equal),
        ("pieces different", "bytes 2 pieces R", "bytes 2 pieces S", Comparison::Retype),
        ("secret add holds", "secret 2", "secret 2 holds other R", Comparison::Widen { integer: false }),
        ("secret drop holds", "secret 2 holds other R", "secret 2", Comparison::Widen { integer: false }),
        ("secret holds family", "secret 2 holds other R", "secret 2 holds third R", Comparison::Retype),
        ("secret holds name", "secret 2 holds other R", "secret 2 holds other S", Comparison::Retype),
        ("secret pieces same", "secret 2 pieces R", "secret 2 pieces R", Comparison::Equal),
        ("secret pieces different", "secret 2 pieces R", "secret 2 pieces S", Comparison::Retype),
        ("named same", "R", "R", Comparison::Equal),
        ("named different", "R", "S", Comparison::Retype),
    ];
    for (name, a, b, expected) in rows {
        let old =
            crate::schema::parse(format!("family f 1 record R {{}} record S {{}} record D {{ f: {a} }}").as_bytes())
                .expect(name);
        let new =
            crate::schema::parse(format!("family f 2 record R {{}} record S {{}} record D {{ f: {b} }}").as_bytes())
                .expect(name);
        // R, S and D are in the same order in both texts.
        let mut matching = List::with_capacity(old.declarations());
        for at in 0..old.declarations() {
            matching.push(Some(DeclIx::new(at))).expect("one match per declaration");
        }
        assert_eq!(
            compare(&matching, &old, &new, old.field(FieldIx::new(0)).ty, new.field(FieldIx::new(0)).ty),
            expected,
            "{name}"
        );
    }
}
