//! Pure row-part encodings, shared by the root and service vocabulary
//! (temper/docs/design/client/forge/parts.md, 3–4).
#![expect(clippy::manual_map, reason = "pure encodings spell out optional fields without closures")]
use crate::{MilestoneTotals, generated::client_wire::milestone_totals_to_record};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::documents::TaskLine;
use skein_lib::{List, Wall};
use temper_client_protocol_wire::v1::Record;
use temper_client_protocol_wire::{self as wire, v1 as w};

pub fn kinds(connector: u16) -> Box<[jig_domain_core_vocabulary::PartKind]> {
    use jig_domain_core_vocabulary::{PartInputs, PartKind};
    let notes = PartInputs { line: false, notes: true, delegates: false, delegate_notes: false, records: false };
    let milestone = PartInputs { line: true, notes: false, delegates: true, delegate_notes: true, records: true };
    Box::new([
        PartKind { owner: wire::FORGE, kind: wire::GOAL_PART, connector, bytes: wire::GOAL_PART_BYTES, inputs: notes },
        PartKind { owner: wire::FORGE, kind: wire::CHAT_PART, connector, bytes: wire::CHAT_PART_BYTES, inputs: notes },
        PartKind {
            owner: wire::FORGE,
            kind: wire::MILESTONE_PART,
            connector,
            bytes: wire::MILESTONE_PART_BYTES,
            inputs: milestone,
        },
    ])
}

pub fn goal(report: Option<&(Box<[u8]>, Wall)>, milestone: Option<u64>) -> Option<Box<[u8]>> {
    let report = match report {
        Some((headline, at)) => Some(w::GoalReport { headline: headline.clone(), at: at.as_nanos() }),
        None => None,
    };
    let record = w::GoalPart { report, milestone };
    record.encode(&w::CEILINGS).ok()
}

pub fn chat(document: Option<&[u8]>) -> Option<Box<[u8]>> {
    let record = w::ChatPart {
        document: match document {
            Some(value) => Some(value.into()),
            None => None,
        },
    };
    record.encode(&w::CEILINGS).ok()
}

#[must_use]
pub fn totals(value: &MilestoneTotals) -> w::MilestoneTotals {
    milestone_totals_to_record(value)
}

fn milestone_goal(line: &TaskLine, headline: Option<Box<[u8]>>) -> Option<w::MilestoneGoal> {
    let line = jig_protocol_engine_clients::translate::task_line(line, &jig_client_protocol_wire::CEILINGS).ok()?;
    let line = line.encode(&jig_client_protocol_wire::CEILINGS).ok()?;
    Some(w::MilestoneGoal { line, headline })
}

pub fn milestone(
    whole: Option<Wall>,
    value: &MilestoneTotals,
    goals: &[(TaskLine, Option<Box<[u8]>>)],
) -> Option<Box<[u8]>> {
    let limits = &w::CEILINGS;
    let total = value.done.checked_add(value.failed)?.checked_add(value.closed)?.checked_add(value.live)?;
    let mut kept = List::with_capacity(16);
    let whole = match whole {
        Some(at) => Some(at.as_nanos()),
        None => None,
    };
    let mut record = w::MilestonePart { whole, totals: totals(value), goals: kept.clone(), more: total };
    for (line, headline) in goals {
        if kept.len() == 16 {
            break;
        }
        let Some(goal) = milestone_goal(line, headline.clone()) else { break };
        kept.push(goal).ok()?;
        let candidate = w::MilestonePart {
            whole,
            totals: totals(value),
            goals: kept.clone(),
            more: total.saturating_sub(kept.len()),
        };
        if candidate.measure(limits).ok()? > wire::MILESTONE_PART_BYTES {
            break;
        }
        record = candidate;
    }
    record.encode(&w::CEILINGS).ok()
}
