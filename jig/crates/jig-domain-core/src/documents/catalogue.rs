//! Public offers and payer readiness (client/domain/contract/documents.md, 1, and client/domain/contract/README.md, 5.1).
//! Closed at once (jig/docs/design/domain/people/acts.md, 2).

use crate::{
    Core, Limits,
    documents::{DocumentLimits, Missing},
};
use jig_domain_accounts as accounts;
pub use jig_domain_core_vocabulary::documents::catalogue::*;
use jig_domain_fleet as fleet;
use jig_domain_tasks as tasks;
use skein_lib::List;

pub(super) fn offers(core: &Core, limits: &DocumentLimits) -> Result<(List<ProviderOffer>, bool), Missing> {
    let mut result = List::with_capacity(limits.lines);
    let mut cut = false;
    for provider in &core.settings.llm_providers.providers {
        if result.room() == 0 {
            cut = true;
            continue;
        }
        let mut programs = List::with_capacity(limits.lines);
        let mut programs_cut = false;
        for name in &provider.offers.programs {
            if programs.room() == 0 {
                programs_cut = true;
                continue;
            }
            if name.len() > 32 {
                return Err(Missing::TooLarge);
            }
            programs.push(name.clone()).or(Err(Missing::TooLarge))?;
        }
        result
            .push(ProviderOffer {
                provider: provider.number,
                key_bytes: core.settings.key_bytes,
                name: super::small::text(&provider.name, limits.text_bytes),
                sign_in: provider.offers.sign_in,
                key: provider.offers.key,
                programs,
                programs_cut,
                none: provider.offers.none(),
                signed_in_plan: provider.signed_in_plan,
                manage_usage: provider.manage_usage.clone(),
            })
            .or(Err(Missing::TooLarge))?;
    }
    Ok((result, cut))
}

pub(super) fn rows(core: &Core, limits: &Limits, party: u64) -> Result<(List<PayerReadiness>, bool), Missing> {
    let mut rows = List::with_capacity(limits.documents.lines);
    let mut cut = false;
    let pending = core.accounts.pending();
    append(core, pending.as_slice(), tasks::Payer::Party(party), &mut rows, &mut cut)?;
    let roles = core.people.project_roles(party).ok_or(Missing::Unknown)?;
    for role in &roles {
        if !core.open_project(role.project) {
            continue;
        }
        append(core, pending.as_slice(), tasks::Payer::Project(role.project), &mut rows, &mut cut)?;
    }
    Ok((rows, cut))
}
fn append(
    core: &Core,
    pending: &[accounts::Pending],
    payer: tasks::Payer,
    rows: &mut List<PayerReadiness>,
    cut: &mut bool,
) -> Result<(), Missing> {
    for provider in &core.settings.llm_providers.providers {
        if rows.room() == 0 {
            *cut = true;
            continue;
        }
        let (readiness, refreshing) = readiness(core, payer, provider.number);
        let mut settled = if provider.offers.programs.is_empty() { Some(true) } else { None };
        if refreshing {
            settled = Some(false);
        }
        for pending in pending {
            if payer == tasks::Payer::Party(pending.by) && pending.provider == provider.number {
                settled = Some(false);
            }
        }
        rows.push(PayerReadiness {
            provider: provider.number,
            payer: crate::runs::coverage::payer(payer),
            readiness,
            settled,
        })
        .or(Err(Missing::TooLarge))?;
    }
    Ok(())
}

/// Full-state selection precedes any document truncation. No task pin is chosen here.
fn readiness(core: &Core, payer: tasks::Payer, provider: u32) -> (Readiness, bool) {
    let coverage = core.account_coverage(payer, None);
    let mut unknown = false;
    let mut refreshing = false;
    for model in &core.settings.models {
        if core.provider(model) != Some(provider) {
            continue;
        }
        let (state, pending) = core.model_readiness(payer, coverage.as_slice(), model);
        if state == Readiness::Usable {
            return (Readiness::Usable, false);
        }
        unknown |= state == Readiness::Unknown;
        refreshing |= pending;
    }
    (if unknown || refreshing { Readiness::Unknown } else { Readiness::Unavailable }, refreshing)
}

impl Core {
    /// An account's current list narrows catalogue models; held names without a row stay usable
    /// (jig/docs/design/domain/accounts.md, 3.7 and 4.3).
    pub(crate) fn account_allows(&self, account: u32, model: &crate::Model) -> bool {
        for number in 1..=tasks::MODELS {
            let Some(value) = self.current.value(crate::settings::Setting::Model(number)) else { continue };
            let Some(row) = value.as_model() else { continue };
            if row.model.name != model.name {
                continue;
            }
            let kind = match row.source {
                crate::settings::ModelSource::Stock => accounts::ModelKind::Stock,
                crate::settings::ModelSource::List => accounts::ModelKind::List,
                crate::settings::ModelSource::Person => accounts::ModelKind::Person,
            };
            return self.accounts.allows(account, row.number, kind);
        }
        true
    }

