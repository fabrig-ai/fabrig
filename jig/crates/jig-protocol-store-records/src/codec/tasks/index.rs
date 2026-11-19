//! Durable task values translated exhaustively (protocol/store/records.md, 1).

use crate::tasks_v13 as wire;

use jig_domain_tasks as domain;
use skein_lib::Wall;

/// Translate `AnswerLine` to its durable record.
pub fn answer_line(source: &domain::AnswerLine, _limits: &wire::Limits) -> Result<wire::AnswerLine, wire::Problem> {
    let domain::AnswerLine { task, position } = source;
    Ok(wire::AnswerLine { task: *task, position: *position })
}
/// Read `AnswerLine` from its durable record.
pub fn read_answer_line(source: wire::AnswerLine) -> Result<domain::AnswerLine, wire::Problem> {
    let wire::AnswerLine { task, position } = source;
    Ok(domain::AnswerLine { task, position })
}

/// Translate `QuestionLine` to its durable record.
pub fn question_line(
    source: &domain::QuestionLine,
    limits: &wire::Limits,
) -> Result<wire::QuestionLine, wire::Problem> {
    let domain::QuestionLine { message, line, answer } = source;
    Ok(wire::QuestionLine { message: *message, line: *line, answer: optional_answer(answer.as_ref(), limits)? })
}
/// Read `QuestionLine` from its durable record.
pub fn read_question_line(source: wire::QuestionLine) -> Result<domain::QuestionLine, wire::Problem> {
    let wire::QuestionLine { message, line, answer } = source;
    Ok(domain::QuestionLine { message, line, answer: read_optional_answer(answer)? })
}

/// Translate `ProposalRow` to its durable record.
pub fn proposal_row(source: &domain::ProposalRow, _limits: &wire::Limits) -> Result<wire::ProposalRow, wire::Problem> {
    let domain::ProposalRow { proposal, revision } = source;
    Ok(wire::ProposalRow { proposal: *proposal, revision: *revision })
}
/// Read `ProposalRow` from its durable record.
pub fn read_proposal_row(source: wire::ProposalRow) -> Result<domain::ProposalRow, wire::Problem> {
    let wire::ProposalRow { proposal, revision } = source;
    Ok(domain::ProposalRow { proposal, revision })
}

/// Translate `Endings` to its durable record.
pub fn endings(source: &domain::Endings, _limits: &wire::Limits) -> Result<wire::Endings, wire::Problem> {
    let domain::Endings { done, failed, cancelled } = source;
    Ok(wire::Endings { done: *done, failed: *failed, cancelled: *cancelled })
}
/// Read `Endings` from its durable record.
pub fn read_endings(source: wire::Endings) -> Result<domain::Endings, wire::Problem> {
    let wire::Endings { done, failed, cancelled } = source;
    Ok(domain::Endings { done, failed, cancelled })
}

/// Translate `UpdateObject` to its durable value.
pub fn update_object(
    source: &domain::UpdateObject,
    _limits: &wire::Limits,
) -> Result<wire::UpdateObject, wire::Problem> {
    Ok(match source {
        domain::UpdateObject::Proposal { proposal } => {
            wire::UpdateObject::Proposal(wire::ProposalUpdate { proposal: proposal.value() })
        }
        domain::UpdateObject::Result => wire::UpdateObject::Result,
        domain::UpdateObject::Reply { message, activity } => {
            wire::UpdateObject::Reply(wire::ReplyObject { message: message.value(), activity: activity.value() })
        }
    })
}
/// Read `UpdateObject` from its durable value.
pub fn read_update_object(source: wire::UpdateObject) -> Result<domain::UpdateObject, wire::Problem> {
    Ok(match source {
        wire::UpdateObject::Proposal(value) => {
            domain::UpdateObject::Proposal { proposal: domain::Newest::of(value.proposal) }
        }
        wire::UpdateObject::Result => domain::UpdateObject::Result,
        wire::UpdateObject::Reply(source) => {
            let wire::ReplyObject { message, activity } = source;
            domain::UpdateObject::Reply { message: domain::Newest::of(message), activity: domain::Newest::of(activity) }
        }
    })
}

