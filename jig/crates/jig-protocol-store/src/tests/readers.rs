//! Additive read-only doors (jig/docs/design/protocol/store/loads.md and jig/docs/design/protocol/store/README.md, 9).
use super::fixtures::{self, ALICE, ENDED};
use crate::{Cursor, Family, KeptMap, Phase, Restore, Row, Store, codec, keys, restore};
use alloc::collections::BTreeMap;
use jig_domain_core_vocabulary::{Census, Key, ReadResults, Record, RestoreRange};
use restore::{Cause, DecodeFailure, Item, JoinFailure, RangeFailure, WalkFailure};
use skein_lib::{List, Token, length::len_u64};

const ROWS: u32 = 8;
const BYTES: u64 = 65_536;
const OWNER: Token = Token::new(5);

fn census() -> Restore {
    Restore::Jig(Box::new([RestoreRange::Census(Census {
        questions: Box::new([]),
        results: Box::new([ReadResults { party: ALICE, after: 0 }]),
    })]))
}
fn live() -> Restore {
    Restore::Jig(Box::new(RestoreRange::LIVE))
}
fn raw_store(rows: &[skein_kv::Op]) -> Store {
    let mut stores = fixtures::stores();
    let mut ops = List::with_capacity(
        u32::try_from(rows.len()).expect("fixture row count").checked_add(4).expect("fixture plus component rows"),
    );
    for op in [
        fixtures::format_put(fixtures::DEPLOYMENT),
        fixtures::layouts_put(),
        fixtures::header_put(fixtures::DEPLOYMENT, 1),
        fixtures::installation_put(fixtures::INSTALLATION),
    ] {
        ops.push(op).expect("component row room");
    }
    for row in rows {
        ops.push(row.clone()).expect("fixture row room");
    }
    fixtures::plant(&mut stores, ops.as_slice());
    let mut store = fixtures::over(stores);
    let events = fixtures::run(
        &mut store,
        crate::Request::Open { owner: OWNER, root: fixtures::ROOT, installation: fixtures::INSTALLATION },
    );
    let [crate::Event::Opened { .. }] = events.as_ref() else { panic!("fixture opens") };
    store
}
fn put(record: Record) -> skein_kv::Op {
    skein_kv::Op::Put {
        key: keys::encode(&record.key(), fixtures::KEY_LIMIT).expect("fixture key"),
        value: codec::encode(&record, &fixtures::codec_limits(), &[]).expect("fixture value"),
    }
}
fn result() -> Record {
    fixtures::result_index(ALICE, 1, ENDED)
}
fn position() -> Record {
    Record::People(jig_domain_people::Stored::EndedRead { task: ENDED, party: ALICE, through: 1 })
}

#[test]
fn restore_rowless_faults_preserve_phase_range_and_cursor() {
    let mut store = fixtures::over(fixtures::stores());
    let error = restore::page(&mut store, live(), None, ROWS, BYTES).expect_err("closed");
    assert_eq!(error, restore::Failure { cause: Cause::Phase { found: Phase::Closed }, row: None });
    let mut store = fixtures::opened();
    store.limits.store.key = 1;
    let range = Restore::Application(Box::new([skein_kv::Range::prefix(b"long")]));
    let error = restore::page(&mut store, range, None, ROWS, BYTES).expect_err("range key too long");
    assert_eq!(error, restore::Failure { cause: Cause::Range(RangeFailure::Encoding), row: None });
    for after in [Cursor { interval: u32::MAX, after: Box::new([]) }, Cursor { interval: 0, after: Box::new([255]) }] {
        let error = restore::page(&mut store, live(), Some(&after), ROWS, BYTES).expect_err("invalid cursor");
        assert_eq!(error, restore::Failure { cause: Cause::Range(RangeFailure::Cursor), row: None });
    }
    store.phase = Phase::Stopped;
    let error = store.retained_page(None, ROWS, BYTES).expect_err("stopped");
    assert_eq!(error, restore::Failure { cause: Cause::Phase { found: Phase::Stopped }, row: None });
}

