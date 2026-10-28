//! The secret and its digest have lowercase hexadecimal digits (protocol/setup.md, 8.2).
use alloc::boxed::Box;
use skein_lib::Writer;

pub(crate) fn digits(bytes: &[u8; 32]) -> Box<[u8]> {
    let mut writer = Writer::new(64);
    for byte in bytes {
        for nibble in [byte >> 4_u8, byte & 15] {
            let digit = b"0123456789abcdef".get(usize::from(nibble)).expect("a nibble is below 16");
            writer.put(&[*digit]).expect("32 bytes have 64 hexadecimal digits");
        }
    }
    writer.finish()
}
