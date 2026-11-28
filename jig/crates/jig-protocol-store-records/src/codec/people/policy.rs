//! Policy values retained by people (protocol/store/records.md, 1).
use super::{
    Wall, decode_authority_note_resources, decode_delegation_kinds, decode_grant_piece, decode_grants, decode_numbers,
    decode_permissions, decode_policy_value_roles, decode_requirement_piece, decode_requirements, decode_segments,
    decode_spec_parameters, domain, encode_authority_note_resources, encode_delegation_kinds, encode_grant_piece,
    encode_grants, encode_numbers, encode_permissions, encode_policy_value_roles, encode_requirement_piece,
    encode_requirements, encode_segments, encode_spec_parameters, permission_sum, shared_sum, wire,
};
use crate::generated::people as mapped;
use alloc::boxed::Box;

pub(crate) fn encode_pattern(value: &domain::Pattern, limits: &wire::Limits) -> Result<wire::Pattern, wire::Problem> {
    let domain::Pattern { segments, last } = value;
    Ok(wire::Pattern {
        segments: encode_segments(segments, limits, shared_sum(limits), limits.pattern_segments)?,
        last: mapped::last_to_record(last),
    })
}
pub(crate) fn decode_pattern(value: wire::Pattern, limits: &wire::Limits) -> Result<domain::Pattern, wire::Problem> {
    let wire::Pattern { segments, last } = value;
    Ok(domain::Pattern {
        segments: decode_segments(segments, limits, shared_sum(limits))?,
        last: mapped::last_to_domain(last),
    })
}

pub(crate) fn delegation_to_record(
    value: &domain::Delegation,
    limits: &wire::Limits,
) -> Result<wire::Delegation, wire::Problem> {
    let domain::Delegation { kinds, tasks, depth } = value;
    Ok(wire::Delegation {
        kinds: encode_delegation_kinds(kinds, limits, shared_sum(limits), limits.delegation_kinds)?,
        tasks: *tasks,
        depth: *depth,
    })
}
pub(crate) fn delegation_to_domain(
    value: wire::Delegation,
    limits: &wire::Limits,
) -> Result<domain::Delegation, wire::Problem> {
    let wire::Delegation { kinds, tasks, depth } = value;
    Ok(domain::Delegation { kinds: decode_delegation_kinds(kinds, limits, shared_sum(limits))?, tasks, depth })
}

pub(crate) fn authority_to_record(
    value: &domain::Authority,
    limits: &wire::Limits,
) -> Result<wire::Authority, wire::Problem> {
    let domain::Authority { tools, models, grants, delegation, spend, deadline, notes, note_resources } = value;
    Ok(wire::Authority {
        tools: *tools,
        models: match models {
            Some(value) => Some(mapped::model_set_to_record(value)),
            None => None,
        },
        grants: encode_grant_piece(grants, limits, shared_sum(limits), limits.authority_grants)?,
        delegation: delegation_to_record(delegation, limits)?,
        spend: *spend,
        deadline: match deadline {
            Some(value) => Some(value.as_nanos()),
            None => None,
        },
        notes: *notes,
        note_resources: encode_authority_note_resources(
            note_resources,
            limits,
            shared_sum(limits),
            limits.authority_note_resources,
        )?,
    })
}
pub(crate) fn authority_to_domain(
    value: wire::Authority,
    limits: &wire::Limits,
) -> Result<domain::Authority, wire::Problem> {
    let wire::Authority { tools, models, grants, delegation, spend, deadline, notes, note_resources } = value;
    Ok(domain::Authority {
        tools,
        models: match models {
            Some(value) => Some(mapped::model_set_to_domain(value)),
            None => None,
        },
        grants: decode_grant_piece(grants, limits, shared_sum(limits))?,
        delegation: delegation_to_domain(delegation, limits)?,
        spend,
        deadline: match deadline {
            Some(value) => Some(Wall::from_nanos(value)),
            None => None,
        },
        notes,
        note_resources: decode_authority_note_resources(note_resources, limits, shared_sum(limits))?,
    })
}