#[test]
fn restore_decode_faults_name_keys_without_values() {
    let person = fixtures::person(ALICE);
    let raw_key = keys::encode(&person.key(), fixtures::KEY_LIMIT).expect("person key");
    let other_value = codec::encode(&fixtures::person(ENDED), &fixtures::codec_limits(), &[]).expect("other person");
    for (key, value, category) in [
        (Box::from([keys::JIG, 2, 0, 255]), Box::from([255]), DecodeFailure::Key),
        (raw_key.clone(), Box::from(b"undecodable private value".as_slice()), DecodeFailure::Value),
        (raw_key.clone(), other_value, DecodeFailure::KeyValueDisagree),
    ] {
        let mut store = raw_store(&[skein_kv::Op::Put { key: key.clone(), value }]);
        let error = restore::page(&mut store, live(), None, ROWS, BYTES).expect_err("decode failure");
        assert_eq!(error.cause, Cause::Decode(category));
        let site = error.row.expect("source row");
        assert_eq!(site.family, Some(Family::People));
        assert_eq!(site.key, key);
        assert!(!alloc::fmt::format(format_args!("{site:?}")).contains("private value"));
        let Phase::Ready { .. } = store.phase else { panic!("reader preserves ready phase") };
    }
}

#[test]
fn diagnostics_bound_raw_keys_and_retain_original_lengths() {
    let mut store = fixtures::opened();
    store.limits.store.key = 3;
    for (key, family) in [(b"\x01\x02\x00bad-key".as_slice(), Some(Family::People)), (b"\xffunknown".as_slice(), None)]
    {
        let site = restore::site(&store, key);
        assert_eq!(site.key.as_ref(), &key[..3]);
        assert_eq!(site.key_bytes, len_u64(key.len()));
        assert_eq!(site.family, family);
        assert!(site.truncated);
    }
    let key = Box::from(b"\x01\x02\x00bad-key".as_slice());
    let mut store = raw_store(&[skein_kv::Op::Put { key, value: Box::new([255]) }]);
    store.limits.store.key = 3;
    let error = restore::page(&mut store, Restore::Jig(Box::new([RestoreRange::People])), None, ROWS, BYTES)
        .expect_err("malformed stored key");
    assert_eq!(error.cause, Cause::Decode(DecodeFailure::Key));
    assert_eq!(
        error.row,
        Some(restore::RowSite {
            family: Some(Family::People),
            key: Box::new([1, 2, 0]),
            key_bytes: 10,
            truncated: true
        })
    );
}

#[test]
fn restore_walk_reports_real_refusal_and_cursor_arithmetic() {
    let mut store = raw_store(&[]);
    let mut above = skein_lib::Queue::uncapped();
    let mut below = skein_lib::Queue::uncapped();
    skein_kv::down(
        &mut store.stores.store,
        &crate::memory_env(store.limits.store),
        skein_kv::Request::Close { owner: OWNER },
        &mut above,
        &mut below,
    );
    let error = restore::page(&mut store, live(), None, ROWS, BYTES).expect_err("underlying map unavailable");
    assert_eq!(
        error,
        restore::Failure { cause: Cause::Walk(WalkFailure::Refused(skein_kv::Refusal::Unavailable)), row: None }
    );
    let mut store = raw_store(&[]);
    store.limits.store.key = u32::MAX;
    let error = store.retained_page(None, ROWS, BYTES).expect_err("exclusive allowance overflow");
    assert_eq!(error, restore::Failure { cause: Cause::Arithmetic(restore::Arithmetic::CursorLength), row: None });
}

