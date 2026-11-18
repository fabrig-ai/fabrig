//! Registered layouts derive field defaults (jig protocol/store/layouts.md, 4 and 5).
#![expect(clippy::disallowed_methods, reason = "ordinary Rust schema contract tests")]

#[test]
fn a_registered_layout_defaults_an_added_list() {
    let old = skein_codec::schema::parse(b"family fixture 1\nrecord Item { taken: u32 }\n").unwrap();
    let new = skein_codec::schema::parse(b"family fixture 1\nrecord Item { taken: u32 waiting: list 16 u64 = [] }\n")
        .unwrap();
    let plan = skein_codec::layout::Plan::derive(
        &old,
        old.root(b"Item", None).unwrap(),
        &[],
        &new,
        new.root(b"Item", None).unwrap(),
        &[],
    )
    .expect("the registry derives the added field's default at open");
    let mut work = skein_codec::layout::Workspace::new();
    assert!(work.fit(&plan));
    let mut out = skein_lib::Buffer::with_capacity(1024);
    plan.convert(&4_u32.to_be_bytes(), &mut out, &mut work).unwrap();
    // The retained count, followed by an empty waiting list.
    assert_eq!(out.as_slice(), &[0, 0, 0, 4, 0, 0, 0, 0]);
}

#[test]
fn a_change_layout_without_source_or_kept_defaults_both_to_none() {
    let family = crate::OWNER_SET.families.iter().find(|family| family.name == "temper-store").unwrap();
    let mut before = family.current.to_owned();
    for field in ["source: option ChangeSource = none ", "kept: option fixed 32 = none "] {
        before = before.replacen(field, "", 1);
    }
    let old = skein_codec::schema::parse(before.as_bytes()).unwrap();
    let new = skein_codec::schema::parse(family.current.as_bytes()).unwrap();
    for (root, bytes) in [
        (b"ChangeCold".as_slice(), golden(family.current, "record_change_cold_smallest.bin")),
        (b"ChangeHot".as_slice(), golden(family.current, "record_change_hot_smallest.bin")),
    ] {
        let plan = skein_codec::layout::Plan::derive(
            &old,
            old.root(root, None).unwrap(),
            &[],
            &new,
            new.root(root, None).unwrap(),
            &[],
        )
        .unwrap();
        let mut work = skein_codec::layout::Workspace::new();
        assert!(work.fit(&plan));
        let mut out = skein_lib::Buffer::with_capacity(1024);
        // The added option's zero byte is absent from the earlier smallest record.
        plan.convert(&bytes[1..], &mut out, &mut work).unwrap();
        if root == b"ChangeCold" {
            let row =
                crate::v25::ChangeCold::decode(&crate::v25::CEILINGS, &mut skein_lib::Reader::new(out.as_slice()))
                    .unwrap();
            assert!(row.source().is_none());
        } else {
            let row = crate::v25::ChangeHot::decode(&crate::v25::CEILINGS, &mut skein_lib::Reader::new(out.as_slice()))
                .unwrap();
            assert!(row.kept().is_none());
        }
    }
}

#[test]
fn the_current_layout_has_no_whole_change_value_type() {
    let family = crate::OWNER_SET.families.iter().find(|family| family.name == "temper-store").unwrap();
    let current = skein_codec::schema::parse(family.current.as_bytes()).unwrap();
    assert!(current.root(b"Row", Some(b"change")).is_none());
    assert!(!family.types.iter().any(|ty| ty.number == 12));
}

#[test]
fn a_change_layout_without_regate_defaults_it_to_none() {
    let family = crate::OWNER_SET.families.iter().find(|family| family.name == "temper-store").unwrap();
    let before = family.current.replacen("regate: option Regate = none ", "", 1);
    let old = skein_codec::schema::parse(before.as_bytes()).unwrap();
    let new = skein_codec::schema::parse(family.current.as_bytes()).unwrap();
    let plan = skein_codec::layout::Plan::derive(
        &old,
        old.root(b"ChangeHot", None).unwrap(),
        &[],
        &new,
        new.root(b"ChangeHot", None).unwrap(),
        &[],
    )
    .expect("an added regate has a default for stored change rows");
    let mut work = skein_codec::layout::Workspace::new();
    assert!(work.fit(&plan));
    let mut out = skein_lib::Buffer::with_capacity(1024);
    let current = golden(family.current, "record_change_hot_smallest.bin");
    // The earlier row lacks the first option's absent byte.
    plan.convert(&current[1..], &mut out, &mut work).unwrap();
    let row =
        crate::v25::ChangeHot::decode(&crate::v25::CEILINGS, &mut skein_lib::Reader::new(out.as_slice())).unwrap();
    assert!(row.regate().is_none());
}

fn golden(text: &str, name: &str) -> Box<[u8]> {
    skein_codegen::goldens(&skein_codegen::parse(text).expect("current store schema"))
        .into_iter()
        .find(|value| value.name == name)
        .expect("named fixed-rule Change row")
        .bytes
        .into_boxed_slice()
}
