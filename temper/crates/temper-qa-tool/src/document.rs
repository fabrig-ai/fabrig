//! The whole JSON document (qa/ledger.md, 2), encoded without derive.
use crate::SessionKind;
use crate::pass_document as q;
use crate::{
    Answer, Ask, Build, BuildState, Case, Exit, Landing, Ledger, Line, Pages, Refusal, Session, Step, refused,
};
use serde_json::{Map, Value, json};

pub(crate) fn object<'a>(value: &'a Value, fields: &[&str]) -> Result<&'a Map<String, Value>, Refusal> {
    let object = value.as_object().ok_or_else(|| refused("expected an object"))?;
    for name in object.keys() {
        if !fields.contains(&name.as_str()) {
            return Err(refused(format!("unknown field {name}")));
        }
    }
    Ok(object)
}
pub(crate) fn required<'a>(object: &'a Map<String, Value>, name: &str) -> Result<&'a Value, Refusal> {
    object.get(name).ok_or_else(|| refused(format!("missing {name}")))
}
pub(crate) fn string(value: &Value) -> Result<String, Refusal> {
    value.as_str().map(String::from).ok_or_else(|| refused("expected a string"))
}
pub(crate) fn text(object: &Map<String, Value>, name: &str) -> Result<String, Refusal> {
    string(required(object, name)?).map_err(|why| refused(format!("{name}: {why}")))
}
pub(crate) fn number(value: &Value) -> Result<u64, Refusal> {
    value.as_u64().ok_or_else(|| refused("expected a nonnegative whole number"))
}
pub(crate) fn boolean(value: &Value) -> Result<bool, Refusal> {
    value.as_bool().ok_or_else(|| refused("expected a boolean"))
}
pub(crate) fn integer(object: &Map<String, Value>, name: &str) -> Result<u64, Refusal> {
    number(required(object, name)?).map_err(|why| refused(format!("{name}: {why}")))
}
pub(crate) fn array(value: &Value) -> Result<&[Value], Refusal> {
    value.as_array().map(Vec::as_slice).ok_or_else(|| refused("expected an array"))
}
pub(crate) fn strings(value: &Value) -> Result<Vec<String>, Refusal> {
    array(value)?.iter().map(string).collect()
}
pub(crate) fn numbers(value: &Value) -> Result<Vec<u64>, Refusal> {
    array(value)?.iter().map(number).collect()
}
pub(crate) fn optional_number(object: &Map<String, Value>, name: &str) -> Result<Option<u64>, Refusal> {
    object.get(name).map(number).transpose()
}
fn flag(object: &Map<String, Value>, name: &str) -> Result<bool, Refusal> {
    Ok(object.get(name).map(boolean).transpose()?.unwrap_or(false))
}

impl Ledger {
    pub fn decode(bytes: &[u8]) -> Result<Self, Refusal> {
        let decode = || {
            let value: Value = serde_json::from_slice(bytes).map_err(|why| refused(why.to_string()))?;
            let object = object(
                &value,
                &[
                    "head",
                    "landings",
                    "cases",
                    "builds",
                    "sessions",
                    "lines",
                    "passes",
                    "findings",
                    "notes",
                    "place_waits",
                ],
            )?;
            Ok(Self {
                head: object.get("head").map(string).transpose()?,
                landings: array(required(object, "landings")?)?.iter().map(landing).collect::<Result<_, _>>()?,
                cases: array(required(object, "cases")?)?.iter().map(case).collect::<Result<_, _>>()?,
                builds: array(required(object, "builds")?)?.iter().map(build).collect::<Result<_, _>>()?,
                sessions: array(required(object, "sessions")?)?.iter().map(session).collect::<Result<_, _>>()?,
                lines: array(required(object, "lines")?)?.iter().map(line).collect::<Result<_, _>>()?,
                passes: q::records(object, "passes")?.iter().map(q::pass).collect::<Result<_, _>>()?,
                findings: q::records(object, "findings")?.iter().map(q::finding).collect::<Result<_, _>>()?,
                notes: q::records(object, "notes")?.iter().map(q::note).collect::<Result<_, _>>()?,
                place_waits: q::records(object, "place_waits")?
                    .iter()
                    .map(|value| {
                        let object = self::object(value, &["from", "to"])?;
                        Ok(crate::PlaceWait { from: integer(object, "from")?, to: optional_number(object, "to")? })
                    })
                    .collect::<Result<_, Refusal>>()?,
            })
        };
        decode().map_err(|why: Refusal| refused(format!("ledger.json: {why}")))
    }