pub(crate) fn policy_value_to_record(
    value: &domain::PolicyValue,
    limits: &wire::Limits,
) -> Result<wire::PolicyValue, wire::Problem> {
    let domain::PolicyValue {
        ceiling,
        escalation_role,
        period_spend,
        projections,
        roles,
        requirements,
        permissions,
        templates,
    } = value;
    Ok(wire::PolicyValue {
        ceiling: authority_to_record(ceiling, limits)?,
        escalation_role: *escalation_role,
        period_spend: *period_spend,
        templates: encode_templates(templates, limits.policy_value_templates)?,
        projections: encode_grants(projections, limits)?,
        roles: encode_policy_value_roles(roles, limits)?,
        requirements: encode_requirements(requirements, limits)?,
        permissions: encode_permissions(permissions, limits, permission_sum(limits), limits.policy_value_permissions)?,
    })
}
pub(crate) fn policy_value_to_domain(
    value: wire::PolicyValue,
    limits: &wire::Limits,
) -> Result<domain::PolicyValue, wire::Problem> {
    let wire::PolicyValue {
        ceiling,
        escalation_role,
        period_spend,
        projections,
        roles,
        requirements,
        permissions,
        templates,
    } = value;
    Ok(domain::PolicyValue {
        ceiling: authority_to_domain(ceiling, limits)?,
        escalation_role,
        templates: decode_templates(templates),
        period_spend,
        projections: decode_grants(projections, limits)?,
        roles: decode_policy_value_roles(roles, limits)?,
        requirements: decode_requirements(requirements, limits)?,
        permissions: decode_permissions(permissions, limits, permission_sum(limits))?,
    })
}

pub(crate) fn policy_change_to_record(
    value: &domain::PolicyChange,
    limits: &wire::Limits,
) -> Result<wire::PolicyChange, wire::Problem> {
    Ok(match value {
        domain::PolicyChange::Templates { overrides } => wire::PolicyChange::Templates(wire::PolicyChangeTemplates {
            overrides: encode_templates(overrides, limits.policy_change_templates_overrides)?,
        }),
        domain::PolicyChange::RoleSpend { role, period_spend } => {
            wire::PolicyChange::RoleSpend(wire::PolicyChangeRoleSpend { role: *role, period_spend: *period_spend })
        }
        domain::PolicyChange::ProjectSpend { period_spend } => {
            wire::PolicyChange::ProjectSpend(wire::PolicyChangeProjectSpend { period_spend: *period_spend })
        }
        domain::PolicyChange::Role(value) => {
            wire::PolicyChange::Role(wire::PolicyChangeRole { value: mapped::policy_role_to_record(value, limits)? })
        }
        domain::PolicyChange::Requirements { requirements } => {
            wire::PolicyChange::Requirements(wire::PolicyChangeRequirements {
                requirements: encode_requirement_piece(
                    requirements,
                    limits,
                    limits.parameter_bytes_value,
                    limits.policy_change_requirements_requirements,
                )?,
            })
        }
        domain::PolicyChange::Projections { grants } => {
            wire::PolicyChange::Projections(wire::PolicyChangeProjections {
                grants: encode_grant_piece(
                    grants,
                    limits,
                    limits.parameter_bytes_value,
                    limits.policy_change_projections_grants,
                )?,
            })
        }
        domain::PolicyChange::Permissions { mappings } => {
            wire::PolicyChange::Permissions(wire::PolicyChangePermissions {
                mappings: encode_permissions(
                    mappings,
                    limits,
                    limits.parameter_bytes_value,
                    limits.policy_change_permissions_mappings,
                )?,
            })
        }
    })
}
pub(crate) fn policy_change_to_domain(
    value: wire::PolicyChange,
    limits: &wire::Limits,
) -> Result<domain::PolicyChange, wire::Problem> {
    Ok(match value {
        wire::PolicyChange::Templates(value) => {
            domain::PolicyChange::Templates { overrides: decode_templates(value.overrides) }
        }
        wire::PolicyChange::RoleSpend(value) => {
            let wire::PolicyChangeRoleSpend { role, period_spend } = value;
            domain::PolicyChange::RoleSpend { role, period_spend }
        }
        wire::PolicyChange::ProjectSpend(value) => {
            let wire::PolicyChangeProjectSpend { period_spend } = value;
            domain::PolicyChange::ProjectSpend { period_spend }
        }
        wire::PolicyChange::Role(value) => {
            let wire::PolicyChangeRole { value } = value;
            domain::PolicyChange::Role(mapped::policy_role_to_domain(value, limits)?)
        }
        wire::PolicyChange::Requirements(value) => {
            let wire::PolicyChangeRequirements { requirements } = value;
            domain::PolicyChange::Requirements {
                requirements: decode_requirement_piece(requirements, limits, limits.parameter_bytes_value)?,
            }
        }
        wire::PolicyChange::Projections(value) => {
            let wire::PolicyChangeProjections { grants } = value;
            domain::PolicyChange::Projections {
                grants: decode_grant_piece(grants, limits, limits.parameter_bytes_value)?,
            }
        }
        wire::PolicyChange::Permissions(value) => {
            let wire::PolicyChangePermissions { mappings } = value;
            domain::PolicyChange::Permissions {
                mappings: decode_permissions(mappings, limits, limits.parameter_bytes_value)?,
            }
        }
    })
}

