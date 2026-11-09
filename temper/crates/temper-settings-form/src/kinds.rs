//! Temper's settings file values and codec records
//! (temper/docs/design/client/tools.md, 5.3; temper/docs/design/protocol/configuration.md, 13.2).
use crate::file::{Decoder, Error, Fields, error, field, optional};
use crate::view as c;
use alloc::boxed::Box;
use skein_json::writer::Encoder;
use skein_lib::{List, Reader, Writer};
use temper_client_protocol_wire::v1 as w;

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
    if kind == b"project-ways" || kind == b"project-questions" {
        if optional(fields, b"number").is_some() {
            return Err(error(b"number", b"project setting has no number"));
        }
        let project = number(d, fields, b"project")?;
        if project == 0 {
            return Err(error(b"project", b"from 1"));
        }
        return Ok(c::Setting::Application {
            kind: if kind == b"project-ways" { 7 } else { 6 },
            scope: project,
            number: 1,
        });
    }
    let number = number(d, fields, b"number")?;
    if number == 0 {
        return Err(error(b"number", b"from 1"));
    }
    let project = optional(fields, b"project");
    if kind == b"forge-charter" || kind == b"template-goal" {
        if project.is_some() {
            return Err(error(b"project", b"unknown member"));
        }
        return Ok(c::Setting::Application { kind: if kind == b"forge-charter" { 3 } else { 11 }, scope: 0, number });
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
    if kind == 6 {
        let word = match d.string(at) {
            Ok(word) => word,
            Err(why) => return Err(why.at(b"project-questions")),
        };
        let route = match word {
            b"none" => w::QuestionsRoute::None,
            b"foreman" => w::QuestionsRoute::Foreman,
            b"up" => w::QuestionsRoute::Up,
            _ => return Err(error(b"project-questions", b"none, foreman or up required")),
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
fn goal(d: &Decoder<'_>, at: u32) -> Result<w::TemplateGoal, Error> {
    let fields = d.object(at, &[b"design", b"close_out"])?;
    w::TemplateGoal::new(
        &d.kinds,
        w::TemplateGoalParts { design: boolean(d, &fields, b"design")?, close_out: boolean(d, &fields, b"close_out")? },
    )
    .or(Err(error(b"", b"wire value bound")))
}
fn ways(d: &Decoder<'_>, at: u32) -> Result<w::ProjectWays, Error> {
    let fields = d.object(at, &[b"drafts", b"goals", b"changes", b"away", b"way"])?;
    w::ProjectWays::new(
        &d.kinds,
        w::ProjectWaysParts {
            skein_bools: [
                boolean(d, &fields, b"drafts")?,
                boolean(d, &fields, b"goals")?,
                boolean(d, &fields, b"changes")?,
                boolean(d, &fields, b"away")?,
            ],
            way: number(d, &fields, b"way")?,
        },
    )
    .or(Err(error(b"", b"wire value bound")))
}
fn rule(d: &Decoder<'_>, at: u32) -> Result<w::LandingRule, Error> {
    let fields = d.object(at, &[b"pattern", b"enforce", b"ci", b"up_to_date", b"gates", b"approvals"])?;
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
        },
    )
    .or(Err(error(b"", b"wire value bound")))
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

pub(super) fn render(kind: u32, bytes: &[u8], out: &mut Encoder) -> bool {
    let mut reader = Reader::new(bytes);
    match kind {
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
                w::QuestionsRoute::Foreman => b"foreman",
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
            out.key(b"away");
            out.boolean(value.away());
            out.key(b"way");
            out.unsigned(u64::from(value.way()));
            out.object_end();
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
    out.object_end();
}
