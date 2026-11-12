//! Values of temper's kinds, typed at the entrance
//! (temper/docs/design/domain/routing.md, 6;
//! temper/docs/design/protocol/configuration/settings.md, 2).

use alloc::boxed::Box;
use skein_lib::{List, Reader, Writer};
use temper_client_protocol_wire::{self as wire, v1};
use temper_engine_domain_forge::{GateLast, GatePattern, JudgeFreshness, settings};
use temper_engine_domain_forge_change::Route;

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
        6 => {
            let record = v1::ProjectQuestions::decode(&v1::CEILINGS, &mut reader).ok()?;
            settings::Value::Questions(match record.route() {
                v1::QuestionsRoute::None => Route::None,
                v1::QuestionsRoute::Up => Route::Up,
            })
        }
        7 => {
            let v1::ProjectWaysParts { drafts, goals, changes, way } =
                v1::ProjectWays::decode(&v1::CEILINGS, &mut reader).ok()?.into_parts();
            settings::Value::Ways(settings::ProjectWays { drafts, goals, changes, way })
        }
        8 => {
            let record = v1::ProjectClasses::decode(&v1::CEILINGS, &mut reader).ok()?;
            settings::Value::Classes(temper_protocol_reaches::classes_from(&record))
        }
        9 => {
            let record = v1::CharterWords::decode(&v1::CEILINGS, &mut reader).ok()?;
            settings::Value::CharterWords(record.into_parts().line)
        }
        10 => {
            let parts = v1::ProjectWords::decode(&v1::CEILINGS, &mut reader).ok()?.into_parts();
            settings::Value::ProjectWords(settings::ProjectWords { headline: parts.headline, lede: parts.lede })
        }
        12 => settings::Value::Follow(settings::follow_from(v1::Follow::decode(&v1::CEILINGS, &mut reader).ok()?)),
        11 => {
            let record = v1::TemplateGoal::decode(&v1::CEILINGS, &mut reader).ok()?;
            settings::Value::Goal(settings::TemplateGoal { design: record.design(), close_out: record.close_out() })
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
        settings::Value::Questions(route) => {
            let record = wire_questions(*route);
            let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
            record.encode(&mut writer).expect("measured room");
            writer.finish()
        }
        settings::Value::Classes(classes) => {
            let record = temper_protocol_reaches::classes_record(classes).expect("admitted classes fit the record");
            let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
            record.encode(&mut writer).expect("measured room");
            writer.finish()
        }
        settings::Value::Follow(follow) => {
            let record = settings::follow_record(follow).expect("admitted follow fits the record");
            let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
            record.encode(&mut writer).expect("measured room");
            writer.finish()
        }
        settings::Value::Goal(goal) => {
            let record = wire_goal(*goal);
            let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
            record.encode(&mut writer).expect("measured room");
            writer.finish()
        }
        settings::Value::Ways(ways) => {
            let record = wire_ways(*ways);
            let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
            record.encode(&mut writer).expect("measured room");
            writer.finish()
        }
        settings::Value::CharterWords(line) => {
            let record = wire_charter_words(line);
            let mut writer = Writer::new(usize::try_from(record.measure()).expect("measured length fits usize"));
            record.encode(&mut writer).expect("measured room");
            writer.finish()
        }
        settings::Value::ProjectWords(words) => {
            let record = wire_project_words(words);
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
        settings::Value::Questions(route) => wire_questions(*route).measure(),
        settings::Value::Ways(ways) => wire_ways(*ways).measure(),
        settings::Value::Classes(classes) => {
            temper_protocol_reaches::classes_record(classes).expect("admitted classes fit the record").measure()
        }
        settings::Value::CharterWords(line) => wire_charter_words(line).measure(),
        settings::Value::ProjectWords(words) => wire_project_words(words).measure(),
        settings::Value::Goal(goal) => wire_goal(*goal).measure(),
        settings::Value::Follow(follow) => {
            settings::follow_record(follow).expect("admitted follow fits the record").measure()
        }
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
        sets: parts.sets.into_boxed(),
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
            sets: List::from_boxed(rule.sets.clone()),
        },
    )
    .expect("rule within schema ceilings")
}

fn wire_charter(charter: u32) -> v1::ForgeCharter {
    v1::ForgeCharter::new(&v1::CEILINGS, v1::ForgeCharterParts { charter }).expect("fixed charter field")
}

fn wire_questions(route: Route) -> v1::ProjectQuestions {
    let route = match route {
        Route::None => v1::QuestionsRoute::None,
        Route::Up => v1::QuestionsRoute::Up,
    };
    v1::ProjectQuestions::new(&v1::CEILINGS, v1::ProjectQuestionsParts { route }).expect("fixed route field")
}

fn wire_ways(ways: settings::ProjectWays) -> v1::ProjectWays {
    v1::ProjectWays::new(
        &v1::CEILINGS,
        v1::ProjectWaysParts { drafts: ways.drafts, goals: ways.goals, changes: ways.changes, way: ways.way },
    )
    .expect("fixed ways fields")
}

fn wire_charter_words(line: &[u8]) -> v1::CharterWords {
    v1::CharterWords::new(&v1::CEILINGS, v1::CharterWordsParts { line: Box::from(line) })
        .expect("line within schema ceiling")
}

fn wire_project_words(words: &settings::ProjectWords) -> v1::ProjectWords {
    v1::ProjectWords::new(
        &v1::CEILINGS,
        v1::ProjectWordsParts { headline: words.headline.clone(), lede: words.lede.clone() },
    )
    .expect("words within schema ceilings")
}

// A goal template's work (temper/docs/design/protocol/configuration/settings.md, 2).
fn wire_goal(goal: settings::TemplateGoal) -> v1::TemplateGoal {
    v1::TemplateGoal::new(&v1::CEILINGS, v1::TemplateGoalParts { design: goal.design, close_out: goal.close_out })
        .expect("fixed goal fields")
}
