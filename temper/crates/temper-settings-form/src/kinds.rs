//! Temper's settings file values and codec records
//! (temper/docs/design/client/ctl/tools.md, 3; temper/docs/design/protocol/configuration/settings.md, 2).
use crate::file::{Decoder, Error, Fields, error, field, optional};
use crate::view as c;
use alloc::boxed::Box;
use skein_json::writer::Encoder;
use skein_lib::{List, Reader, Writer};
use temper_client_protocol_wire::v1 as w;
mod follow;

fn dependency(d: &Decoder<'_>, at: u32) -> Result<w::Dependency, Error> {
    let fields = d.object(at, &[b"project"])?;
    w::Dependency::new(&d.kinds, w::DependencyParts { project: number(d, &fields, b"project")? })
        .or(Err(error(b"", b"wire value bound")))
}

fn hex(d: &Decoder<'_>, at: u32, most: u32, name: &[u8]) -> Result<Box<[u8]>, Error> {
    match d.hex(at, most, name) {
        Ok(value) => Ok(value),
        Err(why) => Err(why.named(name)),
    }
}
fn number(d: &Decoder<'_>, fields: &Fields, name: &[u8]) -> Result<u32, Error> {
    let at = field(fields, name)?;
    let value = match d.number(at) {
        Ok(value) => value,
        Err(why) => return Err(why.at(name)),
    };
    u32::try_from(value).or(Err(error(name, b"u32 bound")))
}
fn boolean(d: &Decoder<'_>, fields: &Fields, name: &[u8]) -> Result<bool, Error> {
    let at = field(fields, name)?;
    match d.boolean(at) {
        Ok(value) => Ok(value),
        Err(why) => Err(why.at(name)),
    }
}
pub(super) fn setting(d: &Decoder<'_>, fields: &Fields, kind: &[u8]) -> Result<c::Setting, Error> {
    let value = setting_value(d, fields, kind);
    match value {
        Ok(value) => Ok(value),
        Err(why) => Err(why.at(kind)),
    }
}
fn setting_value(d: &Decoder<'_>, fields: &Fields, kind: &[u8]) -> Result<c::Setting, Error> {
    if [
        b"project-ways".as_slice(),
        b"project-questions",
        b"project-classes",
        b"project-words",
        b"project-areas",
        b"project-triager",
    ]
    .contains(&kind)
    {
        if optional(fields, b"number").is_some() {
            return Err(error(b"number", b"project setting has no number"));
        }
        let project = number(d, fields, b"project")?;
        if project == 0 {
            return Err(error(b"project", b"from 1"));
        }
        return Ok(c::Setting::Application {
            kind: match kind {
                b"project-ways" => 7,
                b"project-questions" => 6,
                b"project-classes" => 8,
                b"project-words" => 10,
                b"project-areas" => 14,
                b"project-triager" => 15,
                _ => unreachable!("project kind selected"),
            },
            scope: project,
            number: 1,
        });
    }
    if kind == b"follow" || kind == b"dependency" {
        let project = number(d, fields, b"project")?;
        let number = number(d, fields, b"number")?;
        if project == 0 {
            return Err(error(b"project", b"from 1"));
        }
        if number == 0 {
            return Err(error(b"number", b"from 1"));
        }
        return Ok(c::Setting::Application { kind: if kind == b"follow" { 12 } else { 13 }, scope: project, number });
    }
    let number = number(d, fields, b"number")?;
    if number == 0 {
        return Err(error(b"number", b"from 1"));
    }
    let project = optional(fields, b"project");
    if kind == b"forge-charter" || kind == b"charter-words" || kind == b"template-goal" {
        if project.is_some() {
            return Err(error(b"project", b"unknown member"));
        }
        return Ok(c::Setting::Application {
            kind: if kind == b"charter-words" {
                9
            } else if kind == b"template-goal" {
                11
            } else {
                3
            },
            scope: 0,
            number,
        });
    }
    let scope = if project.is_some() {
        let project = self::number(d, fields, b"project")?;
        if project == 0 {
            return Err(error(b"project", b"from 1"));
        }
        project
    } else {
        0
    };
    Ok(c::Setting::Application { kind: if scope == 0 { 1 } else { 2 }, scope, number })
}
pub(super) fn value(d: &Decoder<'_>, at: u32, kind: u32) -> Result<Box<[u8]>, Error> {
    if kind == 14 {
        let record = match areas(d, at) {
            Ok(record) => record,
            Err(why) => return Err(why.at(b"project-areas")),
        };
        let mut out = Writer::new(usize::try_from(record.measure()).expect("record length fits"));
        record.encode(&mut out).expect("measured record");
        return Ok(out.finish());
    }
    if kind == 15 {
        let fields = d.object(at, &[b"owner"])?;
        let owner = area_owner(d, field(&fields, b"owner")?)?;
        let record = w::ProjectTriager::new(&d.kinds, w::ProjectTriagerParts { owner })
            .or(Err(error(b"project-triager", b"wire value bound")))?;
        let mut out = Writer::new(usize::try_from(record.measure()).expect("record length fits"));
        record.encode(&mut out).expect("measured record");
        return Ok(out.finish());
    }
    if kind == 13 {
        let record = match dependency(d, at) {
            Ok(value) => value,
            Err(why) => return Err(why.at(b"dependency")),
        };
        let mut out = Writer::new(usize::try_from(record.measure()).expect("record length fits"));
        record.encode(&mut out).expect("measured record");
        return Ok(out.finish());
    }
    if kind == 12 {
        let record = match follow::read(d, at) {
            Ok(value) => value,
            Err(why) => return Err(why.at(b"follow")),
        };
        let mut out = Writer::new(usize::try_from(record.measure()).expect("record length fits"));
        record.encode(&mut out).expect("measured record");
        return Ok(out.finish());
    }
    if kind == 8 {
        let record = match crate::classes::read(d, at) {
            Ok(record) => record,
            Err(why) => return Err(why.at(b"project-classes")),
        };
        let mut out = Writer::new(usize::try_from(record.measure()).expect("record length fits"));
        record.encode(&mut out).expect("measured record");
        return Ok(out.finish());
    }
    if kind == 9 {
        let line = match d.string(at) {
            Ok(value) => value.into(),
            Err(why) => return Err(why.at(b"charter-words")),
        };
        let record = w::CharterWords::new(&d.kinds, w::CharterWordsParts { line })
            .or(Err(error(b"charter-words", b"wire value bound")))?;
        let mut out = Writer::new(usize::try_from(record.measure()).expect("record length fits"));
        record.encode(&mut out).expect("measured record");
        return Ok(out.finish());
    }
    remaining_value(d, at, kind)
}
fn remaining_value(d: &Decoder<'_>, at: u32, kind: u32) -> Result<Box<[u8]>, Error> {
    if kind == 10 {
        let record = match project_words(d, at) {
            Ok(value) => value,
            Err(why) => return Err(why.at(b"project-words")),
        };
        let mut out = Writer::new(usize::try_from(record.measure()).expect("record length fits"));
        record.encode(&mut out).expect("measured record");
        return Ok(out.finish());
    }
    if kind == 6 {
        let word = match d.string(at) {
            Ok(word) => word,
            Err(why) => return Err(why.at(b"project-questions")),
        };
        let route = match word {
            b"none" => w::QuestionsRoute::None,
            b"up" => w::QuestionsRoute::Up,
            _ => return Err(error(b"project-questions", b"none or up required")),
        };
        let record = w::ProjectQuestions::new(&d.kinds, w::ProjectQuestionsParts { route })
            .or(Err(error(b"project-questions", b"wire value bound")))?;
        let mut out = Writer::new(usize::try_from(record.measure()).expect("record length fits"));
        record.encode(&mut out).expect("measured record");
        return Ok(out.finish());
    }
    if kind == 7 {
        let record = match ways(d, at) {
            Ok(record) => record,
            Err(why) => return Err(why.at(b"project-ways")),
        };
        let mut out = Writer::new(usize::try_from(record.measure()).expect("record length fits"));
        record.encode(&mut out).expect("measured record");
        return Ok(out.finish());
    }
    if kind == 11 {
        let record = match goal(d, at) {
            Ok(record) => record,
            Err(why) => return Err(why.at(b"template-goal")),
        };
        let mut out = Writer::new(usize::try_from(record.measure()).expect("record length fits"));
        record.encode(&mut out).expect("measured record");
        return Ok(out.finish());
    }
    if kind == 3 {
        let charter = match d.number(at) {
            Ok(value) => u32::try_from(value).or(Err(error(b"forge-charter", b"u32 bound")))?,
            Err(why) => return Err(why.at(b"forge-charter")),
        };
        let record = w::ForgeCharter::new(&d.kinds, w::ForgeCharterParts { charter })
            .or(Err(error(b"forge-charter", b"wire value bound")))?;
        let mut out = Writer::new(usize::try_from(record.measure()).expect("record length fits"));
        record.encode(&mut out).expect("measured record");
        return Ok(out.finish());
    }
    let record = match rule(d, at) {
        Ok(record) => record,
        Err(why) => return Err(why.at(b"landing-rule")),
    };
    let mut out = Writer::new(usize::try_from(record.measure()).expect("record length fits"));
    record.encode(&mut out).expect("measured record");
    Ok(out.finish())
}
fn project_words(d: &Decoder<'_>, at: u32) -> Result<w::ProjectWords, Error> {
    let fields = d.object(at, &[b"headline", b"lede"])?;
    let headline = match d.string(field(&fields, b"headline")?) {
        Ok(value) => value.into(),
        Err(why) => return Err(why.at(b"headline")),
    };
    let lede = match d.string(field(&fields, b"lede")?) {
        Ok(value) => value.into(),
        Err(why) => return Err(why.at(b"lede")),
    };
    w::ProjectWords::new(&d.kinds, w::ProjectWordsParts { headline, lede }).or(Err(error(b"", b"wire value bound")))
}
fn goal(d: &Decoder<'_>, at: u32) -> Result<w::TemplateGoal, Error> {
    let fields = d.object(at, &[b"design", b"close_out"])?;
    w::TemplateGoal::new(
        &d.kinds,
        w::TemplateGoalParts { design: boolean(d, &fields, b"design")?, close_out: boolean(d, &fields, b"close_out")? },
    )
    .or(Err(error(b"", b"wire value bound")))
}
fn ways(d: &Decoder<'_>, at: u32) -> Result<w::ProjectWays, Error> {
    let fields = d.object(at, &[b"drafts", b"goals", b"changes", b"way"])?;
    w::ProjectWays::new(
        &d.kinds,
        w::ProjectWaysParts {
            drafts: boolean(d, &fields, b"drafts")?,
            goals: boolean(d, &fields, b"goals")?,
            changes: boolean(d, &fields, b"changes")?,
            way: number(d, &fields, b"way")?,
        },
    )
    .or(Err(error(b"", b"wire value bound")))
}
fn rule(d: &Decoder<'_>, at: u32) -> Result<w::LandingRule, Error> {
    let fields = d.object(at, &[b"pattern", b"enforce", b"ci", b"up_to_date", b"gates", b"approvals", b"sets"])?;
    let pattern = match pattern(d, field(&fields, b"pattern")?) {
        Ok(value) => value,
        Err(why) => return Err(why.at(b"pattern")),
    };
    let positions =
        d.array(field(&fields, b"gates")?, d.kinds.landing_rule_gates.min(w::CEILINGS.landing_rule_gates), b"gates")?;
    let mut gates = List::with_capacity(positions.len());
    for at in &positions {
        let gate = match gate(d, *at) {
            Ok(value) => value,
            Err(why) => return Err(why.at(b"gates")),
        };
        gates.push(gate).expect("counted gates");
    }
    let positions = d.array(
        field(&fields, b"approvals")?,
        d.kinds.landing_rule_approvals.min(w::CEILINGS.landing_rule_approvals),
        b"approvals",
    )?;
    let mut approvals = List::with_capacity(positions.len());
    for at in &positions {
        let approval = match approval(d, *at) {
            Ok(value) => value,
            Err(why) => return Err(why.at(b"approvals")),
        };
        approvals.push(approval).expect("counted approvals");
    }
    w::LandingRule::new(
        &d.kinds,
        w::LandingRuleParts {
            pattern,
            enforce: boolean(d, &fields, b"enforce")?,
            ci: boolean(d, &fields, b"ci")?,
            up_to_date: boolean(d, &fields, b"up_to_date")?,
            gates,
            approvals,
            sets: rule_sets(d, optional(&fields, b"sets"))?,
        },
    )
    .or(Err(error(b"", b"wire value bound")))
}
fn rule_sets(d: &Decoder<'_>, at: Option<u32>) -> Result<List<u8>, Error> {
    let Some(at) = at else { return Ok(List::with_capacity(0)) };
    let positions = d.array(at, d.kinds.landing_rule_sets.min(w::CEILINGS.landing_rule_sets), b"sets")?;
    let mut sets = List::with_capacity(positions.len());
    for at in &positions {
        let number = match d.number(*at) {
            Ok(value) => value,
            Err(why) => return Err(why.at(b"sets")),
        };
        let number = u8::try_from(number).or(Err(error(b"sets", b"u8 bound")))?;
        sets.push(number).expect("counted sets");
    }
    Ok(sets)
}
fn pattern(d: &Decoder<'_>, at: u32) -> Result<w::LandingPattern, Error> {
    let fields = d.object(at, &[b"segments", b"last"])?;
    let most =
        d.limits.pattern_segments.min(d.kinds.landing_pattern_segments).min(w::CEILINGS.landing_pattern_segments);
    let positions = d.array(field(&fields, b"segments")?, most, b"segments")?;
    let mut segments = List::with_capacity(positions.len());
    for at in &positions {
        let segment = hex(
            d,
            *at,
            d.limits
                .pattern_segments_item
                .min(d.kinds.landing_pattern_segments_item)
                .min(w::CEILINGS.landing_pattern_segments_item),
            b"segments",
        )?;
        segments.push(segment).expect("counted segments");
    }
    let last = match last(d, field(&fields, b"last")?) {
        Ok(value) => value,
        Err(why) => return Err(why.at(b"last")),
    };
    w::LandingPattern::new(&d.kinds, w::LandingPatternParts { segments, last })
        .or(Err(error(b"", b"wire pattern bound")))
}
fn last(d: &Decoder<'_>, at: u32) -> Result<w::LandingLast, Error> {
    let fields = d.object(at, &[b"exact", b"open"])?;
    if fields.len() != 1 {
        return Err(error(b"", b"one variant required"));
    }
    if let Some(at) = optional(&fields, b"exact") {
        let value = hex(
            d,
            at,
            d.limits.pattern_end_exact_value.min(d.kinds.landing_exact_value).min(w::CEILINGS.landing_exact_value),
            b"exact",
        )?;
        let record = w::LandingExact::new(&d.kinds, w::LandingExactParts { value })
            .or(Err(error(b"exact", b"wire value bound")))?;
        return Ok(w::LandingLast::Exact(record));
    }
    let value = hex(
        d,
        field(&fields, b"open")?,
        d.limits.pattern_end_open_value.min(d.kinds.landing_open_value).min(w::CEILINGS.landing_open_value),
        b"open",
    )?;
    let record =
        w::LandingOpen::new(&d.kinds, w::LandingOpenParts { value }).or(Err(error(b"open", b"wire value bound")))?;
    Ok(w::LandingLast::Open(record))
}
fn freshness(d: &Decoder<'_>, fields: &Fields) -> Result<w::RuleFreshness, Error> {
    let at = field(fields, b"freshness")?;
    let word = match d.string(at) {
        Ok(word) => word,
        Err(why) => return Err(why.at(b"freshness")),
    };
    match word {
        b"exact" => Ok(w::RuleFreshness::Exact),
        b"clean" => Ok(w::RuleFreshness::Clean),
        _ => Err(error(b"freshness", b"exact or clean required")),
    }
}
fn gate(d: &Decoder<'_>, at: u32) -> Result<w::RuleGate, Error> {
    let fields = d.object(at, &[b"number", b"blocking", b"freshness"])?;
    w::RuleGate::new(
        &d.kinds,
        w::RuleGateParts {
            number: number(d, &fields, b"number")?,
            blocking: boolean(d, &fields, b"blocking")?,
            freshness: freshness(d, &fields)?,
        },
    )
    .or(Err(error(b"", b"wire value bound")))
}
fn approval(d: &Decoder<'_>, at: u32) -> Result<w::RuleApproval, Error> {
    let fields = d.object(at, &[b"role", b"people", b"freshness"])?;
    w::RuleApproval::new(
        &d.kinds,
        w::RuleApprovalParts {
            role: number(d, &fields, b"role")?,
            people: number(d, &fields, b"people")?,
            freshness: freshness(d, &fields)?,
        },
    )
    .or(Err(error(b"", b"wire value bound")))
}

