//! Values of temper's kinds, typed at the entrance
//! (temper/docs/design/domain/root.md, 5.6;
//! temper/docs/design/protocol/configuration.md, 13.2).

use alloc::boxed::Box;
use skein_lib::{List, Reader, Writer};
use temper_client_protocol_wire::{self as wire, v1};
use temper_engine_domain_forge::{GateLast, GatePattern, JudgeFreshness, settings};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bounds {
    pub segments: u32,
    pub segment_bytes: u32,
    pub gates: u32,
    pub approvals: u32,
}

/// Decode one complete value within the receiving side's bounds.
#[must_use]
pub fn decode(kind: u32, bytes: &[u8], bounds: &Bounds) -> Option<settings::SizedValue> {
    let size = u32::try_from(bytes.len()).ok()?;
    let mut reader = Reader::new(bytes);
    let value = match kind {
        1 | 2 => {
            let limits = wire::limits(&wire::Bounds {
                segments: bounds.segments,
                segment_bytes: bounds.segment_bytes,
                gates: bounds.gates,
                approvals: bounds.approvals,
                field_bytes: 0,
                checks: 0,
                contexts: 0,
                refusal_bytes: 0,
            })?;
            let record = v1::LandingRule::decode(&limits, &mut reader).ok()?;
            settings::Value::Rule(rule(record))
        }
        3 => {
            let record = v1::ForgeCharter::decode(&v1::CEILINGS, &mut reader).ok()?;
            settings::Value::Charter(record.charter())
        }
        _ => return None,
    };
    if !reader.is_empty() {
        return None;
    }
    Some(settings::SizedValue { value, size })
}

/// Encode a typed value within the wire's schema ceilings.
#[must_use]
pub fn encode(value: &settings::Value) -> Box<[u8]> {
    match value {
        settings::Value::Rule(rule) => {
            let record = wire_rule(rule);
            let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
            record.encode(&mut writer).expect("measured room");
            writer.finish()
        }
        settings::Value::Charter(charter) => {
            let record = wire_charter(*charter);
            let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
            record.encode(&mut writer).expect("measured room");
            writer.finish()
        }
    }
}

/// The encoding's byte length, as counted for a stock default.
#[must_use]
pub fn size(value: &settings::Value) -> u32 {
    match value {
        settings::Value::Rule(rule) => wire_rule(rule).measure(),
        settings::Value::Charter(charter) => wire_charter(*charter).measure(),
    }
}

fn rule(record: v1::LandingRule) -> settings::LandingRule {
    let parts = record.into_parts();
    let pattern = parts.pattern.into_parts();
    let last = match pattern.last {
        v1::LandingLast::Exact(last) => GateLast::Exact(last.into_parts().value),
        v1::LandingLast::Open(last) => GateLast::Open(last.into_parts().value),
    };
    let mut gates = List::with_capacity(parts.gates.len());
    for gate in parts.gates.into_boxed() {
        gates
            .push(settings::RuleGate {
                number: gate.number(),
                blocking: gate.blocking(),
                freshness: freshness(gate.freshness()),
            })
            .expect("decoded gate count");
    }
    let mut approvals = List::with_capacity(parts.approvals.len());
    for approval in parts.approvals.into_boxed() {
        approvals
            .push(settings::RuleApproval {
                role: approval.role(),
                people: approval.people(),
                freshness: freshness(approval.freshness()),
            })
            .expect("decoded approval count");
    }
    settings::LandingRule {
        pattern: GatePattern { segments: pattern.segments.into_boxed(), last },
        enforce: parts.enforce,
        ci: parts.ci,
        up_to_date: parts.up_to_date,
        gates: gates.into_boxed(),
        approvals: approvals.into_boxed(),
    }
}

fn freshness(value: &v1::RuleFreshness) -> JudgeFreshness {
    match value {
        v1::RuleFreshness::Exact => JudgeFreshness::Exact,
        v1::RuleFreshness::Clean => JudgeFreshness::Clean,
    }
}

fn wire_freshness(value: JudgeFreshness) -> v1::RuleFreshness {
    match value {
        JudgeFreshness::Exact => v1::RuleFreshness::Exact,
        JudgeFreshness::Clean => v1::RuleFreshness::Clean,
    }
}

fn wire_rule(rule: &settings::LandingRule) -> v1::LandingRule {
    let limits = &v1::CEILINGS;
    let mut segments = List::with_capacity(u32::try_from(rule.pattern.segments.len()).expect("bounded segments"));
    for segment in &rule.pattern.segments {
        segments.push(segment.clone()).expect("counted segments");
    }
    let last = match &rule.pattern.last {
        GateLast::Exact(value) => v1::LandingLast::Exact(
            v1::LandingExact::new(limits, v1::LandingExactParts { value: value.clone() })
                .expect("last within schema ceiling"),
        ),
        GateLast::Open(value) => v1::LandingLast::Open(
            v1::LandingOpen::new(limits, v1::LandingOpenParts { value: value.clone() })
                .expect("last within schema ceiling"),
        ),
    };
    let pattern = v1::LandingPattern::new(limits, v1::LandingPatternParts { segments, last })
        .expect("pattern within schema ceilings");
    let mut gates = List::with_capacity(u32::try_from(rule.gates.len()).expect("bounded gates"));
    for gate in &rule.gates {
        let record = v1::RuleGate::new(
            limits,
            v1::RuleGateParts {
                number: gate.number,
                blocking: gate.blocking,
                freshness: wire_freshness(gate.freshness),
            },
        )
        .expect("fixed gate fields");
        gates.push(record).expect("counted gates");
    }
    let mut approvals = List::with_capacity(u32::try_from(rule.approvals.len()).expect("bounded approvals"));
    for approval in &rule.approvals {
        let record = v1::RuleApproval::new(
            limits,
            v1::RuleApprovalParts {
                role: approval.role,
                people: approval.people,
                freshness: wire_freshness(approval.freshness),
            },
        )
        .expect("fixed approval fields");
        approvals.push(record).expect("counted approvals");
    }
    v1::LandingRule::new(
        limits,
        v1::LandingRuleParts {
            pattern,
            enforce: rule.enforce,
            ci: rule.ci,
            up_to_date: rule.up_to_date,
            gates,
            approvals,
        },
    )
    .expect("rule within schema ceilings")
}

fn wire_charter(charter: u32) -> v1::ForgeCharter {
    v1::ForgeCharter::new(&v1::CEILINGS, v1::ForgeCharterParts { charter }).expect("fixed charter field")
}
