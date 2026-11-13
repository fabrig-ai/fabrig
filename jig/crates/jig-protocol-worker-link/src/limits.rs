//! Bounds supplied by the layer (protocol/channel/README.md, 10).
use crate::Unusable;
use jig_protocol_worker_wire::{
    self as wire,
    sizes::{self, Sizes},
};
use skein_channel::{self as channel, Role};
use skein_lib::Duration;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub sizes: Sizes,
    pub chunk: u32,
    /// The most bytes the stream below grants at once: io's
    /// `largest_room` (protocol/channel/README.md, 8 and 10).
    pub room: u32,
    pub handshake: Duration,
    pub ping: Duration,
    pub silence: Duration,
    pub stall: Duration,
    pub backoff: Duration,
    pub ceiling: Duration,
    pub proved: Duration,
    pub grace: Duration,
    pub busy: Duration,
}
pub fn channel_limits(limits: &Limits) -> Result<channel::Limits, Unusable> {
    let bodies = sizes::limits(&limits.sizes).ok_or(Unusable::Limits)?;
    let cap = sizes::worker_cap(&limits.sizes).ok_or(Unusable::Limits)?;
    let mut skip = 0;
    for rule in wire::KINDS {
        let largest = sizes::largest_body(&limits.sizes, rule.body).ok_or(Unusable::Limits)?;
        skip = skip.max(largest);
    }
    let credential = wire::Credential::encoded_bound(&bodies).ok_or(Unusable::Limits)?;
    let Ok(credential) = u32::try_from(credential) else { return Err(Unusable::Limits) };
    let Ok(kinds) = u32::try_from(wire::KINDS.len()) else { return Err(Unusable::Limits) };
    Ok(channel::Limits {
        chunk: limits.chunk,
        credential,
        skip,
        output_bytes: cap.cap.bytes,
        output_frames: cap.cap.frames,
        kinds,
    })
}
/// Whether these limits are representable: every edge bound they size
/// is finite and every capacity they derive fits (programming-model.md, 6.3).
#[must_use]
pub fn valid(limits: &Limits) -> bool {
    if limits.room < 270
        || limits.ping == Duration::ZERO
        || limits.silence < limits.ping.saturating_mul(3)
        || limits.stall == Duration::ZERO
        || limits.grace == Duration::ZERO
        || limits.busy == Duration::ZERO
        || limits.backoff == Duration::ZERO
        || limits.ceiling < limits.backoff
        || limits.handshake == Duration::ZERO
        || limits.proved == Duration::ZERO
    {
        return false;
    }
    let Some(bodies) = sizes::limits(&limits.sizes) else {
        return false;
    };
    let Ok(schema) = wire::schema(&bodies) else {
        return false;
    };
    let Ok(channel) = channel_limits(limits) else {
        return false;
    };
    // Open's fixed body is 14 bytes, followed by the credential; the header is 8.
    let Some(open) = channel.credential.checked_add(22) else {
        return false;
    };
    if open > limits.room {
        return false;
    }
    for rule in wire::KINDS {
        if rule.direction == channel::Direction::FromInitiator {
            let Some(body) = sizes::largest_body(&limits.sizes, rule.body) else {
                return false;
            };
            let Some(frame) = body.checked_add(8) else {
                return false;
            };
            if frame > limits.room {
                return false;
            }
        }
    }
    channel::valid(&schema, Role::Initiator, &channel)
        && limits.sizes.slots.checked_mul(limits.sizes.calls).is_some()
        && limits.sizes.slots.checked_mul(limits.sizes.turns).is_some()
        && limits.sizes.accounts.checked_add(limits.sizes.credentials).is_some()
        && u64::from(channel.skip).checked_add(8).is_some()
        && inventory_bound(&bodies).is_some()
}

fn inventory_bound(bodies: &wire::v12::Limits) -> Option<u64> {
    wire::Logins::encoded_bound(bodies)?.checked_add(8)
}
