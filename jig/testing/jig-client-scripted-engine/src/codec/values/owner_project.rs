//! Wire projections of owner project facts (client/domain/testing.md, 4).
use super::super::{c, facts, list, pieces, w};
use super::{
    wire_document_text, wire_funder, wire_held_role, wire_offered, wire_owner_authority_document, wire_owner_grant,
    wire_owner_numbers, wire_owner_part, wire_party_kind, wire_policy_permission, wire_policy_requirement,
    wire_pool_ledger, wire_recurring, wire_template_override,
};
pub(in crate::codec) fn wire_owner_project(
    value: &c::OwnerProject,
    limits: &w::Limits,
) -> Result<w::OwnerProject, w::Problem> {
    w::OwnerProject::new(
        limits,
        w::OwnerProjectParts {
            acts: pieces(
                value.acts.iter().map(|item| wire_offered(item, limits)),
                w::Offered::measure,
                w::Offered::encode,
            )?,
            number: value.number,
            name: wire_document_text(&value.name, limits)?,
            resources: pieces(
                value.resources.iter().map(|item| wire_owner_project_resource(item, limits)),
                w::OwnerProjectResource::measure,
                w::OwnerProjectResource::encode,
            )?,
            policy: wire_owner_project_policy(&value.policy, limits)?,
            members: list(value.members.iter().map(|item| wire_owner_project_member(item, limits)))?,
            current_period: value.current_period.as_ref().map(|item| wire_pool_ledger(item, limits)).transpose()?,
            pools: list(value.pools.iter().map(|item| wire_pool_ledger(item, limits)))?,
            configured_period: value.configured_period,
            configured_pools: list(value.configured_pools.iter().map(|item| wire_pool_ledger(item, limits)))?,
            configured_pools_cut: value.configured_pools_cut,
            spend: pieces(
                value.spend.iter().map(|item| wire_owner_project_spend(item, limits)),
                w::OwnerProjectSpend::measure,
                w::OwnerProjectSpend::encode,
            )?,
            recurring: pieces(
                value.recurring.iter().map(|item| wire_recurring(item, limits)),
                w::Recurring::measure,
                w::Recurring::encode,
            )?,
            cut: value.cut,
        },
    )
}

pub(in crate::codec) fn wire_owner_project_resource(
    value: &c::OwnerProjectResource,
    limits: &w::Limits,
) -> Result<w::OwnerProjectResource, w::Problem> {
    w::OwnerProjectResource::new(
        limits,
        w::OwnerProjectResourceParts {
            part: wire_owner_part(&value.part, limits)?,
            role: wire_owner_project_resource_role(&value.role, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_owner_project_resource_role(
    value: &c::OwnerProjectResourceRole,
    _limits: &w::Limits,
) -> Result<w::OwnerProjectResourceRole, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerProjectResourceRole::Owned => w::OwnerProjectResourceRole::Owned,
        c::OwnerProjectResourceRole::Participant => w::OwnerProjectResourceRole::Participant,
        c::OwnerProjectResourceRole::Unavailable => w::OwnerProjectResourceRole::Unavailable,
        c::OwnerProjectResourceRole::Context => w::OwnerProjectResourceRole::Context,
    })
}

pub(in crate::codec) fn wire_owner_project_policy(
    value: &c::OwnerProjectPolicy,
    limits: &w::Limits,
) -> Result<w::OwnerProjectPolicy, w::Problem> {
    w::OwnerProjectPolicy::new(
        limits,
        w::OwnerProjectPolicyParts {
            templates: list(value.templates.iter().map(|item| wire_template_override(item, limits)))?,
            ceiling: wire_owner_authority_document(&value.ceiling, limits)?,
            period_spend: value.period_spend,
            escalation_role: value.escalation_role.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            projections: pieces(
                value.projections.iter().map(|item| wire_owner_grant(item, limits)),
                w::OwnerGrant::measure,
                w::OwnerGrant::encode,
            )?,
            roles: pieces(
                value.roles.iter().map(|item| wire_owner_project_policy_role(item, limits)),
                w::OwnerProjectPolicyRole::measure,
                w::OwnerProjectPolicyRole::encode,
            )?,
            requirements: pieces(
                value.requirements.iter().map(|item| wire_policy_requirement(item, limits)),
                w::PolicyRequirement::measure,
                w::PolicyRequirement::encode,
            )?,
            permissions: value
                .permissions
                .as_ref()
                .map(|item| list((item).iter().map(|item| wire_policy_permission(item, limits))))
                .transpose()?,
            permissions_cut: value.permissions_cut,
            cut: value.cut,
        },
    )
}

pub(in crate::codec) fn wire_owner_project_policy_role(
    value: &c::OwnerProjectPolicyRole,
    limits: &w::Limits,
) -> Result<w::OwnerProjectPolicyRole, w::Problem> {
    w::OwnerProjectPolicyRole::new(
        limits,
        w::OwnerProjectPolicyRoleParts {
            number: value.number,
            authority: wire_owner_authority_document(&value.authority, limits)?,
            period_spend: value.period_spend,
            requests: value.requests,
            decides: value.decides,
        },
    )
}

pub(in crate::codec) fn wire_owner_project_member(
    value: &c::OwnerProjectMember,
    limits: &w::Limits,
) -> Result<w::OwnerProjectMember, w::Problem> {
    w::OwnerProjectMember::new(
        limits,
        w::OwnerProjectMemberParts {
            party: value.party,
            kind: wire_party_kind(&value.kind, limits)?,
            role: wire_held_role(&value.role, limits)?,
            login: wire_document_text(&value.login, limits)?,
            name: wire_document_text(&value.name, limits)?,
        },
    )
}

pub(in crate::codec) fn wire_owner_project_spend(
    value: &c::OwnerProjectSpend,
    limits: &w::Limits,
) -> Result<w::OwnerProjectSpend, w::Problem> {
    w::OwnerProjectSpend::new(
        limits,
        w::OwnerProjectSpendParts {
            chip: facts::chip(&value.chip, limits)?,
            funder: wire_funder(&value.funder, limits)?,
            numbers: wire_owner_numbers(&value.numbers, limits)?,
        },
    )
}
