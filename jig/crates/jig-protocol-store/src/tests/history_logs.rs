//! Hand-moved history exercises the serving view before writers append history.
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "history view step tests compare complete answers and construct frozen byte fixtures"
)]
use super::fixtures::{self, ALICE, CHILD, GOAL, GRANDCHILD};
use crate::{Event, Get, Phase, Request, Restore, Store, codec, keys, logs};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::{self as core, Key, Record, reads};
use jig_domain_tasks as tasks;
use skein_lib::{List, Queue, Set, Token};

pub(super) fn move_history(store: &mut Store, task: u64) {
    let ops = move_operations(store, task);
    raw_commit(store, ops);
}
fn move_operations(store: &mut Store, task: u64) -> Box<[skein_kv::Op]> {
    let mut rows = List::uncapped();
    let mut start: Box<[u8]> = Box::new([1]);
    loop {
        let mut above = Queue::uncapped();
        let mut below = Queue::uncapped();
        skein_kv::down(
            &mut store.stores.store,
            &crate::memory_env(store.limits.store),
            skein_kv::Request::Load {
                owner: Token::new(0),
                range: skein_kv::Range { start, end: Some(Box::new([2])) },
                max: store.limits.store.page,
            },
            &mut above,
            &mut below,
        );
        assert!(below.is_empty());
        let Some(skein_kv::Event::Loaded { rows: page, next, .. }) = above.pop() else { panic!("map page") };
        for row in page {
            if keys::logged(&row.key) == Some(task) {
                rows.push((row.key, Some(row.value))).unwrap();
            }
        }
        let Some(next) = next else { break };
        start = next;
    }
    append_operations(store, task, rows.into_boxed(), true)
}
pub(super) fn append(store: &mut Store, task: u64, rows: Box<[logs::record::Row]>, erase_map: bool) {
    let ops = append_operations(store, task, rows, erase_map);
    raw_commit(store, ops);
}
fn append_operations(
    store: &mut Store,
    task: u64,
    rows: Box<[logs::record::Row]>,
    erase_map: bool,
) -> Box<[skein_kv::Op]> {
    let log = logs::history(task).unwrap();
    let at = store.stores.store.log_end(log);
    let body = logs::record::encode(&rows).unwrap();
    let frame = store.stores.store.frame(&body).unwrap();
    let mut ops = List::uncapped();
    ops.push(skein_kv::Op::Append { log, frame }).unwrap();
    let place_key = keys::encode_own(&keys::Own::History { task }, store.limits.store.key).unwrap();
    let mut place = match fixtures::get_kv(store, &place_key) {
        Some(value) => codec::component::HistoryLog::decode(&value).unwrap(),
        None => codec::component::HistoryLog { record: None },
    };
    for (key, value) in rows {
        if key.starts_with(&[1, 1, 6]) {
            place.record = value.map(|_| at);
        }
        if erase_map {
            ops.push(skein_kv::Op::Erase { key }).unwrap();
        }
    }
    ops.push(skein_kv::Op::Put { key: place_key, value: place.encode().unwrap() }).unwrap();
    ops.into_boxed()
}
pub(super) fn raw_commit(store: &mut Store, ops: Box<[skein_kv::Op]>) {
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    skein_kv::down(
        &mut store.stores.store,
        &crate::memory_env(store.limits.store),
        skein_kv::Request::Commit { owner: Token::new(0), ops },
        &mut above,
        &mut below,
    );
    assert!(below.is_empty());
    let Some(skein_kv::Event::Committed { number, .. }) = above.pop() else { panic!("move commit fits: {above:?}") };
    store.phase = Phase::Ready { next: number.checked_add(1).unwrap(), format_due: false };
}
pub(super) fn read(store: &mut Store, range: reads::Range) -> reads::Loaded {
    let events = fixtures::run(store, Request::Read { owner: Token::new(7), range });
    let mut events = events.into_iter();
    let Some(Event::Read { result, .. }) = events.next() else { panic!("one read") };
    assert!(events.next().is_none());
    result
}
pub(super) fn equal(before: &reads::Loaded, after: &reads::Loaded) {
    assert!(format!("{before:?}") == format!("{after:?}"), "read answers differ over map and log");
}
pub(super) fn same(store: &mut Store, tasks: &[u64], range: reads::Range) -> reads::Loaded {
    let before = read(store, range.clone());
    for &task in tasks {
        move_history(store, task);
    }
    let after = read(store, range);
    equal(&before, &after);
    after
}
fn child_store() -> Store {
    let mut store = fixtures::opened();
    fixtures::committed(
        &mut store,
        Box::new([
            fixtures::header(1),
            fixtures::save(fixtures::ended_of(CHILD, tasks::Party::Person(ALICE))),
            fixtures::save(fixtures::terminal(CHILD, 1)),
            fixtures::save(fixtures::history(CHILD, 1)),
        ]),
    );
    store
}
fn got(store: &mut Store) -> Option<Record> {
    let events = fixtures::run(
        store,
        Request::Get {
            owner: Token::new(7),
            key: Get::Jig(
                crate::ended::key(&core::Now::EndedPersonTask {
                    request: Token::new(9),
                    person: ALICE,
                    project: fixtures::PROJECT,
                    task: CHILD,
                })
                .unwrap(),
            ),
        },
    );
    let [Event::Got { row, .. }] = events.as_ref() else { panic!("one get: {events:?}") };
    match row {
        Some(crate::Row::Jig(row)) => Some(row.clone()),
        None => None,
        Some(crate::Row::Application { .. }) => panic!("domain get"),
    }
}
#[test]
fn an_ended_get_and_its_terminal_read_match_the_map() {
    let mut store = child_store();
    let before = got(&mut store);
    let range = fixtures::read(ALICE, reads::Scope::Task { task: CHILD });
    let page = read(&mut store, range.clone());
    move_history(&mut store, CHILD);
    assert_eq!(before, got(&mut store));
    equal(&page, &read(&mut store, range));
}
#[test]
fn a_later_erase_hides_a_fetched_key_and_none_names_no_ended_record() {
    let mut store = child_store();
    move_history(&mut store, CHILD);
    let key = keys::encode(&Key::Tasks(tasks::Key::Ended(CHILD)), store.limits.store.key).unwrap();
    append(&mut store, CHILD, Box::new([(key, None)]), false);
    assert_eq!(got(&mut store), None);
    let range = fixtures::read(ALICE, reads::Scope::TaskOwnRows { task: CHILD });
    let before = read(&mut store, range.clone());
    let key =
        keys::encode(&Key::Tasks(tasks::Key::History { task: CHILD, revision: 1 }), store.limits.store.key).unwrap();
    append(&mut store, CHILD, Box::new([(key.clone(), None)]), false);
    let reads::Loaded::Page { rows: before, .. } = before else { panic!("page") };
    let reads::Loaded::Page { rows: after, .. } = read(&mut store, range) else { panic!("page") };
    assert_eq!(before.len(), after.len().checked_add(1).unwrap());
    assert!(!after.iter().any(|row| keys::encode(&row.value.key(), u32::MAX).unwrap() == key));
}
fn census() -> Restore {
    Restore::Jig(Box::new([core::RestoreRange::Census(core::Census {
        results: Box::new([core::ReadResults { party: ALICE, after: 0 }]),
        questions: Box::new([(GRANDCHILD, 7)]),
    })]))
}
fn restored(store: &mut Store) -> Box<[Record]> {
    let mut found = List::uncapped();
    let mut after = None;
    loop {
        let events = fixtures::run(
            store,
            Request::Restore { owner: Token::new(7), range: census(), after, rows: 4, bytes: 65_536 },
        );
        let [Event::Page { items, next, .. }] = events.as_ref() else { panic!("census: {events:?}") };
        for item in items {
            let crate::restore::Item::Row(crate::Row::Jig(row)) = item else { panic!("joined row") };
            found.push(row.clone()).unwrap();
        }
        after = next.clone();
        if after.is_none() {
            return found.into_boxed();
        }
    }
}
#[test]
fn restart_census_joins_positions_results_and_question_lines_from_logs() {
    let mut store = child_store();
    fixtures::committed(
        &mut store,
        Box::new([
            fixtures::header(2),
            fixtures::save(Record::People(jig_domain_people::Stored::EndedRead {
                task: CHILD,
                party: ALICE,
                through: 0,
            })),
            fixtures::save(fixtures::result_index(ALICE, 1, GOAL)),
            fixtures::save(fixtures::ended_of(GOAL, tasks::Party::Person(ALICE))),
        ]),
    );
    let Record::Tasks(tasks::Stored::Live(mut questioner)) = fixtures::live_task(GRANDCHILD) else {
        panic!("live questioner");
    };
    questioner.question_lines = Box::new([tasks::QuestionLine { message: 5, line: 7, answer: None }]);
    fixtures::committed(
        &mut store,
        Box::new([
            fixtures::header(3),
            fixtures::save(Record::Tasks(tasks::Stored::Live(questioner))),
            fixtures::save(fixtures::question_line(GRANDCHILD, 7, 5)),
        ]),
    );
    let before = restored(&mut store);
    assert_eq!(before.len(), 4);
    for task in [CHILD, GOAL, GRANDCHILD] {
        move_history(&mut store, task);
    }
    let after = restored(&mut store);
    assert_eq!(before.len(), after.len());
    assert_eq!(before, after);
}
#[test]
fn fetch_bytes_cut_a_party_read_and_refuse_a_restart_page() {
    let mut store = child_store();
    fixtures::committed(
        &mut store,
        Box::new([fixtures::header(2), fixtures::save(fixtures::question_line(CHILD, 7, 1))]),
    );
    move_history(&mut store, CHILD);
    store.limits.fetch = 1;
    let range = fixtures::read(ALICE, reads::Scope::Task { task: CHILD });
    equal(&read(&mut store, range), &crate::reads::empty());
    let events = fixtures::run(
        &mut store,
        Request::Restore {
            owner: Token::new(7),
            range: Restore::Jig(Box::new([core::RestoreRange::Census(core::Census {
                results: Box::new([]),
                questions: Box::new([(CHILD, 7)]),
            })])),
            after: None,
            rows: 4,
            bytes: 65_536,
        },
    );
    let [Event::PageRefused { failure, .. }] = events.as_ref() else { panic!("refused fetch: {events:?}") };
    let crate::restore::Cause::RowPastPage { has: 1, .. } = failure.cause else { panic!("fetch bounds") };
}

