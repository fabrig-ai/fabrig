#[must_use]
pub fn repeated(byte: u8, length: u32) -> Box<[u8]> {
    let mut bytes = skein_lib::bytes::zeroed(usize::try_from(length).unwrap());
    bytes.fill(byte);
    bytes
}
