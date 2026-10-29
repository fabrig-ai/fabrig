//! Capacity limits and representability checks (forge-client.md, section 5).
//!
//! The limits bound retained calls, live rows and in-flight writes. This
//! module retains no state and knows no task policy. `valid` rejects
//! incoherent limits and capacities that do not fit their types.
use skein_lib::Duration;
/// Caps on retained calls, live rows, writes and reply bytes.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    /// Logical calls retained, including queued and in flight.
    pub pending: u32,
    pub calls: u32,
    pub rate: u32,
    pub reserve: u32,
    pub window: Duration,
    /// Aggregate owned allocation bytes per operation, including row storage.
    pub op_bytes: u32,
    pub answer_bytes: u32,
    pub rows: u32,
    /// Distinct comments and reviews remembered per participating resource,
    /// each in its own bounded delivery stamp table.
    pub inbox: u32,
    pub read_attempts: u32,
    pub backoff: Duration,
    pub backoff_max: Duration,
    pub entries: u32,
    pub write_attempts: u32,
    /// Longest time a sent write may still arrive at Forgejo.
    pub lifetime: Duration,
    /// Allowance for the wall clock moving forward between hosts or restarts.
    pub clock_margin: Duration,
    pub resources: u32,
    pub repositories: u32,
    pub poll: Duration,
    pub poll_max: Duration,
    pub hinted: Duration,
    pub slow: Duration,
    /// How long Forgejo may settle mergeability and a hinted job's CI.
    pub settle: Duration,
    pub facts: u32,
}
/// Whether these limits are representable: every edge bound they size
/// is finite and every capacity they derive fits (programming-model.md, 6.3).
/// Budgets and retry windows are coherent; alarm and keep capacities fit u32.
#[must_use]
pub fn valid(l: &Limits) -> bool {
    if l.pending == 0
        || l.calls == 0
        || l.calls > l.pending
        || l.rate == 0
        || l.reserve > l.rate
        || l.window == Duration::ZERO
        || l.read_attempts == 0
        || l.backoff == Duration::ZERO
        || l.backoff > l.backoff_max
        || l.entries == 0
        || l.write_attempts == 0
        || l.lifetime == Duration::ZERO
        || l.clock_margin == Duration::ZERO
        || l.rows == 0
        || l.inbox == 0
        || l.resources == 0
        || l.repositories == 0
        || l.poll == Duration::ZERO
        || l.poll > l.poll_max
        || l.hinted == Duration::ZERO
        || l.slow == Duration::ZERO
        || l.settle == Duration::ZERO
    {
        return false;
    }
    capacity_bound(l).is_some()
}

fn capacity_bound(l: &Limits) -> Option<()> {
    l.pending.checked_add(l.entries)?.checked_add(l.resources)?.checked_add(l.repositories)?.checked_add(2)?;
    l.resources.checked_mul(2)?.checked_add(l.repositories.checked_mul(2)?)?.checked_add(1)?;
    Some(())
}
