//! Policy snapshots (jig/docs/design/domain/authority.md, 6).

use crate::translate::{authority_to_people, grant_to_authority, grant_to_people, requirement_to_people};
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_people as people;
use skein_lib::List;

/// Capture the whole policy after a validated policy event.
#[must_use]
pub fn snapshot(
    policy: &authority::Policy,
    permissions: &[people::PermissionRole],
    templates: &[people::TemplateOverride],
) -> Option<people::PolicyValue> {
    let mut roles = List::with_capacity(u32::try_from(policy.roles.len()).ok()?);
    for role in &policy.roles {
        roles
            .push(people::PolicyRole {
                number: role.number,
                authority: authority_to_people(&role.authority)?,
                period_spend: role.period_spend,
                requests: role.requests.0,
                decides: role.decides.0,
            })
            .ok()?;
    }
    let mut requirements = List::with_capacity(u32::try_from(policy.requirements.len()).ok()?);
    for requirement in &policy.requirements {
        requirements.push(requirement_to_people(requirement)).ok()?;
    }
    Some(people::PolicyValue {
        ceiling: authority_to_people(&policy.ceiling)?,
        escalation_role: policy.escalation_role,
        period_spend: policy.period_spend,
        projections: grants_to_people(&policy.projections)?,
        roles: roles.into_boxed(),
        requirements: requirements.into_boxed(),
        permissions: Box::from(permissions),
        templates: Box::from(templates),
    })
}

#[must_use]
pub fn grants_to_authority(grants: Box<[people::Grant]>) -> Option<Box<[authority::Grant]>> {
    let mut result = List::with_capacity(u32::try_from(grants.len()).ok()?);
    for grant in grants {
        result.push(grant_to_authority(grant)).ok()?;
    }
    Some(result.into_boxed())
}
fn grants_to_people(grants: &[authority::Grant]) -> Option<Box<[people::Grant]>> {
    let mut result = List::with_capacity(u32::try_from(grants.len()).ok()?);
    for grant in grants {
        result.push(grant_to_people(grant)).ok()?;
    }
    Some(result.into_boxed())
}
