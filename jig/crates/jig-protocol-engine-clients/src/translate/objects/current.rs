//! Whole single-read owner documents (protocol/client/wire.md, 1, and protocol/client/README.md, 4).
use super::super::{chips, owner, task, w};
use alloc::boxed::Box;
use jig_client_protocol_wire::Record as _;
use jig_domain_core_vocabulary::documents as d;
use skein_lib::{List, Writer};
pub fn result(
    models: &[crate::Model],
    value: &d::ResultDoc,
    limits: &w::Limits,
) -> Result<w::ResultDocument, w::Problem> {
    let mut facts = List::with_capacity(4);
    facts
        .push(w::Fact::Ending(w::EndingRow {
            parent: None,
            value: w::EndingFields::Task(w::EndingTaskFields { value: 1 }),
        }))
        .expect("one ending root");
    chips::append_ending(&value.ending, Some(0), &mut facts, limits)?;
    let ending = w::Ending { rows: chips::fact_pieces(&facts, limits.ending_rows, w::paths::ENDING_ROWS, limits)? };
    let result = match &value.ending {
        d::Ending::Done(item) => Some(result_value(item, limits)?),
        d::Ending::Failed(_) => None,
        d::Ending::Cancelled { result, .. } => match result {
            Some(item) => Some(result_value(item, limits)?),
            None => None,
        },
    };
    Ok(w::ResultDocument {
        task: chips::chip(&value.task, limits)?,
        ending,
        value: result,
        at: match value.at {
            Some(item) => Some(item.as_nanos()),
            None => None,
        },
        details: Some(details(value, limits)?),
        acts: task::offered(models, &value.acts, limits, limits.result_document_acts, w::paths::RESULT_DOCUMENT_ACTS)?,
    })
}
pub fn details(value: &d::ResultDoc, limits: &w::Limits) -> Result<w::ResultDetails, w::Problem> {
    let mut delegates = List::with_capacity(value.delegates.len());
    for row in &value.delegates {
        delegates.push(task::link(row, limits)?).expect("supplied rows fit");
    }
    let delegates =
        task::links_pieces(&delegates, limits.result_details_delegates, w::paths::RESULT_DETAILS_DELEGATES, limits)?;
    let followups = match &value.followups {
        Some(rows) => {
            let mut links = List::with_capacity(rows.len());
            for row in rows {
                links.push(task::link(row, limits)?).expect("supplied rows fit");
            }
            let links = task::links_pieces(
                &links,
                limits.result_details_followups_some,
                w::paths::RESULT_DETAILS_FOLLOWUPS,
                limits,
            )?;
            Some(links)
        }
        None => None,
    };
    Ok(w::ResultDetails {
        contract: owner::contract(&value.contract, limits)?,
        requester: owner::requester(value.requester, limits)?,
        executor: owner::executor(value.executor, limits)?,
        chosen_by: value.chosen_by,
        delegates,
        followups,
        proposal: match &value.proposal {
            Some(item) => Some(task::proposal(item, limits)?),
            None => None,
        },
        cut: value.cut,
    })
}
pub fn result_value(value: &d::ResultValue, limits: &w::Limits) -> Result<w::ResultValue, w::Problem> {
    let mut facts = List::with_capacity(1);
    facts
        .push(w::Fact::ResultValue(w::ResultValueRow { parent: None, value: chips::result_fields(value, limits)? }))
        .expect("one result fact");
    Ok(w::ResultValue {
        rows: chips::fact_pieces(&facts, limits.result_value_rows, w::paths::RESULT_VALUE_ROWS, limits)?,
    })
}
pub fn person_task(
    models: &[crate::Model],
    value: &d::PersonTask,
    limits: &w::Limits,
) -> Result<w::PersonTask, w::Problem> {
    let mut results = List::with_capacity(value.results.len());
    for row in &value.results {
        results
            .push(w::ShownResult {
                task: task::link(&row.task, limits)?,
                result: match &row.result {
                    Some(item) => Some(result(models, item, limits)?),
                    None => None,
                },
            })
            .expect("supplied rows fit");
    }
    let results = shown_result_pieces(&results, limits.person_task_results, w::paths::PERSON_TASK_RESULTS, limits)?;

    Ok(w::PersonTask {
        task: chips::chip(&value.task, limits)?,
        revision: w::Revision { value: value.revision },
        requester: owner::requester(value.requester, limits)?,
        spec: owner::specification(&value.spec, limits)?,
        address: chips::address(value.address, limits)?,
        taken_by: value.taken_by,
        since: match value.since {
            Some(item) => Some(item.as_nanos()),
            None => None,
        },
        stall: value.stall,
        deadline: match value.deadline {
            Some(item) => Some(item.as_nanos()),
            None => None,
        },
        contract: owner::contract(&value.contract, limits)?,
        results,
        cut: value.cut,
        acts: task::offered(models, &value.acts, limits, limits.person_task_acts, w::paths::PERSON_TASK_ACTS)?,
    })
}
pub fn question(models: &[crate::Model], value: &d::Question, limits: &w::Limits) -> Result<w::Question, w::Problem> {
    Ok(w::Question {
        task: chips::chip(&value.task, limits)?,
        from: Some(owner::requester(value.from, limits)?),
        message: value.message,
        words: owner::text(&value.words, limits)?,
        since: value.at.as_nanos(),
        acts: task::offered(models, &value.acts, limits, limits.question_acts, w::paths::QUESTION_ACTS)?,
        ending: None,
    })
}
pub fn escalation(
    value: &d::Escalation,
    models: &[crate::Model],
    limits: &w::Limits,
) -> Result<w::Escalation, w::Problem> {
    Ok(w::Escalation {
        task: task::document(&value.task, models, limits)?,
        acts: task::offered(models, &value.acts, limits, limits.escalation_acts, w::paths::ESCALATION_ACTS)?,
    })
}
pub(super) fn shown_result_pieces(
    rows: &List<w::ShownResult>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(w::Problem { path, reason: skein_codec::Reason::Bound }),
        }
        encoded.push(out.finish()).expect("supplied rows fit");
    }
    super::super::pieces(&encoded, bound, path)
}
