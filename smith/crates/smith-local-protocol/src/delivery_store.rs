//! The durable pre-effect intent and post-effect decision for a local
//! delivery (protocol/host.md, sections 5.3 and 5.5; domain/host.md,
//! section 2). Neither this codec nor its caller may discard an intent after
//! a crash: the local domain reconciles it before the next run. The file is
//! replaced through Skein's synced whole-file store before the effect starts
//! or its terminal is told to the agent.

use crate::local_store_v1::Record;
use crate::{File, StoreError, local_store_v1 as wire};
use alloc::boxed::Box;
use skein_lib::{List, Reader};
use smith_domain::run;
use smith_local_domain::{DeliveryIntent, DeliveryRecord, DeliveryState, IntentDirectory, PushTarget};

/// Encode a generated delivery record under its stable call name.
pub fn save_delivery(record: &DeliveryRecord, max_bytes: u32) -> Result<File, StoreError> {
    if payload_bytes(record)? > u64::from(max_bytes) {
        return Err(StoreError::TooLarge);
    }
    let name = wire::CallName {
        activation: record.name.activation,
        completion: record.name.completion,
        position: record.name.position,
    };
    let state = match &record.state {
        DeliveryState::Intent(intent) => wire::DeliveryState::Intent(encode_intent(intent)?),
        DeliveryState::Answer(answer) => {
            wire::DeliveryState::Answer(wire::DeliveryAnswer { answer: encode_answer(answer)? })
        }
    };
    let value = wire::Delivery { name, state, landed: encode_receipts(&record.landed)? };
    let length = value.measure(&wire::CEILINGS).or(Err(StoreError::TooLarge))?;
    if length > max_bytes {
        return Err(StoreError::TooLarge);
    }
    let bytes = value.encode(&wire::CEILINGS).or(Err(StoreError::TooLarge))?;
    Ok(File { name: delivery_name(record.name), bytes })
}

/// Decode the generated grammar and preserve the domain's constructor checks.
pub fn decode_delivery(bytes: &[u8], max_bytes: u32) -> Result<DeliveryRecord, StoreError> {
    if bytes.len() > usize::try_from(max_bytes).expect("u32 fits usize") {
        return Err(StoreError::TooLarge);
    }
    let value = wire::Delivery::decode(&wire::CEILINGS, &mut Reader::new(bytes)).or(Err(StoreError::Malformed))?;
    let name = value.name;
    if name.activation == 0 || name.completion == 0 {
        return Err(StoreError::Malformed);
    }
    let state = match value.state {
        wire::DeliveryState::Intent(intent) => DeliveryState::Intent(decode_intent(intent)?),
        wire::DeliveryState::Answer(answer) => DeliveryState::Answer(decode_answer(answer.answer)?),
    };
    Ok(DeliveryRecord {
        name: run::CallName { activation: name.activation, completion: name.completion, position: name.position },
        state,
        landed: decode_receipts(value.landed)?,
    })
}

// Check aggregate payload ownership before the generated adapters copy it.
// This is not a wire measurement: generated measure remains the sole encoded
// length, and also checks tags, counts, versions and field overhead below.
fn payload_bytes(record: &DeliveryRecord) -> Result<u64, StoreError> {
    let state = match &record.state {
        DeliveryState::Intent(intent) => {
            count(intent.directories.len())?;
            let mut bytes = 0_u64;
            for directory in &intent.directories {
                if let Some(head) = &directory.head {
                    bytes = add_bytes(bytes, head)?;
                }
                if let Some(push) = &directory.push {
                    bytes = add_bytes(bytes, &push.remote)?;
                    bytes = add_bytes(bytes, &push.branch)?;
                }
            }
            bytes
        }
        DeliveryState::Answer(answer) => answer_bytes(answer)?,
    };
    state.checked_add(receipt_bytes(&record.landed)?).ok_or(StoreError::TooLarge)
}

fn answer_bytes(answer: &run::Delivery) -> Result<u64, StoreError> {
    match answer {
        run::Delivery::Delivered(delivered) => receipt_bytes(delivered.receipts()),
        run::Delivery::Nothing | run::Delivery::Stale => Ok(0),
        run::Delivery::Refused(refused) => {
            let bytes = add_bytes(0, refused.explanation())?;
            match refused.marker() {
                Some(marker) => add_bytes(bytes, marker.path()),
                None => Ok(bytes),
            }
        }
        run::Delivery::Failed(failed) => add_bytes(0, failed.diagnostic.output()),
    }
}

