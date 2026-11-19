//! Wire projections of owner project facts (client/domain/testing.md, 4).
use super::super::{c, facts, list, pieces, w};
use super::{
    wire_authority_document, wire_document_text, wire_funder, wire_grant, wire_held_role, wire_numbers, wire_offered,
    wire_part, wire_party_kind, wire_policy_permission, wire_policy_requirement, wire_pool_ledger, wire_recurring,
    wire_template_override,
};
pub(in crate::codec) fn wire_project(value: &c::Project, limits: &w::Limits) -> Result<w::Project, w::Problem> {
    Ok(w::Project {
            acts: pieces(value.acts.iter().map(|item| wire_offered(item, limits)), limits)?,
            number: value.number,
            name: wire_document_text(&value.name, limits)?,
            resources: pieces(value.resources.iter().map(|item| wire_project_resource(item, limits)), limits)?,
            policy: wire_project_policy(&value.policy, limits)?,
            members: list(value.members.iter().map(|item| wire_project_member(item, limits)))?,
            current_period: value.current_period.as_ref().map(|item| wire_pool_ledger(item, limits)).transpose()?,
            pools: list(value.pools.iter().map(|item| wire_pool_ledger(item, limits)))?,
            configured_period: value.configured_period,
            configured_pools: list(value.configured_pools.iter().map(|item| wire_pool_ledger(item, limits)))?,
            configured_pools_cut: value.configured_pools_cut,
            spend: pieces(value.spend.iter().map(|item| wire_project_spend(item, limits)), limits)?,
            recurring: pieces(value.recurring.iter().map(|item| wire_recurring(item, limits)), limits)?,
            cut: value.cut,
        })
}

pub(in crate::codec) fn wire_project_resource(
    value: &c::ProjectResource,
    limits: &w::Limits,
) -> Result<w::ProjectResource, w::Problem> {
    Ok(w::ProjectResource {
            part: wire_part(&value.part, limits)?,
            role: wire_project_resource_role(&value.role, limits)?,
            home: value.home,
        })
}

pub(in crate::codec) fn wire_project_resource_role(
    value: &c::ProjectResourceRole,
    _limits: &w::Limits,
) -> Result<w::ProjectResourceRole, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ProjectResourceRole::Fork => w::ProjectResourceRole::Fork,
        c::ProjectResourceRole::Owned => w::ProjectResourceRole::Owned,
        c::ProjectResourceRole::Participant => w::ProjectResourceRole::Participant,
        c::ProjectResourceRole::Unavailable => w::ProjectResourceRole::Unavailable,
        c::ProjectResourceRole::Context => w::ProjectResourceRole::Context,
    })
}

pub(in crate::codec) fn wire_project_policy(
    value: &c::ProjectPolicy,
    limits: &w::Limits,
) -> Result<w::ProjectPolicy, w::Problem> {
    Ok(w::ProjectPolicy {
            templates: list(value.templates.iter().map(|item| wire_template_override(item, limits)))?,
            ceiling: wire_authority_document(&value.ceiling, limits)?,
            period_spend: value.period_spend,
            escalation_role: value.escalation_role.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            projections: pieces(value.projections.iter().map(|item| wire_grant(item, limits)), limits)?,
            roles: pieces(value.roles.iter().map(|item| wire_project_policy_role(item, limits)), limits)?,
            requirements: pieces(value.requirements.iter().map(|item| wire_policy_requirement(item, limits)), limits)?,
            permissions: value
                .permissions
                .as_ref()
                .map(|item| list((item).iter().map(|item| wire_policy_permission(item, limits))))
                .transpose()?,
            permissions_cut: value.permissions_cut,
            cut: value.cut,
        })
}

pub(in crate::codec) fn wire_project_policy_role(
    value: &c::ProjectPolicyRole,
    limits: &w::Limits,
) -> Result<w::ProjectPolicyRole, w::Problem> {
    Ok(w::ProjectPolicyRole {
            number: value.number,
            authority: wire_authority_document(&value.authority, limits)?,
            period_spend: value.period_spend,
            requests: value.requests,
            decides: value.decides,
        })
}

pub(in crate::codec) fn wire_project_member(
    value: &c::ProjectMember,
    limits: &w::Limits,
) -> Result<w::ProjectMember, w::Problem> {
    Ok(w::ProjectMember {
            party: value.party,
            kind: wire_party_kind(&value.kind, limits)?,
            role: wire_held_role(&value.role, limits)?,
            login: wire_document_text(&value.login, limits)?,
            name: wire_document_text(&value.name, limits)?,
        })
}

pub(in crate::codec) fn wire_project_spend(
    value: &c::ProjectSpend,
    limits: &w::Limits,
) -> Result<w::ProjectSpend, w::Problem> {
    Ok(w::ProjectSpend {
            chip: facts::chip(&value.chip, limits)?,
            funder: wire_funder(&value.funder, limits)?,
            numbers: wire_numbers(&value.numbers, limits)?,
        })
}
