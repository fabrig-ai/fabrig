//! Field-for-field client contract translation (protocol/client.md, 3 and 5).
use super::{
    Child, active_run, chip_phase, closing_stage, document_text, failure_class, hold_category, person_address,
    run_stage, take_closing, take_ending, take_owner_chip_details, take_owner_chip_ending,
};
use crate::Limits;
use crate::translate::text_problem;
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use jig_client_protocol_wire::v19 as w;
use skein_lib::Map;
use skein_lib::{List, Wall, bytes};
pub(crate) fn chip_phase_fields(
    value: &w::ChipPhaseOwnerFields,
    _limits: &Limits,
    index: u32,
    children: &mut Map<u32, Child>,
) -> Result<c::ChipPhaseFacts, w::Problem> {
    let phase = chip_phase(value.phase());
    let details = take_owner_chip_details(value.details(), index, children)?;
    let required_missing = match &phase {
        c::ChipPhase::Ended => details.ended_at.is_none(),
        c::ChipPhase::Active(c::RunStage::BackingOff { .. }) => details.retry.is_none(),
        c::ChipPhase::Waiting { .. }
        | c::ChipPhase::Person { .. }
        | c::ChipPhase::Closing(_)
        | c::ChipPhase::Held { .. }
        | c::ChipPhase::Active(
            c::RunStage::Idle
            | c::RunStage::Due
            | c::RunStage::Preparing
            | c::RunStage::Claimed { .. }
            | c::RunStage::Running { .. },
        ) => false,
    };
    if required_missing {
        return Err(crate::translate::piece_problem(skein_codec::Reason::Malformed));
    }
    Ok(c::ChipPhaseFacts { phase, details: Box::new(details) })
}

pub(crate) fn closing_fields(
    value: &w::ClosingFields,
    index: u32,
    children: &mut Map<u32, Child>,
) -> Result<c::Closing, w::Problem> {
    let opt0 = match &value.connectors() {
        Some(item) => {
            let mut list0 = List::with_capacity((item).len());
            for item in item {
                list0.push(*item).expect("reserved supplied list");
            }
            Some(list0)
        }
        None => None,
    };
    let opt1 = match &value.stage() {
        Some(item) => Some(closing_stage(item)),
        None => None,
    };
    let opt2 = match &value.ending() {
        Some(item) => Some(take_ending(*item, index, children)?),
        None => None,
    };
    Ok(c::Closing { delegates: value.delegates(), connectors: opt0, stage: opt1, ending: opt2 })
}

pub(crate) fn actor(value: &w::Actor) -> c::Actor {
    match value {
        w::Actor::Party(row) => c::Actor::Party(row.value()),
        w::Actor::Task(row) => c::Actor::Task(row.value()),
        w::Actor::Deployment => c::Actor::Deployment,
        w::Actor::ProjectDeployment(row) => c::Actor::ProjectDeployment { project: row.project() },
    }
}

pub(crate) fn party_kind(value: &w::PartyKind) -> c::PartyKind {
    match value {
        w::PartyKind::Person => c::PartyKind::Person,
        w::PartyKind::Service => c::PartyKind::Service,
    }
}

pub(crate) fn party(value: &w::Party, limits: &Limits) -> Result<c::Party, w::Problem> {
    Ok(c::Party {
        number: value.number(),
        kind: party_kind(value.kind()),
        name: match c::Text::new(bytes::copy_of(value.name()), limits.client().text) {
            Ok(value) => value,
            Err(_) => return Err(text_problem()),
        },
    })
}