fn receipt_bytes(receipts: &[run::Receipt]) -> Result<u64, StoreError> {
    count(receipts.len())?;
    let mut bytes = 0_u64;
    for receipt in receipts {
        bytes = add_bytes(bytes, receipt.text())?;
    }
    Ok(bytes)
}

fn add_bytes(total: u64, bytes: &[u8]) -> Result<u64, StoreError> {
    total.checked_add(skein_lib::len_u64(bytes.len())).ok_or(StoreError::TooLarge)
}

fn count(length: usize) -> Result<u32, StoreError> {
    let length = u32::try_from(length).or(Err(StoreError::TooLarge))?;
    if length > run::MAX_DIRECTORIES {
        return Err(StoreError::TooLarge);
    }
    Ok(length)
}

fn encode_intent(intent: &DeliveryIntent) -> Result<wire::DeliveryIntent, StoreError> {
    let mut directories = List::with_capacity(count(intent.directories.len())?);
    for entry in &intent.directories {
        let push = match &entry.push {
            Some(push) => Some(wire::PushTarget { remote: push.remote.clone(), branch: push.branch.clone() }),
            None => None,
        };
        let value = wire::IntentDirectory {
            directory: entry.directory,
            changed: entry.changed,
            head: entry.head.clone(),
            push,
        };
        directories.push(value).or(Err(StoreError::TooLarge))?;
    }
    Ok(wire::DeliveryIntent { directories })
}

fn decode_intent(intent: wire::DeliveryIntent) -> Result<DeliveryIntent, StoreError> {
    let source = intent.directories;
    let mut directories = List::with_capacity(source.len());
    let mut previous = None;
    for entry in source.into_boxed() {
        if entry.directory >= run::MAX_DIRECTORIES || !increasing(previous, entry.directory) {
            return Err(StoreError::Malformed);
        }
        previous = Some(entry.directory);
        match &entry.head {
            Some(head) if head.is_empty() => return Err(StoreError::Malformed),
            Some(_) | None => {}
        }
        let push = match entry.push {
            Some(push) => Some(PushTarget { remote: push.remote, branch: push.branch }),
            None => None,
        };
        directories
            .push(IntentDirectory { directory: entry.directory, changed: entry.changed, head: entry.head, push })
            .expect("bounded directories");
    }
    Ok(DeliveryIntent { directories: directories.into_boxed() })
}

fn encode_receipts(receipts: &[run::Receipt]) -> Result<List<wire::Receipt>, StoreError> {
    let mut values = List::with_capacity(count(receipts.len())?);
    for receipt in receipts {
        let value = wire::Receipt { directory: receipt.directory(), text: receipt.text().into() };
        values.push(value).or(Err(StoreError::TooLarge))?;
    }
    Ok(values)
}

fn decode_receipts(values: List<wire::Receipt>) -> Result<Box<[run::Receipt]>, StoreError> {
    let mut receipts = List::with_capacity(values.len());
    let mut previous = None;
    for value in values.into_boxed() {
        if !increasing(previous, value.directory) {
            return Err(StoreError::Malformed);
        }
        previous = Some(value.directory);
        receipts
            .push(run::Receipt::new(value.directory, value.text).ok_or(StoreError::Malformed)?)
            .expect("bounded receipts");
    }
    Ok(receipts.into_boxed())
}

fn increasing(previous: Option<u32>, current: u32) -> bool {
    match previous {
        Some(prior) => prior < current,
        None => true,
    }
}

fn encode_answer(answer: &run::Delivery) -> Result<wire::Answer, StoreError> {
    match answer {
        run::Delivery::Delivered(delivered) => {
            Ok(wire::Answer::Delivered(wire::Delivered { receipts: encode_receipts(delivered.receipts())? }))
        }
        run::Delivery::Nothing => Ok(wire::Answer::Nothing),
        run::Delivery::Stale => Ok(wire::Answer::Stale),
        run::Delivery::Refused(refused) => {
            let marker = match refused.marker() {
                Some(marker) => Some(wire::Marker { directory: marker.directory(), path: marker.path().into() }),
                None => None,
            };
            Ok(wire::Answer::Refused(wire::DeliveryRefusal { marker, explanation: refused.explanation().into() }))
        }
        run::Delivery::Failed(failed) => Ok(wire::Answer::Failed(wire::DeliveryFailure {
            directory: failed.directory,
            reason: encode_reason(failed.reason),
            output: failed.diagnostic.output().into(),
            cut: failed.diagnostic.cut(),
        })),
    }
}

