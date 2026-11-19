//! Durable task values translated exhaustively (protocol/store/records.md, 1).
use super::phase;
use crate::tasks_v13 as wire;
use alloc::boxed::Box;
use jig_domain_tasks as domain;
use skein_lib::{Token, Wall};

/// Translate `Status` to its durable tag.
#[must_use]
pub fn status(source: domain::Status) -> wire::Status {
    match source {
        domain::Status::Done => wire::Status::Done,
        domain::Status::Failed => wire::Status::Failed,
        domain::Status::Cancelled => wire::Status::Cancelled,
    }
}
/// Read `Status` from its durable tag.
#[must_use]
pub fn read_status(source: wire::Status) -> domain::Status {
    match source {
        wire::Status::Done => domain::Status::Done,
        wire::Status::Failed => domain::Status::Failed,
        wire::Status::Cancelled => domain::Status::Cancelled,
    }
}

/// Translate `InvalidResult` to its durable tag.
#[must_use]
pub fn invalid_result(source: domain::InvalidResult) -> wire::InvalidResult {
    match source {
        domain::InvalidResult::Form => wire::InvalidResult::Form,
        domain::InvalidResult::Verdict => wire::InvalidResult::Verdict,
        domain::InvalidResult::Words => wire::InvalidResult::Words,
        domain::InvalidResult::Change => wire::InvalidResult::Change,
        domain::InvalidResult::Followups => wire::InvalidResult::Followups,
    }
}
/// Read `InvalidResult` from its durable tag.
#[must_use]
pub fn read_invalid_result(source: wire::InvalidResult) -> domain::InvalidResult {
    match source {
        wire::InvalidResult::Form => domain::InvalidResult::Form,
        wire::InvalidResult::Verdict => domain::InvalidResult::Verdict,
        wire::InvalidResult::Words => domain::InvalidResult::Words,
        wire::InvalidResult::Change => domain::InvalidResult::Change,
        wire::InvalidResult::Followups => domain::InvalidResult::Followups,
    }
}

/// Translate `Stub` to its durable record.
pub fn stub(source: &domain::Stub, _limits: &wire::Limits) -> Result<wire::Stub, wire::Problem> {
    let domain::Stub { task, phase, result } = source;
    Ok(wire::Stub { task: *task, phase: status(*phase), result: result.raw() })
}
/// Read `Stub` from its durable record.
pub fn read_stub(source: wire::Stub) -> Result<domain::Stub, wire::Problem> {
    let wire::Stub { task, phase, result } = source;
    Ok(domain::Stub { task, phase: read_status(phase), result: Token::new(result) })
}

/// Translate `ReplyHint` to its durable record.
pub fn reply_hint(source: &domain::ReplyHint, _limits: &wire::Limits) -> Result<wire::ReplyHint, wire::Problem> {
    let domain::ReplyHint { number, activity, attempt, turn, at } = source;
    Ok(wire::ReplyHint { number: *number, activity: *activity, attempt: *attempt, turn: *turn, at: at.as_nanos() })
}
/// Read `ReplyHint` from its durable record.
pub fn read_reply_hint(source: wire::ReplyHint) -> Result<domain::ReplyHint, wire::Problem> {
    let wire::ReplyHint { number, activity, attempt, turn, at } = source;
    Ok(domain::ReplyHint { number, activity, attempt, turn, at: Wall::from_nanos(at) })
}

/// Translate `EscalationHolder` to its durable value.
pub fn escalation_holder(
    source: &domain::EscalationHolder,
    _limits: &wire::Limits,
) -> Result<wire::EscalationHolder, wire::Problem> {
    Ok(match source {
        domain::EscalationHolder::Task(task) => wire::EscalationHolder::Task(wire::EscalationTask { task: *task }),
        domain::EscalationHolder::Person(person) => {
            wire::EscalationHolder::Person(wire::EscalationPerson { person: *person })
        }
        domain::EscalationHolder::Role { project, role } => {
            wire::EscalationHolder::Role(wire::EscalationRole { project: *project, role: *role })
        }
    })
}
/// Read `EscalationHolder` from its durable value.
pub fn read_escalation_holder(source: wire::EscalationHolder) -> Result<domain::EscalationHolder, wire::Problem> {
    Ok(match source {
        wire::EscalationHolder::Task(source) => {
            let wire::EscalationTask { task } = source;
            domain::EscalationHolder::Task(task)
        }
        wire::EscalationHolder::Person(source) => {
            let wire::EscalationPerson { person } = source;
            domain::EscalationHolder::Person(person)
        }
        wire::EscalationHolder::Role(source) => {
            let wire::EscalationRole { project, role } = source;
            domain::EscalationHolder::Role { project, role }
        }
    })
}

