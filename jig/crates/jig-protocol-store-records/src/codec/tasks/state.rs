//! Durable task values translated exhaustively (protocol/store/records.md, 1).
use crate::generated::tasks as mapped;
use crate::tasks_v13 as wire;
use alloc::boxed::Box;
use jig_domain_tasks as domain;
use skein_lib::Wall;

/// Translate `Status` to its durable tag.
#[must_use]
pub fn status(source: domain::Status) -> wire::Status {
    mapped::status_to_record(&source)
}

/// Translate `InvalidResult` to its durable tag.
#[must_use]
pub fn invalid_result(source: domain::InvalidResult) -> wire::InvalidResult {
    mapped::invalid_result_to_record(&source)
}

/// Translate `Escalation` to its durable value.
#[expect(clippy::match_same_arms, reason = "the stored number and kind are separate fields")]
#[must_use]
pub fn escalation(source: &domain::Escalation) -> wire::Escalation {
    match source {
        domain::Escalation::Unheld { revision } => {
            wire::Escalation::Unheld(wire::UnheldEscalation { revision: *revision })
        }
        domain::Escalation::Routing { revision } => {
            wire::Escalation::Routing(wire::RoutingEscalation { revision: *revision })
        }
        domain::Escalation::Waiting { revision, holder, entry, since } => {
            wire::Escalation::Waiting(wire::WaitingEscalation {
                revision: *revision,
                holder: mapped::escalation_holder_to_record(holder),
                entry: *entry,
                since: since.as_nanos(),
            })
        }
        domain::Escalation::Rejected { revision, by, reason } => wire::Escalation::Rejected(wire::RejectedEscalation {
            revision: *revision,
            by: match by {
                domain::EscalationDecider::Task(number) => *number,
                domain::EscalationDecider::Person(number) => *number,
            },
            reason: reason.clone(),
            by_kind: match by {
                domain::EscalationDecider::Task(_) => wire::DeciderKind::Task,
                domain::EscalationDecider::Person(_) => wire::DeciderKind::Person,
            },
        }),
    }
}
/// Read `Escalation` from its durable value.
#[must_use]
pub fn read_escalation(source: wire::Escalation) -> domain::Escalation {
    match source {
        wire::Escalation::Unheld(source) => {
            let wire::UnheldEscalation { revision } = source;
            domain::Escalation::Unheld { revision }
        }
        wire::Escalation::Routing(source) => {
            let wire::RoutingEscalation { revision } = source;
            domain::Escalation::Routing { revision }
        }
        wire::Escalation::Waiting(source) => {
            let wire::WaitingEscalation { revision, holder, entry, since } = source;
            domain::Escalation::Waiting {
                revision,
                holder: mapped::escalation_holder_to_domain(holder),
                entry,
                since: Wall::from_nanos(since),
            }
        }
        wire::Escalation::Rejected(source) => {
            let wire::RejectedEscalation { revision, by, reason, by_kind } = source;
            domain::Escalation::Rejected {
                revision,
                by: match by_kind {
                    wire::DeciderKind::Person => domain::EscalationDecider::Person(by),
                    wire::DeciderKind::Task => domain::EscalationDecider::Task(by),
                },
                reason,
            }
        }
    }
}

/// Translate `PersonAnswer` to its durable record.
#[must_use]
pub fn person_answer(source: &domain::PersonAnswer) -> wire::PersonAnswer {
    mapped::person_answer_to_record(source)
}

/// Translate `QuestionCredit` to its durable record.
#[must_use]
pub fn question_credit(source: &domain::QuestionCredit) -> wire::QuestionCredit {
    mapped::question_credit_to_record(source)
}

/// Translate `SavedResource` to its durable record.
pub fn saved_resource(
    source: &domain::SavedResource,
    limits: &wire::Limits,
) -> Result<wire::SavedResource, wire::Problem> {
    let domain::SavedResource { connector, path } = source;
    Ok(wire::SavedResource { connector: *connector, path: saved_path(path, limits)? })
}
/// Read `SavedResource` from its durable record.
pub fn read_saved_resource(
    source: wire::SavedResource,
    limits: &wire::Limits,
) -> Result<domain::SavedResource, wire::Problem> {
    let wire::SavedResource { connector, path } = source;
    Ok(domain::SavedResource {
        connector,
        path: super::pieces::read_authority_segments(path, limits, wire::paths::SAVED_RESOURCE_PATH)?,
    })
}

fn saved_path(source: &[Box<[u8]>], limits: &wire::Limits) -> Result<Box<[u8]>, wire::Problem> {
    super::pieces::authority_segments(source, limits, limits.saved_resource_path, wire::paths::SAVED_RESOURCE_PATH)
}
