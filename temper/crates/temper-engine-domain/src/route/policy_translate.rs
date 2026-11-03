//! Landing defaults at the root boundary (temper/docs/design/domain/root.md, 6;
//! temper/docs/design/protocol/configuration.md, 13.3).
use super::{Freshness, LandingRule, authority, forge};
use alloc::boxed::Box;
use forge::settings;
use skein_lib::List;

pub(crate) fn combine_requirements(
    base: &[authority::Requirement],
    added: &[authority::Requirement],
    limit: u32,
) -> Option<Box<[authority::Requirement]>> {
    let mut result = List::with_capacity(limit);
    for requirement in base {
        result.push(requirement.clone()).ok()?;
    }
    for requirement in added {
        if !base.contains(requirement) {
            result.push(requirement.clone()).ok()?;
        }
    }
    Some(result.into_boxed())
}

/// The merge checks the forge's held rules, independently of their criteria.
pub(crate) fn standing_requirements(connector: u16) -> [authority::Requirement; 2] {
    [standing(connector, 5, true), standing(connector, 6, false)]
}

fn standing(connector: u16, requirement: u16, guarded: bool) -> authority::Requirement {
    authority::Requirement {
        connector,
        kind: 4,
        pattern: authority::Pattern {
            segments: Box::new([Box::from(b"forge".as_slice())]),
            last: authority::Last::Open(Box::new([])),
        },
        judge: authority::Judge { connector, requirement, parameters: 0 },
        guard: if guarded {
            authority::Guard::Guarded
        } else {
            authority::Guard::Observed { freshness: skein_lib::Duration::ZERO }
        },
        must_be_guarded: guarded,
    }
}

/// Bridge for worlds constructing Config without encoded kind defaults.
pub(crate) fn forge_stock(
    landing: &crate::LandingRules,
    charter: u32,
    connector: u16,
    limits: &crate::Limits,
) -> Result<Box<[settings::Stock]>, Option<u32>> {
    if landing.projects.capacity() > limits.authority.projects || !rules_within(&landing.deployment, connector, limits)
    {
        return Err(None);
    }
    for (&project, rules) in &landing.projects {
        if !rules_within(rules, connector, limits) {
            return Err(Some(project));
        }
    }
    let mut stock = List::uncapped();
    rules(&mut stock, 1, 0, &landing.deployment);
    for (&project, landing) in &landing.projects {
        rules(&mut stock, 2, project, landing);
    }
    for (number, charter) in [(1, charter), (2, 4)] {
        stock
            .push(settings::Stock {
                kind: 3,
                scope: 0,
                number,
                value: settings::SizedValue { size: 0, value: settings::Value::Charter(charter) },
            })
            .expect("uncapped stock");
    }
    Ok(stock.into_boxed())
}

fn rules(stock: &mut List<settings::Stock>, kind: u32, scope: u32, rules: &[LandingRule]) {
    for (index, rule) in rules.iter().enumerate() {
        let last = match &rule.pattern.last {
            jig_domain_authority::Last::Exact(value) => forge::GateLast::Exact(value.clone()),
            jig_domain_authority::Last::Open(value) => forge::GateLast::Open(value.clone()),
        };
        let mut gates = List::uncapped();
        for gate in &rule.gates {
            gates
                .push(settings::RuleGate {
                    number: gate.number,
                    blocking: gate.blocking,
                    freshness: freshness(gate.freshness),
                })
                .expect("uncapped gates");
        }
        let mut approvals = List::uncapped();
        for approval in &rule.approvals {
            approvals
                .push(settings::RuleApproval {
                    role: approval.role,
                    people: approval.people,
                    freshness: freshness(approval.freshness),
                })
                .expect("uncapped approvals");
        }
        let value = settings::Value::Rule(settings::LandingRule {
            pattern: forge::GatePattern { segments: rule.pattern.segments.clone(), last },
            enforce: rule.enforce,
            ci: rule.ci,
            up_to_date: rule.up_to_date,
            gates: gates.into_boxed(),
            approvals: approvals.into_boxed(),
        });
        stock
            .push(settings::Stock {
                kind,
                scope,
                number: u32::try_from(index).expect("bounded rules").checked_add(1).expect("bounded rule number"),
                value: settings::SizedValue { size: 0, value },
            })
            .expect("uncapped stock");
    }
}

fn freshness(value: Freshness) -> forge::JudgeFreshness {
    match value {
        Freshness::Exact => forge::JudgeFreshness::Exact,
        Freshness::Clean => forge::JudgeFreshness::Clean,
    }
}

fn rules_within(rules: &[LandingRule], connector: u16, limits: &crate::Limits) -> bool {
    if rules.len() > usize::try_from(limits.forge.landing_rules).expect("bounded rules") {
        return false;
    }
    match super::landing::landing_rules_bytes(rules) {
        Some(bytes) if bytes <= u64::from(limits.journal.transcript_bytes) => {}
        Some(_) | None => return false,
    }
    for rule in rules {
        if rule.connector != connector
            || rule.kind != 4
            || rule.gates.len() > usize::try_from(limits.forge.change_policy.gates).expect("bounded gates")
        {
            return false;
        }
    }
    true
}
