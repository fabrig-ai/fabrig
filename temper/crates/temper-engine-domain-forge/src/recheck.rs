//! The one measurement after an unchanged repair (temper/docs/design/domain/forge/change/repairs.md, 1).
use alloc::boxed::Box;
use skein_forge::api::Commit;
use skein_lib::{Decimal, List, bytes};
use temper_engine_domain_forge_change as change;

use crate::{ChangeRow, Receipt, ReceiptReport};

const FAILED_AGAIN: &[u8] = b"; amend this change to ask a fresh repair, measure it again, or cancel it";
const UNCHECKED: &[u8] = b"; amend this change to ask a fresh repair, or cancel it";
const CUT: &[u8] = " …".as_bytes();

pub(crate) fn stands_again(head: Commit, repair: u64, keys: &[Box<[u8]>]) -> Box<[u8]> {
    let mut words = account(head, repair, b"failed again: ");
    for (position, key) in keys.iter().enumerate() {
        let separator = if position == 0 { b"".as_slice() } else { b", " };
        let more = position.saturating_add(1) < keys.len();
        let reserved = FAILED_AGAIN.len().saturating_add(if more { CUT.len() } else { 0 });
        let key = bytes::utf8_replaced(key);
        let needed = key.len().saturating_add(separator.len()).saturating_add(reserved);
        if needed > skein_lib::usize_of(words.room()) {
            append(&mut words, CUT);
            break;
        }
        append(&mut words, separator);
        append(&mut words, &key);
    }
    append(&mut words, FAILED_AGAIN);
    words.into_boxed()
}

pub(crate) fn stands_unchecked(head: Commit, repair: u64, why: &[u8]) -> Box<[u8]> {
    let mut words = account(head, repair, b"can't be checked again: ");
    let room = skein_lib::usize_of(words.room()).checked_sub(UNCHECKED.len()).expect("recovery fits the hold");
    append(&mut words, &crate::text::fit(&bytes::utf8_replaced(why), room, CUT));
    append(&mut words, UNCHECKED);
    words.into_boxed()
}

fn account(head: Commit, repair: u64, measurement: &[u8]) -> List<u8> {
    let mut words = List::with_capacity(jig_domain_tasks::HOLD_ACCOUNT_BYTES);
    append(&mut words, b"its red at ");
    let head = crate::commit_text(&head);
    append(&mut words, head.get(..12).expect("a commit has at least twelve hex digits"));
    append(&mut words, b" stands: repair task ");
    append(&mut words, Decimal::of(repair).as_bytes());
    append(&mut words, b" changed nothing, and the same content ");
    append(&mut words, measurement);
    words
}

fn append(words: &mut List<u8>, part: &[u8]) {
    for byte in part {
        words.push(*byte).expect("measured hold account");
    }
}

pub(super) fn landing(reason: change::Repair) -> bool {
    match reason {
        change::Repair::Landing { .. } | change::Repair::LandingAt { .. } => true,
        change::Repair::Ci
        | change::Repair::Semantic
        | change::Repair::Gate(_)
        | change::Repair::Review { .. }
        | change::Repair::TemperDir { .. } => false,
    }
}

pub(super) fn landing_repair(state: &change::State) -> Option<change::Repair> {
    if let change::State::Repairing { why, .. } = state
        && landing(*why)
    {
        Some(*why)
    } else {
        None
    }
}

pub(super) fn released_repair(change: &change::Change) -> Option<change::Repair> {
    if change.released != Some(change::Cause::RedStands) {
        return None;
    }
    let state = if let change::State::Held { was, .. } = &change.state { was.as_ref() } else { &change.state };
    if let change::State::Repairing { why, .. } = state { Some(*why) } else { None }
}

pub(super) fn completed(row: &mut ChangeRow, unchanged: bool, bound: u32) {
    if let Some(words) = row.delegate_words.take()
        && let Some(repair) = row.repair_tasks.last()
    {
        let prefix: &[u8] = if !unchanged {
            b""
        } else if row.delegate_flake {
            b"changed nothing (flake): "
        } else if row.delegate_no_test {
            b"changed nothing (no failed test): "
        } else {
            b"changed nothing (no change): "
        };
        row.result.completed_read(*repair, prefix, &words, bound);
    }
    row.delegate_flake = false;
    row.delegate_no_test = false;
    row.producer_failure = None;
}

pub(super) fn hold_words(row: &ChangeRow, measured: Option<&Receipt>) -> Option<Box<[u8]>> {
    let head = row.change.last_head?;
    let repair = *row.repair_tasks.last()?;
    let Some(receipt) = measured else {
        return Some(stands_unchecked(head, repair, b"the landing no longer keeps its failed receipt"));
    };
    let mut keys = List::uncapped();
    match &receipt.report {
        ReceiptReport::Tests(tests) if !tests.tests.is_empty() => {
            for test in &tests.tests {
                keys.push(test.key.clone()).expect("receipt's bounded tests");
            }
        }
        ReceiptReport::Tests(_) | ReceiptReport::Many { .. } | ReceiptReport::Unset | ReceiptReport::Unread => {
            for job in &receipt.jobs {
                keys.push(job.name.clone()).expect("receipt's bounded jobs");
            }
        }
    }
    Some(stands_again(head, repair, keys.as_slice()))
}