    #[must_use]
    pub fn document(&self) -> Value {
        let mut document = json!({
            "head": self.head,
            "landings": self.landings.iter().map(|landing| {
                let (answer, why) = match &landing.answer {
                    None => (None, None),
                    Some(Answer::None { why }) => (Some("none"), Some(why)),
                    Some(Answer::Cases) => (Some("cases"), None),
                };
                json!({"number": landing.number, "commit": landing.commit, "goal": landing.goal,
                    "pull": landing.pull, "title": landing.title, "seen": landing.seen,
                    "answer": answer, "why": why, "triaged": landing.triaged,
                    "unanswered": landing.unanswered, "sessions": landing.sessions})
            }).collect::<Vec<_>>(),
            "cases": self.cases.iter().map(|case| json!({"number": case.number, "surface": case.surface,
                "page": case.page, "state": case.state.name(), "pass": case.pass, "blocks": if case.blocks == 0 { None } else { Some(case.blocks) },
                "outcomes": case.outcomes.iter().map(q::outcome_document).collect::<Vec<_>>(), "queued": case.queued, "line": case.line,
                "asks": case.asks.iter().map(|ask| json!({"landing": ask.landing, "setup": ask.setup,
                    "adds": ask.adds, "steps": ask.steps.iter().map(step_document).collect::<Vec<_>>() })).collect::<Vec<_>>() })).collect::<Vec<_>>(),
            "builds": self.builds.iter().map(|build| json!({"commit": build.commit, "kept": build.kept,
                "state": match build.state { BuildState::Kept => "kept", BuildState::Deleted => "deleted" },
                "deleted": build.deleted, "holds": build.holds, "order": build.order, "held": build.held })).collect::<Vec<_>>(),
            "sessions": self.sessions.iter().map(|session| json!({"number": session.number, "kind": match session.kind { SessionKind::Triage => "triage", SessionKind::Pass => "pass" },
                "pass": session.pass, "before": session.before.as_ref().map(q::before_document),
                "landing": session.landing, "effort": session.effort, "started": session.started,
                "ended": session.ended, "exit": session.exit.map(exit_document), "tokens": session.tokens,
                "pages": if session.kind == SessionKind::Triage { Some(&session.pages) } else { None }, "setups": if session.kind == SessionKind::Triage { Some(&session.setups) } else { None } })).collect::<Vec<_>>(),
            "passes": self.passes.iter().map(q::pass_document).collect::<Vec<_>>(),
            "findings": self.findings.iter().map(q::finding_document).collect::<Vec<_>>(),
            "notes": self.notes.iter().map(q::note_document).collect::<Vec<_>>(),
            "place_waits": self.place_waits.iter().map(|wait| json!({"from": wait.from, "to": wait.to})).collect::<Vec<_>>(),
            "lines": self.lines.iter().map(|line| json!({"number": line.number, "time": line.time,
                "area": line.area, "words": line.words, "given": line.given})).collect::<Vec<_>>(),
        });
        omit_absent(&mut document);
        document
    }

