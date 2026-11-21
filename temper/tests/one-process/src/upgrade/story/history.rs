//! Byte-for-byte history, with registry, job and startup differences named.
use super::{CoreRecord, Maps, Record, fixtures, stories};
/// Every key is compared; exceptions are enumerated and checked, never removed silently.
pub(super) fn history(before: &Maps, after: &Maps) {
    assert_eq!(after.active, before.active + 1, "one whole copied generation");
    assert_eq!(before.secrets, after.secrets, "every secret record preserved");
    let registry = crate::upgrade::registry_upgrade(before, after);
    let mut registry_seen = std::collections::BTreeSet::new();
    let keys: std::collections::BTreeSet<_> = before.store.keys().chain(after.store.keys()).collect();
    let mut allowed = Vec::new();
    for key in keys {
        let old = before.store.get(key);
        let new = after.store.get(key);
        let named = jig_protocol_store::keys::decode(key);
        if old == new {
            continue;
        }
        if registry.contains(key) {
            registry_seen.insert(key.clone());
            continue;
        }
        if let Some(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Deployment)) = named {
            let decode = |value: &Vec<u8>| {
                jig_protocol_store::codec::decode(
                    named.as_ref().expect("key"),
                    value,
                    &stories::playground::codec(),
                    fixtures::store_names(),
                )
                .expect("header")
            };
            let Record::Core(CoreRecord::Deployment(mut expected)) = decode(old.expect("header kept")) else {
                panic!("header")
            };
            let Record::Core(CoreRecord::Deployment(found)) = decode(new.expect("header kept")) else {
                panic!("header")
            };
            assert!(found.commits > expected.commits, "upgrade commits carry on");
            expected.commits = found.commits;
            assert_eq!(found, expected, "all identity high-water marks preserved");
            allowed.push("registry, job, restore and wires commit position");
        } else {
            let own = jig_protocol_store::keys::decode_own(key);
            if own == Some(jig_protocol_store::keys::Own::Wires) {
                use jig_protocol_store::component_v3::{CEILINGS, Record as _, Wires};
                let mut expected =
                    Wires::decode(&CEILINGS, &mut skein_lib::Reader::new(old.expect("wires kept"))).expect("old wires");
                let found = Wires::decode(&CEILINGS, &mut skein_lib::Reader::new(new.expect("wires kept")))
                    .expect("current wires");
                let mut seen = false;
                for wire in expected.wires.iter_mut() {
                    if wire.name.as_ref() == b"jig-worker-wire" {
                        assert_eq!(wire.version, 18, "the kept image's worker wire");
                        wire.version = u32::from(jig_protocol_worker_wire::VERSION);
                        seen = true;
                    }
                }
                assert!(seen, "the worker wire is kept");
                assert_eq!(found, expected, "only the worker wire moves at startup");
                allowed.push("startup's current worker wire");
            } else if matches!(own, Some(jig_protocol_store::keys::Own::Opening(_))) {
                assert!(old.is_some() && new.is_none(), "the start's wires clear the previous worker opening");
                allowed.push("wires cleared the previous worker opening");
            } else {
                panic!(
                    "unlisted history difference key {key:02x?}: {named:?}, {own:?}\nold {old:02x?}\nnew {new:02x?}"
                );
            }
        }
    }
    assert_eq!(registry_seen, registry, "only the named current layouts, their texts and jobs' done rows change");
    eprintln!("upgrade history allowed: {allowed:?}");
}
