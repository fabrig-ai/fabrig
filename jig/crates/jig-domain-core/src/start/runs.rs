//! Run adoption and decision opening (jig/docs/design/domain/engine.md, 6, 8).
use crate::{Core, fleet};

impl Core {
    /// True once a start has opened decisions.
    #[must_use]
    pub fn start_opened(&self) -> bool {
        self.opened
    }

    pub(crate) fn start_stopped(&self) -> bool {
        self.start_failure().is_some()
    }

    /// Count every restored worker claim before any reaches the fleet.
    #[expect(clippy::manual_map, reason = "the step-code subset uses exhaustive matches")]
    pub(crate) fn restored_worker_claims(&self) -> Option<(u64, u64)> {
        let mut first = None;
        let mut held = 0_u64;
        for (task, _) in &self.unreported_restored {
            match self.proofs.get(task).expect("restored claim has its proof").host {
                fleet::HostKind::Worker if !self.workers_in_process => {
                    if first.is_none() {
                        first = Some(*task);
                    }
                    held = held.checked_add(1).expect("bounded restored claims");
                }
                fleet::HostKind::Worker | fleet::HostKind::Engine => {}
            }
        }
        match first {
            Some(task) => Some((task, held)),
            None => None,
        }
    }
}
