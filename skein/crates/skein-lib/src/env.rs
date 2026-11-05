use crate::{Duration, Time, Wall};

/// What a step reads besides its own state: the times of the current
/// iteration and its layer's limits. A step receives it behind a shared
/// borrow, so it is read-only (skein/docs/design/lib.md, 9).
#[derive(Debug)]
pub struct Env<L> {
    /// Read once per iteration by the shell (or the simulator), and the same
    /// for every step in the iteration.
    pub now: Time,
    /// The wall-clock time, read with `now`: for things about the world, never
    /// for a deadline (programming-model.md, section 9).
    pub wall: Wall,
    /// The layer's configured limits.
    pub limits: L,
}

impl<L> Env<L> {
    /// Whether a wall-clock expiry has come, on this step's clocks.
    #[must_use]
    pub fn wall_due(&self, expiry: Wall) -> bool {
        let remaining = expiry.as_nanos().saturating_sub(self.wall.as_nanos());
        self.now.saturating_add(Duration::from_nanos(remaining)) <= self.now
    }

    /// The same clocks with another layer's limits.
    #[must_use]
    pub fn with<M>(&self, limits: M) -> Env<M> {
        Env { now: self.now, wall: self.wall, limits }
    }
}