fn decode_answer(answer: wire::Answer) -> Result<run::Delivery, StoreError> {
    match answer {
        wire::Answer::Delivered(delivered) => Ok(run::Delivery::Delivered(
            run::Delivered::new(decode_receipts(delivered.receipts)?).ok_or(StoreError::Malformed)?,
        )),
        wire::Answer::Nothing => Ok(run::Delivery::Nothing),
        wire::Answer::Stale => Ok(run::Delivery::Stale),
        wire::Answer::Refused(refused) => {
            let marker = match refused.marker {
                Some(marker) => Some(run::Marker::new(marker.directory, marker.path).ok_or(StoreError::Malformed)?),
                None => None,
            };
            Ok(run::Delivery::Refused(
                run::DeliveryRefusal::new(marker, refused.explanation).ok_or(StoreError::Malformed)?,
            ))
        }
        wire::Answer::Failed(failed) => Ok(run::Delivery::Failed(run::DeliveryFailure {
            directory: failed.directory,
            reason: decode_reason(failed.reason),
            diagnostic: run::Diagnostic::new(&failed.output, failed.cut),
        })),
    }
}

fn encode_reason(reason: run::DeliveryReason) -> wire::DeliveryReason {
    match reason {
        run::DeliveryReason::Unreachable => wire::DeliveryReason::Unreachable,
        run::DeliveryReason::RefusedByTarget => wire::DeliveryReason::RefusedByTarget,
        run::DeliveryReason::TimedOut => wire::DeliveryReason::TimedOut,
        run::DeliveryReason::Broken => wire::DeliveryReason::Broken,
        run::DeliveryReason::TooLarge => wire::DeliveryReason::TooLarge,
        run::DeliveryReason::Missing => wire::DeliveryReason::Missing,
        run::DeliveryReason::Busy => wire::DeliveryReason::Busy,
        run::DeliveryReason::Unavailable => wire::DeliveryReason::Unavailable,
        run::DeliveryReason::Cancelled => wire::DeliveryReason::Cancelled,
        run::DeliveryReason::Unknown => wire::DeliveryReason::Unknown,
    }
}

fn decode_reason(reason: wire::DeliveryReason) -> run::DeliveryReason {
    match reason {
        wire::DeliveryReason::Unreachable => run::DeliveryReason::Unreachable,
        wire::DeliveryReason::RefusedByTarget => run::DeliveryReason::RefusedByTarget,
        wire::DeliveryReason::TimedOut => run::DeliveryReason::TimedOut,
        wire::DeliveryReason::Broken => run::DeliveryReason::Broken,
        wire::DeliveryReason::TooLarge => run::DeliveryReason::TooLarge,
        wire::DeliveryReason::Missing => run::DeliveryReason::Missing,
        wire::DeliveryReason::Busy => run::DeliveryReason::Busy,
        wire::DeliveryReason::Unavailable => run::DeliveryReason::Unavailable,
        wire::DeliveryReason::Cancelled => run::DeliveryReason::Cancelled,
        wire::DeliveryReason::Unknown => run::DeliveryReason::Unknown,
    }
}

fn delivery_name(name: run::CallName) -> Box<[u8]> {
    let mut out = [b'0'; 42];
    let mut cursor = 0_usize;
    decimal(&mut out, &mut cursor, name.activation, 20);
    let dot = out.get_mut(cursor).expect("name separator position");
    *dot = b'.';
    cursor = cursor.saturating_add(1);
    decimal(&mut out, &mut cursor, u64::from(name.completion), 10);
    let dot = out.get_mut(cursor).expect("name separator position");
    *dot = b'.';
    cursor = cursor.saturating_add(1);
    decimal(&mut out, &mut cursor, u64::from(name.position), 10);
    Box::from(out)
}

