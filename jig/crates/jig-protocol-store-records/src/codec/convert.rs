//! Owner scalar conversions (skein's data/codec/maps.md, 5).

use alloc::boxed::Box;

/// Encoding preserves a share's wire value; restoration checks its meaning.
#[must_use]
pub fn share_to_record(value: &u16) -> u16 {
    *value
}

pub fn share_to_domain(value: u16) -> Result<u16, skein_codec::Reason> {
    if value > jig_domain_accounts::SHARE_WHOLE {
        return Err(skein_codec::Reason::Bound);
    }
    Ok(value)
}

#[must_use]
pub fn speed_name_to_record(value: &skein_lib::Name<32>) -> Box<[u8]> {
    skein_lib::bytes::copy_of(value.as_bytes())
}

pub fn speed_name_to_domain(value: Box<[u8]>) -> Result<skein_lib::Name<32>, skein_codec::Reason> {
    skein_lib::Name::from_checked(&value).ok_or(skein_codec::Reason::Bound)
}
