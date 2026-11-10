//! Same-name rename pairings (data/codec/diff.md, 1).

use super::{DiffError, Rename, diff};
use skein_codec::schema;

fn declaration(old: &str, new: &str) -> Rename {
    Rename::Declaration { old: old.into(), new: new.into() }
}
fn part(decl: &str, old: &str, new: &str) -> Rename {
    Rename::Part { decl: decl.into(), old: old.into(), new: new.into() }
}

#[test]
fn same_name_renames_pair_without_a_line() {
    for (text, rename) in [
        ("record D {}", declaration("D", "D")),
        ("record D { a: u8 }", part("D", "a", "a")),
        ("enum D { a }", part("D", "a", "a")),
    ] {
        let schema = schema::parse(format!("family f 1 {text}").as_bytes()).expect("fixture schema");
        assert_eq!(diff(&schema, &schema, &[rename]).expect(text).to_string(), "family f 1 to 1\n", "{text}");
    }
}

#[test]
fn same_name_renames_reserve_both_names() {
    for (kept, current, renames) in [
        ("record D {}", "record D {} record E {}", [declaration("D", "D"), declaration("D", "E")]),
        ("record D {} record E {}", "record D {}", [declaration("D", "D"), declaration("E", "D")]),
        ("record D { a: u8 }", "record D { a: u8 b: u8 }", [part("D", "a", "a"), part("D", "a", "b")]),
        ("enum D { a b }", "enum D { a }", [part("D", "a", "a"), part("D", "b", "a")]),
    ] {
        let kept = schema::parse(format!("family f 1 {kept}").as_bytes()).expect("kept schema");
        let current = schema::parse(format!("family f 1 {current}").as_bytes()).expect("current schema");
        assert_eq!(diff(&kept, &current, &renames), Err(DiffError::Rename(1)), "{renames:?}");
    }
}
