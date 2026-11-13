//! Byte-for-byte history, with the mark and component differences named.
use super::{CoreRecord, Maps, Record, fixtures, stories};
/// Every key is compared; exceptions are enumerated and checked, never removed silently.
pub(super) fn history(before: &Maps, after: &Maps) {
    assert_eq!(after.active, before.active + 1, "one whole copied generation");
    assert_eq!(before.secrets, after.secrets, "every secret record preserved");
    let keys: std::collections::BTreeSet<_> = before.store.keys().chain(after.store.keys()).collect();
    let mut allowed = Vec::new();
    for key in keys {
        let old = before.store.get(key);
        let new = after.store.get(key);
        let named = jig_protocol_store::keys::decode(key);
        if old == new && !matches!(named, Some(jig_domain_core::Key::Core(jig_domain_core::CoreKey::RunProof(_)))) {
            continue;
        }
        match named {
            Some(jig_domain_core::Key::Core(jig_domain_core::CoreKey::RunProof(_))) => {
                let decode = |value: &Vec<u8>| {
                    jig_protocol_store::codec::decode(
                        named.as_ref().expect("key"),
                        value,
                        &stories::playground::codec(),
                        fixtures::store_names(),
                    )
                    .expect("proof")
                };
                let Record::Core(CoreRecord::RunProof(mut expected)) = decode(old.expect("proof kept")) else {
                    panic!("proof")
                };
                expected.crossed = true;
                if let Some(window) = expected.window.as_mut() {
                    window.closed = true;
                }
                assert_eq!(
                    decode(new.expect("proof kept")),
                    Record::Core(CoreRecord::RunProof(expected)),
                    "only windows closed and claims crossed"
                );
                allowed.push("fresh-windows: closed window and crossed claim");
            }
            Some(jig_domain_core::Key::Core(jig_domain_core::CoreKey::Deployment)) => {
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
                allowed.push("patch, restore and wires commit position");
            }
            _ => {
                let own = jig_protocol_store::keys::decode_own(key);
                if matches!(own, Some(jig_protocol_store::keys::Own::Format)) {
                    let mut expected = before.format();
                    expected.levels.application = expected.levels.application.map(|level| level + 1);
                    assert_eq!(after.format(), expected, "only the fixture owner level changes");
                    allowed.push("fixture owner level");
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
    }
    assert!(allowed.contains(&"fresh-windows: closed window and crossed claim"), "a live window exercised the mark");
    eprintln!("upgrade history allowed: {allowed:?}");
}
