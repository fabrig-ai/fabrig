//! Start flights routed through the real store (protocol/store.md, 7.1 and 14).
use super::{Neighbours, Store, StoreOut, core, root};
use jig_conformance::Input;
impl Neighbours {
    pub(super) fn start(
        &mut self,
        store: &mut Store,
        phase: core::start::Phase,
        census: Option<core::Census>,
    ) -> Vec<Input<root::Event, root::Record>> {
        use core::start::Phase;
        self.start_phases.push(phase);
        let position = match phase {
            Phase::LoadCore => Some(0),
            Phase::RestoreConnector { connector } => Some(u32::from(connector)),
            Phase::ReadAfresh { connector } => Some(2 + u32::from(connector)),
            Phase::LoadInbox | Phase::Sweep | Phase::Switch => None,
            Phase::Link | Phase::Wires | Phase::AdoptRuns | Phase::SettleOutbox { .. } | Phase::Open => {
                panic!("root performs internal start phases")
            }
        };
        if let Some(position) = position {
            self.routes.push(jig_conformance::referee::Observed::RestartStage { position });
        }
        if matches!(phase, Phase::ReadAfresh { .. }) || (phase == Phase::Switch && !store.generation_pending) {
            vec![Input::Event(root::Event::StartDone(phase))]
        } else {
            store.ask(StoreOut::Start { phase, census });
            Vec::new()
        }
    }
}
