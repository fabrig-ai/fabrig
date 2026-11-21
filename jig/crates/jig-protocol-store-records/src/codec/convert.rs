//! Owner scalar conversions (skein's data/codec/maps.md, 5).

/// Encoding preserves a share's wire value; restoration checks its meaning.
pub fn share_to_record(value: &u16) -> Result<u16, skein_codec::Reason> {
    Ok(*value)
}

pub fn share_to_domain(value: u16) -> Result<u16, skein_codec::Reason> {
    if value > jig_domain_accounts::SHARE_WHOLE {
        return Err(skein_codec::Reason::Bound);
    }
    Ok(value)
}
