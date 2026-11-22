//! The finding tool (temper/docs/design/domain/agent/tools.md; domain/areas.md, 7).
use crate::{Problem, calls::Spec, json::Value};
use alloc::boxed::Box;
use temper_engine_domain::FindingCall;
use temper_engine_domain_forge as forge;

pub(crate) const SPEC: Spec = Spec {
    name: b"finding",
    description: b"File, hand, route, backlog or drop a finding; repair: numbers words; blame: number [commit] why; list: [state] [mine]; show: number.",
    schema: br#"{"type":"object","properties":{"act":{"enum":["file","hand","route","backlog","drop","repair","blame","list","show"]},"kind":{"enum":["qa","flake","red","review","feedback"]},"severity":{"enum":["low","medium","high"]},"area":{"type":"string"},"title":{"type":"string"},"body":{"type":"string"},"number":{"type":"integer"},"why":{"type":"string","description":"One line, 1 to 200 UTF-8 bytes."},"numbers":{"type":"array","minItems":1,"maxItems":16,"uniqueItems":true,"items":{"type":"integer","minimum":1,"maximum":4294967295}},"words":{"type":"string","description":"1 to 4096 UTF-8 bytes."},"commit":{"type":"string","pattern":"^([0-9a-f]{40}|[0-9a-f]{64})$"},"goal":{"type":"integer"},"reason":{"type":"string"},"state":{"type":"string"},"mine":{"type":"boolean"}},"required":["act"]}"#,
    effect: smith_domain_run::HostEffect::Write,
    whole: Some(br#"{"type":"object","properties":{"act":{"enum":["file","hand","route","backlog","drop","repair","blame","list","show"],"description":"The act."},"kind":{"enum":["qa","flake","red","review","feedback"],"description":"Finding kind."},"severity":{"enum":["low","medium","high"],"description":"Severity."},"area":{"type":"string","description":"Area name."},"title":{"type":"string","description":"Finding title."},"body":{"type":"string","description":"Finding body."},"number":{"type":"integer","description":"Finding number."},"why":{"type":"string","description":"Why hand it or move blame: one line of 1 to 200 UTF-8 bytes."},"numbers":{"type":"array","minItems":1,"maxItems":16,"uniqueItems":true,"items":{"type":"integer","minimum":1,"maximum":4294967295},"description":"Repair's 1 to 16 distinct finding numbers, oldest first as given."},"words":{"type":"string","description":"Repair's words: 1 to 4,096 UTF-8 bytes."},"commit":{"type":"string","pattern":"^([0-9a-f]{40}|[0-9a-f]{64})$","description":"Blame's landing: 40 or 64 lowercase hex digits; omit for none."},"goal":{"type":"integer","description":"Goal task."},"reason":{"type":"string","description":"Why drop it."},"state":{"type":"string","description":"State filter."},"mine":{"type":"boolean","description":"Only mine."}},"required":["act"]}"#),
    offer: crate::calls::Offer::Family(jig_domain_core::ToolFamilies::JIG.delegate),
};

#[expect(clippy::too_many_lines, reason = "the nine finding acts share one dispatch and member check")]
pub(crate) fn decode(value: &Value) -> Result<FindingCall, Problem> {
    let act = value.required(b"act")?.text(b"act")?;
    let fields: &[&[u8]] = match act.as_ref() {
        b"file" => &[b"act", b"kind", b"severity", b"area", b"title", b"body"],
        b"hand" => &[b"act", b"number", b"area", b"why"],
        b"route" => &[b"act", b"number", b"goal"],
        b"backlog" | b"show" => &[b"act", b"number"],
        b"drop" => &[b"act", b"number", b"reason"],
        b"repair" => &[b"act", b"numbers", b"words"],
        b"blame" => &[b"act", b"number", b"commit", b"why"],
        b"list" => &[b"act", b"state", b"mine"],
        _ => return Err(Problem::Range(b"act: file, hand, route, backlog, drop, repair, blame, list or show")),
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
            if !forge::findings::body_fits(&body) {
                return Err(Problem::Range(b"body: at most 4,096 UTF-8 bytes"));
            }
            Ok(FindingCall::File {
                kind,
                severity,
                area: match value.field(b"area")? {
                    Some(_) => Some(line(value, b"area", 32)?),
                    None => None,
                },
                title,
                body,
            })
        }
        b"hand" => Ok(FindingCall::Hand {
            number: number(value)?,
            area: line(value, b"area", 32)?,
            why: line(value, b"why", 200)?,
        }),
        b"route" => {
            let goal = value.required(b"goal")?.number(b"goal")?;
            if goal == 0 {
                return Err(Problem::Range(b"goal"));
            }
            Ok(FindingCall::Triage { number: number(value)?, act: forge::Triage::Route(goal) })
        }
        b"backlog" => Ok(FindingCall::Triage { number: number(value)?, act: forge::Triage::Backlog }),
        b"drop" => {
            let reason = value.required(b"reason")?.text(b"reason")?;
            if !forge::findings::reason_fits(&reason) {
                return Err(Problem::Range(b"reason: 1 to 400 UTF-8 bytes"));
            }
            Ok(FindingCall::Triage { number: number(value)?, act: forge::Triage::Drop(reason) })
        }
        b"repair" => repair(value),
        b"blame" => {
            let landing = match value.field(b"commit")? {
                Some(value) => Some(
                    crate::forge::commit(value, b"commit")
                        .or(Err(Problem::Range(b"commit: 40 or 64 lowercase hex digits")))?,
                ),
                None => None,
            };
            Ok(FindingCall::Triage {
                number: number(value)?,
                act: forge::Triage::Blame(forge::BlameAct { landing, why: line(value, b"why", 200)? }),
            })
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
            let mine = match value.field(b"mine")? {
                Some(value) => value.boolean(b"mine")?,
                None => false,
            };
            Ok(FindingCall::List { state, mine })
        }
        b"show" => Ok(FindingCall::Show { number: number(value)? }),
        _ => unreachable!("act checked above"),
    }
}
mod render;
pub(crate) use render::render;