fn decimal(out: &mut [u8], cursor: &mut usize, value: u64, width: usize) {
    let digits = skein_lib::Decimal::of(value);
    let end = cursor.checked_add(width).expect("fixed decimal field bound");
    let start = end.checked_sub(digits.as_bytes().len()).expect("value fits its decimal field");
    for (slot, digit) in out.get_mut(start..end).expect("fixed decimal slot").iter_mut().zip(digits.as_bytes()) {
        *slot = *digit;
    }
    *cursor = end;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name() -> run::CallName {
        run::CallName { activation: 7, completion: 3, position: 1 }
    }

    #[test]
    fn pre_effect_intent_survives_file_round_trip() {
        let record = DeliveryRecord {
            name: name(),
            state: DeliveryState::Intent(DeliveryIntent {
                directories: Box::new([
                    IntentDirectory {
                        directory: 0,
                        changed: true,
                        head: Some(Box::from(&b"abc"[..])),
                        push: Some(PushTarget { remote: Box::from(&b"origin"[..]), branch: Box::from(&b"main"[..]) }),
                    },
                    IntentDirectory { directory: 1, changed: true, head: None, push: None },
                ]),
            }),
            landed: Box::new([]),
        };
        let file = save_delivery(&record, 4096).expect("bounded intent");
        assert_eq!(save_delivery(&record, 8), Err(StoreError::TooLarge));
        assert_eq!(decode_delivery(&file.bytes, 4096), Ok(record));
    }

    #[test]
    fn landed_answer_and_receipts_survive_file_round_trip() {
        let receipt = run::Receipt::new(0, Box::from(&b"commit abc"[..])).expect("bounded receipt");
        let record = DeliveryRecord {
            name: name(),
            state: DeliveryState::Answer(run::Delivery::Delivered(
                run::Delivered::new(Box::new([receipt.clone()])).expect("one receipt"),
            )),
            landed: Box::new([receipt]),
        };
        let file = save_delivery(&record, 4096).expect("bounded answer");
        assert_eq!(decode_delivery(&file.bytes, 4096), Ok(record));
        assert_eq!(decode_delivery(&file.bytes, 8), Err(StoreError::TooLarge));
    }

    #[test]
    fn failed_answer_keeps_diagnostic_cut_and_refusal_keeps_marker() {
        let failed = DeliveryRecord {
            name: name(),
            state: DeliveryState::Answer(run::Delivery::Failed(run::DeliveryFailure {
                directory: 1,
                reason: run::DeliveryReason::TimedOut,
                diagnostic: run::Diagnostic::new(b"last bytes", 42),
            })),
            landed: Box::new([]),
        };
        let file = save_delivery(&failed, 4096).expect("bounded failure");
        assert_eq!(decode_delivery(&file.bytes, 4096), Ok(failed));

        let refused = DeliveryRecord {
            name: name(),
            state: DeliveryState::Answer(run::Delivery::Refused(
                run::DeliveryRefusal::new(
                    Some(run::Marker::new(0, Box::from(&b"src/main.rs"[..])).expect("relative marker")),
                    Box::from(&b"markers remain"[..]),
                )
                .expect("bounded refusal"),
            )),
            landed: Box::new([]),
        };
        let file = save_delivery(&refused, 4096).expect("bounded refusal");
        assert_eq!(decode_delivery(&file.bytes, 4096), Ok(refused));
    }
}

#[cfg(test)]
#[expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "codec adapter tests are ordinary Rust (programming-model.md, section 10.2)"
)]
mod validity_tests {
    use super::*;

    fn record(state: DeliveryState) -> DeliveryRecord {
        DeliveryRecord {
            name: run::CallName { activation: 7, completion: 3, position: 1 },
            state,
            landed: Box::new([]),
        }
    }

    fn encoded_wire(value: &wire::Delivery) -> Box<[u8]> {
        value.encode(&wire::CEILINGS).expect("bounded delivery")
    }

    fn empty_answer() -> wire::Delivery {
        let file = save_delivery(&record(DeliveryState::Answer(run::Delivery::Nothing)), 4096).unwrap();
        wire::Delivery::decode(&wire::CEILINGS, &mut Reader::new(&file.bytes)).unwrap()
    }

    #[test]
    fn every_answer_and_failure_reason_survives_the_generated_adapter() {
        let mut answers = vec![
            run::Delivery::Nothing,
            run::Delivery::Stale,
            run::Delivery::Refused(run::DeliveryRefusal::new(None, b"cannot deliver".as_slice().into()).unwrap()),
        ];
        for reason in [
            run::DeliveryReason::Unreachable,
            run::DeliveryReason::RefusedByTarget,
            run::DeliveryReason::TimedOut,
            run::DeliveryReason::Broken,
            run::DeliveryReason::TooLarge,
            run::DeliveryReason::Missing,
            run::DeliveryReason::Busy,
            run::DeliveryReason::Unavailable,
            run::DeliveryReason::Cancelled,
            run::DeliveryReason::Unknown,
        ] {
            answers.push(run::Delivery::Failed(run::DeliveryFailure {
                directory: 1,
                reason,
                diagnostic: run::Diagnostic::new(&[b'x'; run::Diagnostic::CAPACITY], 19),
            }));
        }
        for answer in answers {
            let value = record(DeliveryState::Answer(answer));
            let file = save_delivery(&value, 4096).unwrap();
            assert_eq!(decode_delivery(&file.bytes, 4096), Ok(value));
        }
    }

