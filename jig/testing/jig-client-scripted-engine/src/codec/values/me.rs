//! Wire projections of me facts (client/domain/testing.md, 4).
use super::super::{c, list, pieces, w};
use super::{
    wire_authority_document, wire_beyond, wire_count, wire_document_text, wire_held_role, wire_llm_choice,
    wire_me_model, wire_needs, wire_party_kind, wire_pool_ledger, wire_provider_offer, wire_task_contract_document,
    wire_task_flags, wire_template_budget, wire_template_deadline, wire_template_executor,
};
pub(in crate::codec) fn wire_template(value: &c::Template, limits: &w::Limits) -> Result<w::Template, w::Problem> {
    w::Template::new(
        limits,
        w::TemplateParts {
            resources: skein_lib::List::with_capacity(0),
            number: value.number,
            name: wire_document_text(&value.name, limits)?,
            executor: wire_template_executor(&value.executor, limits)?,
            contract: wire_task_contract_document(&value.contract, limits)?,
            shallower: value.shallower,
            budget: wire_template_budget(&value.budget, limits)?,
            deadline: wire_template_deadline(&value.deadline, limits)?,
            tracked: value.tracked.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            flags: wire_task_flags(&value.flags, limits)?,
            label: value.label,
            grace: value.grace.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            llm: value.llm.as_ref().map(|item| wire_llm_choice(item, limits)).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_bounds_create(
    value: &c::CreateBounds,
    limits: &w::Limits,
) -> Result<w::BoundsCreate, w::Problem> {
    w::BoundsCreate::new(
        limits,
        w::BoundsCreateParts {
            authority: wire_authority_document(&value.authority, limits)?,
            pool_left: value.pool_left,
            charters: list(value.charters.iter().map(|item| Ok::<_, w::Problem>(*(item))))?,
            procedures: list(value.procedures.iter().map(|item| wire_create_procedure(item, limits)))?,
            latest: value.latest.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            budget: value.budget,
            maximum_budget: value.maximum_budget,
            maximum_deadline: value
                .maximum_deadline
                .as_ref()
                .map(|item| Ok::<_, w::Problem>((item).as_nanos()))
                .transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_create_procedure(
    value: &c::CreateProcedure,
    limits: &w::Limits,
) -> Result<w::CreateProcedure, w::Problem> {
    w::CreateProcedure::new(limits, w::CreateProcedureParts { connector: value.connector, code: value.code })
}

pub(in crate::codec) fn wire_me(value: &c::Me, limits: &w::Limits) -> Result<w::Me, w::Problem> {
    w::Me::new(
        limits,
        w::MeParts {
            party: wire_me_party(&value.party, limits)?,
            projects: pieces(
                value.projects.iter().map(|item| wire_me_project(item, limits)),
                w::MeProject::measure,
                w::MeProject::encode,
            )?,
            needs: wire_needs(&value.needs, limits)?,
            needs_total: value.needs_total,
            updates: wire_count(&value.updates, limits)?,
            results_seen: value.results_seen,
            cut: value.cut,
            retention: value.retention,
            deployment_role: value.deployment_role.as_ref().map(|item| wire_held_role(item, limits)).transpose()?,
            providers: pieces(
                value.providers.iter().map(|item| wire_provider_offer(item, limits)),
                w::ProviderOffer::measure,
                w::ProviderOffer::encode,
            )?,
            providers_cut: value.providers_cut,
            key_bytes: value.key_bytes,
            redirect_bytes: value.redirect_bytes,
        },
    )
}

pub(in crate::codec) fn wire_me_party(value: &c::MeParty, limits: &w::Limits) -> Result<w::MeParty, w::Problem> {
    w::MeParty::new(
        limits,
        w::MePartyParts {
            number: value.number,
            kind: wire_party_kind(&value.kind, limits)?,
            login: wire_document_text(&value.login, limits)?,
            name: wire_document_text(&value.name, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_me_project(value: &c::MeProject, limits: &w::Limits) -> Result<w::MeProject, w::Problem> {
    w::MeProject::new(
        limits,
        w::MeProjectParts {
            public: None,
            project: value.project,
            name: wire_document_text(&value.name, limits)?,
            role: wire_held_role(&value.role, limits)?,
            pool: value.pool.as_ref().map(|item| wire_pool_ledger(item, limits)).transpose()?,
            templates: pieces(
                value.templates.iter().map(|item| wire_template(item, limits)),
                w::Template::measure,
                w::Template::encode,
            )?,
            creating: value.creating.as_ref().map(|item| wire_creating_bounds(item, limits)).transpose()?,
            models: list(value.models.iter().map(|item| wire_me_model(item, limits)))?,
        },
    )
}

pub(in crate::codec) fn wire_creating_bounds(
    value: &c::Creating,
    limits: &w::Limits,
) -> Result<w::CreatingBounds, w::Problem> {
    w::CreatingBounds::new(
        limits,
        w::CreatingBoundsParts {
            within: wire_bounds_create(&value.within, limits)?,
            beyond: wire_beyond(&value.beyond, limits)?,
        },
    )
}
