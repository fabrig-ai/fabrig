//! A goal's red families and their patch (protocol/store.md, 2).
use super::{catch_up::patched, fixtures};
use crate::{Undecodable, keys, records, v20, v22};
use skein_lib::{List, Writer};
use temper_engine_domain_forge_vocabulary as d;

fn row() -> d::Red {
    d::Red {
        goal: 7,
        key: b"test a::t x".as_slice().into(),
        head: Box::new([4; 20]),
        why: b"base".as_slice().into(),
        by: d::Recorder::Task(7),
        at: 9,
        generation: 1,
    }
}
fn filled(byte: u8, len: u32) -> Box<[u8]> {
    let mut out = List::with_capacity(len);
    for _ in 0..len {
        out.push(byte).unwrap();
    }
    out.into_boxed()
}
#[test]
fn red_families_have_golden_keys_and_record_bytes() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    for (by, tag) in [(d::Recorder::Task(7), 0), (d::Recorder::Party(7), 1)] {
        let red = row();
        let stored = d::Stored::Red(d::Red { by, ..red.clone() });
        // Literal field framing, independent of the generated codec.
        let mut golden = Writer::new(83);
        golden.put(&[0, 22, 39]).unwrap();
        golden.put(&7_u64.to_be_bytes()).unwrap();
        golden.put(b"\0\0\0\x0btest a::t x").unwrap();
        golden.put(b"\0\0\0\x14").unwrap();
        golden.put(&[4; 20]).unwrap();
        golden.put(b"\0\0\0\x04base").unwrap();
        golden.put(&[tag]).unwrap();
        golden.put(&7_u64.to_be_bytes()).unwrap();
        golden.put(&9_u64.to_be_bytes()).unwrap();
        golden.put(&1_u64.to_be_bytes()).unwrap();
        let golden = golden.finish();
        assert_eq!(records::encode(&stored), golden);
        let key = keys::encode(&d::stored_key(&stored), u32::MAX).unwrap();
        assert_eq!(key.as_ref(), b"\x2c\0\0\0\0\0\0\0\x07test a::t x\0\0");
        assert_eq!(keys::decode(&key, &fixtures::LIMITS), Some(d::Key::Red { goal: 7, key: red.key }));
        assert_eq!(records::row(&key, &golden, &bounds), Ok(stored));
    }
    let number = d::Stored::RedNumber { last: u64::MAX };
    let key = keys::encode(&d::stored_key(&number), u32::MAX).unwrap();
    assert_eq!(key.as_ref(), &[45]);
    assert_eq!(keys::decode(&key, &fixtures::LIMITS), Some(d::Key::RedNumber));
    let golden = b"\0\x16\x28\xff\xff\xff\xff\xff\xff\xff\xff";
    assert_eq!(records::encode(&number).as_ref(), golden);
    assert_eq!(records::row(&key, golden, &bounds), Ok(number));
}
#[test]
fn red_decoding_checks_key_why_head_and_configured_byte_bounds() {
    for cap in [1, 256, 1024] {
        let limits = d::Limits { red_key_bytes: cap, ..fixtures::LIMITS };
        let bounds = crate::bounds(&limits, limits.client.op_bytes).unwrap();
        for width in [20, 32] {
            let red = d::Red { key: filled(b'k', cap), why: filled(b'w', 256), head: filled(7, width), ..row() };
            let stored = d::Stored::Red(red);
            let bytes = records::encode(&stored);
            let key = keys::encode(&d::stored_key(&stored), u32::MAX).unwrap();
            assert_eq!(records::row(&key, &bytes, &bounds), Ok(stored.clone()));
            let wrong = keys::encode(&d::Key::Red { goal: 8, key: filled(b'k', cap) }, u32::MAX).unwrap();
            assert_eq!(records::row(&wrong, &bytes, &bounds), Err(Undecodable::Malformed));
            for end in 0..bytes.len() {
                assert_eq!(records::decode(bytes.get(..end).unwrap(), &bounds), Err(Undecodable::Malformed));
            }
            if cap < 1024 {
                let d::Stored::Red(mut red) = stored else { unreachable!() };
                red.key = filled(b'k', cap + 1);
                let stored = d::Stored::Red(red);
                assert_eq!(records::decode(&records::encode(&stored), &bounds), Err(Undecodable::Malformed));
                assert_eq!(keys::decode(&keys::encode(&d::stored_key(&stored), u32::MAX).unwrap(), &limits), None);
            }
        }
        let worst = d::Key::Red { goal: u64::MAX, key: filled(0, cap) };
        let length = u32::try_from(keys::encode(&worst, u32::MAX).unwrap().len()).unwrap();
        assert!(keys::largest(&limits).unwrap() >= length);
    }
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    for key in [b"".as_slice(), b" a", b"a ", b"a\nb", b"a\tb", b"a\0b", b"\xff", b"\xc2\xa0a"] {
        let stored = d::Stored::Red(d::Red { key: key.into(), ..row() });
        assert_eq!(records::decode(&records::encode(&stored), &bounds), Err(Undecodable::Malformed));
        assert_eq!(keys::decode(&keys::encode(&d::stored_key(&stored), u32::MAX).unwrap(), &fixtures::LIMITS), None);
    }
    for why in [b"".as_slice(), b"a\nb", b"a\tb", b"\xff"] {
        let stored = d::Stored::Red(d::Red { why: why.into(), ..row() });
        assert_eq!(records::decode(&records::encode(&stored), &bounds), Err(Undecodable::Malformed));
    }
    for width in [0, 19, 21, 31] {
        let stored = d::Stored::Red(d::Red { head: filled(1, width), ..row() });
        assert_eq!(records::decode(&records::encode(&stored), &bounds), Err(Undecodable::Malformed));
    }
    for (key, head, why) in [(1025, 20, 1), (1, 33, 1), (1, 20, 257)] {
        v22::Red::new(
            &v22::CEILINGS,
            v22::RedParts {
                goal: 7,
                key: filled(b'k', key),
                head: filled(1, head),
                why: filled(b'w', why),
                by: v22::Recorder::Task(v22::Number::new(&v22::CEILINGS, v22::NumberParts { number: 7 }).unwrap()),
                at: 1,
                generation: 1,
            },
        )
        .unwrap_err();
    }
}
#[test]
fn the_red_patch_keeps_old_rows_without_making_a_red_or_counter() {
    let old = v20::Row::new(
        &v20::CEILINGS,
        v20::RowParts {
            value: v20::Value::ForgeNumber(
                v20::ForgeNumber::new(&v20::CEILINGS, v20::ForgeNumberParts { last: 3 }).unwrap(),
            ),
        },
    )
    .unwrap();
    let mut bytes = Writer::new(usize::try_from(old.measure()).unwrap());
    old.encode(&mut bytes).unwrap();
    let bytes = patched(20, 9, b"Row", bytes.finish());
    let bytes = patched(21, 10, b"Row", bytes);
    assert_eq!(bytes.as_ref(), b"\0\x16\x16\0\x03");
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    assert_eq!(records::decode(&bytes, &bounds), Ok(d::Stored::ForgeNumber { last: 3 }));
    assert_eq!(crate::STORE_VERSION, 22);
}

