//! Whole single-read documents, exhaustive over the core vocabulary (protocol/client/wire.md, 1, and protocol/client/README.md, 4).
use super::{deployment, owner, task, w};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::{documents as d, reads};
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
        reads::EndedValue::Task { document, result } => w::EndedValue::Task(w::EndedValueTask {
            document: task::document(document, models, limits)?,
            result: current::result(models, result, limits)?,
        }),
        reads::EndedValue::Proposal(value) | reads::EndedValue::PartyProposal(value) => {
            w::EndedValue::Proposal(w::EndedValueProposal { value: ended::proposal_history(models, value, limits)? })
        }
        reads::EndedValue::PersonProposal { proposal, decision } => match &proposal.action {
            jig_domain_tasks::PersonAction::Settings { against, items } => {
                w::EndedValue::Proposal(w::EndedValueProposal {
                    value: ended::settings_history(models, proposal, *against, items, limits)?,
                })
            }
            jig_domain_tasks::PersonAction::Batch(_) | jig_domain_tasks::PersonAction::Amend { .. } => {
                w::EndedValue::PersonProposal(w::EndedValuePersonProposal {
                    proposal: ended::person_proposal(models, proposal, limits)?,
                    decision: match *decision {
                        Some(item) => Some(ended::proposal_decision(item, limits)?),
                        None => None,
                    },
                })
            }
        },
        reads::EndedValue::Escalation { task: document, decision, hold } => {
            w::EndedValue::Escalation(w::EndedValueEscalation {
                task: task::document(document, models, limits)?,
                decision: ended::escalation_decision(decision, limits)?,
                hold: match hold {
                    Some(item) => Some(ended::hold(item, limits)?),
                    None => None,
                },
            })
        }
        reads::EndedValue::Question { question, answer, ending } => w::EndedValue::Question(w::EndedValueQuestion {
            question: current::question(models, question, limits)?,
            answer: match answer {
                Some(item) => Some(question_answer::answer(item, limits)?),
                None => None,
            },
            ending: match ending {
                Some(item) => Some(result(models, item, limits)?),
                None => None,
            },
        }),
        reads::EndedValue::PersonTask { document, result } => w::EndedValue::PersonTask(w::EndedValuePersonTask {
            document: person_task(models, document, limits)?,
            result: current::result(models, result, limits)?,
        }),
        reads::EndedValue::Reply(_) | reads::EndedValue::Turn(_) | reads::EndedValue::WithheldTurn(_) => {
            return Err(w::Problem { path: w::paths::ENDED_VALUE, reason: skein_codec::Reason::Tag });
        }
    };
    Ok(w::Ended {
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
    })
}
pub fn document(
    answer: &reads::Answer,
    models: &[crate::Model],
    private: &crate::Private,
    limits: &w::Limits,
) -> Result<w::ReadValue, w::Problem> {
    Ok(match answer {
        reads::Answer::Document(value) => match value.as_ref() {
            d::Document::Task(value) => w::ReadValue::Task(task::document(value, models, limits)?),
            d::Document::Proposal(value) => w::ReadValue::Proposal(proposal(models, value, limits)?),
            d::Document::Escalation(value) => w::ReadValue::Escalation(escalation(value, models, limits)?),
            d::Document::Question(value) => w::ReadValue::Question(question(models, value, limits)?),
            d::Document::PersonTask(value) => w::ReadValue::PersonTask(person_task(models, value, limits)?),
            d::Document::Result(value) => w::ReadValue::Result(result(models, value, limits)?),
            d::Document::Project(value) => w::ReadValue::Project(project(models, value, limits)?),
            d::Document::Settings(value) => w::ReadValue::Settings(super::settings::document(value, limits)?),
            d::Document::Deployment(value) => w::ReadValue::Deployment(deployment::document(value, limits)?),
            // The component translates me with the reading sign-in's expiry.
            d::Document::Frame(_)
            | d::Document::Chips(_)
            | d::Document::Run(_)
            | d::Document::Tree(_)
            | d::Document::Lines(_)
            | d::Document::Terminal { .. }
            | d::Document::Accounts(_)
            | d::Document::Note(_)
            | d::Document::Inbox(_) => {
                return Err(w::Problem { path: w::paths::READ_VALUE_TAG, reason: skein_codec::Reason::Tag });
            }
        },
        reads::Answer::Ended(value) => match &value.value {
            reads::EndedValue::WithheldTurn(record) => {
                w::ReadValue::Turn(super::activity::archive(record, Some(private), limits)?)
            }
            reads::EndedValue::Turn(record) => w::ReadValue::Turn(super::activity::archive(record, None, limits)?),
            reads::EndedValue::Task { .. }
            | reads::EndedValue::Proposal(_)
            | reads::EndedValue::PartyProposal(_)
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
            return Err(w::Problem { path: w::paths::READ_VALUE_TAG, reason: skein_codec::Reason::Tag });
        }
    })
}

fn ended_key(value: &reads::EndedValue, limits: &w::Limits) -> Result<w::ObjectKey, w::Problem> {
    Ok(match value {
        reads::EndedValue::Task { document, .. } => {
            w::ObjectKey::Task(w::ObjectKeyTask { value: document.chip.number })
        }
        reads::EndedValue::Proposal(value) => w::ObjectKey::Proposal(w::ObjectKeyProposal {
            value: task::proposal(
                &d::ProposalRef { proposer: d::Proposer::Task(value.proposer), number: value.number },
                limits,
            )?,
        }),
        reads::EndedValue::PartyProposal(value) => w::ObjectKey::Proposal(w::ObjectKeyProposal {
            value: task::proposal(
                &d::ProposalRef { proposer: d::Proposer::Party(value.proposer), number: value.number },
                limits,
            )?,
        }),
        reads::EndedValue::PersonProposal { proposal, .. } => w::ObjectKey::Proposal(w::ObjectKeyProposal {
            value: task::proposal(
                &d::ProposalRef { proposer: d::Proposer::Party(proposal.proposer), number: proposal.number },
                limits,
            )?,
        }),
        reads::EndedValue::Escalation { task: document, .. } => {
            w::ObjectKey::Escalation(w::ObjectKeyEscalation { value: document.chip.number })
        }
        reads::EndedValue::Question { question, .. } => {
            w::ObjectKey::Question(w::ObjectKeyQuestion { task: question.task.number, message: question.message })
        }
        reads::EndedValue::PersonTask { document, .. } => {
            w::ObjectKey::PersonTask(w::ObjectKeyPersonTask { value: document.task.number })
        }
        reads::EndedValue::Reply(_) | reads::EndedValue::Turn(_) | reads::EndedValue::WithheldTurn(_) => {
            return Err(w::Problem { path: w::paths::ENDED_VALUE, reason: skein_codec::Reason::Tag });
        }
    })
}
