//! Fixed-rule values and record measurements (skein/docs/design/data/codec/README.md, section 6).

pub fn all(family: &str, version: u16) -> Vec<skein_codegen::Golden> {
    let text = match (family, version) {
        ("charter", 5) => include_str!("../../../../../crates/smith-charter/schema/charter-v5.schema"),
        ("transcript", 7) => include_str!("../../../../../crates/smith-transcript/schema/transcript-v7.schema"),
        ("channel", 11) => include_str!("../../../../../crates/smith-channel/schema/channel-v11.schema"),
        ("channel", 9) => include_str!("../../../../../crates/smith-channel/schema/kept/smith-channel-v9.schema"),
        ("channel", 7) => include_str!("../../../../../crates/smith-channel/schema/kept/smith-channel-v7.schema"),
        _ => panic!("no embedded schema for {family} v{version}"),
    };
    skein_codegen::goldens(&skein_codegen::parse(text).expect("valid embedded schema"))
}

pub fn bytes(family: &str, version: u16, name: &str) -> Vec<u8> {
    all(family, version).into_iter().find(|value| value.name == name).expect("named fixed-rule value").bytes
}

/// One test's fixed-rule values, indexed once for all of its measurements.
pub struct Goldens {
    family: &'static str,
    values: std::collections::BTreeMap<String, Vec<u8>>,
}

impl Goldens {
    pub fn new(family: &'static str, version: u16) -> Self {
        let values = all(family, version).into_iter().map(|golden| (golden.name, golden.bytes)).collect();
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
