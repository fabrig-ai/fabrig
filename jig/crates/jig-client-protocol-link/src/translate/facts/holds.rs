//! Field-for-field client contract translation (protocol/client/wire.md, 1, and protocol/client/README.md, 3).
use super::parties::escalation_holder;
use super::{Child, active_run, document_text, part, party, prior_phase, take_closing, take_ending};
use crate::Limits;
use crate::translate::text_problem;
use jig_client_domain::contract as c;
use jig_client_protocol_wire::v21 as w;
use skein_lib::{List, Map, Wall, bytes};

pub(crate) fn hold_failure(value: &w::HoldFailure) -> c::HoldFailure {
    match value {
        w::HoldFailure::Transient => c::HoldFailure::Transient,
        w::HoldFailure::Permanent => c::HoldFailure::Permanent,
        w::HoldFailure::Run => c::HoldFailure::Run,
        w::HoldFailure::Agent => c::HoldFailure::Agent,
        w::HoldFailure::Lost => c::HoldFailure::Lost,
        w::HoldFailure::Invalid => c::HoldFailure::Invalid,
        w::HoldFailure::Infrastructure => c::HoldFailure::Infrastructure,
    }
}

pub(crate) fn hold_category(value: &w::HoldCategory) -> c::HoldCategory {
    match value {
        w::HoldCategory::Exceeded(value) => c::HoldCategory::Exceeded(c::Exceeded {
            bound: match value.bound() {
                w::Bound::Reasoning => c::Bound::Reasoning,
                w::Bound::Metadata => c::Bound::Metadata,
                w::Bound::Items => c::Bound::Items,
                w::Bound::Answer => c::Bound::Answer,
                w::Bound::Tools => c::Bound::Tools,
                w::Bound::HistoryItems => c::Bound::HistoryItems,
                w::Bound::History => c::Bound::History,
                w::Bound::Window => c::Bound::Window,
                w::Bound::Output => c::Bound::Output,
            },
            value: value.value(),
        }),
        w::HoldCategory::Failures(row) => c::HoldCategory::Failures { class: hold_failure(row.class()) },
        w::HoldCategory::Stalled => c::HoldCategory::Stalled,
        w::HoldCategory::Dependency(row) => c::HoldCategory::Dependency { task: row.task() },
        w::HoldCategory::Stopped => c::HoldCategory::Stopped,
        w::HoldCategory::Placed(row) => c::HoldCategory::Placed { hold: row.hold() },
        w::HoldCategory::Drift => c::HoldCategory::Drift,
        w::HoldCategory::Effects => c::HoldCategory::Effects,
        w::HoldCategory::EffectFailed => c::HoldCategory::EffectFailed,
        w::HoldCategory::Uncertain(row) => c::HoldCategory::Uncertain { entry: row.entry() },
        w::HoldCategory::Budget => c::HoldCategory::Budget,
        w::HoldCategory::Deadline => c::HoldCategory::Deadline,
        w::HoldCategory::HoldsWaited => c::HoldCategory::HoldsWaited,
        w::HoldCategory::ResourceUnavailable => c::HoldCategory::ResourceUnavailable,
        w::HoldCategory::ResourceAccess => c::HoldCategory::ResourceAccess,
        w::HoldCategory::PoolLost(row) => c::HoldCategory::PoolLost { pool: row.pool() },
        w::HoldCategory::Procedure => c::HoldCategory::Procedure,
        w::HoldCategory::Circle => c::HoldCategory::Circle,
    }
}

pub(crate) fn failure_class(value: &w::FailureClass) -> c::FailureClass {
    match value {
        w::FailureClass::HostLost => c::FailureClass::HostLost,
        w::FailureClass::Temporary => c::FailureClass::Temporary,
        w::FailureClass::Permanent => c::FailureClass::Permanent,
        w::FailureClass::Stall => c::FailureClass::Stall,
    }
}

