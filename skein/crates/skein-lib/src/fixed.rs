//! A process's fixed start and deadline-driven clock (skein/docs/design/runtime/lib.md, 8).

use alloc::boxed::Box;

use crate::{CryptoRng, Secret, Time, Wall, bytes};

/// The wall time and seed of a fixed run (skein/docs/design/runtime/lib.md, 8).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FixedStart {
    pub wall: Wall,
    pub seed: u64,
}

impl FixedStart {
    /// The monotonic origin of a fixed run, one hour into its clock.
    pub const NOW: Time = Time::from_nanos(3_600_000_000_000);

    /// Reads canonical unsigned decimal `SECONDS,SEED`, with no leading zeros.
    /// Refuses seconds whose nanoseconds do not fit in `Wall`.
    #[must_use]
    pub fn read(text: &[u8]) -> Option<FixedStart> {
        let comma = bytes::find(text, b",")?;
        let seconds = number(text.get(..comma)?)?;
        let seed = number(text.get(comma.checked_add(1)?..)?)?;
        let wall = Wall::from_nanos(seconds.checked_mul(1_000_000_000)?);
        Some(FixedStart { wall, seed })
    }

    /// Fills from the beginning of the seed's `ChaCha20` stream on every call.
    /// These reproducible bytes are for a fixed run, not secret randomness.
    pub fn draw(&self, stream: u64, out: &mut [u8]) {
        let mut key = [0_u8; 32];
        for (byte, seed_byte) in key.iter_mut().zip(self.seed.to_le_bytes()) {
            *byte = seed_byte;
        }
        let key = Secret::new(Box::new(key));
        let mut rng = CryptoRng::new(&key, stream).expect("a fixed start's key holds 32 bytes");
        rng.fill(out);
    }
}

fn number(text: &[u8]) -> Option<u64> {
    if text.len() > 1 && text.first() == Some(&b'0') {
        return None;
    }
    bytes::decimal(text)
}

/// A clock that moves only to requested deadlines once native time reaches them
/// (skein/docs/design/runtime/lib.md, 8).
#[derive(Clone, Copy, Debug)]
pub struct FixedClock {
    wall: Wall,
    at: Time,
    deadline: Option<(Time, Time)>,
}

impl FixedClock {
    /// Starts frozen at `FixedStart::NOW`; native elapsed time alone never moves it.
    #[must_use]
    pub fn new(start: &FixedStart, _native: Time) -> FixedClock {
        FixedClock { wall: start.wall, at: FixedStart::NOW, deadline: None }
    }

    /// Translates a deadline to native time, keeping the first instant for a
    /// repeated request. `None` forgets the request without moving the clock.
    #[must_use]
    pub fn until(&mut self, deadline: Option<Time>, native: Time) -> Option<Time> {
        let Some(deadline) = deadline else {
            self.deadline = None;
            return None;
        };
        if let Some((kept, instant)) = self.deadline
            && kept == deadline
        {
            return Some(instant);
        }
        let instant = native.saturating_add(deadline.saturating_since(self.at));
        self.deadline = Some((deadline, instant));
        Some(instant)
    }

    /// Reads the fixed times, consuming a due request and never moving backward.
    #[must_use]
    pub fn read(&mut self, native: Time) -> (Time, Wall) {
        if let Some((deadline, instant)) = self.deadline
            && native >= instant
        {
            self.at = self.at.max(deadline);
            self.deadline = None;
        }
        let elapsed = self.at.saturating_since(FixedStart::NOW);
        let wall = Wall::from_nanos(self.wall.as_nanos().saturating_add(elapsed.as_nanos()));
        (self.at, wall)
    }
}
