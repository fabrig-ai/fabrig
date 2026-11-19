//! Wire projections of policy edit facts (client/domain/testing.md, 4).
use super::super::{c, w};
use super::{wire_authority, wire_pattern};
pub(in crate::codec) fn wire_template_executor(
    value: &c::TemplateExecutor,
    _limits: &w::Limits,
) -> Result<w::TemplateExecutor, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TemplateExecutor::Charter(value) => {
            w::TemplateExecutor::Charter(w::TemplateCharter { value: *(value) })
        }
        c::TemplateExecutor::Chosen => w::TemplateExecutor::Chosen,
    })
}

pub(in crate::codec) fn wire_template_budget(
    value: &c::TemplateBudget,
    _limits: &w::Limits,
) -> Result<w::TemplateBudget, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TemplateBudget::Amount(value) => {
            w::TemplateBudget::Amount(w::TemplateAmount { value: *(value) })
        }
        c::TemplateBudget::AsAllowed { cap } => w::TemplateBudget::AsAllowed(w::TemplateAsAllowed { cap: (cap).as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()? }),
        c::TemplateBudget::Given => w::TemplateBudget::Given,
    })
}

pub(in crate::codec) fn wire_template_deadline(
    value: &c::TemplateDeadline,
    _limits: &w::Limits,
) -> Result<w::TemplateDeadline, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::TemplateDeadline::None => w::TemplateDeadline::None,
        c::TemplateDeadline::Span(value) => {
            w::TemplateDeadline::Span(w::TemplateSpan { value: *(value) })
        }
        c::TemplateDeadline::Given => w::TemplateDeadline::Given,
    })
}

pub(in crate::codec) fn wire_policy_edit_role(
    value: &c::PolicyEditRole,
    limits: &w::Limits,
) -> Result<w::PolicyEditRole, w::Problem> {
    Ok(w::PolicyEditRole {
            number: value.number,
            authority: wire_authority(&value.authority, limits)?,
            period_spend: value.period_spend,
            requests: value.requests,
            decides: value.decides,
        })
}

pub(in crate::codec) fn wire_policy_requirement(
    value: &c::PolicyRequirement,
    limits: &w::Limits,
) -> Result<w::PolicyRequirement, w::Problem> {
    Ok(w::PolicyRequirement {
            connector: value.connector,
            kind: value.kind,
            pattern: wire_pattern(&value.pattern, limits)?,
            judge: wire_policy_judge(&value.judge, limits)?,
            guard: wire_policy_guard(&value.guard, limits)?,
            must_be_guarded: value.must_be_guarded,
        })
}

pub(in crate::codec) fn wire_policy_judge(
    value: &c::PolicyJudge,
    _limits: &w::Limits,
) -> Result<w::PolicyJudge, w::Problem> {
    Ok(w::PolicyJudge {
            connector: value.connector,
            requirement: value.requirement,
            parameters: value.parameters,
        })
}

pub(in crate::codec) fn wire_policy_guard(
    value: &c::PolicyGuard,
    _limits: &w::Limits,
) -> Result<w::PolicyGuard, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::PolicyGuard::Guarded => w::PolicyGuard::Guarded,
        c::PolicyGuard::Observed { freshness } => w::PolicyGuard::Observed(w::PolicyGuardObserved { freshness: *(freshness) }),
    })
}

pub(in crate::codec) fn wire_policy_permission(
    value: &c::PolicyPermission,
    _limits: &w::Limits,
) -> Result<w::PolicyPermission, w::Problem> {
    Ok(w::PolicyPermission { connector: value.connector, permission: value.permission, role: value.role })
}

pub(in crate::codec) fn wire_template_override(
    value: &c::TemplateOverride,
    limits: &w::Limits,
) -> Result<w::TemplateOverride, w::Problem> {
    Ok(w::TemplateOverride {
            template: value.template,
            executor: value.executor.as_ref().map(|item| wire_template_executor(item, limits)).transpose()?,
            budget: value.budget.as_ref().map(|item| wire_template_budget(item, limits)).transpose()?,
            deadline: value.deadline.as_ref().map(|item| wire_template_deadline(item, limits)).transpose()?,
            priority: value.priority.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
        })
}
