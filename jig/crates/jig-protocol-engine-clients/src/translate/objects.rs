//! Whole single-read documents, exhaustive over the core vocabulary (protocol/client.md, 3 and 6).
use super::{deployment, owner, task, w};
use alloc::boxed::Box;
use jig_domain_core::{documents as d, reads};
mod current;
pub(super) mod ended;
pub(super) mod goal;
mod projects;
pub(super) mod proposals;
pub(crate) mod question_answer;
pub use current::{escalation, person_task, question, result};
pub use projects::project;
pub use proposals::proposal;
pub fn ended(value: &reads::Ended, models: &[crate::Model], limits: &w::Limits) -> Result<w::Ended, w::Problem> {
    let key = ended_key(&value.value, limits)?;
    let record = match &value.value {
        reads::EndedValue::Task { document, result } => w::EndedValue::Task(w::EndedValueTask::new(
            limits,
            w::EndedValueTaskParts {
                document: task::document(document, models, limits)?,
                result: current::result(models, result, limits)?,
            },
        )?),
        reads::EndedValue::Proposal(value) => w::EndedValue::Proposal(w::EndedValueProposal::new(
            limits,
            w::EndedValueProposalParts { value: ended::proposal_history(models, value, limits)? },
        )?),
        reads::EndedValue::PersonProposal { proposal, .. }
            if match proposal.action {
                jig_domain_tasks::PersonAction::Settings { .. } => true,
                jig_domain_tasks::PersonAction::Batch(_) | jig_domain_tasks::PersonAction::Amend { .. } => false,
            } =>
        {
            w::EndedValue::Proposal(w::EndedValueProposal::new(
                limits,
                w::EndedValueProposalParts { value: ended::settings_history(models, proposal, limits)? },
            )?)
        }
        reads::EndedValue::PersonProposal { proposal, decision } => {
            w::EndedValue::PersonProposal(w::EndedValuePersonProposal::new(
                limits,
                w::EndedValuePersonProposalParts {
                    proposal: ended::person_proposal(models, proposal, limits)?,
                    decision: match *decision {
                        Some(item) => Some(ended::proposal_decision(item, limits)?),
                        None => None,
                    },
                },
            )?)
        }
        reads::EndedValue::Escalation { task: document, decision, hold } => {
            w::EndedValue::Escalation(w::EndedValueEscalation::new(
                limits,
                w::EndedValueEscalationParts {
                    task: task::document(document, models, limits)?,
                    decision: ended::escalation_decision(decision, limits)?,
                    hold: match hold {
                        Some(item) => Some(ended::milestone(item, limits)?),
                        None => None,
                    },
                },
            )?)
        }
        reads::EndedValue::Question { question, answer, ending } => {
            w::EndedValue::Question(w::EndedValueQuestion::new(
                limits,
                w::EndedValueQuestionParts {
                    question: current::question(models, question, limits)?,
                    answer: match answer {
                        Some(item) => Some(question_answer::answer(item, limits)?),
                        None => None,
                    },
                    ending: match ending {
                        Some(item) => Some(result(models, item, limits)?),
                        None => None,
                    },
                },
            )?)
        }
        reads::EndedValue::PersonTask { document, result } => w::EndedValue::PersonTask(w::EndedValuePersonTask::new(
            limits,
            w::EndedValuePersonTaskParts {
                document: person_task(models, document, limits)?,
                result: current::result(models, result, limits)?,
            },
        )?),
        reads::EndedValue::Reply(_) | reads::EndedValue::Turn(_) => {
            return Err(w::Problem { path: w::Path::EndedValue, reason: skein_codec::Reason::Tag });
        }
    };
    w::Ended::new(
        limits,
        w::EndedParts {
            key,
            value: record,
            at: match value.at {
                Some(item) => Some(item.as_nanos()),
                None => None,
            },
            by: match value.by {
                Some(item) => Some(owner::requester(item, limits)?),
                None => None,
            },
        },
    )
}
pub fn document(
    answer: &reads::Answer,
    models: &[crate::Model],
    limits: &w::Limits,
) -> Result<w::ReadValue, w::Problem> {
    Ok(match answer {
        reads::Answer::Document(value) => match value.as_ref() {
            d::Document::Frame(value) => w::ReadValue::Me(owner::frame(value, models, limits)?),
            d::Document::Task(value) => w::ReadValue::Task(task::document(value, models, limits)?),
            d::Document::Proposal(value) => w::ReadValue::Proposal(proposal(models, value, limits)?),
            d::Document::Escalation(value) => w::ReadValue::Escalation(escalation(value, models, limits)?),
            d::Document::Question(value) => w::ReadValue::Question(question(models, value, limits)?),
            d::Document::PersonTask(value) => w::ReadValue::PersonTask(person_task(models, value, limits)?),
            d::Document::Result(value) => w::ReadValue::Result(result(models, value, limits)?),
            d::Document::Project(value) => w::ReadValue::Project(project(models, value, limits)?),
            d::Document::Settings(value) => w::ReadValue::Settings(super::settings::document(value, limits)?),
            d::Document::Deployment(value) => w::ReadValue::Deployment(deployment::document(value, limits)?),
            d::Document::Chips(_)
            | d::Document::Run(_)
            | d::Document::Tree(_)
            | d::Document::Lines(_)
            | d::Document::Accounts(_)
            | d::Document::Note(_)
            | d::Document::Inbox(_) => {
                return Err(w::Problem { path: w::Path::ReadValueTag, reason: skein_codec::Reason::Tag });
            }
        },
        reads::Answer::Ended(value) => match &value.value {
            reads::EndedValue::Turn(record) => w::ReadValue::Turn(super::activity::archive(record, limits)?),
            reads::EndedValue::Task { .. }
            | reads::EndedValue::Proposal(_)
            | reads::EndedValue::PersonProposal { .. }
            | reads::EndedValue::Escalation { .. }
            | reads::EndedValue::Question { .. }
            | reads::EndedValue::PersonTask { .. }
            | reads::EndedValue::Reply(_) => w::ReadValue::Ended(Box::new(ended(value, models, limits)?)),
        },
        reads::Answer::Setting(_)
        | reads::Answer::Revision(_)
        | reads::Answer::Revisions(_)
        | reads::Answer::TaskText(_)
        | reads::Answer::Tasks(_)
        | reads::Answer::Page(_)
        | reads::Answer::None(_) => {
            return Err(w::Problem { path: w::Path::ReadValueTag, reason: skein_codec::Reason::Tag });
        }
    })
}

