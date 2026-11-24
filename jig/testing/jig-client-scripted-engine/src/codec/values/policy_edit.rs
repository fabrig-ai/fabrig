//! Wire projections of policy edit facts (client/domain/testing.md, 4).
use super::super::{c, w};
use super::{wire_authority, wire_pattern};
pub(in crate::codec) fn wire_template_executor(value: &c::TemplateExecutor) -> w::TemplateExecutor {
    match value {
        c::TemplateExecutor::Charter(value) => w::TemplateExecutor::Charter(w::TemplateCharter { value: *(value) }),
        c::TemplateExecutor::Chosen => w::TemplateExecutor::Chosen,
    }
}

pub(in crate::codec) fn wire_template_budget(value: &c::TemplateBudget) -> w::TemplateBudget {
    match value {
        c::TemplateBudget::Amount(value) => w::TemplateBudget::Amount(w::TemplateAmount { value: *(value) }),
        c::TemplateBudget::AsAllowed { cap } => w::TemplateBudget::AsAllowed(w::TemplateAsAllowed { cap: *cap }),
        c::TemplateBudget::Given => w::TemplateBudget::Given,
    }
}

pub(in crate::codec) fn wire_template_deadline(value: &c::TemplateDeadline) -> w::TemplateDeadline {
    match value {
        c::TemplateDeadline::None => w::TemplateDeadline::None,
        c::TemplateDeadline::Span(value) => w::TemplateDeadline::Span(w::TemplateSpan { value: *(value) }),
        c::TemplateDeadline::Given => w::TemplateDeadline::Given,
    }
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

pub(in crate::codec) fn wire_policy_requirement(value: &c::PolicyRequirement) -> w::PolicyRequirement {
    w::PolicyRequirement {
        connector: value.connector,
        kind: value.kind,
        pattern: wire_pattern(&value.pattern),
        judge: wire_policy_judge(&value.judge),
        guard: wire_policy_guard(&value.guard),
        must_be_guarded: value.must_be_guarded,
    }
}

pub(in crate::codec) fn wire_policy_judge(value: &c::PolicyJudge) -> w::PolicyJudge {
    w::PolicyJudge { connector: value.connector, requirement: value.requirement, parameters: value.parameters }
}

pub(in crate::codec) fn wire_policy_guard(value: &c::PolicyGuard) -> w::PolicyGuard {
    match value {
        c::PolicyGuard::Guarded => w::PolicyGuard::Guarded,
        c::PolicyGuard::Observed { freshness } => {
            w::PolicyGuard::Observed(w::PolicyGuardObserved { freshness: *(freshness) })
        }
    }
}

pub(in crate::codec) fn wire_policy_permission(value: &c::PolicyPermission) -> w::PolicyPermission {
    w::PolicyPermission { connector: value.connector, permission: value.permission, role: value.role }
}

pub(in crate::codec) fn wire_template_override(value: &c::TemplateOverride) -> w::TemplateOverride {
    w::TemplateOverride {
        template: value.template,
        executor: value.executor.as_ref().map(wire_template_executor),
        budget: value.budget.as_ref().map(wire_template_budget),
        deadline: value.deadline.as_ref().map(wire_template_deadline),
        priority: value.priority,
    }
}
