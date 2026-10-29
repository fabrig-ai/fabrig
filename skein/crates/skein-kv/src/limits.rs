//! Admission limits.

use crate::Page;
use skein_lib::Duration;

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
