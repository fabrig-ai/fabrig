//! Every registered jig value type remains readable from its latest kept text (protocol/store/layouts.md, 8).
#![expect(clippy::disallowed_types, clippy::disallowed_methods, reason = "ordinary Rust schema contract tests")]
#[test]
fn real_registered_families_derive_from_latest_kept_to_current() {
    let families: Vec<_> =
        jig_protocol_accounts::JIG_PATCH_SET.families.iter().filter(|family| !family.texts.is_empty()).collect();
    let mut old: Vec<_> = families
        .iter()
        .map(|family| skein_codec::schema::parse(family.texts.last().unwrap().as_bytes()).unwrap())
        .collect();
    let mut new: Vec<_> =
        families.iter().map(|family| skein_codec::schema::parse(family.current.as_bytes()).unwrap()).collect();
    for set in [crate::SKEIN_PATCH_SET, crate::SMITH_PATCH_SET] {
        for family in set.families {
            if let Some(text) = family.texts.last() {
                old.push(skein_codec::schema::parse(text.as_bytes()).unwrap());
                new.push(skein_codec::schema::parse(family.current.as_bytes()).unwrap());
            }
        }
    }
    let old_held: Vec<_> = old.iter().collect();
    let new_held: Vec<_> = new.iter().collect();
    let mut types = 0_usize;
    for ((family, old), new) in families.iter().zip(&old).zip(&new) {
        for ty in family.types {
            let variant = ty.variant.map(str::as_bytes);
            let old_root = old.root(ty.root.as_bytes(), variant).unwrap();
            let new_root = new.root(ty.root.as_bytes(), variant).unwrap();
            let plan = skein_codec::layout::Plan::derive(old, old_root, &old_held, new, new_root, &new_held);
            assert!(plan.is_ok(), "{} type {}: {plan:?}", family.name, ty.number);
            types += 1;
        }
    }
    assert_ne!(types, 0);
}
