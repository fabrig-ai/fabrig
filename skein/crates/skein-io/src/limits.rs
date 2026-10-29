//! io's limits and capacity checks (programming-model.md, 6.3), and
//! the most each entry point emits (programming-model.md, section 2).

use skein_lib::Duration;

/// io's limits: the admission limits of its entities, and the caps on each
/// stream's bytes (programming-model.md, section 7).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    /// Entity slots shared by listeners, sockets, children and their pipes.
    /// A `Listen`, `Connect` or `Spawn` without enough slots is refused
    /// (`Error::Busy`), and a listener accepts only while one is free.
    pub sockets: u32,
    /// In-process names held by listeners (io.md, 3.5).
    pub names: u32,
    /// Refused `Listen`s, `Connect`s and `Spawn`s held until the next up pass tells
    /// them (io.md, 2). The loop hands io a request only while it can hold one
    /// more (`Io::takes`).
    pub refusals: u32,
    /// The cap on each stream's unparsed input, and so the largest read io can
    /// meet (`Limits::largest_read`).
    pub intake: u32,
    /// The most bytes one receive asks for.
    pub receive: u32,
    /// The cap on each stream's queued output, in bytes, its send in flight
    /// included, and so the largest room io can grant
    /// (`Limits::largest_room`).
    pub output: u32,
    /// The most `Send`s queued on a stream at once, beside the one in flight.
    /// Independent output reserves a free queued/direct-flight slot within
    /// this same envelope; it does not subtract the flight from this count
    /// (lib.md, 7.1; io.md, 3.3).
    pub sends: u32,
    /// The most accepts armed in one iteration, over every listener.
    pub accepts: u32,
    /// The backlog each listener asks the kernel for, a hint it may clamp.
    pub backlog: u32,
    /// How long a graceful close may take before it aborts.
    pub close_timeout: Duration,
    /// How long io waits before it tries again what found the kernel out of
    /// buffers or descriptors: a receive, a send, a half-close, an accept
    /// (io.md, 3.2 and 3.3). Never zero, or io would spin.
    pub retry: Duration,
}

impl Limits {
    /// The largest read demand io can meet: a fill or a scan of at most this
    /// many bytes, the intake's cap. Whoever stacks a machine on a stream
    /// checks at startup that the machine's largest demand is no larger (io.md,
    /// 2); io asserts it, as a larger one could never be met (lib.md, 7).
    #[must_use]
    pub const fn largest_read(&self) -> u32 {
        self.intake
    }

    /// The most output room io can grant at once: the output cap. The same
    /// startup check holds for room.
    #[must_use]
    pub const fn largest_room(&self) -> u32 {
        self.output
    }

    /// Whether io can run under these limits: a refusal held, a byte of
    /// intake, of receive and of output, a send queued, an accept per
    /// iteration and a retry that waits, at least.
    #[must_use]
    pub const fn is_usable(&self) -> bool {
        self.refusals > 0
            && self.intake > 0
            && self.receive > 0
            && self.output > 0
            && self.sends > 0
            && self.accepts > 0
            && self.retry.as_nanos() > 0
    }
}

/// The most operations an entity has in flight at once: a socket's `Recv`,
/// `Send`, and a `Cancel` of each. A pipe has at most one read or write and
/// its cancel; a child has a wait and one signal (io.md, 3.4 and 6).
/// Process usage and terminal size each have an independent slot, outside the entity slab.
const PER_ENTITY: u32 = 4;

/// The most operations io has in flight at once, cancels included, under
/// `limits`: the size of the ring (kernel.md, 5), or `None` past a `u32`.
#[must_use]
pub fn operations(limits: &Limits) -> Option<u32> {
    limits.sockets.checked_mul(PER_ENTITY)?.checked_add(2)
}

/// The deadline table's room: a graceful close's and a retry's per entity.
pub(crate) fn timers(limits: &Limits) -> Option<u32> {
    limits.sockets.checked_mul(2)
}

/// The slots of the operation table: twice the most in flight, as an
/// operation retired in an iteration keeps its slot until the reclaim point,
/// and its entity may submit as many again before then.
pub(crate) fn flights(limits: &Limits) -> Option<u32> {
    operations(limits)?.checked_mul(2)
}

/// Whether these limits are representable: every edge bound they size is finite
/// and every capacity they derive fits (programming-model.md, 6.3).
/// Checks io's usable byte and admission limits, and its operation, flight and
/// timer capacities. The byte bounds are already represented by `u32` limits.
#[must_use]
pub fn valid(limits: &Limits) -> bool {
    limits.is_usable() && operations(limits).is_some() && flights(limits).is_some() && timers(limits).is_some()
}

/// The most kernel submissions an entry point emits in one call. The loop
/// reserves this room at the kernel edge; events use an uncapped queue
/// (programming-model.md, section 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct MaxOut {
    pub submissions: u32,
}

/// `resume`: a receive the intake has room for again, or a listener's accept.
pub const MAX_OUT_RESUME: MaxOut = MaxOut { submissions: 1 };

/// `up`: the operation following the completed one, and a receive the intake
/// has room for again.
pub const MAX_OUT_UP: MaxOut = MaxOut { submissions: 2 };

/// `fire`: cancels of a closing stream's receive and send, retries of those
/// operations or a half-close, a listener's accept, or an append write cancel.
pub const MAX_OUT_FIRE: MaxOut = MaxOut { submissions: 2 };

/// `down`: a bound socket's receive and its listener's next accept; a close's
/// discarding receive and half-close; an abort's two cancels; or a child's
/// spawn or signal.
pub const MAX_OUT_DOWN: MaxOut = MaxOut { submissions: 2 };
