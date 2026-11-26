//! Classify a close job at the branch read after it ends (temper/docs/design/protocol/tooling.md, 4.2;
//! temper/docs/design/domain/forge/change/repairs-releases.md, 1).
use crate::ChangeRow;
use alloc::boxed::Box;
use skein_forge::api::Commit;
use skein_json::{Kind, document::Document};
use skein_lib::{Decimal, List, bytes};
use temper_engine_domain_forge_change as change;

const PREFIX: &[u8] = b"the close check refused the goal's branch at ";
const BASE: &[u8] = b", against main at ";
const MOVES: &[u8] = b"; it checks again once the goal's branch or main moves.";
const BROKE: &[u8] = b"the close check broke and didn't judge the goal: ";
const AGAIN: &[u8] = b"; its release runs it again.";
const FINISHED: &[u8] = b"it finished with ";
const RESULT_PREFIX_BYTES: usize = 64;
const HOLD_BYTES: u32 = jig_domain_tasks::HOLD_ACCOUNT_BYTES;

pub(crate) fn judged(row: &mut ChangeRow, pushed: bool) {
    let Some((_, kind)) = row.delegate else { return };
    if row.delegate_status != change::Status::Passed
        || kind != change::Delegate::Close
        || row.change.state != (change::State::Producing { requested: true })
    {
        return;
    }
    let verdict = result(&row.report, row.close_at, pushed);
    match verdict {
        Some(Verdict::Settled) => {}
        Some(Verdict::Refused { counts, words }) => {
            // The refusal was validated against this attempt above.
            row.close_at.as_mut().expect("a refusal names its attempt").refused = Some(change::CloseRefused { counts });
            row.delegate_status = change::Status::Failed;
            row.producer_failure = Some(words);
            row.report = Box::new([]);
        }
        None => {
            let words = finished_words(&row.report);
            row.delegate_status = change::Status::Failed;
            row.producer_failure = Some(words);
            row.report = Box::new([]);
            clear_refusal(row);
        }
    }
}

pub(crate) fn clear_refusal(row: &mut ChangeRow) {
    if let Some(at) = &mut row.close_at {
        at.refused = None;
    }
}

pub(crate) fn broke_words(reason: &[u8]) -> Box<[u8]> {
    let room = skein_lib::usize_of(HOLD_BYTES).saturating_sub(BROKE.len()).saturating_sub(AGAIN.len());
    let reason = bytes::utf8_replaced(reason);
    bytes::concat(&[BROKE, bytes::utf8_prefix(&reason, room), AGAIN])
}

pub(crate) fn finished_words(result: &[u8]) -> Box<[u8]> {
    let end = result.len().min(RESULT_PREFIX_BYTES);
    let prefix = if bytes::is_utf8(result) {
        bytes::utf8_prefix(result, end)
    } else {
        result.get(..end).expect("bounded result prefix")
    };
    broke_words(&bytes::concat(&[FINISHED, prefix]))
}

enum Verdict {
    Settled,
    Refused { counts: Option<Commit>, words: Box<[u8]> },
}

fn result(raw: &[u8], at: Option<change::CloseAt>, pushed: bool) -> Option<Verdict> {
    let length = u32::try_from(raw.len()).ok()?;
    let limits = skein_json::tokenizer::Limits { depth: 4, string: length, number: 20, chunk: 256, length };
    let document =
        skein_json::parse(raw, &limits, &skein_json::document::Limits { tokens: length, text: length }).ok()?;
    if let Some(refused) = document.member(0, b"refused") {
        let at = at?;
        if !at.asked || pushed || refused != 2 || document.value_end(refused)?.checked_add(1)? != document.len() {
            return None;
        }
        return refusal(&document, refused, at.branch);
    }
    strings(&document, document.member(0, b"settled")?)?;
    Some(Verdict::Settled)
}

fn refusal(document: &Document, object: u32, start: Commit) -> Option<Verdict> {
    let head = commit(document, document.member(object, b"head")?)?;
    if head != start {
        return None;
    }
    let _tree = commit(document, document.member(object, b"tree")?)?;
    let base = nullable_commit(document, document.member(object, b"base")?).ok()?;
    let counts = nullable_commit(document, document.member(object, b"counts")?).ok()?;
    let names = strings(document, document.member(object, b"failures")?)?;
    let (more, expected) = match document.member(object, b"more") {
        Some(value) => (number(document, value)?, 6),
        None => (0, 5),
    };
    let end = document.value_end(object)?.checked_sub(1)?;
    let start = object.checked_add(1)?;
    let mut members = 0_u32;
    let mut cursor = start;
    for index in start..end {
        if index != cursor {
            continue;
        }
        members = members.checked_add(1)?;
        cursor = document.value_end(index.checked_add(1)?)?;
    }
    if members != expected || (names.is_empty() && more == 0) {
        return None;
    }
    let words = refusal_words(document, names, more, head, base, counts);
    Some(Verdict::Refused { counts, words })
}

