//! Fixed-rule seeds built from the families' texts (skein's data/codec/README.md, 6).
use skein_codegen::Golden;
pub(super) fn client() -> &'static [Golden] {
    &skein_codegen::values(temper_client_protocol_wire::CLIENT_SCHEMA).goldens
}

pub(super) fn worker() -> &'static [Golden] {
    &skein_codegen::values(temper_protocol_worker_wire::SCHEMA).goldens
}

pub(super) fn store() -> &'static [Golden] {
    &skein_codegen::values(temper_protocol_store::OWNER_SET.families.first().expect("store family").current).goldens
}

pub(super) fn named<'a>(values: &'a [Golden], name: &str) -> &'a [u8] {
    &values.iter().find(|value| value.name == name).expect("named fixed-rule value").bytes
}