#[test]
fn restore_live_pages_keep_varied_families_in_walk_order() {
    let person = fixtures::person(ALICE);
    let task = fixtures::live_task(ENDED);
    let mut store = raw_store(&[put(person.clone()), put(task.clone())]);
    let mut after = None;
    let mut records = List::with_capacity(ROWS);
    loop {
        let page = restore::page(&mut store, live(), after.as_ref(), 1, BYTES).expect("mixed family page");
        for item in page.items {
            let Item::Row(Row::Jig(record)) = item else { panic!("domain row") };
            records.push(record).expect("fixture records fit");
        }
        after = page.next;
        if after.is_none() {
            break;
        }
    }
    assert_eq!(records.as_slice(), [fixtures::header_record(1), person, task]);
}

#[test]
fn missing_census_owners_emit_exact_keyed_obligations() {
    for record in [result(), position()] {
        let mut store = raw_store(&[put(record.clone())]);
        let page = restore::page(&mut store, census(), None, ROWS, BYTES).expect("missing is a link");
        assert!(page.next.is_none());
        let [Item::MissingLink { record: source, target }] = page.items.as_ref() else { panic!("one obligation") };
        assert_eq!(*source, record.key());
        assert_eq!(*target, Key::Tasks(jig_domain_tasks::Key::Ended(ENDED)));
    }
}

#[test]
fn present_undecodable_census_owners_are_join_decode_failures() {
    let target = keys::encode(&fixtures::ended_task(ENDED).key(), fixtures::KEY_LIMIT).expect("ended key");
    let mut store = raw_store(&[
        put(result()),
        skein_kv::Op::Put { key: target.clone(), value: Box::from(b"private broken owner".as_slice()) },
    ]);
    let error = restore::page(&mut store, census(), None, ROWS, BYTES).expect_err("present corrupt owner");
    let Cause::Join { target: joined, failure } = &error.cause else { panic!("join failure") };
    assert_eq!(*failure, JoinFailure::Decode(DecodeFailure::Value));
    assert_eq!(joined.key, target);
    assert_eq!(
        error.row.as_ref().expect("source").key,
        keys::encode(&result().key(), fixtures::KEY_LIMIT).expect("source key")
    );
    assert!(!alloc::fmt::format(format_args!("{error:?}")).contains("private broken owner"));
}

#[test]
fn restore_terminal_mapping_keeps_low_level_categories() {
    let events = [
        None,
        Some(skein_kv::Event::Closed { owner: OWNER }),
        Some(skein_kv::Event::Refused { owner: OWNER, refusal: skein_kv::Refusal::Unavailable }),
        Some(skein_kv::Event::RecoveryFailed { failure: skein_kv::Failure::Corrupt }),
    ];
    for (event, expected) in events.into_iter().zip([
        WalkFailure::NoTerminal,
        WalkFailure::UnexpectedTerminal,
        WalkFailure::Refused(skein_kv::Refusal::Unavailable),
        WalkFailure::Failed(skein_kv::Failure::Corrupt),
    ]) {
        let error = crate::pages::reader::terminal(event).err().expect("walk mapping error");
        assert_eq!(error, restore::Failure { cause: Cause::Walk(expected), row: None });
    }
    for (event, expected) in [
        (None, JoinFailure::NoTerminal),
        (Some(skein_kv::Event::Closed { owner: OWNER }), JoinFailure::UnexpectedTerminal),
        (
            Some(skein_kv::Event::Refused { owner: OWNER, refusal: skein_kv::Refusal::Busy }),
            JoinFailure::Refused(skein_kv::Refusal::Busy),
        ),
        (
            Some(skein_kv::Event::OpenFailed { owner: OWNER, failure: skein_kv::Failure::Io }),
            JoinFailure::Failed(skein_kv::Failure::Io),
        ),
    ] {
        assert_eq!(restore::reader::joined(event), Err(expected));
    }
}