fn ended_key(value: &reads::EndedValue, limits: &w::Limits) -> Result<w::ObjectKey, w::Problem> {
    Ok(match value {
        reads::EndedValue::Task { document, .. } => {
            w::ObjectKey::Task(w::ObjectKeyTask::new(limits, w::ObjectKeyTaskParts { value: document.chip.number })?)
        }
        reads::EndedValue::Proposal(value) => w::ObjectKey::Proposal(w::ObjectKeyProposal::new(
            limits,
            w::ObjectKeyProposalParts {
                value: task::proposal(
                    &d::ProposalRef { proposer: d::Proposer::Task(value.proposer), number: value.number },
                    limits,
                )?,
            },
        )?),
        reads::EndedValue::PersonProposal { proposal, .. } => w::ObjectKey::Proposal(w::ObjectKeyProposal::new(
            limits,
            w::ObjectKeyProposalParts {
                value: task::proposal(
                    &d::ProposalRef { proposer: d::Proposer::Party(proposal.proposer), number: proposal.number },
                    limits,
                )?,
            },
        )?),
        reads::EndedValue::Escalation { task: document, .. } => w::ObjectKey::Escalation(w::ObjectKeyEscalation::new(
            limits,
            w::ObjectKeyEscalationParts { value: document.chip.number },
        )?),
        reads::EndedValue::Question { question, .. } => w::ObjectKey::Question(w::ObjectKeyQuestion::new(
            limits,
            w::ObjectKeyQuestionParts { task: question.task.number, message: question.message },
        )?),
        reads::EndedValue::PersonTask { document, .. } => w::ObjectKey::PersonTask(w::ObjectKeyPersonTask::new(
            limits,
            w::ObjectKeyPersonTaskParts { value: document.task.number },
        )?),
        reads::EndedValue::Reply(_) | reads::EndedValue::Turn(_) => {
            return Err(w::Problem { path: w::Path::EndedValue, reason: skein_codec::Reason::Tag });
        }
    })
}
