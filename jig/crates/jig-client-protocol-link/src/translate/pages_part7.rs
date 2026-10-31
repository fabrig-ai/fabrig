//! Whole pages and single documents (protocol/client.md, 3 and 6).
use super::{
    Limits, List, Reader, Wall, c, opaque, owner_authority_fields, owner_message_proposal_kind_fields, piece_problem,
    specification_fields, task_contract_document_fields, text, too_large, w,
};
pub(crate) fn owner_goal_holding_fields(
    value: &w::OwnerGoalHolding,
    limits: &Limits,
) -> Result<c::OwnerGoalHolding, w::Problem> {
    Ok(match value {
        w::OwnerGoalHolding::Write(row) => {
            if row.path().len() > limits.client().segments {
                return Err(too_large());
            }
            let mut list0 = List::with_capacity(row.path().len());
            for item in row.path() {
                list0.push(opaque(item, limits)?).expect("reserved supplied list");
            }
            c::OwnerGoalHolding::Write { connector: row.connector(), path: list0, kind: row.kind() }
        }
        w::OwnerGoalHolding::Slot(row) => {
            if row.path().len() > limits.client().segments {
                return Err(too_large());
            }
            let mut list0 = List::with_capacity(row.path().len());
            for item in row.path() {
                list0.push(opaque(item, limits)?).expect("reserved supplied list");
            }
            c::OwnerGoalHolding::Slot { connector: row.connector(), path: list0, kind: row.kind() }
        }
    })
}

pub(crate) fn owner_goal_holding_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::OwnerGoalHolding>, w::Problem> {
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
        let row = w::OwnerGoalHolding::decode(limits.wire(), &mut Reader::new(&item))?;
        let row = owner_goal_holding_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_person_proposal_goal_fields(
    value: &w::OwnerPersonProposalGoal,
    limits: &Limits,
) -> Result<c::OwnerPersonProposalGoal, w::Problem> {
    if value.dependencies().len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list0 = List::with_capacity(value.dependencies().len());
    for item in value.dependencies() {
        list0.push(*item).expect("reserved supplied list");
    }
    let opt1 = match value.tracked() {
        Some(item) => Some(*item),
        None => None,
    };
    Ok(c::OwnerPersonProposalGoal {
        number: value.number(),
        project: value.project(),
        executor: crate::translate::facts::owner_executor(value.executor()),
        spec: specification_fields(value.spec(), limits)?,
        contract: task_contract_document_fields(value.contract(), limits)?,
        authority: owner_authority_fields(value.authority(), limits)?,
        numbers: crate::translate::facts::owner_numbers(value.numbers()),
        funder: crate::translate::facts::funder(value.funder()),
        dependencies: list0,
        holdings: owner_goal_holding_piece(value.holdings(), limits, limits.client().rows)?,
        wake: crate::translate::facts::owner_wake_policy(value.wake()),
        tracked: opt1,
    })
}

pub(crate) fn owner_person_proposal_state_fields(
    value: &w::OwnerPersonProposalState,
    limits: &Limits,
) -> Result<c::OwnerPersonProposalState, w::Problem> {
    Ok(match value {
        w::OwnerPersonProposalState::Pending(row) => {
            c::OwnerPersonProposalState::Pending { since: Wall::from_nanos(row.since()) }
        }
        w::OwnerPersonProposalState::Accepted(row) => {
            c::OwnerPersonProposalState::Accepted { by: crate::translate::facts::owner_requester(row.by()) }
        }
        w::OwnerPersonProposalState::Rejected(row) => c::OwnerPersonProposalState::Rejected {
            by: crate::translate::facts::owner_requester(row.by()),
            reason: text(row.reason(), limits)?,
            message: row.message(),
            at: Wall::from_nanos(row.at()),
        },
    })
}

pub(crate) fn owner_person_proposal_fields(
    value: &w::OwnerPersonProposal,
    limits: &Limits,
) -> Result<c::OwnerPersonProposal, w::Problem> {
    Ok(c::OwnerPersonProposal {
        number: value.number(),
        proposer: value.proposer(),
        project: value.project(),
        goal: owner_person_proposal_goal_fields(value.goal(), limits)?,
        state: owner_person_proposal_state_fields(value.state(), limits)?,
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn owner_proposal_decision_fields(value: &w::OwnerProposalDecision) -> c::OwnerProposalDecision {
    let opt0 = match value.created() {
        Some(item) => Some((item.first(), item.last())),
        None => None,
    };
    c::OwnerProposalDecision {
        project: value.project(),
        proposer: crate::translate::answers::proposer(value.proposer()),
        proposal: value.proposal(),
        kind: owner_message_proposal_kind_fields(value.kind()),
        by: value.by(),
        at: Wall::from_nanos(value.at()),
        choice: crate::translate::answers::owner_proposal_choice(value.choice()),
        created: opt0,
    }
}

pub(crate) fn owner_escalation_decision_fields(
    value: &w::OwnerEscalationDecision,
    limits: &Limits,
) -> Result<c::OwnerEscalationDecision, w::Problem> {
    Ok(match value {
        w::OwnerEscalationDecision::Release => c::OwnerEscalationDecision::Release,
        w::OwnerEscalationDecision::Reject(row) => {
            c::OwnerEscalationDecision::Reject { reason: text(row.reason(), limits)? }
        }
        w::OwnerEscalationDecision::Pass => c::OwnerEscalationDecision::Pass,
    })
}
