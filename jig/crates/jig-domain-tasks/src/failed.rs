//! Failed endings and run exhaustion say why, within their reason bounds
//! (jig/docs/design/domain/tasks/lifecycle.md, 2 and 7).
use crate::{Ending, Executor, Limits, TaskResult};
use alloc::boxed::Box;
use skein_lib::{Decimal, Writer, bytes::utf8_prefix, length::usize_of};

pub(crate) fn ending(result: TaskResult, executor: Executor, limits: &Limits) -> Ending {
    match result {
        TaskResult::Failure { reason } => {
            let reason = if reason.is_empty() {
                let words = match executor {
                    Executor::Agent { .. } | Executor::Job { .. } => {
                        b"Its run declared failure without a reason.".as_slice()
                    }
                    Executor::Person(_) => b"Its person declared failure without a reason.",
                    Executor::Procedure { .. } => b"Its procedure declared failure without a reason.",
                };
                bounded(&[words], limits)
            } else {
                reason
            };
            Ending::Failed { reason }
        }
        TaskResult::Approved { .. }
        | TaskResult::Report { .. }
        | TaskResult::Verdict { .. }
        | TaskResult::Job { .. }
        | TaskResult::NoChange { .. }
        | TaskResult::Change { .. } => Ending::Done(result),
    }
}

pub(crate) fn nothing_awaited(allowance: u32) -> Box<[u8]> {
    past_tries(b"Its runs parked with nothing left to await, past its ", allowance, crate::REASON_BYTES)
}

pub(crate) fn question_owed(limits: &Limits) -> Box<[u8]> {
    past_tries(
        b"Its runs finished without asking its requester the question it owes, past its ",
        limits.retries.run.retries,
        limits.result_bytes,
    )
}

fn past_tries(prefix: &[u8], allowance: u32, bound: u32) -> Box<[u8]> {
    let tries = Decimal::of(u64::from(allowance));
    bounded_to(&[prefix, tries.as_bytes(), b" run tries."], bound)
}

pub(crate) fn closer(reason: &[u8], limits: &Limits) -> Box<[u8]> {
    let tries = Decimal::of(u64::from(limits.retries.run.retries));
    let suffix = if reason.is_empty() { b" run tries.".as_slice() } else { b" run tries: " };
    bounded(&[b"Its closer failed, past its ", tries.as_bytes(), suffix, reason], limits)
}

pub(crate) fn bounded(parts: &[&[u8]], limits: &Limits) -> Box<[u8]> {
    bounded_to(parts, limits.result_bytes)
}

fn bounded_to(parts: &[&[u8]], bound: u32) -> Box<[u8]> {
    let bound = usize_of(bound);
    let mut length = 0_usize;
    for part in parts {
        let left = bound.checked_sub(length).expect("measured words within bound");
        let prefix = utf8_prefix(part, left);
        length = length.checked_add(prefix.len()).expect("prefix within reason bound");
        if prefix.len() < part.len() {
            break;
        }
    }
    let mut words = Writer::new(length);
    let mut left = length;
    for part in parts {
        let prefix = utf8_prefix(part, left);
        words.put(prefix).expect("bounded failure words");
        left = left.checked_sub(prefix.len()).expect("prefix within remaining bound");
        if prefix.len() < part.len() {
            break;
        }
    }
    words.finish()
}
