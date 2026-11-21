//! Scalar meanings owned by the store (skein/docs/design/data/codec/maps.md, 5).
use skein_codec::Reason;

pub fn signed_to_record(value: &i32) -> Result<u32, Reason> {
    Ok(u32::from_be_bytes(value.to_be_bytes()))
}

pub fn signed_to_domain(value: u32) -> Result<i32, Reason> {
    Ok(i32::from_be_bytes(value.to_be_bytes()))
}