    pub fn encode(&self) -> Result<Vec<u8>, Refusal> {
        serde_json::to_vec_pretty(&self.document()).map_err(|why| refused(format!("ledger.json: {why}")))
    }
}
fn omit_absent(value: &mut Value) {
    match value {
        Value::Object(object) => {
            object.retain(|_, value| !value.is_null());
            for value in object.values_mut() {
                omit_absent(value);
            }
        }
        Value::Array(array) => {
            for value in array {
                omit_absent(value);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
    }
}
fn step_document(step: &Step) -> Value {
    json!({"do": step.act, "expect": step.expect, "see": step.see})
}
fn exit_document(exit: Exit) -> Value {
    match exit {
        Exit::Code(code) => json!({"code": code}),
        Exit::Signal(signal) => json!({"signal": signal}),
        Exit::Bound => json!("bound"),
        Exit::Lost => json!("lost"),
        Exit::Unstarted => json!("unstarted"),
    }
}
fn exit(value: &Value) -> Result<Exit, Refusal> {
    match value.as_str() {
        Some("bound") => Ok(Exit::Bound),
        Some("lost") => Ok(Exit::Lost),
        Some("unstarted") => Ok(Exit::Unstarted),
        Some(other) => Err(refused(format!("unknown exit {other}"))),
        None => {
            let object = object(value, &["code", "signal"])?;
            if object.len() != 1 {
                return Err(refused("exit needs one code or signal"));
            }
            let (name, value) = object.iter().next().expect("one exit field");
            let value = value
                .as_i64()
                .and_then(|value| i32::try_from(value).ok())
                .ok_or_else(|| refused("exit needs an i32"))?;
            Ok(if name == "code" { Exit::Code(value) } else { Exit::Signal(value) })
        }
    }
}
fn landing(value: &Value) -> Result<Landing, Refusal> {
    let object = object(
        value,
        &["number", "commit", "goal", "pull", "title", "seen", "answer", "why", "triaged", "unanswered", "sessions"],
    )?;
    let answer = match object.get("answer").map(string).transpose()?.as_deref() {
        None => None,
        Some("none") => Some(Answer::None { why: text(object, "why")? }),
        Some("cases") => Some(Answer::Cases),
        Some(other) => return Err(refused(format!("unknown answer {other}"))),
    };
    Ok(Landing {
        number: integer(object, "number")?,
        commit: text(object, "commit")?,
        goal: integer(object, "goal")?,
        pull: optional_number(object, "pull")?,
        title: text(object, "title")?,
        seen: integer(object, "seen")?,
        answer,
        triaged: optional_number(object, "triaged")?,
        unanswered: flag(object, "unanswered")?,
        sessions: integer(object, "sessions")?,
    })
}
fn case(value: &Value) -> Result<Case, Refusal> {
    let object =
        object(value, &["number", "surface", "page", "asks", "state", "queued", "line", "pass", "blocks", "outcomes"])?;
    let state = q::case_state(&text(object, "state")?)?;
    let asks = array(required(object, "asks")?)?.iter().map(ask).collect::<Result<Vec<_>, _>>()?;
    if asks.is_empty() {
        return Err(refused("case has no asks"));
    }
    let surface = text(object, "surface")?;
    if surface != "web" && surface != "tui" {
        return Err(refused(format!("unknown case surface {surface}")));
    }
    Ok(Case {
        number: integer(object, "number")?,
        surface,
        page: text(object, "page")?,
        asks,
        state,
        pass: optional_number(object, "pass")?,
        blocks: optional_number(object, "blocks")?.unwrap_or(0),
        outcomes: q::records(object, "outcomes")?.iter().map(q::outcome).collect::<Result<_, _>>()?,
        queued: integer(object, "queued")?,
        line: flag(object, "line")?,
    })
}
fn ask(value: &Value) -> Result<Ask, Refusal> {
    let object = object(value, &["landing", "setup", "adds", "steps"])?;
    Ok(Ask {
        landing: integer(object, "landing")?,
        setup: text(object, "setup")?,
        adds: text(object, "adds")?,
        steps: array(required(object, "steps")?)?.iter().map(step).collect::<Result<_, _>>()?,
    })
}
fn step(value: &Value) -> Result<Step, Refusal> {
    let object = object(value, &["do", "expect", "see"])?;
    Ok(Step {
        act: text(object, "do")?,
        expect: text(object, "expect")?,
        see: object.get("see").map(string).transpose()?,
    })
}
fn build(value: &Value) -> Result<Build, Refusal> {
    let object = object(value, &["commit", "kept", "state", "deleted", "holds", "order", "held"])?;
    let state = match text(object, "state")?.as_str() {
        "kept" => BuildState::Kept,
        "deleted" => BuildState::Deleted,
        other => return Err(refused(format!("unknown build state {other}"))),
    };
    Ok(Build {
        commit: text(object, "commit")?,
        kept: integer(object, "kept")?,
        state,
        deleted: optional_number(object, "deleted")?,
        holds: object.get("holds").map(numbers).transpose()?,
        order: optional_number(object, "order")?,
        held: optional_number(object, "held")?,
    })
}
fn session(value: &Value) -> Result<Session, Refusal> {
    let object = object(
        value,
        &[
            "number", "kind", "landing", "effort", "started", "ended", "exit", "tokens", "pages", "setups", "pass",
            "before",
        ],
    )?;
    let kind = match text(object, "kind")?.as_str() {
        "triage" => SessionKind::Triage,
        "pass" => SessionKind::Pass,
        other => return Err(refused(format!("unknown session kind {other}"))),
    };
    let empty = Value::Object(Map::new());
    let pages =
        if kind == SessionKind::Triage { required(object, "pages")? } else { object.get("pages").unwrap_or(&empty) };
    let pages = pages.as_object().ok_or_else(|| refused("pages: expected an object"))?;
    let pages = pages
        .iter()
        .map(|(surface, pages)| Ok((surface.clone(), strings(pages)?)))
        .collect::<Result<Pages, Refusal>>()?;
    Ok(Session {
        number: integer(object, "number")?,
        kind,
        landing: if kind == SessionKind::Triage { Some(integer(object, "landing")?) } else { None },
        pass: if kind == SessionKind::Pass { Some(integer(object, "pass")?) } else { None },
        before: if kind == SessionKind::Pass { Some(q::before(required(object, "before")?)?) } else { None },
        effort: text(object, "effort")?,
        started: integer(object, "started")?,
        ended: optional_number(object, "ended")?,
        exit: object.get("exit").map(exit).transpose()?,
        tokens: optional_number(object, "tokens")?,
        pages,
        setups: if kind == SessionKind::Triage { strings(required(object, "setups")?)? } else { Vec::new() },
    })
}
fn line(value: &Value) -> Result<Line, Refusal> {
    let object = object(value, &["number", "time", "area", "words", "given"])?;
    Ok(Line {
        number: integer(object, "number")?,
        time: integer(object, "time")?,
        area: text(object, "area")?,
        words: text(object, "words")?,
        given: flag(object, "given")?,
    })
}
