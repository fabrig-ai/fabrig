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

#[test]
fn former_names_confirm_renames_alongside_layout_changes() {
    let old = schema::parse(b"family f 1 record Job { owner: u8 } enum State { suspended next }").expect("old");
    let new = schema::parse(b"family f 2 record Task was Job { assignee was owner: u32 } enum State { paused was suspended was halted next }").expect("new");
    let result = diff(&old, &new, &[]).expect("former names");
    assert_eq!(result.proposed, []);
    assert_eq!(
        result.to_string(),
        "family f 1 to 2\nnone rename Job Task\nnone rename Task.owner assignee\nderived widen Task.assignee u8 u32\nnone rename State.suspended paused\n"
    );
}

#[test]
fn former_names_do_not_displace_current_names_or_explicit_renames() {
    for (old, new, renames, expected) in [
        ("enum R { before }", "record R { a was before: u8 }", vec![], "written reshape R\n"),
        ("record R { before: u8 }", "enum R { a was before }", vec![], "written reshape R\n"),
        ("record Before {} record R {}", "record R was Before {}", vec![], "none drop Before\n"),
        ("record R { before: u8 a: u8 }", "record R { a was before: u8 }", vec![], "derived drop R.before\n"),
        ("enum E { before a }", "enum E { a was before }", vec![], "written drop E.before\n"),
        (
            "record Before {} record Other {}",
            "record R was Before {}",
            vec![declaration("Other", "R")],
            "none rename Other R\nnone drop Before\n",
        ),
        (
            "record R { before: u8 other: u8 }",
            "record R { a was before: u8 }",
            vec![part("R", "other", "a")],
            "none rename R.other a\nderived drop R.before\n",
        ),
    ] {
        let old = schema::parse(format!("family f 1 {old}").as_bytes()).expect("old");
        let new = schema::parse(format!("family f 1 {new}").as_bytes()).expect("new");
        let result = diff(&old, &new, &renames).expect("former names");
        assert_eq!(result.to_string(), format!("family f 1 to 1\n{expected}"));
        assert_eq!(result.proposed, []);
    }
}

#[test]
fn added_fields_keep_default_spelling_and_implicit_none() {
    let old = schema::parse(b"family f 1 record R {}").expect("old");
    let new = schema::parse(b"family f 2 record R { a: u32 = 0007 o: option u8 s: option u8 = some 3 t: text 8 = \"hi #\" l: list 4 u8 = [] missing: bool }").expect("new");
    let result = diff(&old, &new, &[]).expect("defaults");
    assert_eq!(
        result.to_string(),
        "family f 1 to 2\nderived add R.a u32 0007\nderived add R.o option u8 none\nderived add R.s option u8 some 3\nderived add R.t text 8 \"hi #\"\nderived add R.l list 4 u8 []\nderived add R.missing bool ?\n"
    );
    assert!(
        matches!(&result.changes.first().expect("added field").op, super::Op::AddField { value: super::Fill::Default(spelling), .. } if spelling == "0007")
    );
}
