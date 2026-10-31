//! Frozen binary record measurements (skein/docs/design/codec.md, section 6).

pub fn assert_measured(family: &str, version: u16, record: &str, measure: impl Fn(&[u8]) -> u32) {
    let root = skein_scratch::paths::tree().join(format!("crates/smith-{family}/golden/v{version}"));
    for shape in ["smallest", "full"] {
        let name = format!("record_{record}_{shape}.bin");
        let bytes = std::fs::read(root.join(&name)).expect("record golden");
        assert_eq!(u32::try_from(bytes.len()).expect("golden length fits"), measure(&bytes), "{family}/{name}");
    }
}