#[test]
fn history_record_bytes_are_frozen_in_both_directions() {
    let rows: Box<[logs::record::Row]> = Box::new([
        (Box::from([1, 1, 6, 0, 0, 0, 0, 0, 0, 0, 7]), Some(Box::from([0, 3, 42]))),
        (Box::from([1, 1, 9, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 1]), None),
    ]);
    let bytes = include_bytes!("golden/history-record.bin");
    assert_eq!(logs::record::encode(&rows).unwrap().as_ref(), bytes);
    assert_eq!(logs::record::decode(bytes).unwrap(), rows);
    for prefix in (0..bytes.len()).filter(|&prefix| prefix != 22) {
        assert_eq!(logs::record::decode(&bytes[..prefix]), None, "short record {prefix}");
    }
    let mut unknown = bytes.to_vec();
    unknown[0] = 2;
    assert_eq!(logs::record::decode(&unknown), None);
    assert_eq!(logs::record::encode(&[]), None);
}
#[test]
fn log_places_ids_extents_and_census_names_round_trip() {
    for task in [0_u64, 1, (1 << 63_u32) - 1] {
        assert_eq!(logs::history(task), task.checked_mul(2));
        assert_eq!(logs::turns(task), task.checked_mul(2).unwrap().checked_add(1));
    }
    assert_eq!(logs::history(1 << 63), None);
    assert_eq!(logs::turns(u64::MAX), None);
    for record in [None, Some(0), Some(u64::MAX)] {
        let row = codec::component::HistoryLog { record };
        assert_eq!(codec::component::HistoryLog::decode(&row.encode().unwrap()), Ok(row));
        let row = codec::component::TurnLog { window: record };
        assert_eq!(codec::component::TurnLog::decode(&row.encode().unwrap()), Ok(row));
    }
    let row = codec::component::LogAttempt { first: u64::MAX };
    assert_eq!(codec::component::LogAttempt::decode(&row.encode().unwrap()), Ok(row));
    assert_eq!(logs::extent(&7_u64.to_be_bytes()), Some(7));
    assert_eq!(logs::extent_row(&logs::extent_key(2), &7_u64.to_be_bytes()), Some((2, 7)));
    assert_eq!(logs::extent_row(&[0, 10], &7_u64.to_be_bytes()), None);
    assert_eq!(logs::extent(&[0; 7]), None);
    for (key, name) in [
        (keys::Own::History { task: 7 }, "component.histories"),
        (keys::Own::TurnLog { task: 7 }, "component.turn-logs"),
        (keys::Own::LogAttempt { task: 7, attempt: 1 }, "component.log-attempts"),
    ] {
        let bytes = keys::encode_own(&key, 32).unwrap();
        assert_eq!(keys::decode_own(&bytes), Some(key));
        assert_eq!(crate::census::name(&bytes[..2], crate::Keyspaces::NONE), name);
    }
    assert_eq!(logs::extent_key(2).as_ref(), &[0, 9, 0, 0, 0, 0, 0, 0, 0, 2]);
    assert_eq!(crate::census::name(&logs::EXTENTS, crate::Keyspaces::NONE), "component.logs");
}

