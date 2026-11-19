//! Values of temper's kinds, typed at the entrance
//! (temper/docs/design/domain/routing.md, 6;
//! temper/docs/design/protocol/configuration/settings.md, 2).

use temper_client_protocol_wire::v1::Record;
use alloc::boxed::Box;
use skein_lib::{List, Reader};
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
        4 => settings::Value::Landing(settings::landing_from(
            v1::ProjectLanding::decode(&v1::CEILINGS, &mut reader).ok()?,
        )),
        5 => settings::Value::Gates(settings::gates_from(v1::ProjectGates::decode(&v1::CEILINGS, &mut reader).ok()?)?),
        3 => {
            let record = v1::ForgeCharter::decode(&v1::CEILINGS, &mut reader).ok()?;
            settings::Value::Charter(record.charter)
        }
        6 => {
            let record = v1::ProjectQuestions::decode(&v1::CEILINGS, &mut reader).ok()?;
            settings::Value::Questions(match &record.route {
                v1::QuestionsRoute::None => Route::None,
                v1::QuestionsRoute::Up => Route::Up,
            })
        }
        7 => {
            let v1::ProjectWays { drafts, goals, changes, way } =
                v1::ProjectWays::decode(&v1::CEILINGS, &mut reader).ok()?;
            settings::Value::Ways(settings::ProjectWays { drafts, goals, changes, way })
        }
        8 => {
            let record = v1::ProjectClasses::decode(&v1::CEILINGS, &mut reader).ok()?;
            settings::Value::Classes(temper_protocol_reaches::classes_from(&record))
        }
        9 => {
            let record = v1::CharterWords::decode(&v1::CEILINGS, &mut reader).ok()?;
            settings::Value::CharterWords(record.line)
        }
        10 => {
            let parts = v1::ProjectWords::decode(&v1::CEILINGS, &mut reader).ok()?;
            settings::Value::ProjectWords(settings::ProjectWords { headline: parts.headline, lede: parts.lede })
        }
        13 => settings::Value::Dependency(settings::Dependency {
            project: v1::Dependency::decode(&v1::CEILINGS, &mut reader).ok()?.project,
        }),
        12 => settings::Value::Follow(settings::follow_from(v1::Follow::decode(&v1::CEILINGS, &mut reader).ok()?)),
        11 => {
            let record = v1::TemplateGoal::decode(&v1::CEILINGS, &mut reader).ok()?;
            settings::Value::Goal(settings::TemplateGoal { design: record.design, close_out: record.close_out })
        }
        14 => {
            let record = v1::ProjectAreas::decode(&v1::CEILINGS, &mut reader).ok()?;
            settings::Value::ProjectAreas(areas_from(record))
        }
        21 => settings::Value::AgentFigures(v1::AgentFigures::decode(&v1::CEILINGS, &mut reader).ok()?),
        15 => {
            let record = v1::ProjectTriager::decode(&v1::CEILINGS, &mut reader).ok()?;
            settings::Value::ProjectTriager(settings::ProjectTriager { owner: owner_from(record.owner) })
        }
        22 => {
            settings::Value::Limits(settings::limits_from(v1::LimitOverrides::decode(&v1::CEILINGS, &mut reader).ok()?))
        }
        20 => {
            let record = v1::Welcome::decode(&v1::CEILINGS, &mut reader).ok()?;
            settings::Value::Welcome(settings::Welcome { shown: record.shown })
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
        settings::Value::AgentFigures(record) => {
            record.encode(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::Gates(value) => {
            let record = settings::gates_record(value);
            record.encode(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::Landing(value) => {
            let record = settings::landing_record(value);
            record.encode(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::Limits(value) => {
            let record = settings::limits_record(value);
            record.encode(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::Welcome(value) => {
            let record = wire_welcome(*value);
            record.encode(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::ProjectAreas(value) => {
            let record = wire_areas(value);
            record.encode(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::ProjectTriager(value) => {
            let record = wire_triager(value);
            record.encode(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::Rule(rule) => {
            let record = wire_rule(rule);
            record.encode(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::Charter(charter) => {
            let record = wire_charter(*charter);
            record.encode(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::Questions(route) => {
            let record = wire_questions(*route);
            record.encode(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::Classes(classes) => {
            let record = temper_protocol_reaches::classes_record(classes).expect("admitted classes fit the record");
            record.encode(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::Dependency(value) => {
            let record = wire_dependency(*value);
            record.encode(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::Follow(follow) => {
            let record = settings::follow_record(follow).expect("admitted follow fits the record");
            record.encode(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::Goal(goal) => {
            let record = wire_goal(*goal);
            record.encode(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::Ways(ways) => {
            let record = wire_ways(*ways);
            record.encode(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::CharterWords(line) => {
            let record = wire_charter_words(line);
            record.encode(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::ProjectWords(words) => {
            let record = wire_project_words(words);
            record.encode(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
    }
}

/// The encoding's byte length, as counted for a stock default.
#[must_use]
pub fn size(value: &settings::Value) -> u32 {
    match value {
        settings::Value::AgentFigures(record) => record.measure(&v1::CEILINGS).expect("admitted setting within ceilings"),
        settings::Value::Gates(v) => settings::gates_record(v).measure(&v1::CEILINGS).expect("admitted setting within ceilings"),
        settings::Value::Landing(v) => settings::landing_record(v).measure(&v1::CEILINGS).expect("admitted setting within ceilings"),
        settings::Value::Rule(rule) => wire_rule(rule).measure(&v1::CEILINGS).expect("admitted setting within ceilings"),
        settings::Value::Charter(charter) => wire_charter(*charter).measure(&v1::CEILINGS).expect("admitted setting within ceilings"),
        settings::Value::Questions(route) => wire_questions(*route).measure(&v1::CEILINGS).expect("admitted setting within ceilings"),
        settings::Value::Ways(ways) => wire_ways(*ways).measure(&v1::CEILINGS).expect("admitted setting within ceilings"),
        settings::Value::Classes(classes) => {
            temper_protocol_reaches::classes_record(classes).expect("admitted classes fit the record").measure(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::CharterWords(line) => wire_charter_words(line).measure(&v1::CEILINGS).expect("admitted setting within ceilings"),
        settings::Value::ProjectWords(words) => wire_project_words(words).measure(&v1::CEILINGS).expect("admitted setting within ceilings"),
        settings::Value::Goal(goal) => wire_goal(*goal).measure(&v1::CEILINGS).expect("admitted setting within ceilings"),
        settings::Value::Dependency(value) => wire_dependency(*value).measure(&v1::CEILINGS).expect("admitted setting within ceilings"),
        settings::Value::Follow(follow) => {
            settings::follow_record(follow).expect("admitted follow fits the record").measure(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::ProjectAreas(value) => wire_areas(value).measure(&v1::CEILINGS).expect("admitted setting within ceilings"),
        settings::Value::ProjectTriager(value) => wire_triager(value).measure(&v1::CEILINGS).expect("admitted setting within ceilings"),
        settings::Value::Limits(value) => settings::limits_record(value).measure(&v1::CEILINGS).expect("admitted setting within ceilings"),
        settings::Value::Welcome(value) => wire_welcome(*value).measure(&v1::CEILINGS).expect("admitted setting within ceilings"),
    }
}

fn wire_dependency(value: settings::Dependency) -> v1::Dependency {
    v1::Dependency { project: value.project }
}

fn rule(record: v1::LandingRule) -> settings::LandingRule {
    let parts = record;
    let pattern = parts.pattern;
    let last = match pattern.last {
        v1::LandingLast::Exact(last) => GateLast::Exact(last.value),
        v1::LandingLast::Open(last) => GateLast::Open(last.value),
    };
    let mut gates = List::with_capacity(parts.gates.len());
    for gate in parts.gates.into_boxed() {
        gates
            .push(settings::RuleGate {
                number: gate.number,
                blocking: gate.blocking,
                freshness: freshness(&gate.freshness),
            })
            .expect("decoded gate count");
    }
    let mut approvals = List::with_capacity(parts.approvals.len());
    for approval in parts.approvals.into_boxed() {
        approvals
            .push(settings::RuleApproval {
                role: approval.role,
                people: approval.people,
                freshness: freshness(&approval.freshness),
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
    let mut segments = List::with_capacity(u32::try_from(rule.pattern.segments.len()).expect("bounded segments"));
    for segment in &rule.pattern.segments {
        segments.push(segment.clone()).expect("counted segments");
    }
    let last = match &rule.pattern.last {
        GateLast::Exact(value) => v1::LandingLast::Exact(
            v1::LandingExact { value: value.clone() },
        ),
        GateLast::Open(value) => v1::LandingLast::Open(
            v1::LandingOpen { value: value.clone() },
        ),
    };
    let pattern = v1::LandingPattern { segments, last };
    let mut gates = List::with_capacity(u32::try_from(rule.gates.len()).expect("bounded gates"));
    for gate in &rule.gates {
        let record = v1::RuleGate {
                number: gate.number,
                blocking: gate.blocking,
                freshness: wire_freshness(gate.freshness),
            };
        gates.push(record).expect("counted gates");
    }
    let mut approvals = List::with_capacity(u32::try_from(rule.approvals.len()).expect("bounded approvals"));
    for approval in &rule.approvals {
        let record = v1::RuleApproval {
                role: approval.role,
                people: approval.people,
                freshness: wire_freshness(approval.freshness),
            };
        approvals.push(record).expect("counted approvals");
    }
    v1::LandingRule {
            pattern,
            enforce: rule.enforce,
            ci: rule.ci,
            up_to_date: rule.up_to_date,
            gates,
            approvals,
            sets: List::from_boxed(rule.sets.clone()),
        }
}

fn wire_charter(charter: u32) -> v1::ForgeCharter {
    v1::ForgeCharter { charter }
}

fn wire_questions(route: Route) -> v1::ProjectQuestions {
    let route = match route {
        Route::None => v1::QuestionsRoute::None,
        Route::Up => v1::QuestionsRoute::Up,
    };
    v1::ProjectQuestions { route }
}

fn wire_ways(ways: settings::ProjectWays) -> v1::ProjectWays {
    v1::ProjectWays { drafts: ways.drafts, goals: ways.goals, changes: ways.changes, way: ways.way }
}

fn wire_charter_words(line: &[u8]) -> v1::CharterWords {
    v1::CharterWords { line: Box::from(line) }
}

fn wire_project_words(words: &settings::ProjectWords) -> v1::ProjectWords {
    v1::ProjectWords { headline: words.headline.clone(), lede: words.lede.clone() }
}

// A goal template's work (temper/docs/design/protocol/configuration/settings.md, 2).
fn wire_goal(goal: settings::TemplateGoal) -> v1::TemplateGoal {
    v1::TemplateGoal { design: goal.design, close_out: goal.close_out }
}

fn owner_record(value: settings::AreaOwner) -> v1::AreaOwner {
    match value {
        settings::AreaOwner::Task(number) => v1::AreaOwner::Task(
            v1::AreaTask { number },
        ),
        settings::AreaOwner::Party(number) => v1::AreaOwner::Party(
            v1::AreaParty { number },
        ),
        settings::AreaOwner::Role(number) => v1::AreaOwner::Role(
            v1::AreaRole { number },
        ),
        settings::AreaOwner::Charter(number) => v1::AreaOwner::Charter(
            v1::AreaCharter { number },
        ),
    }
}
fn owner_from(value: v1::AreaOwner) -> settings::AreaOwner {
    match value {
        v1::AreaOwner::Task(number) => settings::AreaOwner::Task(number.number),
        v1::AreaOwner::Party(number) => settings::AreaOwner::Party(number.number),
        v1::AreaOwner::Role(number) => settings::AreaOwner::Role(number.number),
        v1::AreaOwner::Charter(number) => settings::AreaOwner::Charter(number.number),
    }
}
fn wire_areas(value: &settings::ProjectAreas) -> v1::ProjectAreas {
    let mut areas = List::with_capacity(u32::try_from(value.areas.len()).expect("admitted area count"));
    for area in &value.areas {
        areas
            .push(
                v1::Area {
                        name: area.name.clone(),
                        words: area.words.clone(),
                        owner: owner_record(area.owner),
                    },
            )
            .expect("counted areas");
    }
    v1::ProjectAreas { areas }
}
fn wire_triager(value: &settings::ProjectTriager) -> v1::ProjectTriager {
    v1::ProjectTriager { owner: owner_record(value.owner) }
}
fn areas_from(value: v1::ProjectAreas) -> settings::ProjectAreas {
    let mut areas = List::uncapped();
    for area in value.areas.into_boxed() {
        let v1::Area { name, words, owner } = area;
        areas.push(settings::Area { name, words, owner: owner_from(owner) }).expect("decoded areas");
    }
    settings::ProjectAreas { areas: areas.into_boxed() }
}

fn wire_welcome(value: settings::Welcome) -> v1::Welcome {
    v1::Welcome { shown: value.shown }
}
