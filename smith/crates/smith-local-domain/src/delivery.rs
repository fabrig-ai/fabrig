//! One in-place delivery, in workspace order (domain/host.md, section 8;
//! domain/run.md, sections 8.2–8.4). It keeps the child call's name, bounded
//! message, survey and receipts while IO decides git state. It never reads a
//! tree or knows git syntax. The parent saves the intent before the first git
//! write, then saves the answer before returning it to the child.

use alloc::boxed::Box;
use skein_lib::{List, Map};
use smith_domain::run::{self, outcome};

use crate::DeliveryRecord;

/// Return the first durable answer for a repeated call name.
pub(crate) fn cached(records: &Map<run::CallName, DeliveryRecord>, name: run::CallName) -> Option<run::Delivery> {
    records.get(&name)?.answer(name)
}

/// Render the host's configured title first, then all other declared fields
/// in their result order. The run already bounded each field and their total.
#[must_use]
pub fn commit_message(change: &outcome::Change, title_field: &[u8], rules: &[outcome::FieldRule]) -> Option<Box<[u8]>> {
    let mut title = None;
    let mut size = 0_usize;
    for field in &change.fields {
        if field.name.as_ref() == title_field {
            title = Some(&field.value);
        }
        size = size.checked_add(field.value.len())?.checked_add(2)?;
    }
    let title = title?;
    if title.is_empty() {
        return None;
    }
    let cap = u32::try_from(size).ok()?;
    let mut bytes = List::with_capacity(cap);
    append(&mut bytes, title);
    for rule in rules {
        if rule.name.as_ref() == title_field {
            continue;
        }
        for field in &change.fields {
            if field.name == rule.name {
                append(&mut bytes, b"\n\n");
                append(&mut bytes, &field.value);
            }
        }
    }
    for field in &change.fields {
        let mut required = false;
        for rule in rules {
            if field.name == rule.name {
                required = true;
            }
        }
        if field.name.as_ref() != title_field && !required {
            append(&mut bytes, b"\n\n");
            append(&mut bytes, &field.value);
        }
    }
    Some(bytes.into_boxed())
}

fn append(out: &mut List<u8>, bytes: &[u8]) {
    for byte in bytes {
        out.push(*byte).expect("precomputed message bound");
    }
}

/// Append a delivery key as a git trailer, so a committed effect can be found.
pub(crate) fn named_message(message: Box<[u8]>, name: run::CallName) -> Option<Box<[u8]>> {
    let capacity = u32::try_from(message.len()).ok()?.checked_add(80)?;
    let mut out = List::with_capacity(capacity);
    append(&mut out, &message);
    append(&mut out, b"\n\n");
    append(&mut out, &delivery_trailer(name));
    Some(out.into_boxed())
}

/// The stable trailer searched by git when reconciling a delivery intent.
#[must_use]
pub fn delivery_trailer(name: run::CallName) -> Box<[u8]> {
    let mut out = List::with_capacity(80);
    append(&mut out, b"Smith-Delivery: ");
    decimal(&mut out, name.activation);
    append(&mut out, b"/");
    decimal(&mut out, u64::from(name.completion));
    append(&mut out, b"/");
    decimal(&mut out, u64::from(name.position));
    out.into_boxed()
}

/// The bounded receipt for an inspected or newly created commit.
#[must_use]
pub fn commit_receipt(head: &[u8]) -> Option<Box<[u8]>> {
    let size = head.len().checked_add(7)?;
    if head.is_empty() || size > run::Receipt::CAPACITY {
        return None;
    }
    let mut out = List::with_capacity(u32::try_from(size).ok()?);
    append(&mut out, b"commit ");
    append(&mut out, head);
    Some(out.into_boxed())
}

/// Append a count under the caller's diagnostic bound.
pub(crate) fn decimal(out: &mut List<u8>, number: u64) {
    append_bounded(out, skein_lib::Decimal::of(number).as_bytes());
}

/// Keep an interrupted-delivery diagnostic within the sealed 512-byte tail.
pub(crate) fn append_bounded(out: &mut List<u8>, bytes: &[u8]) {
    for byte in bytes {
        if out.room() == 0 {
            break;
        }
        out.push(*byte).expect("room checked");
    }
}

#[cfg(test)]
mod tests {
    use super::{cached, commit_message};
    use crate::DeliveryRecord;
    use smith_domain::run::outcome::{Change, Field, FieldRule};
    use smith_domain::run::{CallName, Delivery};

    #[test]
    fn delivery_evidence_keeps_the_largest_key_and_receipt_boundary() {
        let name = CallName { activation: u64::MAX, completion: u32::MAX, position: u32::MAX };
        assert_eq!(
            super::delivery_trailer(name).as_ref(),
            b"Smith-Delivery: 18446744073709551615/4294967295/4294967295"
        );
        let head = [b'a'; smith_domain::run::Receipt::CAPACITY - 7];
        assert_eq!(
            super::commit_receipt(&head).expect("head at the receipt bound").len(),
            smith_domain::run::Receipt::CAPACITY
        );
        assert!(super::commit_receipt(&[b'a'; smith_domain::run::Receipt::CAPACITY - 6]).is_none());
        assert!(super::commit_receipt(b"").is_none());
    }

    #[test]
    fn the_configured_title_precedes_other_result_fields() {
        let change = Change {
            fields: Box::new([
                Field { name: b"body".as_slice().into(), value: b"Second".as_slice().into() },
                Field { name: b"title".as_slice().into(), value: b"First".as_slice().into() },
                Field { name: b"context".as_slice().into(), value: b"Third".as_slice().into() },
            ]),
        };
        let rules = [
            FieldRule { name: b"title".as_slice().into(), max: 32 },
            FieldRule { name: b"body".as_slice().into(), max: 32 },
        ];
        assert_eq!(commit_message(&change, b"title", &rules).as_deref(), Some(b"First\n\nSecond\n\nThird".as_slice()));
        assert_eq!(commit_message(&change, b"missing", &rules), None);
    }

    #[test]
    fn a_delivery_asked_again_under_its_name_gets_its_first_answer() {
        let name = CallName { activation: 2, completion: 3, position: 1 };
        let record =
            DeliveryRecord { name, state: crate::DeliveryState::Answer(Delivery::Nothing), landed: Box::new([]) };
        let mut records = skein_lib::Map::with_capacity(1);
        records.insert(name, record).expect("one entry");
        assert_eq!(cached(&records, name), Some(Delivery::Nothing));
        assert_eq!(cached(&records, CallName { activation: 2, completion: 4, position: 1 }), None);
    }
}