    #[test]
    fn nonzero_call_names_and_ordered_bounded_intent_directories_are_still_required() {
        for (activation, completion) in [(0, 3), (7, 0)] {
            let mut value = record(DeliveryState::Answer(run::Delivery::Nothing));
            value.name.activation = activation;
            value.name.completion = completion;
            let file = save_delivery(&value, 4096).unwrap();
            assert_eq!(decode_delivery(&file.bytes, 4096), Err(StoreError::Malformed));
        }
        for directories in [vec![1, 0], vec![0, 0], vec![run::MAX_DIRECTORIES]] {
            let value = record(DeliveryState::Intent(DeliveryIntent {
                directories: directories
                    .into_iter()
                    .map(|directory| IntentDirectory { directory, changed: false, head: None, push: None })
                    .collect(),
            }));
            let file = save_delivery(&value, 4096).unwrap();
            assert_eq!(decode_delivery(&file.bytes, 4096), Err(StoreError::Malformed));
        }
        let value = record(DeliveryState::Intent(DeliveryIntent {
            directories: Box::new([IntentDirectory {
                directory: 0,
                changed: false,
                head: Some(Box::new([])),
                push: None,
            }]),
        }));
        let file = save_delivery(&value, 4096).unwrap();
        assert_eq!(decode_delivery(&file.bytes, 4096), Err(StoreError::Malformed));
    }

    #[test]
    fn receipt_constructors_and_order_survive_the_generated_grammar() {
        for rows in [
            vec![(0, b"".as_slice())],
            vec![(run::MAX_DIRECTORIES, b"ok")],
            vec![(1, b"first"), (0, b"second")],
            vec![(0, b"first"), (0, b"second")],
        ] {
            let mut parts = empty_answer();
            let mut landed = List::with_capacity(u32::try_from(rows.len()).unwrap());
            for (directory, text) in rows {
                landed.push(wire::Receipt { directory, text: text.into() }).unwrap();
            }
            parts.landed = landed;
            let value = parts;
            assert_eq!(decode_delivery(&encoded_wire(&value), 4096), Err(StoreError::Malformed));
        }
    }

    #[test]
    fn delivered_and_refused_constructors_still_reject_invalid_payloads() {
        for answer in [
            wire::Answer::Delivered(wire::Delivered { receipts: List::with_capacity(0) }),
            wire::Answer::Refused(wire::DeliveryRefusal { marker: None, explanation: Box::new([]) }),
            wire::Answer::Refused(wire::DeliveryRefusal {
                marker: Some(wire::Marker { directory: 0, path: b"../outside".as_slice().into() }),
                explanation: b"markers".as_slice().into(),
            }),
        ] {
            let mut parts = empty_answer();
            parts.state = wire::DeliveryState::Answer(wire::DeliveryAnswer { answer });
            let value = parts;
            assert_eq!(decode_delivery(&encoded_wire(&value), 4096), Err(StoreError::Malformed));
        }
    }
}

#[cfg(test)]
mod aggregate_tests {
    use super::*;

    #[test]
    fn generated_adaptation_checks_aggregate_payload_before_copying_intent_fields() {
        let record = DeliveryRecord {
            name: run::CallName { activation: 1, completion: 1, position: 0 },
            state: DeliveryState::Intent(DeliveryIntent {
                directories: Box::new([
                    IntentDirectory {
                        directory: 0,
                        changed: true,
                        head: None,
                        push: Some(PushTarget { remote: Box::new([b'x'; 2048]), branch: Box::new([b'y'; 2048]) }),
                    },
                    IntentDirectory {
                        directory: 1,
                        changed: true,
                        head: None,
                        push: Some(PushTarget { remote: b"r".as_slice().into(), branch: b"b".as_slice().into() }),
                    },
                ]),
            }),
            landed: Box::new([]),
        };
        assert_eq!(save_delivery(&record, 4096), Err(StoreError::TooLarge));
        let file = save_delivery(&record, 8192).expect("payload and generated field overhead fit");
        assert_eq!(decode_delivery(&file.bytes, 8192), Ok(record));
    }
}
