//! Failed endings always say why, within the configured reason bound
//! (jig/docs/design/domain/tasks/lifecycle.md, 7).
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

pub(crate) fn nothing_awaited(limits: &Limits) -> Box<[u8]> {
    past_tries(b"Its runs parked with nothing left to await, past its ", b" run tries.", limits)
}

pub(crate) fn question_owed(limits: &Limits) -> Box<[u8]> {
    past_tries(
        b"Its runs finished without asking its requester the question it owes, past its ",
        b" run tries.",
        limits,
    )
}

fn past_tries(prefix: &[u8], suffix: &[u8], limits: &Limits) -> Box<[u8]> {
    let tries = Decimal::of(u64::from(limits.retries.run.retries));
    bounded(&[prefix, tries.as_bytes(), suffix], limits)
}

pub(crate) fn closer(reason: &[u8], limits: &Limits) -> Box<[u8]> {
    let tries = Decimal::of(u64::from(limits.retries.run.retries));
    let suffix = if reason.is_empty() { b" run tries.".as_slice() } else { b" run tries: " };
    bounded(&[b"Its closer failed, past its ", tries.as_bytes(), suffix, reason], limits)
}

fn bounded(parts: &[&[u8]], limits: &Limits) -> Box<[u8]> {
    let bound = usize_of(limits.result_bytes);
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
