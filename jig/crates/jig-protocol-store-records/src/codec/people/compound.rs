//! Bounded compound task operations in the people v3 family.

pub(super) fn encode_grace(grace: Option<skein_lib::Duration>) -> Option<u64> {
    Some(grace?.as_nanos())
}
pub(super) fn decode_grace(grace: Option<u64>) -> Option<skein_lib::Duration> {
    Some(skein_lib::Duration::from_nanos(grace?))
}
