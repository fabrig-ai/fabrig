//! Byte-for-byte history, with registry, job and startup differences named.
use super::{Maps, Wall};
use jig_domain_core::WaitKind;
/// Every key is compared; exceptions are enumerated and checked, never removed silently.
pub(super) fn history(before: &Maps, after: &Maps, upgrade_started: Wall) {
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
            let mut expected = before.deployment().unwrap_or_else(|problem| panic!("old header: {problem}"));
            let found = after.deployment().unwrap_or_else(|problem| panic!("new header: {problem}"));
            assert!(found.commits > expected.commits, "upgrade commits carry on");
            expected.commits = found.commits;
            allowed.push("registry, job, restore and wires commit position");
            if expected.wait_floors.is_none() {
                // Only jig's own kinds acquire floors at startup (waits.md, 7).
                for kind in [
                    WaitKind::Party,
                    WaitKind::Question,
                    WaitKind::Dependency,
                    WaitKind::Resource,
                    WaitKind::Account,
                    WaitKind::Host,
                    WaitKind::Delegate,
                    WaitKind::News,
                ] {
                    let at = found.wait_floor(kind).unwrap_or_else(|| panic!("missing startup floor for {kind:?}"));
                    assert!(at >= upgrade_started, "{kind:?} floor {at:?} precedes upgrade start {upgrade_started:?}");
                    assert!(expected.set_wait_floor(kind, at));
                }
                allowed.push("wait floors set at the upgrade's first start");
            }
            assert_eq!(found, expected, "all other header fields, including any previously kept floors, preserved");
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
                        let current = u32::from(jig_protocol_worker_wire::VERSION);
                        if wire.version != current {
                            assert_eq!(wire.version, 21, "the kept image's worker wire");
                            wire.version = current;
                        }
                        seen = true;
                    } else if wire.name.as_ref() == b"smith-transcript" && wire.version != 8 {
                        assert_eq!(wire.version, 7, "the kept image's transcript");
                        wire.version = 8;
                    }
                }
                assert!(seen, "the worker wire is kept");
                assert_eq!(found, expected, "only the worker wire and transcript move at startup");
                allowed.push("startup's current worker wire and transcript");
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
