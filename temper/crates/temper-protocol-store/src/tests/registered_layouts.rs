//! Registered application layouts stay readable from their latest kept text (jig protocol/store/layouts.md, 8).
#![expect(clippy::disallowed_types, clippy::disallowed_methods, reason = "ordinary Rust schema contract tests")]
#[test]
fn real_registered_families_derive_from_latest_kept_to_current() {
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