#[test]
fn successful_census_pairs_are_atomic_and_exclusive() {
    let mut store = raw_store(&[put(result()), put(position()), put(fixtures::ended_task(ENDED))]);
    let first = restore::page(&mut store, census(), None, 2, BYTES).expect("pair");
    let mut records = List::with_capacity(ROWS);
    for item in first.items {
        let Item::Row(Row::Jig(record)) = item else { panic!("domain rows") };
        records.push(record).expect("fixture pair fits");
    }
    assert_eq!(records.as_slice(), [position(), fixtures::ended_task(ENDED)]);
    let second = restore::page(&mut store, census(), first.next.as_ref(), 2, BYTES).expect("result");
    assert!(second.next.is_none());
    let [Item::Row(Row::Jig(record))] = second.items.as_ref() else { panic!("one result owner") };
    assert_eq!(*record, fixtures::ended_task(ENDED));
}

#[test]
#[should_panic(expected = "empty nonterminal restore page must advance its cursor")]
fn restore_pages_panic_when_the_byte_budget_prevents_progress() {
    let mut store = raw_store(&[put(position()), put(fixtures::ended_task(ENDED))]);
    restore::page(&mut store, census(), None, ROWS, 1).expect("budget assertion");
}

#[test]
#[should_panic(expected = "empty nonterminal restore page must advance its cursor")]
fn retained_pages_panic_when_the_row_budget_prevents_progress() {
    raw_store(&[]).retained_page(None, 0, BYTES).expect("budget assertion");
}

#[test]
fn malformed_join_keys_and_disagreeing_owners_stay_distinct() {
    let mut wrong = put(fixtures::ended_task(ENDED + 1));
    let target_key = fixtures::ended_task(ENDED).key();
    let raw_target = keys::encode(&target_key, fixtures::KEY_LIMIT).expect("target key");
    match &mut wrong {
        skein_kv::Op::Put { key, .. } => *key = raw_target.clone(),
        skein_kv::Op::Erase { .. } => panic!("put"),
    }
    let mut store = raw_store(&[put(result()), wrong]);
    let error = restore::page(&mut store, census(), None, ROWS, BYTES).expect_err("owner mismatch");
    let Cause::Join { failure, .. } = error.cause else { panic!("join failure") };
    assert_eq!(failure, JoinFailure::Decode(DecodeFailure::KeyValueDisagree));
    // A valid writer configuration holds the fixed-width owner key. Exercise the
    // encoding boundary directly after reducing the map's allowance.
    store.limits.store.key = 3;
    let source = restore::site(&store, &keys::encode(&result().key(), fixtures::KEY_LIMIT).expect("source"));
    let error = restore::reader::ended(&mut store, &target_key, &source).expect_err("key encoding");
    let Cause::Join { target, failure } = error.cause else { panic!("join") };
    assert_eq!(failure, JoinFailure::KeyEncoding);
    assert_eq!(target.key.as_ref(), &raw_target[..3]);
    assert_eq!(target.key_bytes, len_u64(raw_target.len()));
    assert!(target.truncated);
    assert_eq!(error.row, Some(source));
}

#[test]
fn a_restore_people_eof_leaves_orphan_secrets_untouched() {
    let mut store = fixtures::opened();
    fixtures::committed(&mut store, Box::new([fixtures::header(1), crate::Write::Wires]));
    fixtures::run(&mut store, crate::Request::Secrets { owner: OWNER, request: fixtures::sign_in_secret(4, 41) });
    let before = (store.stores.store.last(), store.stores.secrets.last());
    let page = restore::page(&mut store, Restore::Jig(Box::new([RestoreRange::People])), None, ROWS, BYTES)
        .expect("typed people EOF");
    assert!(page.items.is_empty() && page.next.is_none());
    store.retained_page(None, ROWS, BYTES).expect("retained inventory");
    assert_eq!((store.stores.store.last(), store.stores.secrets.last()), before);
    let events = fixtures::run(
        &mut store,
        crate::Request::Secrets {
            owner: OWNER,
            request: crate::Secrets::Get { owner: crate::SecretOwner::SignIn, number: 4 },
        },
    );
    let [crate::Event::Secret { value, .. }] = events.as_ref() else { panic!("secret terminal") };
    assert_eq!(*value, Some(crate::SecretValue::Digest { expiry: 0, digest: [41; 32] }));
    // The request door performs its EOF cleanup.
    fixtures::run(
        &mut store,
        crate::Request::Restore {
            owner: OWNER,
            range: Restore::Jig(Box::new([RestoreRange::People])),
            after: None,
            rows: ROWS,
            bytes: BYTES,
        },
    );
    let events = fixtures::run(
        &mut store,
        crate::Request::Secrets {
            owner: OWNER,
            request: crate::Secrets::Get { owner: crate::SecretOwner::SignIn, number: 4 },
        },
    );
    let [crate::Event::Secret { value, .. }] = events.as_ref() else { panic!("secret terminal") };
    assert_eq!(*value, None);
}

