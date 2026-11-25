//! Values of temper's kinds, typed at the entrance
//! (temper/docs/design/domain/routing.md, 6;
//! temper/docs/design/protocol/configuration/settings-kinds.md, 1).

use crate::generated::client_wire::{
    dependency_to_domain, dependency_to_record, landing_rule_to_domain, landing_rule_to_record,
    project_areas_to_domain, project_areas_to_record, project_triager_to_domain, project_triager_to_record,
    project_ways_to_domain, project_ways_to_record, project_words_to_domain, project_words_to_record,
    questions_route_to_domain, questions_route_to_record, template_goal_to_domain, template_goal_to_record,
    welcome_to_domain, welcome_to_record,
};
use alloc::boxed::Box;
use skein_lib::Reader;
use temper_client_protocol_wire::v1::Record;
use temper_client_protocol_wire::{self as wire, v1};
use temper_engine_domain_forge::settings;
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
            settings::Value::Rule(landing_rule_to_domain(record))
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
            settings::Value::Questions(questions_route_to_domain(record.route))
        }
        7 => {
            let record = v1::ProjectWays::decode(&v1::CEILINGS, &mut reader).ok()?;
            settings::Value::Ways(project_ways_to_domain(record))
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
            settings::Value::ProjectWords(project_words_to_domain(parts))
        }
        13 => {
            let record = v1::Dependency::decode(&v1::CEILINGS, &mut reader).ok()?;
            settings::Value::Dependency(dependency_to_domain(record))
        }
        12 => settings::Value::Follow(settings::follow_from(v1::Follow::decode(&v1::CEILINGS, &mut reader).ok()?)),
        11 => {
            let record = v1::TemplateGoal::decode(&v1::CEILINGS, &mut reader).ok()?;
            settings::Value::Goal(template_goal_to_domain(record))
        }
        14 => {
            let record = v1::ProjectAreas::decode(&v1::CEILINGS, &mut reader).ok()?;
            settings::Value::ProjectAreas(project_areas_to_domain(record))
        }
        21 => settings::Value::AgentFigures(v1::AgentFigures::decode(&v1::CEILINGS, &mut reader).ok()?),
        15 => {
            let record = v1::ProjectTriager::decode(&v1::CEILINGS, &mut reader).ok()?;
            settings::Value::ProjectTriager(project_triager_to_domain(record))
        }
        22 => {
            settings::Value::Limits(settings::limits_from(v1::LimitOverrides::decode(&v1::CEILINGS, &mut reader).ok()?))
        }
        20 => {
            let record = v1::Welcome::decode(&v1::CEILINGS, &mut reader).ok()?;
            settings::Value::Welcome(welcome_to_domain(record))
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
            let record = welcome_to_record(value);
            record.encode(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::ProjectAreas(value) => {
            let record = project_areas_to_record(value).expect("admitted area count");
            record.encode(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::ProjectTriager(value) => {
            let record = project_triager_to_record(value);
            record.encode(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::Rule(rule) => {
            let record = landing_rule_to_record(rule).expect("admitted landing rule counts");
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
            let record = dependency_to_record(value);
            record.encode(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::Follow(follow) => {
            let record = settings::follow_record(follow).expect("admitted follow fits the record");
            record.encode(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::Goal(goal) => {
            let record = template_goal_to_record(goal);
            record.encode(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::Ways(ways) => {
            let record = project_ways_to_record(ways);
            record.encode(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::CharterWords(line) => {
            let record = wire_charter_words(line);
            record.encode(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::ProjectWords(words) => {
            let record = project_words_to_record(words);
            record.encode(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
    }
}

/// The encoding's byte length, as counted for a stock default.
#[must_use]
pub fn size(value: &settings::Value) -> u32 {
    match value {
        settings::Value::AgentFigures(record) => {
            record.measure(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::Gates(v) => {
            settings::gates_record(v).measure(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::Landing(v) => {
            settings::landing_record(v).measure(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::Rule(rule) => landing_rule_to_record(rule)
            .expect("admitted landing rule counts")
            .measure(&v1::CEILINGS)
            .expect("admitted setting within ceilings"),
        settings::Value::Charter(charter) => {
            wire_charter(*charter).measure(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::Questions(route) => {
            wire_questions(*route).measure(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::Ways(ways) => {
            project_ways_to_record(ways).measure(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::Classes(classes) => temper_protocol_reaches::classes_record(classes)
            .expect("admitted classes fit the record")
            .measure(&v1::CEILINGS)
            .expect("admitted setting within ceilings"),
        settings::Value::CharterWords(line) => {
            wire_charter_words(line).measure(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::ProjectWords(words) => {
            project_words_to_record(words).measure(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::Goal(goal) => {
            template_goal_to_record(goal).measure(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::Dependency(value) => {
            dependency_to_record(value).measure(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::Follow(follow) => settings::follow_record(follow)
            .expect("admitted follow fits the record")
            .measure(&v1::CEILINGS)
            .expect("admitted setting within ceilings"),
        settings::Value::ProjectAreas(value) => project_areas_to_record(value)
            .expect("admitted area count")
            .measure(&v1::CEILINGS)
            .expect("admitted setting within ceilings"),
        settings::Value::ProjectTriager(value) => {
            project_triager_to_record(value).measure(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::Limits(value) => {
            settings::limits_record(value).measure(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
        settings::Value::Welcome(value) => {
            welcome_to_record(value).measure(&v1::CEILINGS).expect("admitted setting within ceilings")
        }
    }
}

fn wire_charter(charter: u32) -> v1::ForgeCharter {
    v1::ForgeCharter { charter }
}

fn wire_questions(route: Route) -> v1::ProjectQuestions {
    v1::ProjectQuestions { route: questions_route_to_record(&route) }
}

fn wire_charter_words(line: &[u8]) -> v1::CharterWords {
    v1::CharterWords { line: Box::from(line) }
}