pub(crate) fn decision(value: &w::Decision, limits: &Limits) -> Result<c::Decision, w::Problem> {
    Ok(match value {
        w::Decision::Accepted => c::Decision::Accepted,
        w::Decision::Rejected(row) => c::Decision::Rejected {
            reason: match c::Text::new(bytes::copy_of(row.reason()), limits.client().text) {
                Ok(value) => value,
                Err(_) => return Err(text_problem()),
            },
        },
        w::Decision::PassedUp => c::Decision::PassedUp,
        w::Decision::Released(row) => {
            let opt0 = match row.again() {
                Some(item) => Some(*item),
                None => None,
            };
            c::Decision::Released { again: opt0 }
        }
        w::Decision::LeftHeld(row) => c::Decision::LeftHeld {
            reason: match c::Text::new(bytes::copy_of(row.reason()), limits.client().text) {
                Ok(value) => value,
                Err(_) => return Err(text_problem()),
            },
        },
    })
}

pub(crate) fn ending_fields(
    value: &w::EndingFields,
    limits: &Limits,
    index: u32,
    children: &mut Map<u32, Child>,
) -> Result<c::Ending, w::Problem> {
    Ok(match value {
        w::EndingFields::Task(row) => c::Ending::Task(Box::new(take_owner_chip_ending(row.value(), index, children)?)),
        w::EndingFields::Done => c::Ending::Done,
        w::EndingFields::Failed(row) => {
            let opt0 = match &row.reason() {
                Some(item) => Some(match c::Text::new(bytes::copy_of(item), limits.client().text) {
                    Ok(value) => value,
                    Err(_) => return Err(text_problem()),
                }),
                None => None,
            };
            c::Ending::Failed { reason: opt0 }
        }
        w::EndingFields::Cancelled(row) => {
            let opt0 = match &row.reason() {
                Some(item) => Some(match c::Text::new(bytes::copy_of(item), limits.client().text) {
                    Ok(value) => value,
                    Err(_) => return Err(text_problem()),
                }),
                None => None,
            };
            let opt1 = match row.at() {
                Some(item) => Some(Wall::from_nanos(*item)),
                None => None,
            };
            c::Ending::Cancelled { by: actor(row.by()), reason: opt0, at: opt1 }
        }
        w::EndingFields::Withdrawn(row) => c::Ending::Withdrawn {
            reason: match c::Text::new(bytes::copy_of(row.reason()), limits.client().text) {
                Ok(value) => value,
                Err(_) => return Err(text_problem()),
            },
        },
        w::EndingFields::Decided(row) => {
            let opt0 = match &row.party() {
                Some(item) => Some(party(item, limits)?),
                None => None,
            };
            let opt1 = match &row.how() {
                Some(item) => Some(decision(item, limits)?),
                None => None,
            };
            c::Ending::Decided { by: row.by(), party: opt0, how: opt1, at: Wall::from_nanos(row.at()) }
        }
        w::EndingFields::Deleted => c::Ending::Deleted,
    })
}

pub(crate) fn held_runtime_fields(
    value: &w::HeldRuntimeFields,
    limits: &Limits,
    index: u32,
    children: &mut Map<u32, Child>,
) -> Result<c::HeldRuntime, w::Problem> {
    Ok(match value {
        w::HeldRuntimeFields::Waiting => c::HeldRuntime::Waiting,
        w::HeldRuntimeFields::Active(row) => c::HeldRuntime::Active(active_run(row.value(), limits)?),
        w::HeldRuntimeFields::Closing(row) => c::HeldRuntime::Closing(take_closing(row.value(), index, children)?),
    })
}

pub(crate) fn attempt_failure(value: &w::AttemptFailure, limits: &Limits) -> Result<c::AttemptFailure, w::Problem> {
    Ok(c::AttemptFailure {
        attempt: value.attempt(),
        class: failure_class(value.class()),
        reason: match c::Text::new(bytes::copy_of(value.reason()), limits.client().text) {
            Ok(value) => value,
            Err(_) => return Err(text_problem()),
        },
        at: Wall::from_nanos(value.at()),
    })
}