fn repair(value: &Value) -> Result<FindingCall, Problem> {
    let numbers = value.required(b"numbers")?.array(b"numbers")?;
    if numbers.is_empty() || numbers.len() > 16 {
        return Err(Problem::Range(b"numbers: 1 to 16 distinct finding numbers"));
    }
    let mut checked = skein_lib::List::with_capacity(16);
    for value in numbers {
        let number = value.small(b"numbers")?;
        if number == 0 || checked.as_slice().contains(&number) {
            return Err(Problem::Range(b"numbers: 1 to 16 distinct finding numbers"));
        }
        checked.push(number).expect("bounded repair numbers");
    }
    let number = *checked.as_slice().first().expect("nonempty repair numbers");
    let more = checked.as_slice().get(1..).expect("first repair number").into();
    let words = value.required(b"words")?.text(b"words")?;
    if words.is_empty() || !forge::findings::body_fits(&words) {
        return Err(Problem::Range(b"words: 1 to 4,096 UTF-8 bytes"));
    }
    Ok(FindingCall::Triage { number, act: forge::Triage::Repair(forge::RepairAct { more, words }) })
}

fn optional_text(value: &Value, name: &'static [u8]) -> Result<Option<Box<[u8]>>, Problem> {
    match value.field(name)? {
        Some(value) => Ok(Some(value.text(name)?)),
        None => Ok(None),
    }
}
fn number(value: &Value) -> Result<u32, Problem> {
    let number = value.required(b"number")?.small(b"number")?;
    if number == 0 {
        return Err(Problem::Range(b"number"));
    }
    Ok(number)
}
fn line(value: &Value, name: &'static [u8], bound: u32) -> Result<Box<[u8]>, Problem> {
    let text = value.required(name)?.text(name)?;
    if !forge::settings::words_fit(&text, bound) {
        let rule: &[u8] = match name {
            b"area" => b"area: 1 to 32 UTF-8 bytes, no control characters",
            b"title" => b"title: 1 to 200 UTF-8 bytes, no control characters",
            b"why" => b"why: one line of 1 to 200 UTF-8 bytes, no control characters",
            _ => unreachable!("finding line field"),
        };
        return Err(Problem::Range(rule));
    }
    Ok(text)
}
