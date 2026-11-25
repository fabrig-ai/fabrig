//! Field-for-field client contract translation (protocol/client/wire.md, 1, and protocol/client/README.md, 3).
use super::{Child, active_run, part, party, prior_phase, take_closing, take_ending};
use crate::Limits;
use crate::generated::client;
use crate::translate::text_problem;
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use jig_client_protocol_wire as w;
use skein_lib::{List, Map, Wall, bytes};

pub(crate) fn hold_category(value: &w::HoldCategory) -> c::HoldCategory {
    client::hold_category_to_domain(value.clone())
}

pub(crate) fn failure_class(value: &w::FailureClass) -> c::FailureClass {
    client::failure_class_to_domain(value.clone())
}

pub(crate) fn retry_notice(value: &w::RetryNotice) -> Result<c::RetryNotice, w::Problem> {
    retry_notice_to_domain(value.clone())
}

pub(crate) fn retry_notice_to_domain(record: w::RetryNotice) -> Result<c::RetryNotice, w::Problem> {
    Ok(match record {
        w::RetryNotice::OwnerFailed(row) => c::RetryNotice::OwnerFailed {
            class: client::hold_failure_to_domain(row.class),
            count: row.count,
            allowance: row.allowance,
            reason: retry_reason(row.reason)?,
        },
        w::RetryNotice::Failed(row) => c::RetryNotice::Failed {
            class: client::failure_class_to_domain(row.class),
            count: row.count,
            allowance: row.allowance,
            reason: retry_reason(row.reason)?,
        },
        w::RetryNotice::Refused(row) => c::RetryNotice::Refused { count: row.count, reason: retry_reason(row.reason)? },
    })
}

fn retry_reason(reason: Option<Box<[u8]>>) -> Result<Option<c::DocumentText>, w::Problem> {
    match reason {
        Some(reason) => match c::Text::new(reason, 128) {
            Ok(bytes) => Ok(Some(c::DocumentText { bytes, cut: false })),
            Err(_) => Err(text_problem()),
        },
        None => Ok(None),
    }
}

pub(crate) fn held_runtime_fields(
    value: &w::HeldRuntimeFields,
    limits: &Limits,
    index: u32,
    children: &mut Map<u32, Child>,
) -> Result<c::HeldRuntime, w::Problem> {
    Ok(match value {
        w::HeldRuntimeFields::Waiting => c::HeldRuntime::Waiting,
        w::HeldRuntimeFields::Active(row) => c::HeldRuntime::Active(active_run(&row.value, limits)?),
        w::HeldRuntimeFields::Closing(row) => c::HeldRuntime::Closing(take_closing(row.value, index, children)?),
    })
}

pub(crate) fn attempt_failure(value: &w::AttemptFailure, limits: &Limits) -> Result<c::AttemptFailure, w::Problem> {
    Ok(c::AttemptFailure {
        attempt: value.attempt,
        class: failure_class(&value.class),
        reason: match c::Text::new(bytes::copy_of(&value.reason), limits.client().text) {
            Ok(value) => value,
            Err(_) => return Err(text_problem()),
        },
        at: Wall::from_nanos(value.at),
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
            let mut list0 = List::with_capacity(row.failures.len());
            for item in &row.failures {
                list0.push(attempt_failure(item, limits)?).expect("reserved supplied list");
            }
            c::HoldReason::Tries { failures: list0 }
        }
        w::HoldReasonFields::Stalled(row) => c::HoldReason::Stalled {
            since: Wall::from_nanos(row.since),
            waiting: match c::Text::new(bytes::copy_of(&row.waiting), limits.client().text) {
                Ok(value) => value,
                Err(_) => return Err(text_problem()),
            },
        },
        w::HoldReasonFields::Dependency(row) => {
            c::HoldReason::Dependency { task: row.task, how: take_ending(row.how, index, children)? }
        }
        w::HoldReasonFields::Budget(row) => c::HoldReason::Budget { spent: row.spent, limit: row.limit },
        w::HoldReasonFields::Deadline(row) => c::HoldReason::Deadline { at: Wall::from_nanos(row.at) },
        w::HoldReasonFields::Stopped(row) => {
            c::HoldReason::Stopped { by: party(&row.by, limits)?, at: Wall::from_nanos(row.at) }
        }
    })
}

pub(crate) fn hold_escalation(value: &w::HoldEscalation, limits: &Limits) -> Result<c::HoldEscalation, w::Problem> {
    client::hold_escalation_to_domain(value.clone(), limits)
}

pub(crate) fn failure_tries(value: &w::FailureTries) -> c::FailureTries {
    client::failure_tries_to_domain(value.clone())
}