#[expect(clippy::too_many_lines, reason = "one dispatch renders each settings kind with its codec")]
pub(super) fn render(kind: u32, bytes: &[u8], out: &mut Encoder) -> bool {
    let mut reader = Reader::new(bytes);
    match kind {
        14 => {
            let Ok(value) = w::ProjectAreas::decode(&w::CEILINGS, &mut reader) else { return false };
            if !reader.is_empty() {
                return false;
            }
            out.object_start();
            out.key(b"areas");
            out.array_start();
            for area in value.areas() {
                out.object_start();
                out.key(b"name");
                out.string(area.name());
                out.key(b"words");
                out.string(area.words());
                out.key(b"owner");
                render_owner(area.owner(), out);
                out.object_end();
            }
            out.array_end();
            out.object_end();
        }
        15 => {
            let Ok(value) = w::ProjectTriager::decode(&w::CEILINGS, &mut reader) else { return false };
            if !reader.is_empty() {
                return false;
            }
            out.object_start();
            out.key(b"owner");
            render_owner(value.owner(), out);
            out.object_end();
        }
        1 | 2 => {
            let Ok(value) = w::LandingRule::decode(&w::CEILINGS, &mut reader) else { return false };
            if !reader.is_empty() {
                return false;
            }
            render_rule(&value, out);
        }
        3 => {
            let Ok(value) = w::ForgeCharter::decode(&w::CEILINGS, &mut reader) else { return false };
            if !reader.is_empty() {
                return false;
            }
            out.unsigned(u64::from(value.charter()));
        }
        6 => {
            let Ok(value) = w::ProjectQuestions::decode(&w::CEILINGS, &mut reader) else { return false };
            if !reader.is_empty() {
                return false;
            }
            out.string(match value.route() {
                w::QuestionsRoute::None => b"none",
                w::QuestionsRoute::Up => b"up",
            });
        }
        7 => {
            let Ok(value) = w::ProjectWays::decode(&w::CEILINGS, &mut reader) else { return false };
            if !reader.is_empty() {
                return false;
            }
            out.object_start();
            out.key(b"drafts");
            out.boolean(value.drafts());
            out.key(b"goals");
            out.boolean(value.goals());
            out.key(b"changes");
            out.boolean(value.changes());
            out.key(b"way");
            out.unsigned(u64::from(value.way()));
            out.object_end();
        }
        8 => {
            let Ok(value) = w::ProjectClasses::decode(&w::CEILINGS, &mut reader) else { return false };
            if !reader.is_empty() {
                return false;
            }
            return crate::classes::render(&value, out);
        }
        9 => {
            let Ok(value) = w::CharterWords::decode(&w::CEILINGS, &mut reader) else { return false };
            if !reader.is_empty() {
                return false;
            }
            out.string(value.line());
        }
        10 => {
            let Ok(value) = w::ProjectWords::decode(&w::CEILINGS, &mut reader) else { return false };
            if !reader.is_empty() {
                return false;
            }
            out.object_start();
            out.key(b"headline");
            out.string(value.headline());
            out.key(b"lede");
            out.string(value.lede());
            out.object_end();
        }
        13 => {
            let Ok(value) = w::Dependency::decode(&w::CEILINGS, &mut reader) else { return false };
            if !reader.is_empty() {
                return false;
            }
            out.object_start();
            out.key(b"project");
            out.unsigned(u64::from(value.project()));
            out.object_end();
        }
        12 => {
            let Ok(value) = w::Follow::decode(&w::CEILINGS, &mut reader) else { return false };
            if !reader.is_empty() {
                return false;
            }
            follow::render(&value, out);
        }
        11 => {
            let Ok(value) = w::TemplateGoal::decode(&w::CEILINGS, &mut reader) else { return false };
            if !reader.is_empty() {
                return false;
            }
            out.object_start();
            out.key(b"design");
            out.boolean(value.design());
            out.key(b"close_out");
            out.boolean(value.close_out());
            out.object_end();
        }
        _ => return false,
    }
    true
}
fn render_freshness(value: &w::RuleFreshness, out: &mut Encoder) {
    out.string(match value {
        w::RuleFreshness::Exact => b"exact",
        w::RuleFreshness::Clean => b"clean",
    });
}
fn render_rule(value: &w::LandingRule, out: &mut Encoder) {
    out.object_start();
    out.key(b"pattern");
    out.object_start();
    out.key(b"segments");
    out.array_start();
    for segment in value.pattern().segments() {
        crate::hex(segment, out);
    }
    out.array_end();
    out.key(b"last");
    out.object_start();
    match value.pattern().last() {
        w::LandingLast::Exact(value) => {
            out.key(b"exact");
            crate::hex(value.value(), out);
        }
        w::LandingLast::Open(value) => {
            out.key(b"open");
            crate::hex(value.value(), out);
        }
    }
    out.object_end();
    out.object_end();
    out.key(b"enforce");
    out.boolean(value.enforce());
    out.key(b"ci");
    out.boolean(value.ci());
    out.key(b"up_to_date");
    out.boolean(value.up_to_date());
    out.key(b"gates");
    out.array_start();
    for gate in value.gates() {
        out.object_start();
        out.key(b"number");
        out.unsigned(u64::from(gate.number()));
        out.key(b"blocking");
        out.boolean(gate.blocking());
        out.key(b"freshness");
        render_freshness(gate.freshness(), out);
        out.object_end();
    }
    out.array_end();
    out.key(b"approvals");
    out.array_start();
    for approval in value.approvals() {
        out.object_start();
        out.key(b"role");
        out.unsigned(u64::from(approval.role()));
        out.key(b"people");
        out.unsigned(u64::from(approval.people()));
        out.key(b"freshness");
        render_freshness(approval.freshness(), out);
        out.object_end();
    }
    out.array_end();
    if !value.sets().is_empty() {
        out.key(b"sets");
        out.array_start();
        for set in value.sets() {
            out.unsigned(u64::from(*set));
        }
        out.array_end();
    }
    out.object_end();
}