pub(crate) fn spec_to_record(value: &domain::Spec, limits: &wire::Limits) -> Result<wire::Spec, wire::Problem> {
    let domain::Spec { asks, words, parameters, inputs } = value;
    Ok(wire::Spec {
        asks: *asks,
        words: words.clone(),
        parameters: encode_spec_parameters(parameters, limits, limits.parameter_bytes_value, limits.spec_parameters)?,
        inputs: encode_numbers(inputs, limits, limits.parameter_bytes_value, limits.spec_inputs)?,
    })
}
pub(crate) fn spec_to_domain(value: wire::Spec, limits: &wire::Limits) -> Result<domain::Spec, wire::Problem> {
    let wire::Spec { words, parameters, inputs, asks } = value;
    Ok(domain::Spec {
        asks,
        words,
        parameters: decode_spec_parameters(parameters, limits, limits.parameter_bytes_value)?,
        inputs: decode_numbers(inputs, limits, limits.parameter_bytes_value)?,
    })
}

pub(crate) fn encode_amendment(
    value: &domain::Amendment,
    limits: &wire::Limits,
) -> Result<wire::Amendment, wire::Problem> {
    check_amendment(value, limits)?;
    let domain::Amendment { tracked, addressed, instructions, spec, wake, dependencies, authority, llm, reason } =
        value;
    Ok(wire::Amendment {
        llm: match llm {
            Some(value) => Some(mapped::llm_choice_to_record(value)?),
            None => None,
        },
        tracked: match tracked {
            Some(value) => Some(mapped::tracking_to_record(value)),
            None => None,
        },
        addressed: match addressed {
            Some(value) => Some(mapped::address_change_to_record(value)),
            None => None,
        },
        instructions: instructions.clone(),
        spec: match spec {
            Some(value) => Some(spec_to_record(value, limits)?),
            None => None,
        },
        wake: match wake {
            Some(value) => Some(mapped::wake_policy_to_record(value)),
            None => None,
        },
        dependencies: match dependencies {
            Some(value) => {
                Some(encode_numbers(value, limits, limits.parameter_bytes_value, limits.amendment_dependencies_some)?)
            }
            None => None,
        },
        authority: match authority {
            Some(value) => Some(authority_to_record(value, limits)?),
            None => None,
        },
        reason: reason.clone(),
    })
}
pub(crate) fn decode_amendment(
    value: wire::Amendment,
    limits: &wire::Limits,
) -> Result<domain::Amendment, wire::Problem> {
    let wire::Amendment { tracked, addressed, instructions, spec, wake, dependencies, authority, llm, reason } = value;
    let value = domain::Amendment {
        llm: match llm {
            Some(value) => Some(mapped::llm_choice_to_domain(value)?),
            None => None,
        },
        tracked: match tracked {
            Some(value) => Some(mapped::tracking_to_domain(value)),
            None => None,
        },
        addressed: match addressed {
            Some(value) => Some(mapped::address_change_to_domain(value)),
            None => None,
        },
        instructions,
        spec: match spec {
            Some(value) => Some(spec_to_domain(value, limits)?),
            None => None,
        },
        wake: match wake {
            Some(value) => Some(mapped::wake_policy_to_domain(value)),
            None => None,
        },
        dependencies: match dependencies {
            Some(value) => Some(decode_numbers(value, limits, limits.parameter_bytes_value)?),
            None => None,
        },
        authority: match authority {
            Some(value) => Some(authority_to_domain(value, limits)?),
            None => None,
        },
        reason,
    };
    check_amendment(&value, limits)?;
    Ok(value)
}

fn check_amendment(value: &domain::Amendment, limits: &wire::Limits) -> Result<(), wire::Problem> {
    match domain::amendment_bytes(value) {
        Some(bytes) if bytes <= u64::from(limits.parameter_bytes_value) => Ok(()),
        Some(_) | None => Err(super::problem()),
    }
}

fn encode_templates(
    entries: &[domain::TemplateOverride],
    bound: u32,
) -> Result<skein_lib::List<wire::TemplateOverride>, wire::Problem> {
    let mut values = skein_lib::List::with_capacity(bound);
    for entry in entries {
        if values.push(mapped::template_override_to_record(entry)).is_err() {
            return Err(super::problem());
        }
    }
    Ok(values)
}
fn decode_templates(entries: skein_lib::List<wire::TemplateOverride>) -> Box<[domain::TemplateOverride]> {
    let mut values = skein_lib::List::with_capacity(entries.len());
    for entry in entries.into_boxed() {
        values.push(mapped::template_override_to_domain(entry)).expect("decoded overrides bounded");
    }
    values.into_boxed()
}
