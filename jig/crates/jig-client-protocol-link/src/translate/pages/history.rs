//! Whole pages and single documents (protocol/client/wire.md, 1, and protocol/client/README.md, 4).
use super::{
    Box, Limits, List, Reader, Wall, authority_document_fields, c, chip_ending_fields, cursor_fields,
    message_proposal_kind_fields, proposed_task_piece, specification_document_fields, too_large, w,
};

pub(crate) fn history_change_fields(value: &w::HistoryChange) -> c::HistoryChange {
    match value {
        w::HistoryChange::Cancelled => c::HistoryChange::Cancelled,
        w::HistoryChange::Released => c::HistoryChange::Released,
        w::HistoryChange::Amended => c::HistoryChange::Amended,
        w::HistoryChange::Moved => c::HistoryChange::Moved,
        w::HistoryChange::Proposed => c::HistoryChange::Proposed,
        w::HistoryChange::ProposalPassed => c::HistoryChange::ProposalPassed,
        w::HistoryChange::ProposalAccepted => c::HistoryChange::ProposalAccepted,
        w::HistoryChange::ProposalRejected => c::HistoryChange::ProposalRejected,
        w::HistoryChange::ProposalWithdrawn => c::HistoryChange::ProposalWithdrawn,
    }
}

pub(crate) fn address_change_fields(value: &w::AddressChange) -> c::AddressChange {
    c::AddressChange {
        revision: crate::translate::answers::revision(value.revision()),
        address: crate::translate::facts::person_address(value.address()),
    }
}

pub(crate) fn retained_amendment_fields(
    value: &w::RetainedAmendment,
    limits: &Limits,
) -> Result<c::RetainedAmendment, w::Problem> {
    let opt0 = match value.addressed() {
        Some(item) => Some(address_change_fields(item)),
        None => None,
    };
    let opt1 = match value.instructions() {
        Some(item) => Some(crate::translate::facts::document_text(item, limits)?),
        None => None,
    };
    let opt2 = match value.release() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt3 = match value.spec() {
        Some(item) => Some(specification_document_fields(item, limits)?),
        None => None,
    };
    let opt4 = match value.authority() {
        Some(item) => Some(authority_document_fields(item, limits)?),
        None => None,
    };
    let opt5 = match value.wake() {
        Some(item) => Some(crate::translate::facts::wake_policy(item)),
        None => None,
    };
    let opt6 = match value.dependencies() {
        Some(item) => {
            if (item).len() > limits.client().rows {
                return Err(too_large());
            }
            let mut list0 = List::with_capacity((item).len());
            for item in item {
                list0.push(*item).expect("reserved supplied list");
            }
            Some(list0)
        }
        None => None,
    };
    Ok(c::RetainedAmendment {
        addressed: opt0,
        instructions: opt1,
        release: opt2,
        spec: opt3,
        authority: opt4,
        wake: opt5,
        dependencies: opt6,
        reason: crate::translate::facts::document_text(value.reason(), limits)?,
        cut: value.cut(),
    })
}

pub(crate) fn retained_action_fields(
    value: &w::RetainedAction,
    limits: &Limits,
) -> Result<c::RetainedAction, w::Problem> {
    Ok(match value {
        w::RetainedAction::Settings(row) => c::RetainedAction::Settings(
            crate::translate::settings_proposals::restore_owner_retained_action_settings(row, limits)?,
        ),
        w::RetainedAction::Batch(row) => {
            c::RetainedAction::Batch(proposed_task_piece(row.value(), limits, limits.client().rows)?)
        }
        w::RetainedAction::Effect(row) => c::RetainedAction::Effect {
            part: crate::translate::facts::part(row.part()),
            authority: authority_document_fields(row.authority(), limits)?,
            attempt: row.attempt(),
            completion: row.completion(),
            position: row.position(),
            purpose: row.purpose(),
        },
        w::RetainedAction::ProcedureEffect(row) => c::RetainedAction::ProcedureEffect {
            part: crate::translate::facts::part(row.part()),
            authority: authority_document_fields(row.authority(), limits)?,
            step: row.step(),
            purpose: row.purpose(),
        },
        w::RetainedAction::Amend(row) => c::RetainedAction::Amend {
            task: row.task(),
            change: Box::new(retained_amendment_fields(row.change(), limits)?),
        },
        w::RetainedAction::Widen(row) => c::RetainedAction::Widen {
            task: row.task(),
            authority: authority_document_fields(row.authority(), limits)?,
        },
        w::RetainedAction::Release(row) => c::RetainedAction::Release { task: row.task() },
    })
}

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
        reason: crate::translate::facts::document_text(value.reason(), limits)?,
        action: retained_action_fields(value.action(), limits)?,
        number: value.number(),
        proposer: value.proposer(),
        as_holder: value.as_holder(),
        state: proposal_history_state_fields(value.state(), limits)?,
    })
}

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
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
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
