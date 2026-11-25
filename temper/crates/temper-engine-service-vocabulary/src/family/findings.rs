//! Findings at the client boundary (temper/docs/design/client/forge/records.md; domain/areas.md, 5 and 7).
use super::{Context, forge, maps, w};
use alloc::boxed::Box;
use skein_codec::Reason;
use skein_lib::{List, usize_of};
use temper_client_protocol_wire::v1::Record;

// A 20-byte head occupies the prefix of the 32-byte domain commit.
// Whole-request admission checks the allowed widths (client/forge/records.md).
pub(crate) fn finding_blame_act_to_domain(value: w::FindingBlameAct) -> forge::BlameAct {
    let w::FindingBlameAct { landing, why } = value;
    let landing = match landing {
        Some(bytes) => {
            let mut commit = [0; 32];
            for (cell, byte) in commit.iter_mut().zip(&bytes) {
                *cell = *byte;
            }
            Some(commit)
        }
        None => None,
    };
    forge::BlameAct { landing, why }
}

fn refused(value: &forge::FindingRefusal) -> Option<w::Outcome> {
    let reason = maps::finding_refusal_to_record(value).ok()?;
    Some(w::Outcome::FindingRefused(w::FindingRefused { reason }))
}

pub(super) fn file_answer(value: forge::FileFindingAnswer) -> Option<w::Outcome> {
    match value {
        forge::FileFindingAnswer::Filed { number } => Some(w::Outcome::Filed(w::Filed { number })),
        forge::FileFindingAnswer::Refused(value) => refused(&value),
    }
}
pub(super) fn hand_answer(value: forge::HandFindingAnswer) -> Option<w::Outcome> {
    match value {
        forge::HandFindingAnswer::Handed => Some(w::Outcome::Handed),
        forge::HandFindingAnswer::Refused(value) => refused(&value),
    }
}
pub(super) fn triage_answer(value: forge::TriageFindingAnswer) -> Option<w::Outcome> {
    match value {
        forge::TriageFindingAnswer::Triaged => Some(w::Outcome::Triaged(w::Triaged { goal: None })),
        forge::TriageFindingAnswer::Repaired(goal) => Some(w::Outcome::Triaged(w::Triaged { goal: Some(goal) })),
        forge::TriageFindingAnswer::Refused(value) => refused(&value),
    }
}

pub(super) fn document(value: &forge::Findings, limits: &w::Limits) -> Option<Box<[u8]>> {
    maps::findings_to_record(value).ok()?.encode(limits).ok()
}

pub(super) fn read_answer(value: Option<Box<forge::FindingAnswer>>, limits: &w::Limits) -> Option<Box<[u8]>> {
    let result = match value {
        None => w::FindingResult::NoFinding,
        Some(value) => {
            w::FindingResult::Finding(Box::new(maps::finding_detail_to_record(&value, &Context::Sending(limits)).ok()?))
        }
    };
    let record = w::FindingAnswer { result };
    record.encode(limits).ok()
}

// Blame keeps the wire's nested count checks at the list writer.
pub(crate) fn finding_blame_to_record(
    value: &forge::FindingBlame,
    context: &Context<'_>,
) -> Result<w::FindingBlame, w::Problem> {
    let limits = context.limits();
    let capacity = limits.finding_blame_landings;
    if value.landings.len() > usize_of(capacity) {
        return Err(w::paths::FINDING_BLAME_LANDINGS.problem(Reason::Bound));
    }
    let mut landings = List::with_capacity(capacity);
    for landing in &value.landings {
        if landing.goals.len() > usize_of(limits.blamed_landing_goals) {
            return Err(w::paths::BLAMED_LANDING_GOALS.problem(Reason::Bound));
        }
        let record = maps::blamed_landing_to_record(landing)?;
        landings.push(record).expect("checked blamed landing count");
    }
    let blame = w::FindingBlame { strength: maps::blame_strength_to_record(&value.strength), landings };
    blame.measure(limits)?;
    Ok(blame)
}