pub(crate) fn hold_reason_fields(
    value: &w::HoldReasonFields,
    limits: &Limits,
    index: u32,
    children: &mut Map<u32, Child>,
) -> Result<c::HoldReason, w::Problem> {
    Ok(match value {
        w::HoldReasonFields::Tries(row) => {
            let mut list0 = List::with_capacity(row.failures().len());
            for item in row.failures() {
                list0.push(attempt_failure(item, limits)?).expect("reserved supplied list");
            }
            c::HoldReason::Tries { failures: list0 }
        }
        w::HoldReasonFields::Stalled(row) => c::HoldReason::Stalled {
            since: Wall::from_nanos(row.since()),
            waiting: match c::Text::new(bytes::copy_of(row.waiting()), limits.client().text) {
                Ok(value) => value,
                Err(_) => return Err(text_problem()),
            },
        },
        w::HoldReasonFields::Dependency(row) => {
            c::HoldReason::Dependency { task: row.task(), how: take_ending(row.how(), index, children)? }
        }
        w::HoldReasonFields::Budget(row) => c::HoldReason::Budget { spent: row.spent(), limit: row.limit() },
        w::HoldReasonFields::Deadline(row) => c::HoldReason::Deadline { at: Wall::from_nanos(row.at()) },
        w::HoldReasonFields::Stopped(row) => {
            c::HoldReason::Stopped { by: party(row.by(), limits)?, at: Wall::from_nanos(row.at()) }
        }
    })
}

pub(crate) fn executor(value: &w::Executor) -> c::Executor {
    match value {
        w::Executor::Job(row) => c::Executor::Job { kind: row.kind() },
        w::Executor::Agent(row) => c::Executor::Agent { charter: row.charter() },
        w::Executor::Procedure(row) => c::Executor::Procedure { connector: row.connector(), code: row.code() },
        w::Executor::Person(row) => c::Executor::Person(person_address(row.value())),
    }
}

pub(crate) fn wake_rule(value: &w::WakeRule) -> c::WakeRule {
    match value {
        w::WakeRule::Never => c::WakeRule::Never,
        w::WakeRule::Immediate => c::WakeRule::Immediate,
        w::WakeRule::Batch(row) => c::WakeRule::Batch { count: row.count(), age: row.age() },
        w::WakeRule::Paced(row) => c::WakeRule::Paced { span: row.span() },
    }
}

pub(crate) fn results_wake(value: &w::ResultsWake) -> c::ResultsWake {
    match value {
        w::ResultsWake::Never => c::ResultsWake::Never,
        w::ResultsWake::Each => c::ResultsWake::Each,
        w::ResultsWake::LastOrFailure => c::ResultsWake::LastOrFailure,
    }
}

pub(crate) fn wake_policy(value: &w::WakePolicy) -> c::WakePolicy {
    c::WakePolicy {
        words: wake_rule(value.words()),
        notices: wake_rule(value.notices()),
        news: wake_rule(value.news()),
        results: results_wake(value.results()),
        questions: value.questions(),
        answers: value.answers(),
        timers: value.timers(),
    }
}

pub(crate) fn person_stall_notice(value: &w::PersonStallNotice) -> c::PersonStallNotice {
    c::PersonStallNotice { message: value.message(), at: Wall::from_nanos(value.at()), activity: value.activity() }
}

pub(crate) fn person_wait(value: &w::PersonWait) -> c::PersonWait {
    let opt0 = match &value.notice() {
        Some(item) => Some(person_stall_notice(item)),
        None => None,
    };
    c::PersonWait {
        since: Wall::from_nanos(value.since()),
        stall: value.stall(),
        deadline: Wall::from_nanos(value.deadline()),
        notice: opt0,
    }
}

pub(crate) fn part_kind(value: &w::PartKind) -> c::PartKind {
    match value {
        w::PartKind::Resource => c::PartKind::Resource,
        w::PartKind::Pool => c::PartKind::Pool,
        w::PartKind::Result => c::PartKind::Result,
        w::PartKind::Procedure => c::PartKind::Procedure,
        w::PartKind::Effect => c::PartKind::Effect,
        w::PartKind::Outbox => c::PartKind::Outbox,
        w::PartKind::Hold => c::PartKind::Hold,
    }
}

