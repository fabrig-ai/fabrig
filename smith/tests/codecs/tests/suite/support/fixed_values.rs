//! Fixed-rule values and record measurements (skein/docs/design/data/codec/golden.md, section 1).

pub fn all(family: &str, version: u16) -> Vec<skein_codegen::FixedValue> {
    let text = match (family, version) {
        ("charter", smith_charter::wire::VERSION) => smith_charter::wire::TEXT,
        ("transcript", smith_transcript::wire::VERSION) => smith_transcript::wire::TEXT,
        ("channel", smith_channel::wire::VERSION) => smith_channel::wire::TEXT,
        _ => panic!("no embedded schema for {family} v{version}"),
    };
    skein_codegen::values(text).fixed.clone()
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
