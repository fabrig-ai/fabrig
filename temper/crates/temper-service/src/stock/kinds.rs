//! temper's kind declarations
//! (temper/docs/design/protocol/configuration/settings.md, 2 and temper/docs/design/protocol/configuration/stock.md and temper/docs/design/protocol/configuration/settings.md, 5).
use alloc::boxed::Box;
use jig_domain_core::settings::{Kind, KindScope};
use skein_lib::{List, bytes};
use temper_engine_domain::Limits;
use temper_engine_domain_forge::{self as forge, settings};
use temper_engine_smith::kinds::size;

/// Declare the forge's kinds at the same bounds as its settings judge.
#[must_use]
#[expect(clippy::too_many_lines, reason = "each setting kind is declared with its admission bounds")]
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
        sets: Box::new([1, 2, 3, 4, 5, 6]),
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
    let mut paths = List::with_capacity(limits.forge.follow_paths);
    for _ in 0..limits.forge.follow_paths {
        paths.push(word(256)).expect("bounded follow paths");
    }
    let follow_bytes = size(&settings::Value::Follow(settings::Follow {
        source: settings::FollowSource {
            repository: skein_forge::api::Repository { forge: 1, repository: 1 },
            branch: word(limits.forge.name_bytes),
            paths: paths.into_boxed(),
        },
        target: settings::FollowTarget {
            repository: skein_forge::api::Repository { forge: 1, repository: 2 },
            branch: word(limits.forge.name_bytes),
        },
        owner: settings::FollowOwner::Person(1),
        start: settings::FollowStart::Code,
        producer: Some(word(8192)),
        check: settings::FollowCheck {
            workflow: word(256),
            artifact: word(256),
            format: temper_engine_domain_forge_tiers::Format::Lines,
        },
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
        Kind {
            number: 6,
            scope: KindScope::Project,
            connector,
            value_bytes: size(&settings::Value::Questions(temper_engine_domain_forge_change::Route::None)),
            per_scope: 1,
            charters: 0,
        },
        Kind {
            number: 7,
            scope: KindScope::Project,
            connector,
            value_bytes: size(&settings::Value::Ways(settings::ProjectWays {
                drafts: true,
                goals: false,
                changes: false,
                way: 0,
            })),
            per_scope: 1,
            charters: 0,
        },
        Kind {
            number: 8,
            scope: KindScope::Project,
            connector,
            value_bytes: size(&settings::Value::Classes(temper_protocol_reaches::largest())),
            per_scope: 1,
            charters: 0,
        },
        Kind {
            number: 9,
            scope: KindScope::Deployment,
            connector,
            value_bytes: size(&settings::Value::CharterWords(word(settings::LINE_BYTES))),
            per_scope: limits.tasks.charters,
            charters: 0,
        },
        Kind {
            number: 10,
            scope: KindScope::Project,
            connector,
            value_bytes: size(&settings::Value::ProjectWords(settings::ProjectWords {
                headline: word(settings::HEADLINE_BYTES),
                lede: word(settings::LEDE_BYTES),
            })),
            per_scope: 1,
            charters: 0,
        },
        Kind {
            number: 11,
            scope: KindScope::Deployment,
            connector,
            value_bytes: size(&settings::Value::Goal(settings::TemplateGoal { design: true, close_out: true })),
            per_scope: limits.templates,
            charters: 0,
        },
        Kind {
            number: 12,
            scope: KindScope::Project,
            connector,
            value_bytes: follow_bytes,
            per_scope: limits.forge.follows,
            charters: 1,
        },
        Kind {
            number: 13,
            scope: KindScope::Project,
            connector,
            value_bytes: size(&settings::Value::Dependency(settings::Dependency { project: 1 })),
            per_scope: limits.forge.dependencies,
            charters: 0,
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