pub(super) fn page_signature(rows: &[reads::Row], next: Option<&reads::Position>) -> Box<str> {
    format!("{rows:?} {next:?}").into_boxed_str()
}

mod disk;

#[test]
fn every_generated_family_routes_only_its_history_keys() {
    let mut rng = skein_lib::Rng::new(71);
    let mut seen = Set::with_capacity(u32::try_from(keys::KEYSPACES.len()).unwrap());
    for number in 0_u32..8192 {
        for key in [
            Key::Core(fixtures::core_key(&mut rng)),
            Key::Tasks(fixtures::tasks_key(&mut rng)),
            Key::People(fixtures::people_key(&mut rng)),
            Key::Notes(fixtures::notes_key(&mut rng)),
            Key::Accounts(fixtures::accounts_key(&mut rng)),
            Key::People(jig_domain_people::Key::Deleted(number)),
            Key::People(jig_domain_people::Key::Deletion(number)),
            Key::People(jig_domain_people::Key::Public(number)),
            Key::People(jig_domain_people::Key::Acted(jig_domain_people::ActKey { connector: 1, key: [7; 16] })),
            Key::Accounts(jig_domain_accounts::Key::Facts { account: number }),
        ] {
            let expected = if let Key::Core(
                core::CoreKey::Terminal { task, .. }
                | core::CoreKey::EscalationDecision { task, .. }
                | core::CoreKey::ProposalDecision { proposer: jig_domain_people::Proposer::Task(task), .. },
            ) = &key
            {
                Some(*task)
            } else if let Key::Tasks(
                tasks::Key::Ended(task)
                | tasks::Key::History { task, .. }
                | tasks::Key::Milestone { task, .. }
                | tasks::Key::Activity { task, .. }
                | tasks::Key::Attempt { task, .. }
                | tasks::Key::Hold { task, .. },
            ) = &key
            {
                Some(*task)
            } else {
                None
            };
            let encoded = keys::encode(&key, fixtures::KEY_LIMIT).unwrap();
            assert_eq!(keys::logged(&encoded), expected, "{key:?}");
            key_family(&mut seen, &encoded);
        }
        for own in [
            fixtures::own(&mut rng),
            keys::Own::Wires,
            keys::Own::Opening("worker".into()),
            keys::Own::Layout { family: "family".into(), number: u16::try_from(number).unwrap(), id: 1 },
            keys::Own::Text([7; 32]),
            keys::Own::Job { owner: crate::Owner::Application, name: "job".into() },
        ] {
            let encoded = keys::encode_own(&own, fixtures::KEY_LIMIT).unwrap();
            assert_eq!(keys::logged(&encoded), None);
            key_family(&mut seen, &encoded);
        }
    }
    for key in [logs::extent_key(2), keys::installation(fixtures::KEY_LIMIT).unwrap()] {
        assert_eq!(keys::logged(&key), None);
        key_family(&mut seen, &key);
    }
    for &(prefix, name) in keys::KEYSPACES {
        assert!(seen.contains(prefix), "generated family {name}");
    }
    for key in [Box::from([]), Box::from([1, 1, 6]), Box::from([1, 1, 24]), keys::application(&[1, 6], 32).unwrap()] {
        assert_eq!(keys::logged(&key), None);
    }
}

fn key_family(seen: &mut Set<Box<[u8]>>, key: &[u8]) {
    let width = if key.first() == Some(&keys::COMPONENT) { 2 } else { 3 };
    seen.insert(Box::from(key.get(..width).unwrap())).unwrap();
}

#[test]
fn ended_erasure_pages_find_logged_records_by_their_summaries() {
    let mut store = child_store();
    let Record::Tasks(tasks::Stored::Ended(owner)) = got(&mut store).unwrap() else { panic!("ended owner") };
    fixtures::committed(
        &mut store,
        Box::new([
            fixtures::header(2),
            fixtures::save(Record::Tasks(tasks::Stored::Summary(Box::new(tasks::TaskSummary::of(&owner))))),
        ]),
    );
    store.limits.joins = 0;
    same(&mut store, &[CHILD], fixtures::read(ALICE, reads::Scope::EndedTasks));
}
