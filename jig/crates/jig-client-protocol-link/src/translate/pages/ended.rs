//! Whole pages and single documents (protocol/client/wire.md, 1, and protocol/client/README.md, 4).
use super::{
    Box, Limits, List, Reader, c, cursor_fields, person_task_fields, question_fields, result_fields, task_from_record,
    too_large, w,
};
use crate::generated::client;
use jig_client_protocol_wire::Record as _;

pub(crate) fn ended_delegate_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
) -> Result<List<c::EndedDelegate>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let row = w::EndedDelegate::decode(limits.wire(), &mut Reader::new(&item))?;
        let row = client::ended_delegate_to_domain(row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn ended_page_fields(value: &w::EndedPage, limits: &Limits) -> Result<c::EndedPage, w::Problem> {
    let opt0 = match &value.before {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    let opt1 = match &value.after {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    Ok(c::EndedPage {
        rows: ended_delegate_piece(&value.rows, limits, limits.client().rows)?,
        before: opt0,
        after: opt1,
        cut: value.cut,
    })
}

pub(crate) fn ended_value_fields(value: w::EndedValue, limits: &Limits) -> Result<c::EndedValue, w::Problem> {
    Ok(match value {
        w::EndedValue::Task(row) => c::EndedValue::Task {
            document: Box::new(task_from_record(row.document, limits)?),
            result: Box::new(result_fields(&row.result, limits)?),
        },
        w::EndedValue::Proposal(row) => c::EndedValue::Proposal(client::proposal_history_to_domain(row.value, limits)?),
        w::EndedValue::PersonProposal(row) => {
            let opt0 = match row.decision {
                Some(item) => Some(client::proposal_decision_to_domain(item)),
                None => None,
            };
            c::EndedValue::PersonProposal {
                proposal: Box::new(client::person_proposal_to_domain(row.proposal, limits)?),
                decision: opt0,
            }
        }
        w::EndedValue::Escalation(row) => {
            let opt0 = match row.hold {
                Some(item) => Some(client::hold_line_to_domain(item, limits)?),
                None => None,
            };
            c::EndedValue::Escalation {
                task: Box::new(task_from_record(row.task, limits)?),
                decision: client::escalation_decision_record_to_domain(row.decision, limits)?,
                hold: opt0,
            }
        }
        w::EndedValue::Question(row) => {
            let opt0 = match row.answer {
                Some(item) => Some(client::activity_line_to_domain(item, limits)?),
                None => None,
            };
            let opt1 = match &row.ending {
                Some(item) => Some(Box::new(result_fields(item, limits)?)),
                None => None,
            };
            c::EndedValue::Question { question: question_fields(&row.question, limits)?, answer: opt0, ending: opt1 }
        }
        w::EndedValue::PersonTask(row) => c::EndedValue::PersonTask {
            document: Box::new(person_task_fields(&row.document, limits)?),
            result: Box::new(result_fields(&row.result, limits)?),
        },
    })
}