pub(crate) fn part_key(value: &w::PartKey) -> c::PartKey {
    match value {
        w::PartKey::Path(row) => {
            let mut list0 = List::with_capacity(row.value().len());
            for item in row.value() {
                list0.push(bytes::copy_of(item)).expect("reserved supplied list");
            }
            c::PartKey::Path(list0)
        }
        w::PartKey::Number(row) => c::PartKey::Number(row.value()),
    }
}

pub(crate) fn part(value: &w::Part) -> c::Part {
    c::Part { connector: value.connector(), kind: part_kind(value.kind()), key: part_key(value.key()) }
}

pub(crate) fn prior_phase(value: &w::PriorPhase) -> c::PriorPhase {
    match value {
        w::PriorPhase::Waiting => c::PriorPhase::Waiting,
        w::PriorPhase::Active(row) => c::PriorPhase::Active(run_stage(row.value())),
        w::PriorPhase::Closing(row) => c::PriorPhase::Closing(closing_stage(row.value())),
    }
}

pub(crate) fn escalation_holder(value: &w::EscalationHolder) -> c::EscalationHolder {
    match value {
        w::EscalationHolder::Task(row) => c::EscalationHolder::Task(row.value()),
        w::EscalationHolder::Person(row) => c::EscalationHolder::Person(row.value()),
        w::EscalationHolder::Role(row) => c::EscalationHolder::Role { project: row.project(), role: row.role() },
    }
}

pub(crate) fn hold_escalation(value: &w::HoldEscalation, limits: &Limits) -> Result<c::HoldEscalation, w::Problem> {
    Ok(match value {
        w::HoldEscalation::Routing => c::HoldEscalation::Routing,
        w::HoldEscalation::Waiting(row) => c::HoldEscalation::Waiting {
            holder: escalation_holder(row.holder()),
            entry: row.entry(),
            since: Wall::from_nanos(row.since()),
        },
        w::HoldEscalation::Rejected(row) => {
            c::HoldEscalation::Rejected { by: row.by(), reason: document_text(row.reason(), limits)? }
        }
    })
}

pub(crate) fn failure_tries(value: &w::FailureTries) -> c::FailureTries {
    c::FailureTries {
        transient: value.transient(),
        permanent: value.permanent(),
        run: value.run(),
        agent: value.agent(),
        lost: value.lost(),
        invalid: value.invalid(),
    }
}

pub(crate) fn chip_hold(value: &w::ChipHold, limits: &Limits) -> Result<c::ChipHold, w::Problem> {
    let opt0 = match &value.account() {
        Some(item) => Some(part(item)),
        None => None,
    };
    let opt1 = match &value.source() {
        Some(item) => Some(part(item)),
        None => None,
    };
    let opt2 = match &value.escalation() {
        Some(item) => Some(hold_escalation(item, limits)?),
        None => None,
    };
    let opt3 = match value.hold_wait_since() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::ChipHold {
        placed: crate::translate::answers::placed(value.placed().as_ref(), limits)?,
        words: match value.words() {
            Some(text) => Some(document_text(text, limits)?),
            None => None,
        },
        circle: match value.circle() {
            Some(circle) => Some(c::Circle { tasks: circle.tasks().clone(), cut: circle.cut() }),
            None => None,
        },
        account: opt0,
        source: opt1,
        reason: hold_category(value.reason()),
        revision: value.revision(),
        before: prior_phase(value.before()),
        escalation: opt2,
        tries: failure_tries(value.tries()),
        hold_wait_since: opt3,
    })
}

pub(crate) fn numbers(value: &w::Numbers) -> c::Numbers {
    c::Numbers {
        budget: value.budget(),
        spent: value.spent(),
        spent_below: value.spent_below(),
        reserved: value.reserved(),
    }
}
