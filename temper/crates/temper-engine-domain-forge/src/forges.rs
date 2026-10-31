//! Forge records and the machine's places (temper/docs/design/draft/playground-reset.md, 3).
use crate::{Domain, ForgeRecord, Machine};

impl Domain {
    /// Keep the machine's places for new forges.
    pub fn set_machine(&mut self, machine: Machine) {
        self.machine = Some(machine);
    }

    /// The kept record for a forge number.
    #[must_use]
    pub fn record(&self, _forge: u16) -> Option<&ForgeRecord> {
        None
    }
}
