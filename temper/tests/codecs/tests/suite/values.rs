//! Fixed-rule seeds built from the families' texts (skein's data/codec/README.md, 6).
use skein_codegen::Golden;
use std::sync::OnceLock;

pub(super) fn client() -> &'static [Golden] {
    static VALUES: OnceLock<Vec<Golden>> = OnceLock::new();
    VALUES.get_or_init(|| {
        built(include_str!("../../../../crates/temper-client-protocol-wire/schema/client-wire-v1.schema"))
    })
}

pub(super) fn worker() -> &'static [Golden] {
    static VALUES: OnceLock<Vec<Golden>> = OnceLock::new();
    VALUES.get_or_init(|| {
        built(include_str!("../../../../crates/temper-protocol-worker-wire/schema/worker-wire-v1.schema"))
    })
}

pub(super) fn store() -> &'static [Golden] {
    static VALUES: OnceLock<Vec<Golden>> = OnceLock::new();
    VALUES.get_or_init(|| built(include_str!("../../../../crates/temper-protocol-store/schema/store-v25.schema")))
}

fn built(text: &str) -> Vec<Golden> {
    skein_codegen::goldens(&skein_codegen::parse(text).expect("family schema"))
}

pub(super) fn named<'a>(values: &'a [Golden], name: &str) -> &'a [u8] {
    &values.iter().find(|value| value.name == name).expect("named fixed-rule value").bytes
}