#[test]
fn retained_byte_and_encoded_row_limits_cut_exclusively() {
    let mut store = raw_store(&[skein_kv::Op::Put { key: Box::new([255]), value: Box::new([42]) }]);
    let after = keys::installation(fixtures::KEY_LIMIT).expect("installation key");
    let page = store.retained_page(Some(&after), ROWS, 2).expect("exact two bytes");
    assert_eq!(page.rows.as_ref(), [skein_kv::Row { key: Box::new([255]), value: Box::new([42]) }]);
    assert!(page.next.is_none());
    store.limits.store.page.rows = 1;
    let page = store.retained_page(None, ROWS, BYTES).expect("encoded row cap");
    assert_eq!(page.rows.len(), 1);
    assert!(page.next.is_some());
    let after = skein_lib::bytes::zeroed(
        skein_lib::length::usize_of(fixtures::KEY_LIMIT).checked_add(1).expect("overlong cursor"),
    );
    assert_eq!(
        store.retained_page(Some(&after), ROWS, BYTES).expect_err("cursor too long"),
        restore::Failure { cause: Cause::Range(RangeFailure::Cursor), row: None }
    );
}

#[test]
fn retained_inventory_matches_independent_committed_raw_inputs() {
    let extra = [
        put(fixtures::person(ALICE)),
        skein_kv::Op::Put { key: Box::new([keys::APPLICATION, 0, 255]), value: Box::new([41, 42]) },
    ];
    let mut store = raw_store(&extra);
    let mut expected = BTreeMap::new();
    for op in [
        fixtures::format_put(fixtures::DEPLOYMENT),
        fixtures::layouts_put(),
        fixtures::header_put(fixtures::DEPLOYMENT, 1),
        fixtures::installation_put(fixtures::INSTALLATION),
    ]
    .into_iter()
    .chain(extra)
    {
        let skein_kv::Op::Put { key, value } = op else { panic!("committed puts") };
        assert!(expected.insert(key, value).is_none());
    }
    let mut found = BTreeMap::new();
    let mut after = None;
    loop {
        let page = store.retained_page(after.as_deref(), 1, BYTES).expect("raw page");
        let replay = store.retained_page(after.as_deref(), 1, BYTES).expect("reusable reader");
        assert_eq!(page.rows, replay.rows);
        assert_eq!(page.next, replay.next);
        for row in page.rows {
            if let Some(after) = after.as_deref() {
                assert!(row.key.as_ref() > after);
            }
            assert!(found.insert(row.key, row.value).is_none(), "exactly once");
        }
        after = page.next;
        if after.is_none() {
            break;
        }
    }
    assert_eq!(found, expected);
    let last = expected.last_key_value().expect("rows").0;
    let eof = store.retained_page(Some(last), ROWS, BYTES).expect("EOF");
    assert!(eof.rows.is_empty() && eof.next.is_none());
}

