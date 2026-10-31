//! Wire projections of policy edit facts (client/domain/testing.md, 4).
use super::super::{c, w};
use super::{wire_owner_authority, wire_owner_pattern};
pub(in crate::codec) fn wire_template_executor(
    value: &c::TemplateExecutor,
    limits: &w::Limits,
) -> Result<w::TemplateExecutor, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TemplateExecutor::Charter(value) => {
            w::TemplateExecutor::Charter(w::TemplateCharter::new(limits, w::TemplateCharterParts { value: *(value) })?)
        }
        c::TemplateExecutor::Chosen => w::TemplateExecutor::Chosen,
    })
}

pub(in crate::codec) fn wire_template_budget(
    value: &c::TemplateBudget,
    limits: &w::Limits,
) -> Result<w::TemplateBudget, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TemplateBudget::Amount(value) => {
            w::TemplateBudget::Amount(w::TemplateAmount::new(limits, w::TemplateAmountParts { value: *(value) })?)
        }
        c::TemplateBudget::AsAllowed { cap } => w::TemplateBudget::AsAllowed(w::TemplateAsAllowed::new(
            limits,
            w::TemplateAsAllowedParts { cap: (cap).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()? },
        )?),
        c::TemplateBudget::Given => w::TemplateBudget::Given,
    })
}

pub(in crate::codec) fn wire_template_deadline(
    value: &c::TemplateDeadline,
    limits: &w::Limits,
) -> Result<w::TemplateDeadline, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TemplateDeadline::None => w::TemplateDeadline::None,
        c::TemplateDeadline::Span(value) => {
            w::TemplateDeadline::Span(w::TemplateSpan::new(limits, w::TemplateSpanParts { value: *(value) })?)
        }
        c::TemplateDeadline::Given => w::TemplateDeadline::Given,
    })
}

pub(in crate::codec) fn wire_policy_edit_role(
    value: &c::PolicyEditRole,
    limits: &w::Limits,
) -> Result<w::PolicyEditRole, w::Problem> {
    w::PolicyEditRole::new(
        limits,
        w::PolicyEditRoleParts {
            number: value.number,
            authority: wire_owner_authority(&value.authority, limits)?,
            period_spend: value.period_spend,
            requests: value.requests,
            decides: value.decides,
        },
    )
}

pub(in crate::codec) fn wire_policy_requirement(
    value: &c::PolicyRequirement,
    limits: &w::Limits,
) -> Result<w::PolicyRequirement, w::Problem> {
    w::PolicyRequirement::new(
        limits,
        w::PolicyRequirementParts {
            connector: value.connector,
            kind: value.kind,
            pattern: wire_owner_pattern(&value.pattern, limits)?,
            judge: wire_policy_judge(&value.judge, limits)?,
            guard: wire_policy_guard(&value.guard, limits)?,
            must_be_guarded: value.must_be_guarded,
        },
    )
}

pub(in crate::codec) fn wire_policy_judge(
    value: &c::PolicyJudge,
    limits: &w::Limits,
) -> Result<w::PolicyJudge, w::Problem> {
    w::PolicyJudge::new(
        limits,
        w::PolicyJudgeParts {
            connector: value.connector,
            requirement: value.requirement,
            parameters: value.parameters,
        },
    )
}

pub(in crate::codec) fn wire_policy_guard(
    value: &c::PolicyGuard,
    limits: &w::Limits,
) -> Result<w::PolicyGuard, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::PolicyGuard::Guarded => w::PolicyGuard::Guarded,
        c::PolicyGuard::Observed { freshness } => w::PolicyGuard::Observed(w::PolicyGuardObserved::new(
            limits,
            w::PolicyGuardObservedParts { freshness: *(freshness) },
        )?),
    })
}

pub(in crate::codec) fn wire_policy_permission(
    value: &c::PolicyPermission,
    limits: &w::Limits,
) -> Result<w::PolicyPermission, w::Problem> {
    w::PolicyPermission::new(
        limits,
        w::PolicyPermissionParts { connector: value.connector, permission: value.permission, role: value.role },
    )
}

pub(in crate::codec) fn wire_template_override(
    value: &c::TemplateOverride,
    limits: &w::Limits,
) -> Result<w::TemplateOverride, w::Problem> {
    w::TemplateOverride::new(
        limits,
        w::TemplateOverrideParts {
            template: value.template,
            executor: value.executor.as_ref().map(|item| wire_template_executor(item, limits)).transpose()?,
            budget: value.budget.as_ref().map(|item| wire_template_budget(item, limits)).transpose()?,
            deadline: value.deadline.as_ref().map(|item| wire_template_deadline(item, limits)).transpose()?,
            priority: value.priority.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        },
    )
}
