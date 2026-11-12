//! Findings (temper/docs/design/protocol/store.md, 2).
use super::{FITS, w};
use crate::Undecodable;
use skein_lib::Wall;
use temper_engine_domain_forge as d;
fn encode_findingkind(value: d::FindingKind) -> w::FindingKind {
    match value {
        d::FindingKind::Qa => w::FindingKind::Qa,
        d::FindingKind::Flake => w::FindingKind::Flake,
        d::FindingKind::Red => w::FindingKind::Red,
        d::FindingKind::Review => w::FindingKind::Review,
        d::FindingKind::Feedback => w::FindingKind::Feedback,
    }
}
fn decode_findingkind(value: w::FindingKind) -> d::FindingKind {
    match value {
        w::FindingKind::Qa => d::FindingKind::Qa,
        w::FindingKind::Flake => d::FindingKind::Flake,
        w::FindingKind::Red => d::FindingKind::Red,
        w::FindingKind::Review => d::FindingKind::Review,
        w::FindingKind::Feedback => d::FindingKind::Feedback,
    }
}
fn encode_severity(value: d::Severity) -> w::Severity {
    match value {
        d::Severity::Low => w::Severity::Low,
        d::Severity::Medium => w::Severity::Medium,
        d::Severity::High => w::Severity::High,
    }
}
fn decode_severity(value: w::Severity) -> d::Severity {
    match value {
        w::Severity::Low => d::Severity::Low,
        w::Severity::Medium => d::Severity::Medium,
        w::Severity::High => d::Severity::High,
    }
}
fn encode_findingstate(value: d::FindingState) -> w::FindingState {
    match value {
        d::FindingState::New => w::FindingState::New,
        d::FindingState::Routed => w::FindingState::Routed,
        d::FindingState::Backlog => w::FindingState::Backlog,
        d::FindingState::Dropped => w::FindingState::Dropped,
    }
}
fn decode_findingstate(value: w::FindingState) -> d::FindingState {
    match value {
        w::FindingState::New => d::FindingState::New,
        w::FindingState::Routed => d::FindingState::Routed,
        w::FindingState::Backlog => d::FindingState::Backlog,
        w::FindingState::Dropped => d::FindingState::Dropped,
    }
}
fn encode_filer(value: d::Filer) -> w::Filer {
    match value {
        d::Filer::Task(number) => w::Filer::Task(w::Number::new(&w::CEILINGS, w::NumberParts { number }).expect(FITS)),
        d::Filer::Party(number) => w::Filer::Party(w::Number::new(&w::CEILINGS, w::NumberParts { number }).expect(FITS)),
    }
}
fn decode_filer(value: w::Filer) -> d::Filer {
    match value { w::Filer::Task(value) => d::Filer::Task(value.number()), w::Filer::Party(value) => d::Filer::Party(value.number()) }
}
pub(super) fn encode(row: &d::Finding) -> w::Finding {
    w::Finding::new(&w::CEILINGS, w::FindingParts {
        project: row.project, number: row.number, kind: encode_findingkind(row.kind), severity: encode_severity(row.severity),
        area: row.area.clone(), state: encode_findingstate(row.state), title: row.title.clone(), body: row.body.clone(),
        filer: encode_filer(row.filer), at: row.at.as_nanos(), why: row.why.clone(),
        hander: match row.hander { Some(value) => Some(encode_filer(value)), None => None },
        goal: row.goal, reason: row.reason.clone(),
    }).expect(FITS)
}
pub(super) fn decode(row: w::Finding) -> Result<d::Finding, Undecodable> {
    let w::FindingParts { project, number, kind, severity, area, state, title, body, filer, at, why, hander, goal, reason } = row.into_parts();
    if number == 0 || !d::settings::words_fit(&title, 200) || !d::findings::body_fits(&body) { return Err(Undecodable); }
    if let Some(words) = &why && !d::settings::words_fit(words, 200) { return Err(Undecodable); }
    if let Some(words) = &reason && !d::settings::words_fit(words, 400) { return Err(Undecodable); }
    if why.is_some() != hander.is_some() { return Err(Undecodable); }
    Ok(d::Finding { project, number, kind: decode_findingkind(kind), severity: decode_severity(severity), area,
        state: decode_findingstate(state), title, body, filer: decode_filer(filer), at: Wall::from_nanos(at), why,
        hander: match hander { Some(value) => Some(decode_filer(value)), None => None }, goal, reason })
}
