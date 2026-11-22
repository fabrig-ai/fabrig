//! Fixed-rule client values (skein/docs/design/data/codec/README.md, 6).
use skein_codegen::FixedValue;
use std::sync::OnceLock;

pub(super) fn client() -> &'static [FixedValue] {
    static SORTED: OnceLock<Vec<FixedValue>> = OnceLock::new();
    SORTED.get_or_init(|| {
        let mut values = skein_codegen::values(jig_client_protocol_wire::CLIENT_SCHEMA).fixed.clone();
        values.sort_by(|left, right| left.name.cmp(&right.name));
        values
    })
}

pub(super) fn fixed_value(name: &str) -> &'static [u8] {
    &client().iter().find(|value| value.name == name).expect("named client value").bytes
}
