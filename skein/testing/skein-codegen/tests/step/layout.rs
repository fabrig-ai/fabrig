//! A generated module names the layout of its family (data/codec/README.md, 4).

fn emitted_layout(text: &str) -> [u8; 32] {
    let schema = skein_codegen::parse(text).expect("valid schema");
    let output = skein_codegen::generate(&schema).records;
    let (_, layout) = output.split_once("pub const LAYOUT: [u8; 32] = [").expect("generated layout");
    let (bytes, _) = layout.split_once("];").expect("layout initializer");
    bytes
        .split(',')
        .filter_map(|part| part.trim().strip_prefix("0x"))
        .map(|hex| u8::from_str_radix(hex, 16).expect("hexadecimal byte"))
        .collect::<Vec<_>>()
        .try_into()
        .expect("32 layout bytes")
}

#[test]
fn layout_names_the_family_fingerprint_and_changes_with_a_field_type() {
    let before = "family sample 1\nrecord A { x: u32 }\n";
    let after = "family sample 1\nrecord A { x: u64 }\n";
    let layout = emitted_layout(before);
    assert_eq!(
        layout,
        skein_codec::schema::parse(before.as_bytes()).expect("valid schema").fingerprints().family().digest()
    );
    assert_ne!(layout, emitted_layout(after));
}
