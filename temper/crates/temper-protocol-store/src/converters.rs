//! Scalar meanings owned by the store (skein/docs/design/data/codec/maps.md, 5).

#[must_use]
pub fn signed_to_record(value: &i32) -> u32 {
    u32::from_be_bytes(value.to_be_bytes())
}

#[must_use]
pub fn signed_to_domain(value: u32) -> i32 {
    i32::from_be_bytes(value.to_be_bytes())
}
