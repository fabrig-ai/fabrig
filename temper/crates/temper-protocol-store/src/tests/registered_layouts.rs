//! Registered application layouts stay readable from their latest kept text (jig protocol/store/layouts.md, 8).
#![expect(clippy::disallowed_types, clippy::disallowed_methods, reason = "ordinary Rust schema contract tests")]
#[test]
fn questions_from_a_layout_without_waiting_words_default_to_none() {
    let family = crate::PATCH_SET.families.iter().find(|family| family.name == "temper-store").unwrap();
    let old = skein_codec::schema::parse(family.texts.last().unwrap().as_bytes()).unwrap();
    let new = skein_codec::schema::parse(family.current.as_bytes()).unwrap();
    let plan = skein_codec::layout::Plan::derive(
        &old,
        old.root(b"Questions", None).unwrap(),
        &[],
        &new,
        new.root(b"Questions", None).unwrap(),
        &[],
    )
    .expect("the registry derives the added field's default at open");
    let mut work = skein_codec::layout::Workspace::new();
    assert!(work.fit(&plan));
    let mut out = skein_lib::Buffer::with_capacity(1024);
    // Up, four questions taken, none open: the pre-words Questions record.
    plan.convert(&[2, 0, 0, 0, 4, 0, 0, 0, 0], &mut out, &mut work).unwrap();
    let decoded =
        crate::v25::Questions::decode(&crate::v25::CEILINGS, &mut skein_lib::Reader::new(out.as_slice())).unwrap();
    assert_eq!(decoded.route(), &crate::v25::Route::Up);
    assert_eq!(decoded.taken(), 4);
    assert!(decoded.waiting().is_empty());
}

#[test]
fn real_registered_families_derive_from_latest_kept_to_current() {
    let family = crate::PATCH_SET.families.iter().find(|family| family.name == "temper-store").unwrap();
    let current = skein_codec::schema::parse(family.current.as_bytes()).unwrap();
    assert!(current.root(b"Row", Some(b"change")).is_none());
    assert!(!family.types.iter().any(|ty| ty.number == 12));
    let sets = [
        jig_protocol_store::SKEIN_PATCH_SET,
        jig_protocol_store::SMITH_PATCH_SET,
        jig_protocol_accounts::JIG_PATCH_SET,
        crate::PATCH_SET,
    ];
    let families: Vec<_> = sets.iter().flat_map(|set| set.families).filter(|family| !family.texts.is_empty()).collect();
    let old: Vec<_> = families
        .iter()
        .map(|family| skein_codec::schema::parse(family.texts.last().unwrap().as_bytes()).unwrap())
        .collect();
    let new: Vec<_> =
        families.iter().map(|family| skein_codec::schema::parse(family.current.as_bytes()).unwrap()).collect();
    let old_held: Vec<_> = old.iter().collect();
    let new_held: Vec<_> = new.iter().collect();
    let mut types = 0_usize;
    let mut added = Vec::new();
    for ((family, old), new) in families.iter().zip(&old).zip(&new) {
        if !crate::PATCH_SET.families.iter().any(|own| own.name == family.name) {
            continue;
        }
        for ty in family.types {
            let variant = ty.variant.map(str::as_bytes);
            let old_root = old.root(ty.root.as_bytes(), variant);
            let new_root = new.root(ty.root.as_bytes(), variant).unwrap();
            let Some(old_root) = old_root else {
                added.push(ty.number);
                continue;
            };
            let plan = skein_codec::layout::Plan::derive(old, old_root, &old_held, new, new_root, &new_held);
            assert!(plan.is_ok(), "{} type {}: {plan:?}", family.name, ty.number);
            types += 1;
        }
    }
    assert_ne!(types, 0);
    assert_eq!(added, [47, 48]);
}
