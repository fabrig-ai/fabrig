//! Observe and transport the start's first requests (protocol/store/loads.md, 1).
use super::{Neighbours, Store, StoreOut, core};

impl Neighbours {
    pub(super) fn start(&mut self, store: &mut Store, phase: core::start::Phase, out: StoreOut) {
        self.start_phases.push(phase);
        if let Some(position) = jig_conformance::core::peers::restart_position(phase, 2) {
            self.routes.push(jig_conformance::referee::Observed::RestartStage { position });
        }
        store.ask(out);
    }
}
