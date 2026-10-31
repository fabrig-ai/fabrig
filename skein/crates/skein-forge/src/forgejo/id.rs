//! Integer and object id spelling.
use alloc::boxed::Box;
use skein_lib::Writer;
use skein_lib::hex::{self, Case};
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
pub fn read(text: &[u8]) -> Option<ObjectId> {
    let format = match text.len() {
        40 => ObjectFormat::Sha1,
        64 => ObjectFormat::Sha256,
        _ => return None,
    };
    let mut bytes = [0; 32];
    for (pair, output) in text.chunks_exact(2).zip(bytes.iter_mut()) {
        let high = hex::read_lower(*pair.first()?)?;
        let low = hex::read_lower(*pair.get(1)?)?;
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
        out.put(&hex::pair(*byte, Case::Lower)).expect("two digits per byte");
    }
    out.finish()
}
