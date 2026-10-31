//! Integer and object id spelling.
use alloc::boxed::Box;
use skein_lib::{Writer, hex};
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
#[must_use]
pub fn unsigned(text: &[u8]) -> Option<u64> {
    if text.is_empty() {
        return None;
    }
    let mut number = 0u64;
    for byte in text {
        if !byte.is_ascii_digit() {
            return None;
        }
        number = number.checked_mul(10)?.checked_add(u64::from(byte.checked_sub(b'0')?))?;
    }
    Some(number)
}
#[must_use]
pub fn read(hex: &[u8]) -> Option<ObjectId> {
    let format = match hex.len() {
        40 => ObjectFormat::Sha1,
        64 => ObjectFormat::Sha256,
        _ => return None,
    };
    let mut bytes = [0; 32];
    for (pair, output) in hex.chunks_exact(2).zip(bytes.iter_mut()) {
        let high = hex::lower_value(*pair.first()?)?;
        let low = hex::lower_value(*pair.get(1)?)?;
        *output = high.checked_mul(16)?.checked_add(low)?;
    }
    Some(ObjectId { format, bytes })
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
            out.put(&[hex::lower(value)]).expect("two digits per byte");
        }
    }
    out.finish()
}
