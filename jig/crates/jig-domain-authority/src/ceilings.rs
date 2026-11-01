//! Policy ceilings before the open period's room (jig/docs/design/domain/authority.md, 6).
use crate::{Authority, Implies, Policy, Rules};

/// The holder of a ceiling a policy edit passes (jig/docs/design/domain/authority.md, 6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CeilingHolder {
    Rules,
    Policy,
    Role(u32),
}

/// A ceiling's dimension; list dimensions carry no contents.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CeilingDimension {
    PeriodSpend,
    Spend,
    Tools,
    Tasks,
    Depth,
    Executors,
    Grants,
    Deadline,
    Notes,
}

/// The first ceiling a policy edit passes, with the scalar values supplied.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Ceiling {
    pub holder: CeilingHolder,
    pub dimension: CeilingDimension,
    pub value: Option<u64>,
    pub asked: Option<u64>,
}

/// Check the rules, then the roles; a cut below a role names that role's floor.
#[must_use]
pub fn policy_ceiling(rules: &Rules, before: &Policy, policy: &Policy) -> Option<Ceiling> {
    if policy.period_spend > rules.period_spend {
        return Some(Ceiling {
            holder: CeilingHolder::Rules,
            dimension: CeilingDimension::PeriodSpend,
            value: Some(rules.period_spend),
            asked: Some(policy.period_spend),
        });
    }
    if let Some(ceiling) = authority_ceiling(&policy.ceiling, &rules.ceiling, &rules.implies, CeilingHolder::Rules) {
        return Some(ceiling);
    }
    for grant in &policy.projections {
        let mut covered = false;
        for ceiling in &policy.ceiling.grants {
            covered |= crate::grant_at_most(grant, ceiling, &rules.implies);
        }
        if !covered {
            return Some(Ceiling {
                holder: CeilingHolder::Policy,
                dimension: CeilingDimension::Grants,
                value: None,
                asked: None,
            });
        }
    }
    for role in &policy.roles {
        if role.period_spend > policy.period_spend {
            let cut = policy.period_spend < before.period_spend;
            return Some(Ceiling {
                holder: if cut { CeilingHolder::Role(role.number) } else { CeilingHolder::Policy },
                dimension: CeilingDimension::PeriodSpend,
                value: Some(if cut { role.period_spend } else { policy.period_spend }),
                asked: Some(if cut { policy.period_spend } else { role.period_spend }),
            });
        }
        if let Some(ceiling) =
            authority_ceiling(&role.authority, &policy.ceiling, &rules.implies, CeilingHolder::Policy)
        {
            return Some(ceiling);
        }
    }
    None
}

fn authority_ceiling(a: &Authority, b: &Authority, implies: &Implies, holder: CeilingHolder) -> Option<Ceiling> {
    let lacks = crate::order::differences(a, b, b.delegation.tasks, b.delegation.depth, b.budget.spend, implies);
    let (dimension, value, asked) = if lacks.spend {
        (CeilingDimension::Spend, Some(b.budget.spend), Some(a.budget.spend))
    } else if lacks.tools {
        (CeilingDimension::Tools, None, None)
    } else if lacks.tasks {
        (CeilingDimension::Tasks, Some(u64::from(b.delegation.tasks)), Some(u64::from(a.delegation.tasks)))
    } else if lacks.depth {
        (CeilingDimension::Depth, Some(u64::from(b.delegation.depth)), Some(u64::from(a.delegation.depth)))
    } else if lacks.executors {
        (CeilingDimension::Executors, None, None)
    } else if lacks.grants {
        (CeilingDimension::Grants, None, None)
    } else if lacks.deadline {
        (
            CeilingDimension::Deadline,
            match b.budget.deadline {
                Some(wall) => Some(wall.as_nanos()),
                None => None,
            },
            match a.budget.deadline {
                Some(wall) => Some(wall.as_nanos()),
                None => None,
            },
        )
    } else if lacks.notes {
        (CeilingDimension::Notes, None, None)
    } else {
        return None;
    };
    Some(Ceiling { holder, dimension, value, asked })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Budget, Delegation, Scopes, Tools, tests_fixtures};
    use alloc::boxed::Box;
    use skein_lib::Wall;

    #[test]
    fn authority_ceiling_checks_name_each_dimension_and_scalar() {
        let given = tests_fixtures::authority();
        let policy = tests_fixtures::policy(given.clone());
        for (ceiling, dimension, value, asked) in [
            (
                Authority { budget: Budget { spend: 99, ..given.budget }, ..given.clone() },
                CeilingDimension::Spend,
                Some(99),
                Some(100),
            ),
            (Authority { tools: Tools(0), ..given.clone() }, CeilingDimension::Tools, None, None),
            (
                Authority { delegation: Delegation { tasks: 9, ..given.delegation.clone() }, ..given.clone() },
                CeilingDimension::Tasks,
                Some(9),
                Some(10),
            ),
            (
                Authority { delegation: Delegation { depth: 4, ..given.delegation.clone() }, ..given.clone() },
                CeilingDimension::Depth,
                Some(4),
                Some(5),
            ),
            (
                Authority {
                    delegation: Delegation { kinds: Box::new([]), ..given.delegation.clone() },
                    ..given.clone()
                },
                CeilingDimension::Executors,
                None,
                None,
            ),
            (Authority { grants: Box::new([]), ..given.clone() }, CeilingDimension::Grants, None, None),
            (
                Authority { budget: Budget { deadline: Some(Wall::from_nanos(10)), ..given.budget }, ..given.clone() },
                CeilingDimension::Deadline,
                Some(10),
                None,
            ),
            (Authority { notes: Scopes(0), ..given }, CeilingDimension::Notes, None, None),
        ] {
            let rules = tests_fixtures::rules(ceiling);
            assert_eq!(
                policy_ceiling(&rules, &policy, &policy),
                Some(Ceiling { holder: CeilingHolder::Rules, dimension, value, asked })
            );
        }
    }
}