#[test]
fn retained_inventory_resumes_exclusively_after_an_empty_raw_key() {
    let mut store = raw_store(&[skein_kv::Op::Put { key: Box::new([]), value: Box::new([42]) }]);
    let first = store.retained_page(None, 1, BYTES).expect("empty raw key");
    assert_eq!(first.rows.as_ref(), [skein_kv::Row { key: Box::new([]), value: Box::new([42]) }]);
    assert_eq!(first.next.as_deref(), Some(b"".as_slice()));
    let second = store.retained_page(first.next.as_deref(), 1, BYTES).expect("exclusive empty key");
    for row in second.rows {
        assert!(!row.key.is_empty());
    }
}

// The one-row walk is the reference for page boundaries and row-site failures.
fn walk_by_rows(
    store: &mut Store,
    map: KeptMap,
    intervals: &[skein_kv::Range],
    after: Option<&Cursor>,
    rows: u32,
) -> Result<crate::pages::Walked, restore::Failure> {
    if let Some(after) = after {
        let index = usize::try_from(after.interval)
            .ok()
            .ok_or(restore::Failure::rowless(Cause::Range(RangeFailure::Cursor)))?;
        let Some(range) = intervals.get(index) else {
            return Err(restore::Failure::rowless(Cause::Range(RangeFailure::Cursor)));
        };
        let limit = skein_lib::length::usize_of(crate::pages::map_limits(store, map).key);
        if after.after.len() > limit
            || (after.after < range.start
                || match &range.end {
                    Some(end) => after.after >= *end,
                    None => false,
                })
        {
            return Err(restore::Failure::rowless(Cause::Range(RangeFailure::Cursor)));
        }
    }
    let capacity = rows.min(crate::pages::map_limits(store, map).page.bytes);
    let mut found = List::with_capacity(capacity);
    let mut encoded = 0_u64;
    let mut last = after.cloned().unwrap_or(Cursor { interval: 0, after: Box::new([]) });
    let first = skein_lib::length::usize_of(last.interval);
    for (index, interval) in intervals.iter().enumerate().skip(first) {
        let index = u32::try_from(index)
            .ok()
            .ok_or(restore::Failure::rowless(Cause::Arithmetic(restore::Arithmetic::RangeCount)))?;
        let mut range = interval.clone();
        if index == last.interval && after.is_some() {
            range.start = range.start.max(crate::pages::reader::past(&last.after)?);
        }
        for _ in 0..=capacity {
            if let Some(end) = &range.end
                && range.start >= *end
            {
                break;
            }
            let page = crate::pages::reader::load_map(store, map, range.clone())?;
            let mut rows = page.rows.into_iter();
            let Some(row) = rows.next() else { break };
            let size = crate::pages::reader::row_bytes(store, &row)?;
            let total = encoded.checked_add(size).ok_or(restore::Failure::at(
                Cause::Arithmetic(restore::Arithmetic::PageBytes),
                restore::site(store, &row.key),
            ))?;
            if found.len() == capacity || total > u64::from(crate::pages::map_limits(store, map).page.bytes) {
                return Ok(crate::pages::Walked { rows: found.into_boxed(), next: Some(last) });
            }
            encoded = total;
            last = Cursor { interval: index, after: row.key.clone() };
            range.start = match crate::pages::reader::past(&row.key) {
                Ok(next) => next,
                Err(failure) => return Err(restore::Failure::at(failure.cause, restore::site(store, &row.key))),
            };
            found.push(crate::pages::WalkRow { row, cursor: last.clone() }).expect("encoded and row bounds checked");
            if page.next.is_none() {
                break;
            }
        }
    }
    Ok(crate::pages::Walked { rows: found.into_boxed(), next: None })
}
fn assert_walks_match(
    store: &mut Store,
    map: KeptMap,
    intervals: &[skein_kv::Range],
    after: Option<&Cursor>,
    rows: u32,
) -> Option<Cursor> {
    let expected = walk_by_rows(store, map, intervals, after, rows);
    let found = crate::pages::reader::walk_map(store, map, intervals, after, rows);
    let limits = crate::pages::map_limits(store, map).page;
    match (found, expected) {
        (Ok(found), Ok(expected)) => {
            assert_eq!(found.next, expected.next, "next: {map:?}, {limits:?}, {rows}, {intervals:?}, {after:?}");
            assert_eq!(found.rows.len(), expected.rows.len(), "row count: {limits:?}, {rows}, {after:?}");
            for (found, expected) in found.rows.iter().zip(expected.rows.iter()) {
                assert_eq!(found.row, expected.row, "same row bytes");
                assert_eq!(found.cursor, expected.cursor, "same exclusive row cursor");
            }
            found.next
        }
        (Err(found), Err(expected)) => {
            assert_eq!(found, expected, "same failure and row site");
            None
        }
        (Ok(_), Err(expected)) => panic!("walk succeeded instead of {expected:?}"),
        (Err(found), Ok(_)) => panic!("walk failed unexpectedly: {found:?}"),
    }
}

