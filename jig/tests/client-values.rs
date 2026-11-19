//! Fixed-rule client values (skein/docs/design/data/codec/README.md, 6).
use skein_codegen::Golden;
use std::sync::OnceLock;

pub(super) fn client() -> &'static [Golden] {
    static SORTED: OnceLock<Vec<Golden>> = OnceLock::new();
    SORTED.get_or_init(|| {
        let mut values = skein_codegen::values(jig_client_protocol_wire::CLIENT_SCHEMA).goldens.clone();
        values.sort_by(|left, right| left.name.cmp(&right.name));
        values
    })
}

pub(super) fn value(name: &str) -> &'static [u8] {
    &client().iter().find(|value| value.name == name).expect("named client value").bytes
}
<<<<<<< HEAD
=======

fn text() -> String {
    include_str!("../crates/jig-client-protocol-wire/schema/client-v38.schema").to_owned()
        + include_str!("../crates/jig-client-protocol-wire/schema/client-v38-part1.schema")
}
>>>>>>> refs/hand5/g9120