fn strings(document: &Document, array: u32) -> Option<core::ops::Range<u32>> {
    if document.token(array)?.kind != Kind::ArrayStart {
        return None;
    }
    let names = array.checked_add(1)?..document.value_end(array)?.checked_sub(1)?;
    for index in names.clone() {
        if document.token(index)?.kind != Kind::String {
            return None;
        }
    }
    Some(names)
}

fn number(document: &Document, index: u32) -> Option<u64> {
    let token = document.token(index)?;
    if token.kind != Kind::Number {
        return None;
    }
    bytes::decimal(document.text(token)?)
}

fn nullable_commit(document: &Document, index: u32) -> Result<Option<Commit>, ()> {
    if document.token(index).ok_or(())?.kind == Kind::Null {
        Ok(None)
    } else {
        let commit = commit(document, index).ok_or(())?;
        Ok(Some(commit))
    }
}

fn commit(document: &Document, index: u32) -> Option<Commit> {
    let token = document.token(index)?;
    if token.kind != Kind::String {
        return None;
    }
    let text = document.text(token)?;
    if text.len() != 40 && text.len() != 64 {
        return None;
    }
    let mut commit = [0; 32];
    for (byte, pair) in commit.iter_mut().zip(text.as_chunks::<2>().0) {
        let high = bytes::hex_value(*pair.first()?)?;
        let low = bytes::hex_value(*pair.get(1)?)?;
        *byte = high.checked_mul(16)?.checked_add(low)?;
    }
    Some(commit)
}

fn refusal_words(
    document: &Document,
    names: core::ops::Range<u32>,
    more: u64,
    head: Commit,
    base: Option<Commit>,
    counts: Option<Commit>,
) -> Box<[u8]> {
    let mut words = List::with_capacity(HOLD_BYTES);
    append(&mut words, PREFIX);
    short_commit(&mut words, &head);
    append(&mut words, BASE);
    if let Some(base) = base {
        short_commit(&mut words, &base);
    } else {
        append(&mut words, b"no base");
    }
    append(&mut words, b" with counts ");
    if let Some(counts) = counts {
        short_commit(&mut words, &counts);
    } else {
        append(&mut words, b"none");
    }
    append(&mut words, b": ");
    let room = skein_lib::usize_of(words.room()).saturating_sub(MOVES.len());
    let kept = fitting_names(document, names.clone(), more, room);
    for index in names.start..kept {
        if index != names.start {
            append(&mut words, b", ");
        }
        append(&mut words, name(document, index));
    }
    let omitted = omitted(kept, names.end, more);
    if omitted != 0 {
        if kept != names.start {
            append(&mut words, b" and ");
        }
        append(&mut words, Decimal::of(omitted).as_bytes());
        append(&mut words, b" more");
    }
    append(&mut words, MOVES);
    words.into_boxed()
}

fn fitting_names(document: &Document, names: core::ops::Range<u32>, more: u64, room: usize) -> u32 {
    let mut length = 0_usize;
    let mut kept = names.start;
    for index in names.clone() {
        let separator = if index == names.start { 0 } else { b", ".len() };
        length = length.saturating_add(separator).saturating_add(name(document, index).len());
        if length > room {
            break;
        }
        let end = index.saturating_add(1);
        let omitted = omitted(end, names.end, more);
        let tail =
            if omitted == 0 { 0 } else { b" and  more".len().saturating_add(Decimal::of(omitted).as_bytes().len()) };
        if length.saturating_add(tail) <= room {
            kept = end;
        }
    }
    kept
}

fn omitted(kept: u32, end: u32, more: u64) -> u64 {
    more.saturating_add(u64::from(end.saturating_sub(kept)))
}

fn name(document: &Document, index: u32) -> &[u8] {
    let token = document.token(index).expect("name from the validated string array");
    document.text(token).expect("decoded name from the validated string array")
}

fn append(words: &mut List<u8>, text: &[u8]) {
    for byte in text {
        words.push(*byte).expect("fitted close refusal words");
    }
}

fn short_commit(words: &mut List<u8>, commit: &Commit) {
    for byte in commit.get(..6).expect("six bytes of a commit") {
        append(words, &crate::hex(*byte));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AT: change::CloseAt = change::CloseAt { branch: [0x11; 32], landing: [0x22; 32], asked: true, refused: None };
    const REFUSAL: &[u8] = br#"{"refused":{"head":"1111111111111111111111111111111111111111111111111111111111111111","tree":"2222222222222222222222222222222222222222","base":null,"counts":null,"failures":["test A::rules rule_00"]}}"#;

    #[test]
    fn an_unasked_or_pushed_refusal_is_not_a_judgment() {
        for (at, pushed) in [(change::CloseAt { asked: false, ..AT }, false), (AT, true)] {
            assert!(result(REFUSAL, Some(at), pushed).is_none(), "{at:?}, pushed: {pushed}");
        }
    }
}