fn areas(d: &Decoder<'_>, at: u32) -> Result<w::ProjectAreas, Error> {
    let fields = d.object(at, &[b"areas"])?;
    let positions = d.array(field(&fields, b"areas")?, d.kinds.project_areas_areas.min(16), b"areas")?;
    let mut areas = List::with_capacity(positions.len());
    for at in &positions {
        let fields = d.object(*at, &[b"name", b"words", b"owner"])?;
        let name = d.string(field(&fields, b"name")?)?.into();
        let words = d.string(field(&fields, b"words")?)?.into();
        let owner = area_owner(d, field(&fields, b"owner")?)?;
        areas
            .push(
                w::Area::new(&d.kinds, w::AreaParts { name, words, owner })
                    .or(Err(error(b"areas", b"wire area bound")))?,
            )
            .expect("counted areas");
    }
    w::ProjectAreas::new(&d.kinds, w::ProjectAreasParts { areas }).or(Err(error(b"areas", b"wire value bound")))
}
fn area_owner(d: &Decoder<'_>, at: u32) -> Result<w::AreaOwner, Error> {
    let fields = d.object(at, &[b"task", b"party", b"role", b"charter"])?;
    if fields.len() != 1 {
        return Err(error(b"owner", b"one variant required"));
    }
    if let Some(at) = optional(&fields, b"task") {
        return Ok(w::AreaOwner::Task(
            w::AreaTask::new(&d.kinds, w::AreaTaskParts { number: d.number(at)? })
                .or(Err(error(b"owner", b"wire value bound")))?,
        ));
    }
    if let Some(at) = optional(&fields, b"party") {
        return Ok(w::AreaOwner::Party(
            w::AreaParty::new(&d.kinds, w::AreaPartyParts { number: d.number(at)? })
                .or(Err(error(b"owner", b"wire value bound")))?,
        ));
    }
    if let Some(at) = optional(&fields, b"role") {
        return Ok(w::AreaOwner::Role(
            w::AreaRole::new(
                &d.kinds,
                w::AreaRoleParts { number: u32::try_from(d.number(at)?).or(Err(error(b"owner", b"u32 bound")))? },
            )
            .or(Err(error(b"owner", b"wire value bound")))?,
        ));
    }
    if let Some(at) = optional(&fields, b"charter") {
        return Ok(w::AreaOwner::Charter(
            w::AreaCharter::new(
                &d.kinds,
                w::AreaCharterParts { number: u32::try_from(d.number(at)?).or(Err(error(b"owner", b"u32 bound")))? },
            )
            .or(Err(error(b"owner", b"wire value bound")))?,
        ));
    }
    unreachable!("one known owner variant")
}

fn render_owner(value: &w::AreaOwner, out: &mut Encoder) {
    let (kind, number) = match value {
        w::AreaOwner::Task(n) => (b"task".as_slice(), n.number()),
        w::AreaOwner::Party(n) => (b"party".as_slice(), n.number()),
        w::AreaOwner::Role(n) => (b"role".as_slice(), u64::from(n.number())),
        w::AreaOwner::Charter(n) => (b"charter".as_slice(), u64::from(n.number())),
    };
    out.object_start();
    out.key(kind);
    out.unsigned(number);
    out.object_end();
}
