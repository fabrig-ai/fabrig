//! Inbox projections use the start's people entrance (domain/engine.md, 6, 11.4).
use crate::{Core, Limits, people};
use skein_lib::Map;

impl Core {
    pub(crate) fn project(&mut self, limits: &Limits, input: people::RestoreProjection) {
        {
            let projected = self.people.start_project(&limits.people, input);
            self.start_collect_failure();
            assert!(projected || self.start_failure().is_some(), "people projection refusal names its fit");
        }
    }

    pub(super) fn start_inbox_done(&mut self) {
        self.census_progress = Map::with_capacity(self.census_progress.capacity());
    }
}