pub(crate) fn retry_notice(value: &w::RetryNotice) -> Result<c::RetryNotice, w::Problem> {
    let reason = match value {
        w::RetryNotice::OwnerFailed(row) => row.reason(),
        w::RetryNotice::Failed(row) => row.reason(),
        w::RetryNotice::Refused(row) => row.reason(),
    };
    let reason = match reason {
        Some(reason) => Some(c::DocumentText {
            bytes: match c::Text::new(bytes::copy_of(reason), 128) {
                Ok(bytes) => bytes,
                Err(_) => return Err(text_problem()),
            },
            cut: false,
        }),
        None => None,
    };
    Ok(match value {
        w::RetryNotice::OwnerFailed(row) => c::RetryNotice::OwnerFailed {
            class: hold_failure(row.class()),
            count: row.count(),
            allowance: row.allowance(),
            reason,
        },
        w::RetryNotice::Failed(row) => c::RetryNotice::Failed {
            class: failure_class(row.class()),
            count: row.count(),
            allowance: row.allowance(),
            reason,
        },
        w::RetryNotice::Refused(row) => c::RetryNotice::Refused { count: row.count(), reason },
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
        infrastructure: value.infrastructure(),
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
            Some(text) => Some(c::DocumentText {
                bytes: match c::Text::new(bytes::copy_of(text.bytes()), crate::limits::HOLD_ACCOUNT_BYTES) {
                    Ok(bytes) => bytes,
                    Err(_) => return Err(text_problem()),
                },
                cut: text.cut(),
            }),
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

#[cfg(test)]
mod tests {
    use super::*;
    use skein_lib::{Reader, Writer};

    #[test]
    fn infrastructure_holds_and_tries_round_trip_through_wire_and_client_facts() {
        let class = w::HoldFailure::Infrastructure;
        let mut writer = Writer::new(usize::try_from(class.measure()).unwrap());
        class.encode(&mut writer).unwrap();
        let wire = writer.finish();
        let class = w::HoldFailure::decode(&w::CEILINGS, &mut Reader::new(&wire)).unwrap();
        assert_eq!(hold_failure(&class), c::HoldFailure::Infrastructure);
        let tries = w::FailureTries::new(
            &w::CEILINGS,
            w::FailureTriesParts {
                transient: 1,
                permanent: 2,
                run: 3,
                agent: 4,
                lost: 5,
                invalid: 6,
                infrastructure: 13,
            },
        )
        .unwrap();
        let mut writer = Writer::new(usize::try_from(tries.measure()).unwrap());
        tries.encode(&mut writer).unwrap();
        let wire = writer.finish();
        let tries = w::FailureTries::decode(&w::CEILINGS, &mut Reader::new(&wire)).unwrap();
        assert_eq!(
            failure_tries(&tries),
            c::FailureTries { transient: 1, permanent: 2, run: 3, agent: 4, lost: 5, invalid: 6, infrastructure: 13 }
        );
    }

    #[test]
    fn retry_reasons_round_trip_through_wire_and_client_facts() {
        const REASON_BYTES: usize = 128; // client-v21's retry reason bound (domain/engine/briefs.md, 1.1).
        let limits = &w::CEILINGS;
        let reason = Some(bytes::copy_of(&[b'x'; REASON_BYTES]));
        let expected = Some(c::DocumentText::complete(
            c::Text::new(bytes::copy_of(&[b'x'; REASON_BYTES]), 128).expect("reason at its wire bound"),
        ));
        for (wire, expected) in [
            (
                w::RetryNotice::OwnerFailed(
                    w::RetryNoticeOwnerFailed::new(
                        limits,
                        w::RetryNoticeOwnerFailedParts {
                            class: w::HoldFailure::Permanent,
                            count: 2,
                            allowance: 3,
                            reason: reason.clone(),
                        },
                    )
                    .expect("bounded reason"),
                ),
                c::RetryNotice::OwnerFailed {
                    class: c::HoldFailure::Permanent,
                    count: 2,
                    allowance: 3,
                    reason: expected.clone(),
                },
            ),
            (
                w::RetryNotice::Failed(
                    w::RetryNoticeFailed::new(
                        limits,
                        w::RetryNoticeFailedParts {
                            class: w::FailureClass::Temporary,
                            count: 2,
                            allowance: 3,
                            reason: reason.clone(),
                        },
                    )
                    .expect("bounded reason"),
                ),
                c::RetryNotice::Failed {
                    class: c::FailureClass::Temporary,
                    count: 2,
                    allowance: 3,
                    reason: expected.clone(),
                },
            ),
            (
                w::RetryNotice::Refused(
                    w::RetryNoticeRefused::new(limits, w::RetryNoticeRefusedParts { count: 2, reason })
                        .expect("bounded reason"),
                ),
                c::RetryNotice::Refused { count: 2, reason: expected },
            ),
        ] {
            let mut writer = Writer::new(skein_lib::length::usize_of(wire.measure()));
            wire.encode(&mut writer).expect("measured room");
            let encoded = writer.finish();
            let decoded = w::RetryNotice::decode(limits, &mut Reader::new(&encoded)).expect("wire round trip");
            assert_eq!(retry_notice(&decoded), Ok(expected));
        }
    }
}
