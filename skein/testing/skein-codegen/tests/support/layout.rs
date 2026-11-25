//! Split the emitted family fingerprint from the rest of the module (skein/docs/design/data/codec/README.md, 4).

pub(super) fn split_layout(text: &str) -> ([u8; 32], (&str, &str)) {
    let (before, layout) = text.split_once("pub const LAYOUT: [u8; 32] = [").expect("generated layout");
    let (bytes, after) = layout.split_once("];\n").expect("end of layout initializer");
    let digest = bytes
        .trim()
        .split_terminator(',')
        .map(|part| {
            let hex = part.trim().strip_prefix("0x").expect("hexadecimal layout byte");
            u8::from_str_radix(hex, 16).expect("layout byte value")
        })
        .collect::<Vec<_>>()
        .try_into()
        .expect("32 layout bytes");
    (digest, (before, after))
}
