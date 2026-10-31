//! Whole pages and single documents (protocol/client.md, 3 and 6).
use super::{
    Box, Limits, Wall, activity_line_fields, c, escalation_decision_fields, milestone_line_fields, object_key_fields,
    person_proposal_fields, person_task_fields, proposal_decision_fields, proposal_history_fields, question_fields,
    result_fields, task_fields, w,
};
pub(crate) fn escalation_decision_record_fields(
    value: &w::EscalationDecisionRecord,
    limits: &Limits,
) -> Result<c::EscalationDecisionRecord, w::Problem> {
    Ok(c::EscalationDecisionRecord {
        project: value.project(),
        requester: value.requester(),
        task: value.task(),
        revision: value.revision(),
        by: value.by(),
        at: Wall::from_nanos(value.at()),
        decision: escalation_decision_fields(value.decision(), limits)?,
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn ended_value_fields(value: &w::EndedValue, limits: &Limits) -> Result<c::EndedValue, w::Problem> {
    Ok(match value {
        w::EndedValue::Task(row) => c::EndedValue::Task {
            document: Box::new(task_fields(row.document(), limits)?),
            result: Box::new(result_fields(row.result(), limits)?),
        },
        w::EndedValue::Proposal(row) => c::EndedValue::Proposal(proposal_history_fields(row.value(), limits)?),
        w::EndedValue::PersonProposal(row) => {
            let opt0 = match row.decision() {
                Some(item) => Some(proposal_decision_fields(item)),
                None => None,
            };
            c::EndedValue::PersonProposal {
                proposal: Box::new(person_proposal_fields(row.proposal(), limits)?),
                decision: opt0,
            }
        }
        w::EndedValue::Escalation(row) => {
            let opt0 = match row.hold() {
                Some(item) => Some(milestone_line_fields(item, limits)?),
                None => None,
            };
            c::EndedValue::Escalation {
                task: Box::new(task_fields(row.task(), limits)?),
                decision: escalation_decision_record_fields(row.decision(), limits)?,
                hold: opt0,
            }
        }
        w::EndedValue::Question(row) => {
            let opt0 = match row.answer() {
                Some(item) => Some(activity_line_fields(item, limits)?),
                None => None,
            };
            let opt1 = match row.ending() {
                Some(item) => Some(Box::new(result_fields(item, limits)?)),
                None => None,
            };
            c::EndedValue::Question { question: question_fields(row.question(), limits)?, answer: opt0, ending: opt1 }
        }
        w::EndedValue::PersonTask(row) => c::EndedValue::PersonTask {
            document: Box::new(person_task_fields(row.document(), limits)?),
            result: Box::new(result_fields(row.result(), limits)?),
        },
    })
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn ended_fields(value: &w::Ended, limits: &Limits) -> Result<c::Ended, w::Problem> {
    let opt0 = match value.at() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt1 = match value.by() {
        Some(item) => Some(crate::translate::facts::requester(item)),
        None => None,
    };
    Ok(c::Ended {
        key: object_key_fields(value.key()),
        value: ended_value_fields(value.value(), limits)?,
        at: opt0,
        by: opt1,
    })
}
