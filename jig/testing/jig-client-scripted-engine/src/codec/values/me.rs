//! Wire projections of me facts (client/domain/testing.md, 4).
use super::super::{c, list, pieces, try_pieces, w};
use super::{
    wire_authority_document, wire_beyond, wire_count, wire_document_text, wire_held_role, wire_llm_choice,
    wire_me_model, wire_needs, wire_party_kind, wire_pool_ledger, wire_provider_offer, wire_task_contract_document,
    wire_task_flags, wire_template_budget, wire_template_deadline, wire_template_executor,
};
pub(in crate::codec) fn wire_template(value: &c::Template, limits: &w::Limits) -> Result<w::Template, w::Problem> {
    Ok(w::Template {
        resources: skein_lib::List::with_capacity(0),
        number: value.number,
        name: wire_document_text(&value.name),
        executor: wire_template_executor(&value.executor),
        contract: wire_task_contract_document(&value.contract, limits)?,
        shallower: value.shallower,
        budget: wire_template_budget(&value.budget),
        deadline: wire_template_deadline(&value.deadline),
        tracked: value.tracked,
        flags: wire_task_flags(&value.flags),
        label: value.label,
        grace: value.grace,
        llm: value.llm.as_ref().map(wire_llm_choice),
    })
}

pub(in crate::codec) fn wire_bounds_create(
    value: &c::CreateBounds,
    limits: &w::Limits,
) -> Result<w::BoundsCreate, w::Problem> {
    Ok(w::BoundsCreate {
        authority: wire_authority_document(&value.authority, limits)?,
        pool_left: value.pool_left,
        charters: list(value.charters.iter().copied()),
        procedures: list(value.procedures.iter().map(wire_create_procedure)),
        latest: value.latest.as_ref().map(|item| (item).as_nanos()),
        budget: value.budget,
        maximum_budget: value.maximum_budget,
        maximum_deadline: value.maximum_deadline.as_ref().map(|item| (item).as_nanos()),
    })
}

pub(in crate::codec) fn wire_create_procedure(value: &c::CreateProcedure) -> w::CreateProcedure {
    w::CreateProcedure { connector: value.connector, code: value.code }
}

pub(in crate::codec) fn wire_me(value: &c::Me, limits: &w::Limits) -> Result<w::Me, w::Problem> {
    Ok(w::Me {
        party: wire_me_party(&value.party),
        projects: try_pieces(value.projects.iter().map(|item| wire_me_project(item, limits)), limits)?,
        needs: wire_needs(&value.needs),
        needs_total: value.needs_total,
        updates: wire_count(&value.updates),
        results_seen: value.results_seen,
        cut: value.cut,
        retention: value.retention,
        deployment_role: value.deployment_role.as_ref().map(wire_held_role),
        providers: pieces(value.providers.iter().map(wire_provider_offer), limits)?,
        providers_cut: value.providers_cut,
        key_bytes: value.key_bytes,
        redirect_bytes: value.redirect_bytes,
        words_bytes: value.words_bytes,
        sign_in_expires: value.sign_in_expires.as_nanos(),
    })
}

pub(in crate::codec) fn wire_me_party(value: &c::MeParty) -> w::MeParty {
    w::MeParty {
        number: value.number,
        kind: wire_party_kind(&value.kind),
        login: wire_document_text(&value.login),
        name: wire_document_text(&value.name),
    }
}

pub(in crate::codec) fn wire_me_project(value: &c::MeProject, limits: &w::Limits) -> Result<w::MeProject, w::Problem> {
    Ok(w::MeProject {
        left_out: value.left_out,
        public: None,
        project: value.project,
        name: wire_document_text(&value.name),
        role: wire_held_role(&value.role),
        pool: value.pool.as_ref().map(wire_pool_ledger),
        templates: try_pieces(value.templates.iter().map(|item| wire_template(item, limits)), limits)?,
        creating: value.creating.as_ref().map(|item| wire_creating_bounds(item, limits)).transpose()?,
        models: list(value.models.iter().map(wire_me_model)),
        defaults: list(
            value
                .defaults
                .iter()
                .map(|item| w::MeDefault { template: item.template, choice: super::wire_llm_choice(&item.choice) }),
        ),
    })
}

pub(in crate::codec) fn wire_creating_bounds(
    value: &c::Creating,
    limits: &w::Limits,
) -> Result<w::CreatingBounds, w::Problem> {
    Ok(w::CreatingBounds {
        within: wire_bounds_create(&value.within, limits)?,
        beyond: wire_beyond(&value.beyond, limits)?,
    })
}
