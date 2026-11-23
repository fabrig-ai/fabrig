//! Fixed-rule values and record measurements (skein/docs/design/data/codec/golden.md, section 1).

pub fn all(family: &str, version: u16) -> Vec<skein_codegen::FixedValue> {
    let text = match (family, version) {
        ("charter", 5) => smith_charter::SCHEMA,
        ("transcript", 7) => smith_transcript::SCHEMA,
        ("channel", 13) => smith_channel::CHANNEL_SCHEMA,
        ("channel", 9) => include_str!("../../../../../crates/smith-channel/schema/kept/smith-channel-v9.schema"),
        ("channel", 7) => include_str!("../../../../../crates/smith-channel/schema/kept/smith-channel-v7.schema"),
        _ => panic!("no embedded schema for {family} v{version}"),
    };
    if (family == "channel" && version == 13) || family == "charter" || family == "transcript" {
        skein_codegen::values(text).fixed.clone()
    } else {
        skein_codegen::fixed_values(&skein_codegen::parse(text).expect("valid embedded schema"))
    }
}

pub fn fixed_value(family: &str, version: u16, name: &str) -> Vec<u8> {
    all(family, version).into_iter().find(|value| value.name == name).expect("named fixed-rule value").bytes
}

/// One test's fixed-rule values, indexed once for all of its measurements.
pub struct FixedValues {
    family: &'static str,
    values: std::collections::BTreeMap<String, Vec<u8>>,
}

impl FixedValues {
    pub fn new(family: &'static str, version: u16) -> Self {
        let values =
            all(family, version).into_iter().map(|fixed_value| (fixed_value.name, fixed_value.bytes)).collect();
        Self { family, values }
    }

    pub fn assert_measured(&self, record: &str, measure: impl Fn(&[u8]) -> u32) {
        for shape in ["smallest", "full"] {
            let name = format!("record_{record}_{shape}.bin");
            let bytes = self.values.get(&name).expect("named fixed-rule value");
            assert_eq!(
                u32::try_from(bytes.len()).expect("value length fits"),
                measure(bytes),
                "{}/{name}",
                self.family
            );
        }
    }
}
