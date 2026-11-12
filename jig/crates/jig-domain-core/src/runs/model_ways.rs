//! Source ways and run endpoints, shared by observation, admission and preparation
//! (jig/docs/design/domain/accounts.md, 3.4 and 4.3). Stored models keep their endpoint.
use crate::{Core, Model};
use jig_domain_fleet::ModelNeed;

impl Core {
    pub(crate) fn source_need(&self, model: &Model, source: u32) -> Option<ModelNeed> {
        let endpoint = self.settings.model_endpoint(model, self.accounts.source(source)?.kind)?;
        Some(ModelNeed { endpoint, name: model.name.clone(), window: model.window, output: model.output })
    }

    pub(crate) fn source_supported(
        &self,
        placement: jig_domain_fleet::PlacementConstraint,
        model: &Model,
        source: u32,
    ) -> bool {
        let Some(need) = self.source_need(model, source) else { return false };
        self.fleet.supports(placement, core::slice::from_ref(&need))
    }
}
