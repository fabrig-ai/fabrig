//! Opening limits for deterministic channel tests (smith/docs/design/protocol/channel.md, section 2).
use crate::Limits;

/// Small framing queues and empty run tables for opening and refusal tests.
#[must_use]
pub fn small() -> Limits {
    Limits {
        bodies: smith_channel::CEILINGS,
        start_bytes: u32::try_from(
            smith_channel::Start::encoded_bound(&smith_channel::CEILINGS).expect("fixture Start bound"),
        )
        .expect("fixture frame"),
        charter: smith_charter::CEILINGS,
        transcript: smith_transcript::CEILINGS,
        resume_turns: 64,
        search_hits: 256,
        endpoints: 0,
        calls: 0,
        turns: 0,
        fact_reserve_frames: 1,
        fact_reserve_bytes: 128,
        grants: 0,
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
