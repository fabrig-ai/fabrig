use alloc::boxed::Box;
use skein_lib::bytes;

use crate::encode::{Encoder, Item};

pub use hearth_test_fixtures::digits::hex;

pub fn text(value: &[u8]) -> Item {
    Item::Bytes(bytes::copy_of(value))
}

pub fn put(encoder: &mut Encoder, items: &[Item]) {
    for item in items {
        encoder.item(item);
    }
}

pub fn encode(items: &[Item]) -> Box<[u8]> {
    let mut measure = Encoder::measuring();
    put(&mut measure, items);
    let mut writer = Encoder::writing(measure.measured());
    put(&mut writer, items);
    writer.written()
}

pub fn digest(text: &str) -> crate::encode::Digest {
    let bytes = hex(text);
    crate::encode::Digest(bytes.as_ref().try_into().expect("a digest has thirty-two bytes"))
}
