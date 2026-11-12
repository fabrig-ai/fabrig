//! Small channel bounds for deterministic worlds (protocol/channel.md, section 2).
//! Shared fixtures (smith/docs/design/testing.md, section 7).

#![forbid(unsafe_code)]

use smith_host_protocol::Limits;

/// Codec ceilings with a small transport queue and eight outstanding calls.
#[must_use]
pub fn small() -> Limits {
    Limits {
        bodies: smith_channel::CEILINGS,
        charter: smith_charter::CEILINGS,
        transcript: smith_transcript::CEILINGS,
        endpoints: 1,
        calls: 8,
        start_bytes: u32::try_from(
            smith_channel::v7::Start::encoded_bound(&smith_channel::CEILINGS).expect("Start ceiling"),
        )
        .expect("Start frame range"),
        channel: skein_channel::Limits {
            chunk: 8,
            credential: 0,
            skip: 8,
            output_bytes: u32::MAX,
            output_frames: 2,
            kinds: 20,
        },
    }
}
