//! Fixed-rule seeds built from the families' texts (skein's data/codec/README.md, 6).
use skein_codegen::FixedValue;
pub(super) fn client() -> &'static [FixedValue] {
    &skein_codegen::values(temper_client_protocol_wire::CLIENT_SCHEMA).fixed
}

pub(super) fn worker() -> &'static [FixedValue] {
    &skein_codegen::values(temper_protocol_worker_wire::SCHEMA).fixed
}

pub(super) fn store() -> &'static [FixedValue] {
    &skein_codegen::values(temper_protocol_store::OWNER_SET.families.first().expect("store family").current).fixed
}

pub(super) fn fixed_value<'a>(values: &'a [FixedValue], name: &str) -> &'a [u8] {
    &values.iter().find(|value| value.name == name).expect("named fixed-rule value").bytes
}
