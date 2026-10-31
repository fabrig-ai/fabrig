//! Whole pages and single documents (protocol/client.md, 3 and 6).
use super::{
    Box, Limits, List, Reader, Wall, authority_document_fields, c, chip_ending_fields, cursor_fields,
    history_change_fields, message_proposal_kind_fields, piece_problem, retained_action_fields,
    specification_document_fields, too_large, w,
};
pub(crate) fn history_holder_fields(value: &w::HistoryHolder) -> c::HistoryHolder {
    match value {
        w::HistoryHolder::Task(row) => c::HistoryHolder::Task(row.value()),
        w::HistoryHolder::Person(row) => c::HistoryHolder::Person(row.value()),
        w::HistoryHolder::Policy(row) => {
            c::HistoryHolder::Policy { project: row.project(), kind: message_proposal_kind_fields(row.kind()) }
        }
    }
}

pub(crate) fn proposal_history_state_fields(
    value: &w::ProposalHistoryState,
    limits: &Limits,
) -> Result<c::ProposalHistoryState, w::Problem> {
    Ok(match value {
        w::ProposalHistoryState::Pending(row) => c::ProposalHistoryState::Pending {
            holder: history_holder_fields(row.holder()),
            since: Wall::from_nanos(row.since()),
        },
        w::ProposalHistoryState::Accepted(row) => {
            c::ProposalHistoryState::Accepted { by: crate::translate::facts::requester(row.by()) }
        }
        w::ProposalHistoryState::Rejected(row) => c::ProposalHistoryState::Rejected {
            by: crate::translate::facts::requester(row.by()),
            reason: crate::translate::facts::document_text(row.reason(), limits)?,
        },
        w::ProposalHistoryState::Withdrawn => c::ProposalHistoryState::Withdrawn,
    })
}

pub(crate) fn proposal_history_fields(
    value: &w::ProposalHistory,
    limits: &Limits,
) -> Result<c::ProposalHistory, w::Problem> {
    Ok(c::ProposalHistory {
        action: retained_action_fields(value.action(), limits)?,
        number: value.number(),
        proposer: value.proposer(),
        as_holder: value.as_holder(),
        state: proposal_history_state_fields(value.state(), limits)?,
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn amendment_state_fields(
    value: &w::AmendmentState,
    limits: &Limits,
) -> Result<c::AmendmentState, w::Problem> {
    let opt0 = match value.addressed() {
        Some(item) => Some(crate::translate::facts::person_address(item)),
        None => None,
    };
    let opt1 = match value.taken_by() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt2 = match value.instructions() {
        Some(item) => Some(crate::translate::facts::document_text(item, limits)?),
        None => None,
    };
    if value.dependencies().len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list3 = List::with_capacity(value.dependencies().len());
    for item in value.dependencies() {
        list3.push(*item).expect("reserved supplied list");
    }
    Ok(c::AmendmentState {
        addressed: opt0,
        taken_by: opt1,
        instructions: opt2,
        spec: specification_document_fields(value.spec(), limits)?,
        authority: authority_document_fields(value.authority(), limits)?,
        wake: crate::translate::facts::wake_policy(value.wake()),
        dependencies: list3,
        cut: value.cut(),
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn history_line_fields(value: &w::HistoryLine, limits: &Limits) -> Result<c::HistoryLine, w::Problem> {
    let opt0 = match value.proposal() {
        Some(item) => Some(proposal_history_fields(item, limits)?),
        None => None,
    };
    let opt1 = match value.amendment() {
        Some(item) => Some((
            Box::new(amendment_state_fields(item.before(), limits)?),
            Box::new(amendment_state_fields(item.after(), limits)?),
        )),
        None => None,
    };
    let opt2 = match value.at() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::HistoryLine {
        task: value.task(),
        revision: value.revision(),
        by: crate::translate::facts::requester(value.by()),
        change: history_change_fields(value.change()),
        reason: crate::translate::facts::document_text(value.reason(), limits)?,
        proposal: opt0,
        amendment: opt1,
        at: opt2,
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn milestone_phase_fields(
    value: &w::MilestonePhase,
    limits: &Limits,
) -> Result<c::MilestonePhase, w::Problem> {
    Ok(match value {
        w::MilestonePhase::Waiting => c::MilestonePhase::Waiting,
        w::MilestonePhase::Active(row) => c::MilestonePhase::Active(crate::translate::facts::run_stage(row.value())),
        w::MilestonePhase::Closing(row) => c::MilestonePhase::Closing {
            stage: crate::translate::facts::closing_stage(row.stage()),
            ending: chip_ending_fields(row.ending(), limits)?,
        },
        w::MilestonePhase::Held(row) => {
            let opt0 = match row.source() {
                Some(item) => Some(crate::translate::facts::part(item)),
                None => None,
            };
            let opt1 = match row.ending() {
                Some(item) => Some(chip_ending_fields(item, limits)?),
                None => None,
            };
            c::MilestonePhase::Held {
                before: crate::translate::facts::prior_phase(row.before()),
                why: crate::translate::answers::hold_category(row.why()),
                source: opt0,
                ending: opt1,
            }
        }
        w::MilestonePhase::Ended(row) => c::MilestonePhase::Ended(chip_ending_fields(row.value(), limits)?),
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn milestone_line_fields(value: &w::MilestoneLine, limits: &Limits) -> Result<c::MilestoneLine, w::Problem> {
    let opt0 = match value.at() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::MilestoneLine {
        task: value.task(),
        position: value.position(),
        phase: milestone_phase_fields(value.phase(), limits)?,
        at: opt0,
    })
}

pub(crate) fn history_row_fields(value: &w::HistoryRow, limits: &Limits) -> Result<c::HistoryRow, w::Problem> {
    Ok(match value {
        w::HistoryRow::Change(row) => c::HistoryRow::Change(Box::new(history_line_fields(row.value(), limits)?)),
        w::HistoryRow::Milestone(row) => {
            c::HistoryRow::Milestone(Box::new(milestone_line_fields(row.value(), limits)?))
        }
    })
}

pub(crate) fn history_row_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::HistoryRow>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let item = match skein_codec::piece::split(&mut reader) {
            Ok(value) => value,
            Err(reason) => return Err(piece_problem(reason)),
        };
        let Some(item) = item else {
            return Ok(rows);
        };
        sum = sum
            .checked_add(u64::try_from(item.len()).expect("piece fits u64"))
            .ok_or(too_large())?
            .checked_add(4)
            .ok_or(too_large())?;
        if sum > limits.client().document_bytes {
            return Err(too_large());
        }
        let row = w::HistoryRow::decode(limits.wire(), &mut Reader::new(&item))?;
        let row = history_row_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn history_page_fields(value: &w::HistoryPage, limits: &Limits) -> Result<c::HistoryPage, w::Problem> {
    let opt0 = match value.before() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    let opt1 = match value.after() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    Ok(c::HistoryPage {
        cut: value.cut(),
        rows: history_row_piece(value.rows(), limits, limits.client().rows)?,
        before: opt0,
        after: opt1,
    })
}