    /// The offer for `payer` in `project`, in catalogue number order
    /// (jig/docs/design/domain/engine/views.md, 1).
    #[must_use]
    pub fn offer(&self, payer: tasks::Payer, project: u32, models: tasks::Models) -> List<Offered> {
        let coverage = self.account_coverage(payer, Some(project));
        let mut offered = List::with_capacity(tasks::MODELS);
        for (at, model) in self.settings.models.iter().enumerate() {
            if model.is_hole() {
                continue;
            }
            let number = u32::try_from(at).expect("model position").checked_add(1).expect("model number");
            if let Some(value) = self.current.value(crate::settings::Setting::Model(number))
                && let Some(value) = value.as_model()
                && value.withdrawn
            {
                continue;
            }
            let index = u8::try_from(at).expect("configured models bounded");
            if models.bits() & 1_u64.checked_shl(u32::from(index)).expect("model bit") == 0 {
                continue;
            }
            let accounts = self.offered_accounts(payer, coverage.as_slice(), model);
            if !accounts.is_empty() {
                offered.push(Offered { model: index, accounts }).expect("configured models bounded");
            }
        }
        offered
    }

    fn offered_accounts(
        &self,
        payer: tasks::Payer,
        coverage: &[accounts::Coverage],
        model: &crate::Model,
    ) -> List<u32> {
        let mut result = List::with_capacity(self.accounts.accounts());
        let Some(provider) = self.provider(model) else { return result };
        let payer = crate::runs::coverage::payer(payer);
        let candidates =
            self.accounts.candidates(payer, provider, self.settings.model_ways(model), None, None, coverage);
        let hosts = self.fleet.host_views();
        let mut facts = List::with_capacity(self.host_fact_capacity());
        for account in &candidates {
            if !self.account_allows(*account, model) {
                continue;
            }
            for host in &hosts {
                if !host.connected
                    || host.draining
                    || host.slots == 0
                    || (host.kind == fleet::HostKind::Worker && self.fleet.awaiting_logins(host.channel))
                {
                    continue;
                }
                let placement = fleet::PlacementConstraint { channel: host.channel, kind: host.kind };
                self.host_facts(placement, self.host_rule(placement), &mut facts);
                match self.accounts.select(
                    payer,
                    provider,
                    self.settings.model_ways(model),
                    Some(*account),
                    None,
                    coverage,
                    facts.as_slice(),
                    self.run_now,
                ) {
                    accounts::Selection::Chosen { source, .. } if self.source_supported(placement, model, source) => {
                        result.push(*account).expect("candidate accounts bounded");
                        break;
                    }
                    accounts::Selection::Chosen { .. }
                    | accounts::Selection::Wait { .. }
                    | accounts::Selection::NoAccount => {}
                }
            }
        }
        result
    }

    /// Whether `model` has an account usable for `payer`, on a connected host
    /// (jig/docs/design/domain/accounts.md, 4.3).
    pub(crate) fn model_readiness(
        &self,
        payer: tasks::Payer,
        coverage: &[accounts::Coverage],
        model: &crate::Model,
    ) -> (Readiness, bool) {
        let Some(provider) = self.provider(model) else { return (Readiness::Unavailable, false) };
        let mut unknown = false;
        let mut refreshing = false;
        let hosts = self.fleet.host_views();
        let candidates = self.accounts.candidates(
            crate::runs::coverage::payer(payer),
            provider,
            self.settings.model_ways(model),
            None,
            None,
            coverage,
        );
        let mut allowed = false;
        for account in &candidates {
            allowed |= self.account_allows(*account, model);
        }
        if !allowed {
            return (Readiness::Unavailable, false);
        }
        let mut facts = List::with_capacity(self.host_fact_capacity());
        for host in &hosts {
            if !host.connected || host.draining || host.slots == 0 {
                continue;
            }
            if host.kind == fleet::HostKind::Worker && self.fleet.awaiting_logins(host.channel) {
                unknown = true;
                continue;
            }
            let placement = fleet::PlacementConstraint { channel: host.channel, kind: host.kind };
            self.host_facts(placement, self.host_rule(placement), &mut facts);
            for account in &candidates {
                if !self.account_allows(*account, model) {
                    continue;
                }
                match self.accounts.select(
                    crate::runs::coverage::payer(payer),
                    provider,
                    self.settings.model_ways(model),
                    Some(*account),
                    None,
                    coverage,
                    facts.as_slice(),
                    self.run_now,
                ) {
                    accounts::Selection::Chosen { source, .. } => {
                        if self.source_supported(placement, model, source) {
                            return (Readiness::Usable, false);
                        }
                    }
                    accounts::Selection::Wait { why: accounts::WaitWhy::Host, .. } => unknown = true,
                    accounts::Selection::Wait { why: accounts::WaitWhy::Refreshing, .. } => refreshing = true,
                    accounts::Selection::Wait {
                        why: accounts::WaitWhy::Source(_) | accounts::WaitWhy::Empty, ..
                    }
                    | accounts::Selection::NoAccount => {}
                }
            }
        }
        (if unknown || refreshing { Readiness::Unknown } else { Readiness::Unavailable }, refreshing)
    }
}
