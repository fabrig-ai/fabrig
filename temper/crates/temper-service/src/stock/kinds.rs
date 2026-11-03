//! temper's kind declarations and document defaults
//! (temper/docs/design/protocol/configuration.md, 13.2, 13.4 and 13.6).
use alloc::boxed::Box;
use jig_domain_core::settings::{Kind, KindScope};
use skein_lib::{List, bytes};
use temper_engine_domain::{LandingPolicy, Limits};
use temper_engine_domain_forge::{self as forge, settings};
use temper_engine_smith::kinds::size;

/// Declare the forge's kinds at the same bounds as its settings judge.
#[must_use]
pub fn kinds(limits: &Limits, connector: u16) -> Box<[Kind]> {
    let segment_bytes = limits.authority.segment_bytes.min(limits.forge.name_bytes);
    let mut segments = List::with_capacity(limits.authority.segments);
    if limits.authority.segments > 0 {
        segments.push(bytes::copy_of(b"forge")).expect("first pattern segment");
    }
    for _ in 1..limits.authority.segments {
        segments.push(word(segment_bytes)).expect("bounded pattern segments");
    }
    let mut gates = List::with_capacity(limits.forge.change_policy.gates);
    for number in 0..limits.forge.change_policy.gates {
        gates
            .push(settings::RuleGate { number, blocking: true, freshness: forge::JudgeFreshness::Clean })
            .expect("bounded gates");
    }
    let mut approvals = List::with_capacity(limits.forge.roles);
    for role in 0..limits.forge.roles {
        approvals
            .push(settings::RuleApproval { role, people: 1, freshness: forge::JudgeFreshness::Clean })
            .expect("bounded approvals");
    }
    let value_bytes = size(&settings::Value::Rule(settings::LandingRule {
        pattern: forge::GatePattern {
            segments: segments.into_boxed(),
            last: forge::GateLast::Exact(word(segment_bytes)),
        },
        enforce: true,
        ci: true,
        up_to_date: true,
        gates: gates.into_boxed(),
        approvals: approvals.into_boxed(),
    }));
    Box::new([
        Kind {
            number: 1,
            scope: KindScope::Deployment,
            connector,
            value_bytes,
            per_scope: limits.forge.landing_rules,
            charters: 0,
        },
        Kind {
            number: 2,
            scope: KindScope::Project,
            connector,
            value_bytes,
            per_scope: limits.forge.landing_rules,
            charters: 0,
        },
        Kind {
            number: 3,
            scope: KindScope::Deployment,
            connector,
            value_bytes: size(&settings::Value::Charter(1)),
            per_scope: 2,
            charters: 1,
        },
    ])
}

fn word(len: u32) -> Box<[u8]> {
    let mut word = List::with_capacity(len);
    for _ in 0..len {
        word.push(b'x').expect("bounded segment bytes");
    }
    word.into_boxed()
}

/// The document's rules and change charter, with the stock review charter.
#[must_use]
pub fn forge_defaults(landing: &LandingPolicy, charter: u32) -> Box<[settings::Stock]> {
    let mut stock = List::uncapped();
    rules(&mut stock, 1, 0, &landing.deployment);
    for (&project, landing) in &landing.projects {
        rules(&mut stock, 2, project, landing);
    }
    for (number, charter) in [(1, charter), (2, 4)] {
        stock
            .push(settings::Stock { kind: 3, scope: 0, number, value: sized(settings::Value::Charter(charter)) })
            .expect("uncapped stock");
    }
    stock.into_boxed()
}

fn sized(value: settings::Value) -> settings::SizedValue {
    settings::SizedValue { size: size(&value), value }
}

fn rules(stock: &mut List<settings::Stock>, kind: u32, scope: u32, rules: &[temper_engine_domain::LandingRule]) {
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
                value: sized(value),
            })
            .expect("uncapped stock");
    }
}

fn freshness(value: temper_engine_domain::Freshness) -> forge::JudgeFreshness {
    match value {
        temper_engine_domain::Freshness::Exact => forge::JudgeFreshness::Exact,
        temper_engine_domain::Freshness::Clean => forge::JudgeFreshness::Clean,
    }
}
