//! A type absent from a kept text has no seed layout (protocol/store/layouts.md, 5.1).
use super::*;

fn added(variant: bool) -> PatchSet {
    let (kept, current, root, selected) = if variant {
        (
            format!("{KEPT}\nenum Value {{ old }}\nversioned record Broad {{ value: Value }}\n"),
            format!("{CURRENT}\nenum Value {{ old, added }}\nversioned record Broad {{ value: Value }}\n"),
            "Broad",
            Some("added"),
        )
    } else {
        (KEPT.to_owned(), format!("{CURRENT}\nversioned record Added {{ n: u8 }}\n"), "Added", None)
    };
    let kept: &'static str = Box::leak(kept.into_boxed_str());
    let family = KeptFamily {
        current: Box::leak(current.into_boxed_str()),
        texts: Box::leak(Box::new([kept])),
        keys: Box::leak(Box::new([FAMILY.keys[0], FAMILY.keys[1], Keys { from: b"e", to: b"f", root }])),
        types: Box::leak(Box::new([
            FAMILY.types[0],
            FAMILY.types[1],
            ValueType { number: 2, root, variant: selected, keys: &[KeyRange { from: b"e", to: b"f" }] },
        ])),
        ..FAMILY
    };
    PatchSet { families: Box::leak(Box::new([family])), ..SET }
}

#[test]
fn a_type_absent_from_the_kept_text_registers_only_its_current_layout_at_the_family_version() {
    for variant in [false, true] {
        let set = added(variant);
        let mut disk = Disk::new(&base(false, true));
        let mut store = store(&disk, set);
        let events = disk.open(&mut store);
        let Some(Event::Opened { pending: true, .. }) = events.first() else { panic!("{events:?}") };
        let schema = skein_codec::schema::parse(set.families[0].current.as_bytes()).unwrap();
        let layout = row(&mut store, 2, schema.version()).expect("registered at the family's version");
        let ty = set.families[0].types[2];
        assert_eq!(layout.root(), ty.root.as_bytes());
        assert_eq!(layout.variant().as_deref(), ty.variant.map(str::as_bytes));
        assert_eq!(*layout.text(), hash(set.families[0].current));
        assert_eq!(row(&mut store, 2, schema.version() + 1), None, "no seed layout to advance past");
        switch(&mut disk, &mut store);
        let mut reopened = self::store(&disk, set);
        let events = disk.open(&mut reopened);
        let Some(Event::Opened { pending: false, .. }) = events.first() else { panic!("{events:?}") };
    }
}

#[test]
fn a_type_absent_from_the_kept_text_refuses_existing_rows_in_its_range() {
    for variant in [false, true] {
        let mut disk = Disk::new(&base(false, false));
        let old = KeptFamily { types: &[], current: KEPT, ..FAMILY };
        let old = PatchSet { families: Box::leak(Box::new([old])), ..SET };
        let mut store = store(&disk, old);
        assert_opened(&disk.open(&mut store), 1);
        raw(
            &mut disk,
            &mut store,
            vec![skein_kv::Op::Put { key: b"\x02e/0".as_slice().into(), value: Box::from([0, 2, 1]) }],
        );
        let mut disk = Disk::new(&disk.items());
        let before = snapshot(&mut disk);
        let mut store = self::store(&disk, added(variant));
        let events = disk.open(&mut store);
        let [Event::OpenRefused { why: OpenRefusal::Set { owner: Owner::Application, patch: 0 }, .. }] =
            events.as_ref()
        else {
            panic!("{events:?}")
        };
        assert_eq!(snapshot(&mut disk), before);
    }
}