#[test]
fn a_walk_by_pages_matches_a_walk_by_rows() {
    // Prefix-related keys exercise exclusive cursors; row sizes of 2, 4, 4, 2,
    // and 2 bytes put a byte cut both within and between underlying pages.
    let inputs = [
        skein_kv::Op::Put { key: Box::new([200]), value: Box::new([1]) },
        skein_kv::Op::Put { key: Box::new([200, 0]), value: Box::new([2, 2]) },
        skein_kv::Op::Put { key: Box::new([201]), value: Box::new([3, 3, 3]) },
        skein_kv::Op::Put { key: Box::new([202]), value: Box::new([4]) },
        skein_kv::Op::Put { key: Box::new([204]), value: Box::new([5]) },
    ];
    let interval = |from, to| skein_kv::Range { start: Box::new([from]), end: Some(Box::new([to])) };
    for map in [KeptMap::Store, KeptMap::Secrets] {
        let mut store = raw_store(&inputs);
        if map == KeptMap::Secrets {
            core::mem::swap(&mut store.stores.store, &mut store.stores.secrets);
        }
        let original_page = crate::pages::map_limits(&store, map).page;
        for intervals in [
            Box::from([interval(200, 205)]),
            Box::from([interval(200, 201), interval(202, 205)]),
            Box::from([interval(203, 204)]),
            Box::from([interval(203, 203)]),
        ] {
            let complete = walk_by_rows(&mut store, map, &intervals, None, ROWS).expect("fixture fits");
            for page_rows in [1, 2, 8] {
                for page_bytes in [2, 5, 32] {
                    match map {
                        KeptMap::Store => {
                            store.limits.store.page = skein_kv::Page { rows: page_rows, bytes: page_bytes }
                        }
                        KeptMap::Secrets => {
                            store.limits.secrets.page = skein_kv::Page { rows: page_rows, bytes: page_bytes };
                        }
                    }
                    for rows in [0, 1, 2, 3, ROWS] {
                        if let Some(next) = assert_walks_match(&mut store, map, &intervals, None, rows) {
                            let _next = assert_walks_match(&mut store, map, &intervals, Some(&next), rows);
                        }
                        for item in &complete.rows {
                            let _next = assert_walks_match(&mut store, map, &intervals, Some(&item.cursor), rows);
                        }
                    }
                }
            }
            match map {
                KeptMap::Store => store.limits.store.page = original_page,
                KeptMap::Secrets => store.limits.secrets.page = original_page,
            }
        }
        match map {
            KeptMap::Store => store.limits.store.page.bytes = 1,
            KeptMap::Secrets => store.limits.secrets.page.bytes = 1,
        }
        let intervals = [interval(200, 205)];
        let error =
            crate::pages::reader::walk_map(&mut store, map, &intervals, None, ROWS).err().expect("oversized row");
        assert_eq!(error, restore::Failure::rowless(Cause::Walk(WalkFailure::Refused(skein_kv::Refusal::TooLarge))));
        let _next = assert_walks_match(&mut store, map, &intervals, None, ROWS);
    }
}
