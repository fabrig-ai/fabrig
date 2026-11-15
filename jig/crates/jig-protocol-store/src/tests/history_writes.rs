//! History routing at the byte commit edge (protocol/store/logs.md, 3 and 6).
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "ordinary Rust step fixtures inspect complete records"
)]
use super::{fixtures, history_logs};
use crate::{CommitCheck, CommitFailure, Event, Request, Store, codec, keys, logs};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::{Key, Record};
use jig_domain_tasks as tasks;
use skein_lib::{Queue, Token};

fn key(record: &Record) -> Box<[u8]> {
    keys::encode(&record.key(), fixtures::KEY_LIMIT).unwrap()
}
fn put(store: &Store, record: &Record) -> skein_kv::Op {
    skein_kv::Op::Put { key: key(record), value: codec::encode(record, &store.codec_limits, &store.names).unwrap() }
}
fn submit(store: &mut Store, ops: Box<[skein_kv::Op]>) -> Box<[Event]> {
    let number = store.stores.store.last().checked_add(1).unwrap();
    let mut above = Queue::uncapped();
    let mut below = fixtures::below();
    crate::commits::submit(store, &fixtures::env(), number, ops, &mut above, &mut below);
    assert!(below.store.is_empty());
    fixtures::events(&mut above)
}
fn committed(events: &[Event]) {
    let [Event::Committed { .. }] = events else { panic!("{events:?}") };
}
fn failed(events: &[Event], check: CommitCheck) {
    let [Event::Failed { why: CommitFailure::Check(why), .. }] = events else { panic!("{events:?}") };
    assert_eq!(*why, check);
}
const SETS: crate::PatchSets =
    crate::PatchSets { sets: &[crate::SKEIN_PATCH_SET, crate::SMITH_PATCH_SET, crate::JIG_STORE_PATCH_SET] };
fn limits() -> crate::Limits {
    let mut limits = fixtures::store_limits();
    limits.store.ops = 128;
    limits.store.budget = 2_097_152;
    limits
}
fn open(limits: &crate::Limits) -> Store {
    let stores = crate::Stores {
        store: skein_kv::Store::memory(&limits.store),
        secrets: skein_kv::Store::memory(&limits.secrets),
    };
    let mut store = Store::over(limits, fixtures::APPLICATION, &[], stores);
    store.patch_sets(SETS);
    let events = fixtures::run(
        &mut store,
        Request::Open { owner: Token::new(1), root: fixtures::ROOT, installation: fixtures::INSTALLATION },
    );
    let Some(Event::Opened { .. }) = events.first() else { panic!("{events:?}") };
    fixtures::committed(&mut store, Box::new([fixtures::header(1)]));
    store
}
fn opened() -> Store {
    open(&limits())
}
fn place(store: &mut Store, task: u64) -> codec::component::HistoryLog {
    let key = keys::encode_own(&keys::Own::History { task }, fixtures::KEY_LIMIT).unwrap();
    codec::component::HistoryLog::decode(&fixtures::get_kv(store, &key).unwrap()).unwrap()
}
fn records(store: &mut Store, task: u64) -> Vec<(u64, Box<[logs::record::Row]>)> {
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    skein_kv::down(
        &mut store.stores.store,
        &crate::memory_env(store.limits.store),
        skein_kv::Request::ReadLog {
            owner: Token::new(0),
            log: logs::history(task).unwrap(),
            at: 0,
            direction: skein_kv::Direction::Forward,
            max: store.limits.store.page.bytes,
        },
        &mut above,
        &mut below,
    );
    assert!(below.is_empty());
    let Some(skein_kv::Event::LogRead { records, next: None, .. }) = above.pop() else { panic!("whole log") };
    records
        .into_vec()
        .into_iter()
        .map(|record| (record.at, logs::record::decode(record.record.as_deref().unwrap()).unwrap()))
        .collect()
}

#[test]
fn a_commit_groups_history_in_key_order_and_places_then_erases_ended() {
    let mut store = opened();
    let rows = [
        fixtures::activity_line(5, 1),
        fixtures::attempt_row(5, 1),
        fixtures::terminal(5, 1),
        fixtures::ended_of(6, tasks::Party::Person(fixtures::ALICE)),
    ];
    let ops = rows.iter().map(|row| put(&store, row)).collect();
    committed(&submit(&mut store, ops));
    for task in [5, 6] {
        let found = records(&mut store, task);
        assert_eq!(found.len(), 1);
        let mut expected: Vec<_> = rows
            .iter()
            .filter(|row| keys::logged(&key(row)) == Some(task))
            .map(|row| {
                let skein_kv::Op::Put { key, value } = put(&store, row) else { unreachable!() };
                (key, Some(value))
            })
            .collect();
        expected.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(found[0].1.as_ref(), expected);
        assert_eq!(place(&mut store, task).record, if task == 6 { Some(found[0].0) } else { None });
    }
    for row in rows {
        assert_eq!(fixtures::get_kv(&mut store, &key(&row)), None);
    }
    let ended = keys::encode(&Key::Tasks(tasks::Key::Ended(6)), fixtures::KEY_LIMIT).unwrap();
    committed(&submit(&mut store, Box::new([skein_kv::Op::Erase { key: ended.clone() }])));
    let found = records(&mut store, 6);
    assert_eq!(found.len(), 2);
    assert_eq!(found[1].1.as_ref(), &[(ended, None)]);
    assert_eq!(place(&mut store, 6).record, None);
}