/// Translate `Escalation` to its durable value.
#[expect(clippy::match_same_arms, reason = "the stored number and kind are separate fields")]
pub fn escalation(source: &domain::Escalation, limits: &wire::Limits) -> Result<wire::Escalation, wire::Problem> {
    Ok(match source {
        domain::Escalation::Unheld { revision } => {
            wire::Escalation::Unheld(wire::UnheldEscalation { revision: *revision })
        }
        domain::Escalation::Routing { revision } => {
            wire::Escalation::Routing(wire::RoutingEscalation { revision: *revision })
        }
        domain::Escalation::Waiting { revision, holder, entry, since } => {
            wire::Escalation::Waiting(wire::WaitingEscalation {
                revision: *revision,
                holder: escalation_holder(holder, limits)?,
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
    })
}
/// Read `Escalation` from its durable value.
pub fn read_escalation(source: wire::Escalation) -> Result<domain::Escalation, wire::Problem> {
    Ok(match source {
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
                holder: read_escalation_holder(holder)?,
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
    })
}

/// Translate `PersonAnswer` to its durable record.
pub fn person_answer(
    source: &domain::PersonAnswer,
    _limits: &wire::Limits,
) -> Result<wire::PersonAnswer, wire::Problem> {
    let domain::PersonAnswer { by, at } = source;
    Ok(wire::PersonAnswer { by: *by, at: at.as_nanos() })
}
/// Read `PersonAnswer` from its durable record.
pub fn read_person_answer(source: wire::PersonAnswer) -> Result<domain::PersonAnswer, wire::Problem> {
    let wire::PersonAnswer { by, at } = source;
    Ok(domain::PersonAnswer { by, at: Wall::from_nanos(at) })
}

/// Translate `PersonStallNotice` to its durable record.
pub fn person_stall_notice(
    source: &domain::PersonStallNotice,
    _limits: &wire::Limits,
) -> Result<wire::PersonStallNotice, wire::Problem> {
    let domain::PersonStallNotice { message, at, activity } = source;
    Ok(wire::PersonStallNotice { message: *message, at: at.as_nanos(), activity: *activity })
}
/// Read `PersonStallNotice` from its durable record.
pub fn read_person_stall_notice(source: wire::PersonStallNotice) -> Result<domain::PersonStallNotice, wire::Problem> {
    let wire::PersonStallNotice { message, at, activity } = source;
    Ok(domain::PersonStallNotice { message, at: Wall::from_nanos(at), activity })
}

/// Translate `PersonWait` to its durable record.
pub fn person_wait(source: &domain::PersonWait, limits: &wire::Limits) -> Result<wire::PersonWait, wire::Problem> {
    let domain::PersonWait { since, stall, deadline, notice } = source;
    Ok(wire::PersonWait {
        since: since.as_nanos(),
        stall: *stall,
        deadline: deadline.as_nanos(),
        notice: optional_notice(notice.as_ref(), limits)?,
    })
}
/// Read `PersonWait` from its durable record.
pub fn read_person_wait(source: wire::PersonWait) -> Result<domain::PersonWait, wire::Problem> {
    let wire::PersonWait { since, stall, deadline, notice } = source;
    Ok(domain::PersonWait {
        since: Wall::from_nanos(since),
        stall,
        deadline: Wall::from_nanos(deadline),
        notice: read_optional_notice(notice)?,
    })
}

/// Translate `RecurringState` to its durable record.
pub fn recurring_state(
    source: &domain::RecurringState,
    limits: &wire::Limits,
    moved: Option<&[u64; 64]>,
) -> Result<wire::RecurringState, wire::Problem> {
    let domain::RecurringState { template, last_batch, last_period, pending_period } = source;
    Ok(wire::RecurringState {
        template: phase::template(template, limits, moved)?,
        last_batch: phase::references(
            last_batch,
            limits.recurring_state_last_batch,
            wire::paths::RECURRING_STATE_LAST_BATCH,
            limits,
        )?,
        last_period: *last_period,
        pending_period: *pending_period,
    })
}
/// Read `RecurringState` from its durable record.
pub fn read_recurring_state(
    source: wire::RecurringState,
    limits: &wire::Limits,
) -> Result<domain::RecurringState, wire::Problem> {
    let wire::RecurringState { template, last_batch, last_period, pending_period } = source;
    Ok(domain::RecurringState {
        template: phase::read_template(template, limits)?,
        last_batch: phase::read_references(last_batch),
        last_period,
        pending_period,
    })
}

/// Translate `QuestionCredit` to its durable record.
pub fn question_credit(
    source: &domain::QuestionCredit,
    limits: &wire::Limits,
) -> Result<wire::QuestionCredit, wire::Problem> {
    let domain::QuestionCredit { number, answerer, asked, line } = source;
    Ok(wire::QuestionCredit {
        number: *number,
        answerer: super::value::party(answerer, limits)?,
        asked: super::value::party(asked, limits)?,
        line: *line,
    })
}
/// Read `QuestionCredit` from its durable record.
pub fn read_question_credit(source: wire::QuestionCredit) -> Result<domain::QuestionCredit, wire::Problem> {
    let wire::QuestionCredit { number, answerer, asked, line } = source;
    Ok(domain::QuestionCredit {
        number,
        answerer: super::value::read_party(answerer),
        asked: super::value::read_party(asked),
        line,
    })
}

/// Translate `SubscriptionKind` to its durable value.
pub fn subscription_kind(
    source: &domain::SubscriptionKind,
    _limits: &wire::Limits,
) -> Result<wire::SubscriptionKind, wire::Problem> {
    Ok(match source {
        domain::SubscriptionKind::Task { target, held, result } => {
            wire::SubscriptionKind::Task(wire::TaskInterest { target: *target, held: *held, result: *result })
        }
        domain::SubscriptionKind::Timer { at, period } => {
            wire::SubscriptionKind::Timer(wire::TimerInterest { at: at.as_nanos(), period: *period })
        }
        domain::SubscriptionKind::Topic { connector, topic } => {
            wire::SubscriptionKind::Topic(wire::TopicInterest { connector: *connector, topic: *topic })
        }
    })
}
/// Read `SubscriptionKind` from its durable value.
pub fn read_subscription_kind(source: wire::SubscriptionKind) -> Result<domain::SubscriptionKind, wire::Problem> {
    Ok(match source {
        wire::SubscriptionKind::Task(source) => {
            let wire::TaskInterest { target, held, result } = source;
            domain::SubscriptionKind::Task { target, held, result }
        }
        wire::SubscriptionKind::Timer(source) => {
            let wire::TimerInterest { at, period } = source;
            domain::SubscriptionKind::Timer { at: Wall::from_nanos(at), period }
        }
        wire::SubscriptionKind::Topic(source) => {
            let wire::TopicInterest { connector, topic } = source;
            domain::SubscriptionKind::Topic { connector, topic }
        }
    })
}

/// Translate `Subscription` to its durable record.
pub fn subscription(source: &domain::Subscription, limits: &wire::Limits) -> Result<wire::Subscription, wire::Problem> {
    let domain::Subscription { number, kind } = source;
    Ok(wire::Subscription { number: *number, kind: subscription_kind(kind, limits)? })
}
/// Read `Subscription` from its durable record.
pub fn read_subscription(source: wire::Subscription) -> Result<domain::Subscription, wire::Problem> {
    let wire::Subscription { number, kind } = source;
    Ok(domain::Subscription { number, kind: read_subscription_kind(kind)? })
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

fn optional_notice(
    source: Option<&domain::PersonStallNotice>,
    limits: &wire::Limits,
) -> Result<Option<wire::PersonStallNotice>, wire::Problem> {
    match source {
        Some(source) => Ok(Some(person_stall_notice(source, limits)?)),
        None => Ok(None),
    }
}
fn read_optional_notice(
    source: Option<wire::PersonStallNotice>,
) -> Result<Option<domain::PersonStallNotice>, wire::Problem> {
    match source {
        Some(source) => Ok(Some(read_person_stall_notice(source)?)),
        None => Ok(None),
    }
}

fn saved_path(source: &[Box<[u8]>], limits: &wire::Limits) -> Result<Box<[u8]>, wire::Problem> {
    super::pieces::authority_segments(source, limits, limits.saved_resource_path, wire::paths::SAVED_RESOURCE_PATH)
}
