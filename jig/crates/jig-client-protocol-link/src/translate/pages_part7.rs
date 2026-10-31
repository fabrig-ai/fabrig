//! Whole pages and single documents (protocol/client.md, 3 and 6).
use super::{
    Limits, List, Reader, Wall, authority_fields, c, message_proposal_kind_fields, opaque, piece_problem,
    specification_fields, task_contract_document_fields, text, too_large, w,
};
pub(crate) fn goal_holding_fields(value: &w::GoalHolding, limits: &Limits) -> Result<c::GoalHolding, w::Problem> {
    Ok(match value {
        w::GoalHolding::Write(row) => {
            if row.path().len() > limits.client().segments {
                return Err(too_large());
            }
            let mut list0 = List::with_capacity(row.path().len());
            for item in row.path() {
                list0.push(opaque(item, limits)?).expect("reserved supplied list");
            }
            c::GoalHolding::Write { connector: row.connector(), path: list0, kind: row.kind() }
        }
        w::GoalHolding::Slot(row) => {
            if row.path().len() > limits.client().segments {
                return Err(too_large());
            }
            let mut list0 = List::with_capacity(row.path().len());
            for item in row.path() {
                list0.push(opaque(item, limits)?).expect("reserved supplied list");
            }
            c::GoalHolding::Slot { connector: row.connector(), path: list0, kind: row.kind() }
        }
    })
}

pub(crate) fn goal_holding_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::GoalHolding>, w::Problem> {
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
        let row = w::GoalHolding::decode(limits.wire(), &mut Reader::new(&item))?;
        let row = goal_holding_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn person_proposal_goal_fields(
    value: &w::PersonProposalGoal,
    limits: &Limits,
) -> Result<c::PersonProposalGoal, w::Problem> {
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
    Ok(c::PersonProposalGoal {
        number: value.number(),
        project: value.project(),
        executor: crate::translate::facts::executor(value.executor()),
        spec: specification_fields(value.spec(), limits)?,
        contract: task_contract_document_fields(value.contract(), limits)?,
        authority: authority_fields(value.authority(), limits)?,
        numbers: crate::translate::facts::numbers(value.numbers()),
        funder: crate::translate::facts::funder(value.funder()),
        dependencies: list0,
        holdings: goal_holding_piece(value.holdings(), limits, limits.client().rows)?,
        wake: crate::translate::facts::wake_policy(value.wake()),
        tracked: opt1,
    })
}

pub(crate) fn person_proposal_state_fields(
    value: &w::PersonProposalState,
    limits: &Limits,
) -> Result<c::PersonProposalState, w::Problem> {
    Ok(match value {
        w::PersonProposalState::Pending(row) => {
            c::PersonProposalState::Pending { since: Wall::from_nanos(row.since()) }
        }
        w::PersonProposalState::Accepted(row) => {
            c::PersonProposalState::Accepted { by: crate::translate::facts::requester(row.by()) }
        }
        w::PersonProposalState::Rejected(row) => c::PersonProposalState::Rejected {
            by: crate::translate::facts::requester(row.by()),
            reason: text(row.reason(), limits)?,
            message: row.message(),
            at: Wall::from_nanos(row.at()),
        },
    })
}

pub(crate) fn person_proposal_fields(
    value: &w::PersonProposal,
    limits: &Limits,
) -> Result<c::PersonProposal, w::Problem> {
    Ok(c::PersonProposal {
        number: value.number(),
        proposer: value.proposer(),
        project: value.project(),
        goal: person_proposal_goal_fields(value.goal(), limits)?,
        state: person_proposal_state_fields(value.state(), limits)?,
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn proposal_decision_fields(value: &w::ProposalDecision) -> c::ProposalDecision {
    let opt0 = match value.created() {
        Some(item) => Some((item.first(), item.last())),
        None => None,
    };
    c::ProposalDecision {
        project: value.project(),
        proposer: crate::translate::answers::proposer(value.proposer()),
        proposal: value.proposal(),
        kind: message_proposal_kind_fields(value.kind()),
        by: value.by(),
        at: Wall::from_nanos(value.at()),
        choice: crate::translate::answers::proposal_choice(value.choice()),
        created: opt0,
    }
}

pub(crate) fn escalation_decision_fields(
    value: &w::EscalationDecision,
    limits: &Limits,
) -> Result<c::EscalationDecision, w::Problem> {
    Ok(match value {
        w::EscalationDecision::Release => c::EscalationDecision::Release,
        w::EscalationDecision::Reject(row) => c::EscalationDecision::Reject { reason: text(row.reason(), limits)? },
        w::EscalationDecision::Pass => c::EscalationDecision::Pass,
    })
}
