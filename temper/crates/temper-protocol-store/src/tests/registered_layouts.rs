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
fn the_current_layout_has_no_whole_change_value_type() {
    let family = crate::OWNER_SET.families.iter().find(|family| family.name == "temper-store").unwrap();
    let current = skein_codec::schema::parse(family.current.as_bytes()).unwrap();
    assert!(current.root(b"Row", Some(b"change")).is_none());
    assert!(!family.types.iter().any(|ty| ty.number == 12));
}
