//! Fixed-rule charter shapes for channel scenarios.

pub(crate) fn charter(shape: &str) -> Vec<u8> {
    let schema = skein_codegen::parse(include_str!("../../../../crates/smith-charter/schema/charter-v5.schema"))
        .expect("charter schema");
    let name = format!("record_charter_{shape}.bin");
    skein_codegen::goldens(&schema).into_iter().find(|value| value.name == name).expect("charter shape").bytes
}
