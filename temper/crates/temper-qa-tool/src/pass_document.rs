//! Q3's JSON records (qa/ledger.md, 2). Unknown fields are refused at every level.
use crate::document::{self as d, integer, optional_number, required, text};
use crate::{
    Before, CaseBefore, CaseState, Check, Finding, FindingState, Note, Outcome, OutcomeResult, Pass, PassState,
    Refusal, Request, refused,
};
use serde_json::{Map, Value, json};

pub(crate) fn records<'a>(object: &'a Map<String, Value>, name: &str) -> Result<&'a [Value], Refusal> {
    object.get(name).map(d::array).transpose().map(|array| array.unwrap_or(&[]))
}
pub(crate) fn case_state(word: &str) -> Result<CaseState, Refusal> {
    match word {
        "queued" => Ok(CaseState::Queued),
        "assigned" => Ok(CaseState::Assigned),
        "passed" => Ok(CaseState::Passed),
        "failed" => Ok(CaseState::Failed),
        "blocked" => Ok(CaseState::Blocked),
        other => Err(refused(format!("unknown case state {other}"))),
    }
}
fn pass_state(word: &str) -> Result<PassState, Refusal> {
    match word {
        "running" => Ok(PassState::Running),
        "waiting" => Ok(PassState::Waiting),
        "ended" => Ok(PassState::Ended),
        other => Err(refused(format!("unknown pass state {other}"))),
    }
}
fn finding_state(word: &str) -> Result<FindingState, Refusal> {
    match word {
        "open" => Ok(FindingState::Open),
        "linked" => Ok(FindingState::Linked),
        "closed" => Ok(FindingState::Closed),
        "handed" => Ok(FindingState::Handed),
        other => Err(refused(format!("unknown finding state {other}"))),
    }
}
pub(crate) fn outcome_document(outcome: &Outcome) -> Value {
    json!({"session":outcome.session,"time":outcome.time,"result":outcome.result.name(),"notes":outcome.notes,"findings":outcome.findings})
}
pub(crate) fn outcome(value: &Value) -> Result<Outcome, Refusal> {
    let object = d::object(value, &["session", "time", "result", "notes", "findings"])?;
    Ok(Outcome {
        session: integer(object, "session")?,
        time: integer(object, "time")?,
        result: OutcomeResult::parse(&text(object, "result")?)?,
        notes: text(object, "notes")?,
        findings: d::numbers(required(object, "findings")?)?,
    })
}
pub(crate) fn pass_document(pass: &Pass) -> Value {
    json!({"number":pass.number,"place":pass.place,"build":pass.build,"state":pass.state.name(),"started":pass.started,"ended":pass.ended,
        "cases":pass.cases,"sessions":pass.sessions,"requests":pass.requests.iter().map(request_document).collect::<Vec<_>>()})
}
pub(crate) fn pass(value: &Value) -> Result<Pass, Refusal> {
    let object =
        d::object(value, &["number", "place", "build", "state", "started", "ended", "cases", "sessions", "requests"])?;
    Ok(Pass {
        number: integer(object, "number")?,
        place: text(object, "place")?,
        build: text(object, "build")?,
        state: pass_state(&text(object, "state")?)?,
        started: integer(object, "started")?,
        ended: optional_number(object, "ended")?,
        cases: d::numbers(required(object, "cases")?)?,
        sessions: d::numbers(required(object, "sessions")?)?,
        requests: records(object, "requests")?.iter().map(request).collect::<Result<_, _>>()?,
    })
}
fn request_document(request: &Request) -> Value {
    json!({"number":request.number,"time":request.time,"findings":request.findings,"recipes":request.recipes,"filed":request.filed,"goal":request.goal,"ended":request.ended})
}
fn request(value: &Value) -> Result<Request, Refusal> {
    let object = d::object(value, &["number", "time", "findings", "recipes", "filed", "goal", "ended"])?;
    Ok(Request {
        number: integer(object, "number")?,
        time: integer(object, "time")?,
        findings: d::numbers(required(object, "findings")?)?,
        recipes: d::strings(required(object, "recipes")?)?,
        filed: optional_number(object, "filed")?,
        goal: optional_number(object, "goal")?,
        ended: optional_number(object, "ended")?,
    })
}
pub(crate) fn before_document(before: &Before) -> Value {
    json!({"state":before.state.map(PassState::name),"place":before.place,"build":before.build,
        "cases":before.cases.iter().map(|case| json!({"number":case.number,"state":case.state.name(),"pass":case.pass})).collect::<Vec<_>>()})
}
pub(crate) fn before(value: &Value) -> Result<Before, Refusal> {
    let object = d::object(value, &["state", "place", "build", "cases"])?;
    Ok(Before {
        state: object.get("state").map(d::string).transpose()?.map(|word| pass_state(&word)).transpose()?,
        place: object.get("place").map(d::string).transpose()?,
        build: object.get("build").map(d::string).transpose()?,
        cases: records(object, "cases")?
            .iter()
            .map(|value| {
                let object = d::object(value, &["number", "state", "pass"])?;
                Ok(CaseBefore {
                    number: integer(object, "number")?,
                    state: case_state(&text(object, "state")?)?,
                    pass: optional_number(object, "pass")?,
                })
            })
            .collect::<Result<_, _>>()?,
    })
}
pub(crate) fn finding_document(finding: &Finding) -> Value {
    json!({"number":finding.number,"pass":finding.pass,"case":finding.case,"session":finding.session,"time":finding.time,
        "title":finding.title,"severity":finding.severity,"area":finding.area,"setup":finding.setup,"lines":finding.lines,
        "expected":finding.expected,"seen":finding.seen,"evidence":finding.evidence,"surface":finding.surface,"page":finding.page,"landings":finding.landings,
        "state":finding.state.name(),"to":finding.to,"closed":finding.closed,"closed_time":finding.closed_time,"goals":finding.goals,
        "checks":finding.checks.iter().map(|check| json!({"session":check.session,"time":check.time,"commit":check.commit,"holds":check.holds,"notes":check.notes})).collect::<Vec<_>>()})
}
pub(crate) fn finding(value: &Value) -> Result<Finding, Refusal> {
    let object = d::object(
        value,
        &[
            "number",
            "pass",
            "case",
            "session",
            "time",
            "title",
            "severity",
            "area",
            "setup",
            "lines",
            "expected",
            "seen",
            "evidence",
            "surface",
            "page",
            "landings",
            "state",
            "to",
            "closed",
            "closed_time",
            "goals",
            "checks",
        ],
    )?;
    let severity = text(object, "severity")?;
    if !["low", "medium", "high"].contains(&severity.as_str()) {
        return Err(refused(format!("unknown finding severity {severity}")));
    }
    let surface = text(object, "surface")?;
    if surface != "web" && surface != "tui" {
        return Err(refused(format!("unknown finding surface {surface}")));
    }
    Ok(Finding {
        number: integer(object, "number")?,
        pass: integer(object, "pass")?,
        case: integer(object, "case")?,
        session: integer(object, "session")?,
        time: integer(object, "time")?,
        title: text(object, "title")?,
        severity,
        area: text(object, "area")?,
        setup: text(object, "setup")?,
        lines: d::strings(required(object, "lines")?)?,
        expected: text(object, "expected")?,
        seen: text(object, "seen")?,
        evidence: d::strings(required(object, "evidence")?)?,
        surface,
        page: text(object, "page")?,
        landings: d::numbers(required(object, "landings")?)?,
        state: finding_state(&text(object, "state")?)?,
        to: optional_number(object, "to")?,
        closed: object.get("closed").map(d::string).transpose()?,
        closed_time: optional_number(object, "closed_time")?,
        goals: d::numbers(required(object, "goals")?)?,
        checks: records(object, "checks")?.iter().map(check).collect::<Result<_, _>>()?,
    })
}
fn check(value: &Value) -> Result<Check, Refusal> {
    let object = d::object(value, &["session", "time", "commit", "holds", "notes"])?;
    Ok(Check {
        session: integer(object, "session")?,
        time: integer(object, "time")?,
        commit: text(object, "commit")?,
        holds: d::boolean(required(object, "holds")?)?,
        notes: text(object, "notes")?,
    })
}
pub(crate) fn note_document(note: &Note) -> Value {
    json!({"number":note.number,"pass":note.pass,"session":note.session,"time":note.time,"words":note.words})
}
pub(crate) fn note(value: &Value) -> Result<Note, Refusal> {
    let object = d::object(value, &["number", "pass", "session", "time", "words"])?;
    Ok(Note {
        number: integer(object, "number")?,
        pass: integer(object, "pass")?,
        session: integer(object, "session")?,
        time: integer(object, "time")?,
        words: text(object, "words")?,
    })
}