#[test]
fn old_map_keys_are_removed_and_erases_do_not_create_logs() {
    let mut store = opened();
    let old = fixtures::history(5, 1);
    let erased = fixtures::history(6, 1);
    let ops = Box::new([put(&store, &old), put(&store, &erased)]);
    history_logs::raw_commit(&mut store, ops);
    let mut replacement = old.clone();
    let Record::Tasks(tasks::Stored::History(ref mut history)) = replacement else { unreachable!() };
    history.reason = Box::from(b"last write".as_slice());
    let ops = Box::new([
        put(&store, &old),
        put(&store, &replacement),
        skein_kv::Op::Erase { key: key(&erased) },
        skein_kv::Op::Erase { key: key(&fixtures::history(7, 1)) },
    ]);
    let events = submit(&mut store, ops);
    committed(&events);
    assert_eq!(fixtures::get_kv(&mut store, &key(&old)), None);
    assert_eq!(fixtures::get_kv(&mut store, &key(&erased)), None);
    let skein_kv::Op::Put { key, value } = put(&store, &replacement) else { unreachable!() };
    assert_eq!(records(&mut store, 5)[0].1.as_ref(), &[(key, Some(value))]);
    for task in [6, 7] {
        assert_eq!(store.stores.store.log_end(logs::history(task).unwrap()), 0);
    }
}

#[test]
fn an_oversized_history_record_fails_encoding_at_the_record_edge() {
    let row = fixtures::terminal(5, 1);
    let store = opened();
    let skein_kv::Op::Put { key, value } = put(&store, &row) else { unreachable!() };
    let length = u32::try_from(logs::record::encode(&[(key, Some(value))]).unwrap().len()).unwrap();
    for limit in [length, length.checked_sub(1).unwrap()] {
        let mut limits = limits();
        limits.store.record = limit;
        let mut store = open(&limits);
        let op = put(&store, &row);
        let events = submit(&mut store, Box::new([op]));
        if limit == length {
            committed(&events);
        } else {
            failed(&events, CommitCheck::Encoding);
        }
    }
}

#[test]
fn a_drop_and_history_write_or_excess_routed_operations_fail_capacity() {
    for drop in [false, true] {
        let mut store = opened();
        let row = fixtures::terminal(5, 1);
        let mut ops = vec![put(&store, &row)];
        if drop {
            ops.push(skein_kv::Op::Drop { log: logs::history(5).unwrap() });
        } else {
            store.limits.store.ops = 2;
        }
        failed(&submit(&mut store, ops.into_boxed_slice()), CommitCheck::Capacity);
        assert_eq!(store.stores.store.log_end(logs::history(5).unwrap()), 0);
    }
}

#[test]
fn dropping_history_with_erases_removes_map_copies_without_appending() {
    for held in [false, true] {
        let mut store = opened();
        let ended = fixtures::ended_of(5, tasks::Party::Person(fixtures::ALICE));
        if held {
            fixtures::committed(&mut store, Box::new([fixtures::header(2), fixtures::save(ended.clone())]));
            assert!(store.stores.store.log_end(logs::history(5).unwrap()) > 0);
        }
        let old = fixtures::history(5, 1);
        let op = put(&store, &old);
        history_logs::raw_commit(&mut store, Box::new([op]));
        assert!(fixtures::get_kv(&mut store, &key(&old)).is_some());
        let number = store.stores.store.last().checked_add(1).unwrap();
        let events = fixtures::run(
            &mut store,
            fixtures::commit(
                number,
                Box::new([
                    fixtures::header(number),
                    crate::Write::DropHistory { task: 5 },
                    crate::Write::Erase(ended.key()),
                    crate::Write::Erase(old.key()),
                ]),
            ),
        );
        committed(&events);
        assert_eq!(store.stores.store.log_end(logs::history(5).unwrap()), 0);
        for key in [
            key(&ended),
            key(&old),
            logs::extent_key(logs::history(5).unwrap()),
            keys::encode_own(&keys::Own::History { task: 5 }, fixtures::KEY_LIMIT).unwrap(),
        ] {
            assert_eq!(fixtures::get_kv(&mut store, &key), None);
        }
    }
}

#[test]
fn task_logs_consumes_only_the_rows_within_quarter_writes_and_all_bytes() {
    let rows = [fixtures::terminal(5, 1), fixtures::terminal(6, 1), fixtures::terminal(7, 1)];
    let store = opened();
    let rows: Vec<_> = rows
        .iter()
        .map(|row| {
            let skein_kv::Op::Put { key, value } = put(&store, row) else { unreachable!() };
            crate::JobRow { key, value }
        })
        .collect();
    let bytes = crate::JobWrite::Put { key: rows[0].key.clone(), value: rows[0].value.clone() }.bytes();
    let mut page = crate::JobPage {
        rows: rows.into(),
        end: false,
        layout: None,
        room: crate::JobRoom { writes: 8, bytes: u64::MAX },
    };
    for (writes, limit, count) in [(8, u64::MAX, 2), (12, bytes, 1), (12, bytes.checked_sub(1).unwrap(), 0)] {
        page.room = crate::JobRoom { writes, bytes: limit };
        let answer = crate::jobs::page("task-logs", &page);
        if count == 0 {
            assert!(answer.is_err());
            continue;
        }
        let answer = answer.unwrap();
        assert_eq!(answer.writes.len(), count);
        assert_eq!(answer.through.as_ref(), Some(&page.rows[count.checked_sub(1).unwrap()].key));
    }
    page.rows = Box::new([]);
    page.end = true;
    page.room = crate::JobRoom { writes: 0, bytes: 0 };
    assert!(crate::jobs::page("task-logs", &page).unwrap().writes.is_empty());
}