pub(crate) fn chip_hold(value: &w::ChipHold, limits: &Limits) -> Result<c::ChipHold, w::Problem> {
    let opt0 = match &(&value.account) {
        Some(item) => Some(part(item)),
        None => None,
    };
    let opt1 = match &(&value.source) {
        Some(item) => Some(part(item)),
        None => None,
    };
    let opt2 = match &(&value.escalation) {
        Some(item) => Some(hold_escalation(item, limits)?),
        None => None,
    };
    let opt3 = match &value.hold_wait_since {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::ChipHold {
        placed: crate::translate::answers::placed(value.placed.as_ref(), limits)?,
        words: match &value.words {
            Some(text) => Some(c::DocumentText {
                bytes: match c::Text::new(bytes::copy_of(&text.bytes), crate::limits::HOLD_ACCOUNT_BYTES) {
                    Ok(bytes) => bytes,
                    Err(_) => return Err(text_problem()),
                },
                cut: text.cut,
            }),
            None => None,
        },
        circle: match &value.circle {
            Some(circle) => Some(client::circle_to_domain(circle.clone())),
            None => None,
        },
        account: opt0,
        source: opt1,
        reason: hold_category(&value.reason),
        revision: value.revision,
        before: prior_phase(&value.before),
        escalation: opt2,
        tries: failure_tries(&value.tries),
        hold_wait_since: opt3,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use jig_client_protocol_wire::Record as _;
    use skein_lib::{Reader, Writer};

    #[test]
    fn infrastructure_holds_and_tries_round_trip_through_wire_and_client_facts() {
        let class = w::HoldFailure::Infrastructure;
        let mut writer = Writer::new(
            usize::try_from(class.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap(),
        );
        class.write(&jig_client_protocol_wire::CEILINGS, &mut writer).unwrap();
        let wire = writer.finish();
        let class = w::HoldFailure::decode(&w::CEILINGS, &mut Reader::new(&wire)).unwrap();
        assert_eq!(client::hold_failure_to_domain(class), c::HoldFailure::Infrastructure);
        let tries =
            w::FailureTries { transient: 1, permanent: 2, run: 3, agent: 4, lost: 5, invalid: 6, infrastructure: 13 };
        let mut writer = Writer::new(
            usize::try_from(tries.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap(),
        );
        tries.write(&jig_client_protocol_wire::CEILINGS, &mut writer).unwrap();
        let wire = writer.finish();
        let tries = w::FailureTries::decode(&w::CEILINGS, &mut Reader::new(&wire)).unwrap();
        assert_eq!(
            failure_tries(&tries),
            c::FailureTries { transient: 1, permanent: 2, run: 3, agent: 4, lost: 5, invalid: 6, infrastructure: 13 }
        );
    }

    #[test]
    fn retry_reasons_round_trip_through_wire_and_client_facts() {
        const REASON_BYTES: usize = 128; // client-v35's retry reason bound (domain/engine/briefs.md, 1.1).
        let limits = &w::CEILINGS;
        let reason = Some(bytes::copy_of(&[b'x'; REASON_BYTES]));
        let expected = Some(c::DocumentText::complete(
            c::Text::new(bytes::copy_of(&[b'x'; REASON_BYTES]), 128).expect("reason at its wire bound"),
        ));
        for (wire, expected) in [
            (
                w::RetryNotice::OwnerFailed(w::RetryNoticeOwnerFailed {
                    class: w::HoldFailure::Permanent,
                    count: 2,
                    allowance: 3,
                    reason: reason.clone(),
                }),
                c::RetryNotice::OwnerFailed {
                    class: c::HoldFailure::Permanent,
                    count: 2,
                    allowance: 3,
                    reason: expected.clone(),
                },
            ),
            (
                w::RetryNotice::Failed(w::RetryNoticeFailed {
                    class: w::FailureClass::Temporary,
                    count: 2,
                    allowance: 3,
                    reason: reason.clone(),
                }),
                c::RetryNotice::Failed {
                    class: c::FailureClass::Temporary,
                    count: 2,
                    allowance: 3,
                    reason: expected.clone(),
                },
            ),
            (
                w::RetryNotice::Refused(w::RetryNoticeRefused { count: 2, reason }),
                c::RetryNotice::Refused { count: 2, reason: expected },
            ),
        ] {
            let mut writer = Writer::new(skein_lib::length::usize_of(
                wire.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
            ));
            wire.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("measured room");
            let encoded = writer.finish();
            let decoded = w::RetryNotice::decode(limits, &mut Reader::new(&encoded)).expect("wire round trip");
            assert_eq!(retry_notice(&decoded), Ok(expected));
        }
    }
}
