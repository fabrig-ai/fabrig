//! The finding tool (temper/docs/design/domain/agent/tools.md; domain/areas.md, 7).
use alloc::boxed::Box;
use crate::{Problem, calls::Spec, json::Value};
use temper_engine_domain::FindingCall;
use temper_engine_domain_forge as forge;

pub(crate) const SPEC: Spec = Spec {
    name: b"finding",
    description: b"File, hand on, route, backlog, drop, list or show findings in your task's project. Triage only your own findings.",
    schema: br#"{"type":"object","additionalProperties":false,"properties":{"act":{"type":"string","enum":["file","hand","route","backlog","drop","list","show"]},"kind":{"type":"string","enum":["qa","flake","red","review","feedback"]},"severity":{"type":"string","enum":["low","medium","high"]},"area":{"type":"string","description":"An area of this project, 1 to 32 bytes."},"title":{"type":"string","description":"1 to 200 UTF-8 bytes, no control characters."},"body":{"type":"string","description":"At most 4,096 UTF-8 bytes."},"number":{"type":"integer","minimum":1,"maximum":4294967295},"why":{"type":"string","description":"Why hand it on: one line, 1 to 200 bytes."},"goal":{"type":"integer","minimum":1},"reason":{"type":"string","description":"Why drop it: 1 to 400 bytes."},"state":{"type":"string","enum":["new","routed","backlog","dropped"]},"mine":{"type":"boolean","default":false}},"required":["act"],"oneOf":[{"properties":{"act":{"const":"file"}},"required":["kind","severity","title","body"]},{"properties":{"act":{"const":"hand"}},"required":["number","area","why"]},{"properties":{"act":{"const":"route"}},"required":["number","goal"]},{"properties":{"act":{"const":"backlog"}},"required":["number"]},{"properties":{"act":{"const":"drop"}},"required":["number","reason"]},{"properties":{"act":{"const":"list"}}},{"properties":{"act":{"const":"show"}},"required":["number"]}]}"#,
    effect: smith_domain_run::HostEffect::Write,
};

pub(crate) fn decode(value: &Value) -> Result<FindingCall, Problem> {
    let act = value.required(b"act")?.text(b"act")?;
    let fields: &[&[u8]] = match act.as_ref() {
        b"file" => &[b"act", b"kind", b"severity", b"area", b"title", b"body"],
        b"hand" => &[b"act", b"number", b"area", b"why"],
        b"route" => &[b"act", b"number", b"goal"],
        b"backlog" | b"show" => &[b"act", b"number"],
        b"drop" => &[b"act", b"number", b"reason"],
        b"list" => &[b"act", b"state", b"mine"],
        _ => return Err(Problem::Range(b"act: file, hand, route, backlog, drop, list or show")),
    };
    let Value::Object(members) = value else { return Err(Problem::Type(b"finding")) };
    for (name, _) in members {
        if !fields.contains(&name.as_ref()) {
            return Err(Problem::Range(b"finding: member does not belong to this act"));
        }
    }
    match act.as_ref() {
        b"file" => {
            let kind = match value.required(b"kind")?.text(b"kind")?.as_ref() {
                b"qa" => forge::FindingKind::Qa,
                b"flake" => forge::FindingKind::Flake,
                b"red" => forge::FindingKind::Red,
                b"review" => forge::FindingKind::Review,
                b"feedback" => forge::FindingKind::Feedback,
                _ => return Err(Problem::Range(b"kind: qa, flake, red, review or feedback")),
            };
            let severity = match value.required(b"severity")?.text(b"severity")?.as_ref() {
                b"low" => forge::Severity::Low,
                b"medium" => forge::Severity::Medium,
                b"high" => forge::Severity::High,
                _ => return Err(Problem::Range(b"severity: low, medium or high")),
            };
            let title = line(value, b"title", 200)?;
            let body = value.required(b"body")?.text(b"body")?;
            if !forge::findings::body_fits(&body) { return Err(Problem::Range(b"body: at most 4,096 UTF-8 bytes")); }
            Ok(FindingCall::File { kind, severity, area: match value.field(b"area")? { Some(_) => Some(line(value, b"area", 32)?), None => None }, title, body })
        }
        b"hand" => Ok(FindingCall::Hand { number: number(value)?, area: line(value, b"area", 32)?, why: line(value, b"why", 200)? }),
        b"route" => {
            let goal = value.required(b"goal")?.number(b"goal")?;
            if goal == 0 { return Err(Problem::Range(b"goal")); }
            Ok(FindingCall::Triage { number: number(value)?, act: forge::Triage::Route(goal) })
        }
        b"backlog" => Ok(FindingCall::Triage { number: number(value)?, act: forge::Triage::Backlog }),
        b"drop" => {
            let reason = value.required(b"reason")?.text(b"reason")?;
            if !forge::findings::reason_fits(&reason) { return Err(Problem::Range(b"reason: 1 to 400 UTF-8 bytes")); }
            Ok(FindingCall::Triage { number: number(value)?, act: forge::Triage::Drop(reason) })
        }
        b"list" => {
            let state = match optional_text(value, b"state")?.as_deref() {
                Some(b"new") => Some(forge::FindingState::New),
                Some(b"routed") => Some(forge::FindingState::Routed),
                Some(b"backlog") => Some(forge::FindingState::Backlog),
                Some(b"dropped") => Some(forge::FindingState::Dropped),
                None => None,
                Some(_) => return Err(Problem::Range(b"state: new, routed, backlog or dropped")),
            };
            let mine = match value.field(b"mine")? { Some(value) => value.boolean(b"mine")?, None => false };
            Ok(FindingCall::List { state, mine })
        }
        b"show" => Ok(FindingCall::Show { number: number(value)? }),
        _ => unreachable!("act checked above"),
    }
}
mod render;
pub(crate) use render::render;

fn optional_text(value: &Value, name: &'static [u8]) -> Result<Option<Box<[u8]>>, Problem> {
    match value.field(name)? { Some(value) => Ok(Some(value.text(name)?)), None => Ok(None) }
}
fn number(value: &Value) -> Result<u32, Problem> {
    let number = value.required(b"number")?.small(b"number")?;
    if number == 0 { return Err(Problem::Range(b"number")); }
    Ok(number)
}
fn line(value: &Value, name: &'static [u8], bound: u32) -> Result<Box<[u8]>, Problem> {
    let text = value.required(name)?.text(name)?;
    if !forge::settings::words_fit(&text, bound) { return Err(Problem::Range(name)); }
    Ok(text)
}
