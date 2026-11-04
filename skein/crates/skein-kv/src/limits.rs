//! Admission limits.

use crate::Page;
use skein_lib::Duration;

/// The least log since a snapshot's start that the next waits for (skein/docs/design/kv.md, 5).
pub const SNAPSHOT_FLOOR: u64 = 65_536;
/// The next snapshot's trigger, in multiples of the live map's encoded size (skein/docs/design/kv.md, 5).
pub const SNAPSHOT_MULTIPLE: u64 = 2;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub key: u32,
    pub value: u32,
    pub ops: u32,
    pub commit: u32,
    pub queued: u32,
    pub queued_bytes: u64,
    pub budget: u64,
    pub segment: u64,
    pub snapshot_after: u64,
    pub chunk: u32,
    pub page: Page,
    pub deadline: Duration,
}

impl Limits {
    /// The log since the last snapshot's start past which the next starts (skein/docs/design/kv.md, 5).
    pub(crate) fn snapshot_bound(&self, live: u64) -> u64 {
        SNAPSHOT_MULTIPLE.saturating_mul(live).max(SNAPSHOT_FLOOR).min(self.snapshot_after)
    }

    #[must_use]
    pub fn is_usable(&self) -> bool {
        self.key > 0
            && self.value > 0
            && self.ops > 0
            && self.commit >= 26
            && self.queued > 0
            && self.queued <= 32
            && self.queued_bytes > 0
            && self.budget > 0
            && self.segment > 0
            && self.snapshot_after > 0
            && self.chunk > 0
            && self.page.rows > 0
            && self.page.bytes > 0
            && self.deadline.as_nanos() > 0
            && u64::from(self.chunk) >= u64::from(self.key).saturating_add(u64::from(self.value)).saturating_add(20)
    }
}