/// Translate `Update` to its durable record.
pub fn update(source: &domain::Update, limits: &wire::Limits) -> Result<wire::Update, wire::Problem> {
    let domain::Update { party, at, task, object } = source;
    Ok(wire::Update { party: *party, at: at.as_nanos(), task: *task, object: update_object(object, limits)? })
}
/// Read `Update` from its durable record.
pub fn read_update(source: wire::Update) -> Result<domain::Update, wire::Problem> {
    let wire::Update { party, at, task, object } = source;
    Ok(domain::Update { party, at: Wall::from_nanos(at), task, object: read_update_object(object)? })
}

/// Translate `ResultRow` to its durable record.
pub fn result_row(source: &domain::ResultRow, _limits: &wire::Limits) -> Result<wire::ResultRow, wire::Problem> {
    let domain::ResultRow { party, position, task } = source;
    Ok(wire::ResultRow { party: *party, position: *position, task: *task })
}
/// Read `ResultRow` from its durable record.
pub fn read_result_row(source: wire::ResultRow) -> Result<domain::ResultRow, wire::Problem> {
    let wire::ResultRow { party, position, task } = source;
    Ok(domain::ResultRow { party, position, task })
}

/// Translate `Edge` to its durable record.
pub fn edge(source: &domain::Edge, _limits: &wire::Limits) -> Result<wire::Edge, wire::Problem> {
    let domain::Edge { requester, task } = source;
    Ok(wire::Edge { requester: *requester, task: *task })
}
/// Read `Edge` from its durable record.
pub fn read_edge(source: wire::Edge) -> Result<domain::Edge, wire::Problem> {
    let wire::Edge { requester, task } = source;
    Ok(domain::Edge { requester, task })
}

fn optional_answer(
    source: Option<&domain::AnswerLine>,
    limits: &wire::Limits,
) -> Result<Option<wire::AnswerLine>, wire::Problem> {
    match source {
        Some(source) => Ok(Some(answer_line(source, limits)?)),
        None => Ok(None),
    }
}
fn read_optional_answer(source: Option<wire::AnswerLine>) -> Result<Option<domain::AnswerLine>, wire::Problem> {
    match source {
        Some(source) => Ok(Some(read_answer_line(source)?)),
        None => Ok(None),
    }
}

/// The ended-by-requester row (protocol/store/loads.md, 4).
pub fn ended_by_party(row: &domain::EndedByParty, _limits: &wire::Limits) -> Result<wire::EndedByParty, wire::Problem> {
    Ok(wire::EndedByParty { party: row.party, label: row.label, ended_at: row.ended_at.as_nanos(), task: row.task })
}
/// Decode the ended-by-requester row.
#[must_use]
pub fn read_ended_by_party(row: wire::EndedByParty) -> domain::EndedByParty {
    let wire::EndedByParty { party, label, ended_at, task } = row;
    domain::EndedByParty { party, label, ended_at: Wall::from_nanos(ended_at), task }
}

/// The ended-by-project row (protocol/store/loads.md, 4).
pub fn ended_by_project(
    row: &domain::EndedByProject,
    _limits: &wire::Limits,
) -> Result<wire::EndedByProject, wire::Problem> {
    Ok(wire::EndedByProject {
        project: row.project,
        label: row.label,
        ended_at: row.ended_at.as_nanos(),
        task: row.task,
    })
}
/// Decode the ended-by-project row.
#[must_use]
pub fn read_ended_by_project(row: wire::EndedByProject) -> domain::EndedByProject {
    let wire::EndedByProject { project, label, ended_at, task } = row;
    domain::EndedByProject { project, label, ended_at: Wall::from_nanos(ended_at), task }
}
