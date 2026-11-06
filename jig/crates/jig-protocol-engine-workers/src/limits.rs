//! Checked channel capacity (protocol/channel.md, section 11).
use jig_protocol_worker_wire::{
    self as wire,
    sizes::{self, Sizes},
};
use skein_channel::{Role, Schema};
use skein_lib::Duration;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub sizes: Sizes,
    pub names: u32,
    pub workers: u32,
    pub opening: u32,
    pub chunk: u32,
    pub handshake: Duration,
    pub hello: Duration,
    pub ping: Duration,
    pub silence: Duration,
    pub stall: Duration,
}
pub(crate) fn channel(limits: &Limits) -> Option<skein_channel::Limits> {
    let codec = sizes::limits(&limits.sizes)?;
    let credential = u32::try_from(wire::Credential::encoded_bound(&codec)?).ok()?;
    let cap = sizes::engine_cap(&limits.sizes)?;
    Some(skein_channel::Limits {
        chunk: limits.chunk,
        credential,
        skip: limits.chunk,
        output_bytes: cap.bytes,
        output_frames: cap.frames,
        kinds: u32::try_from(wire::KINDS.len()).ok()?,
    })
}
pub(crate) fn schema(limits: &Limits) -> Option<Schema> {
    wire::schema(&sizes::limits(&limits.sizes)?).ok()
}
/// Whether these limits are representable: every edge bound they size
/// is finite and every capacity they derive fits (programming-model.md, 6.3).
#[must_use]
pub fn valid(limits: &Limits) -> bool {
    if limits.handshake == Duration::ZERO
        || limits.hello == Duration::ZERO
        || limits.stall == Duration::ZERO
        || limits.ping == Duration::ZERO
        || limits.silence < limits.ping.saturating_mul(3)
    {
        return false;
    }
    let Some(schema) = schema(limits) else {
        return false;
    };
    let Some(channel) = channel(limits) else {
        return false;
    };
    let Some(codec) = sizes::limits(&limits.sizes) else {
        return false;
    };
    limits.workers.checked_add(limits.opening).is_some()
        && skein_channel::valid(&schema, Role::Responder, &channel)
        && wire::v9::decoded_bound(&codec).is_some()
}
