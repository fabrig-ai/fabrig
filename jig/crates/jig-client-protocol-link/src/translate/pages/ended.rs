//! Whole pages and single documents (protocol/client/wire.md, 1, and protocol/client/README.md, 4).
use super::{
    Box, Limits, List, Reader, Wall, activity_line_fields, c, chip_ending_fields, chip_fields, cursor_fields,
    escalation_decision_record_fields, milestone_line_fields, object_key_fields, person_proposal_fields,
    person_task_fields, proposal_decision_fields, proposal_history_fields, question_fields, result_fields, task_fields,
    too_large, w,
};

pub(crate) fn ended_delegate_fields(value: &w::EndedDelegate, limits: &Limits) -> Result<c::EndedDelegate, w::Problem> {
    let opt0 = match value.at() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::EndedDelegate {
        chip: chip_fields(value.chip(), limits)?,
        ended: chip_ending_fields(value.ended(), limits)?,
        at: opt0,
    })
}

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
        let row = ended_delegate_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn ended_page_fields(value: &w::EndedPage, limits: &Limits) -> Result<c::EndedPage, w::Problem> {
    let opt0 = match value.before() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    let opt1 = match value.after() {
        Some(item) => Some(cursor_fields(item, limits)?),
        None => None,
    };
    Ok(c::EndedPage {
        rows: ended_delegate_piece(value.rows(), limits, limits.client().rows)?,
        before: opt0,
        after: opt1,
        cut: value.cut(),
    })
}

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
