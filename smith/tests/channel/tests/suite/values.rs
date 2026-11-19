//! Fixed-rule charter shapes for channel scenarios.

pub(crate) fn charter(shape: &str) -> Vec<u8> {
    let values = skein_codegen::values(smith_charter::SCHEMA);
    let name = format!("record_charter_{shape}.bin");
    values.goldens.clone().into_iter().find(|value| value.name == name).expect("charter shape").bytes
}
