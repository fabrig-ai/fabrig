//! Independent streams and named parts of a world's seed.
//!
//! The derivation is fixed by `skein/docs/design/testing/sweeps.md`, section 2;
//! comparable runs use it as `temper/docs/design/draft/failure-worlds.md`,
//! section 3.5 describes. Drawing one stream or part never advances another.

use skein_lib::Rng;

/// The independent sources of a world's workload, faults, actions and order.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Stream {
    Workload,
    Faults,
    Parties,
    Scheduling,
}

/// A world's seed, from which each stream and named part is derived afresh.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Streams {
    seed: u64,
}

impl Streams {
    #[must_use]
    pub const fn new(seed: u64) -> Streams {
        Streams { seed }
    }

    /// The seed itself, which a finding names.
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// The seed of one part of a stream, for a component or fake taking a `u64`.
    /// The empty part is the stream's own; other names belong to the world.
    #[must_use]
    pub fn seed_of(&self, stream: Stream, part: &str) -> u64 {
        mix(mix(self.seed ^ tag(stream)) ^ fnv(part))
    }

    /// A fresh generator for a stream's part, independent of all earlier draws.
    #[must_use]
    pub fn rng(&self, stream: Stream, part: &str) -> Rng {
        Rng::new(self.seed_of(stream, part))
    }
}

const fn tag(stream: Stream) -> u64 {
    match stream {
        Stream::Workload => u64::from_le_bytes(*b"workload"),
        Stream::Faults => u64::from_le_bytes(*b"faults\0\0"),
        Stream::Parties => u64::from_le_bytes(*b"parties\0"),
        Stream::Scheduling => u64::from_le_bytes(*b"schedule"),
    }
}

pub(crate) fn mix(seed: u64) -> u64 {
    Rng::new(seed).next_u64()
}

pub(crate) fn fnv(part: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in part.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}