fn golden_red() -> Box<[u8]> {
    skein_lib::bytes::concat(&[
        7_u64.to_be_bytes().as_slice(),
        b"\0\0\0\x0btest a::t x",
        b"\0\0\0\x14",
        &[4; 20],
        b"\0\0\0\x04base",
        b"\0",
        &7_u64.to_be_bytes(),
        &9_u64.to_be_bytes(),
        &1_u64.to_be_bytes(),
    ])
}
#[test]
fn named_red_answers_have_golden_bytes_and_charge_every_snapshot() {
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    let outcomes = [
        (
            Ok(d::RedOutcome::Recorded {
                head: Box::new([5; 20]),
                recorded: d::RedRecorded { generation: 2, replaced: None },
            }),
            skein_lib::bytes::concat(&[b"\0\0\0\0\x14".as_slice(), &[5; 20], &2_u64.to_be_bytes(), b"\0"]),
        ),
        (
            Ok(d::RedOutcome::Recorded {
                head: Box::new([5; 32]),
                recorded: d::RedRecorded { generation: 2, replaced: Some(row()) },
            }),
            skein_lib::bytes::concat(&[
                b"\0\0\0\0\x20".as_slice(),
                &[5; 32],
                &2_u64.to_be_bytes(),
                b"\x01",
                &golden_red(),
            ]),
        ),
        (Ok(d::RedOutcome::Ended(row())), skein_lib::bytes::concat(&[b"\x01".as_slice(), &golden_red()])),
        (Err(d::RedRefusal::NoGoal), b"\x02\0".as_slice().into()),
        (Err(d::RedRefusal::Full { bound: 64 }), b"\x02\x01\0\0\0\x40".as_slice().into()),
        (Err(d::RedRefusal::NoSuch), b"\x02\x02".as_slice().into()),
    ];
    for (result, body) in outcomes {
        let row = d::Stored::Call {
            key: d::NamedCall { task: 7, attempt: 1, completion: 2, position: 3 },
            answer: d::CallPayload::Red(Box::new(d::RedReply {
                goal: 7,
                key: b"test a::t x".as_slice().into(),
                result,
            })),
        };
        let golden = skein_lib::bytes::concat(&[
            b"\0\x16\0".as_slice(),
            &7_u64.to_be_bytes(),
            &1_u64.to_be_bytes(),
            &2_u32.to_be_bytes(),
            &3_u32.to_be_bytes(),
            b"\x06",
            &7_u64.to_be_bytes(),
            b"\0\0\0\x0btest a::t x",
            &body,
        ]);
        assert_eq!(records::encode(&row).as_ref(), golden.as_ref());
        assert_eq!(records::decode(&golden, &bounds), Ok(row.clone()));
        let meter = skein_world::domain::heap::Meter::new();
        meter.start();
        let cloned = row.clone();
        let held = meter.end().held();
        assert_eq!(d::stored_bytes(&cloned), Some(held));
        drop(cloned);
        for end in 0..golden.len() {
            assert_eq!(records::decode(golden.get(..end).unwrap(), &bounds), Err(Undecodable::Malformed));
        }
    }
}
#[test]
fn the_red_reply_patch_preserves_old_named_calls_without_making_reds() {
    use crate::v21 as old;
    assert_eq!(crate::STORE_VERSION, 22);
    let call = old::Call::new(
        &old::CEILINGS,
        old::CallParts {
            key: old::NamedCall::new(
                &old::CEILINGS,
                old::NamedCallParts { task: 7, attempt: 1, completion: 2, position: 3 },
            )
            .unwrap(),
            answer: old::CallPayload::Follow(
                old::CallFollow::new(&old::CEILINGS, old::CallFollowParts { follow: 4, refusal: None }).unwrap(),
            ),
        },
    )
    .unwrap();
    let old = old::Row::new(&old::CEILINGS, old::RowParts { value: old::Value::Call(call) }).unwrap();
    let mut bytes = Writer::new(usize::try_from(old.measure()).unwrap());
    old.encode(&mut bytes).unwrap();
    let before = bytes.finish();
    let after = patched(21, 10, b"Row", before.clone());
    assert_eq!(after.get(..2).unwrap(), b"\0\x16");
    assert_eq!(after.get(2..).unwrap(), before.get(2..).unwrap());
    let bounds = crate::bounds(&fixtures::LIMITS, fixtures::LIMITS.client.op_bytes).unwrap();
    assert_eq!(
        records::decode(&after, &bounds),
        Ok(d::Stored::Call {
            key: d::NamedCall { task: 7, attempt: 1, completion: 2, position: 3 },
            answer: d::CallPayload::Follow { follow: 4, refusal: None },
        })
    );
}
