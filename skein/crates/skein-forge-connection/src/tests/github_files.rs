//! Component allocation spans used by the remaining crate harness.
// Sum only component/harness spans. Each next step starts with the retained
// growth of the preceding steps; fake allocations between them are excluded.
#[derive(Debug)]
pub struct CallHeap {
    pub(super) peak: i64,
    pub(super) held: i64,
}
impl CallHeap {
    pub(super) fn count(&mut self, growth: skein_heap::Grown) {
        self.peak = self.peak.max(self.held.checked_add(growth.peak).expect("peak growth"));
        self.held = self.held.checked_add(growth.net).expect("retained growth");
    }
}
