//! Source ways and run endpoints, shared by observation, admission and preparation
//! (jig/docs/design/domain/accounts.md, 3.4 and 4.3). Stored models keep their endpoint.
use crate::{Core, Model, Settings};
use jig_domain_accounts::{Kind, Ways};
use jig_domain_fleet::ModelNeed;

impl Settings {
    pub(crate) fn model_ways(&self, model: &Model) -> Ways {
        Ways {
            program: model.program.is_some(),
            direct: model.program.is_none() || self.direct_endpoint(model.provider).is_some(),
        }
    }

    pub(crate) fn model_endpoint(&self, model: &Model, kind: Kind) -> Option<u32> {
        if !self.model_ways(model).allows(kind) {
            return None;
        }
        if kind == Kind::Program || model.program.is_none() {
            Some(model.endpoint)
        } else {
            Some(self.direct_endpoint(model.provider)?.number)
        }
    }
}

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
