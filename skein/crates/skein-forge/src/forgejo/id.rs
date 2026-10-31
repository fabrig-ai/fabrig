//! Integer and object id spelling.
use alloc::boxed::Box;
use skein_lib::{Writer, bytes};
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ObjectFormat {
    Sha1,
    Sha256,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ObjectId {
    pub format: ObjectFormat,
    pub bytes: [u8; 32],
}
/// `1*DIGIT` as a `u64`: lib's [`bytes::decimal`], under the name the forge
/// readers and their dependants call.
#[must_use]
pub fn unsigned(text: &[u8]) -> Option<u64> {
    bytes::decimal(text)
}
#[must_use]
pub fn read(hex: &[u8]) -> Option<ObjectId> {
    let format = match hex.len() {
        40 => ObjectFormat::Sha1,
        64 => ObjectFormat::Sha256,
        _ => return None,
    };
    // Object ids are spelled in lower case only.
    for byte in hex {
        if byte.is_ascii_uppercase() {
            return None;
        }
    }
    let mut out = [0; 32];
    for (pair, output) in hex.chunks_exact(2).zip(out.iter_mut()) {
        let high = bytes::hex_value(*pair.first()?)?;
        let low = bytes::hex_value(*pair.get(1)?)?;
        *output = high.checked_mul(16)?.checked_add(low)?;
    }
    Some(ObjectId { format, bytes: out })
}
#[must_use]
pub fn write(id: &ObjectId) -> Box<[u8]> {
    let size: usize = match id.format {
        ObjectFormat::Sha1 => 20,
        ObjectFormat::Sha256 => 32,
    };
    let mut out = Writer::new(size.saturating_mul(2));
    for byte in id.bytes.iter().take(size) {
        for value in [byte / 16, byte % 16] {
            out.put(&[bytes::hex_digit(value)]).expect("two digits per byte");
        }
    }
    out.finish()
}
