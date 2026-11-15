//! Delayed history reads and commit ordering at file-operation steps.
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "ordinary Rust disk step tests inspect requests and complete answers"
)]
use super::*;
use crate::{Below, Io};
use fixtures::disk::Disk;
use skein_io::file;

fn setup() -> (Disk, Store) {
    let mut disk = Disk::new(&[]);
    let mut store = disk.store();
    let opened = disk.open(&mut store);
    let [Event::Opened { .. }, Event::Census { .. }] = opened.as_ref() else { panic!("opened: {opened:?}") };
    for (task, requester) in [(CHILD, tasks::Party::Task(GOAL)), (GOAL, tasks::Party::Person(ALICE))] {
        let number = store.stores.store.last().checked_add(1).unwrap();
        let events = disk.request(
            &mut store,
            fixtures::commit(
                number,
                Box::new([
                    fixtures::header(number),
                    fixtures::save(fixtures::ended_of(task, requester)),
                    fixtures::save(fixtures::terminal(task, 1)),
                    fixtures::save(fixtures::history(task, 1)),
                ]),
            ),
        );
        let [Event::Committed { .. }] = events.as_ref() else { panic!("committed: {events:?}") };
    }
    (disk, store)
}
fn commit_ops(disk: &mut Disk, store: &mut Store, ops: Box<[skein_kv::Op]>) {
    let number = store.stores.store.last().checked_add(1).unwrap();
    store.phase = Phase::Ready { next: number.checked_add(1).unwrap(), format_due: false };
    let mut above = Queue::uncapped();
    let mut below = fixtures::below();
    crate::commits::submit(store, &fixtures::env(), number, ops, &mut above, &mut below);
    disk.drive(&mut below, |io, below| crate::up(store, &fixtures::env(), io, &mut above, below));
    let events = fixtures::events(&mut above);
    let [Event::Committed { .. }] = events.as_ref() else { panic!("committed: {events:?}") };
}
fn moved(disk: &mut Disk, store: &mut Store, task: u64) {
    let ops = move_operations(store, task);
    commit_ops(disk, store, ops);
}
fn begin(store: &mut Store, range: reads::Range) -> (Queue<Event>, Below) {
    let mut above = Queue::uncapped();
    let mut below = fixtures::below();
    crate::down(store, &fixtures::env(), Request::Read { owner: Token::new(7), range }, &mut above, &mut below);
    assert!(above.is_empty(), "a history fetch answers later");
    assert!(!below.store.is_empty());
    (above, below)
}
fn drive(disk: &mut Disk, store: &mut Store, above: &mut Queue<Event>, below: &mut Below) -> Vec<u64> {
    let mut starts = Vec::new();
    for _ in 0_u32..256 {
        if let Some(request) = below.store.pop() {
            if let file::Request::ReadAt { offset, .. } = &request {
                starts.push(*offset);
            }
            let arrived = disk.file(0, request);
            crate::up(store, &fixtures::env(), Io::Store(arrived), above, below);
        } else {
            return starts;
        }
    }
    panic!("bounded history fetch");
}
fn answer(events: Box<[Event]>) -> reads::Loaded {
    let mut events = events.into_iter();
    let Some(Event::Read { result, .. }) = events.next() else { panic!("one read") };
    assert!(events.next().is_none());
    result
}
#[test]
fn dropping_histories_erases_extents_places_and_durable_files() {
    let (mut disk, mut store) = setup();
    moved(&mut disk, &mut store, CHILD);
    moved(&mut disk, &mut store, GOAL);
    let files: Vec<_> = disk
        .tree()
        .into_iter()
        .filter(|(path, kind, _)| path.windows(5).any(|part| part == b"logs/") && *kind == skein_io::kernel::Kind::File)
        .map(|(path, _, _)| path)
        .collect();
    assert_eq!(files.len(), 2, "the helper appended both logs through skein-kv");
    for task in [CHILD, GOAL] {
        let extent = logs::extent_key(logs::history(task).unwrap());
        let place = keys::encode_own(&keys::Own::History { task }, store.limits.store.key).unwrap();
        assert!(fixtures::get_kv(&mut store, &extent).is_some());
        assert!(fixtures::get_kv(&mut store, &place).is_some());
    }
    let number = store.stores.store.last().checked_add(1).unwrap();
    let events = disk.request(
        &mut store,
        fixtures::commit(
            number,
            Box::new([
                fixtures::header(number),
                crate::Write::DropHistory { task: CHILD },
                crate::Write::DropHistory { task: GOAL },
            ]),
        ),
    );
    let [Event::Committed { number: found, .. }] = events.as_ref() else { panic!("drop committed: {events:?}") };
    assert_eq!(*found, number);
    for task in [CHILD, GOAL] {
        let extent = logs::extent_key(logs::history(task).unwrap());
        let place = keys::encode_own(&keys::Own::History { task }, store.limits.store.key).unwrap();
        assert_eq!(fixtures::get_kv(&mut store, &extent), None);
        assert_eq!(fixtures::get_kv(&mut store, &place), None);
    }
    let tree = disk.tree();
    for path in files {
        assert!(!tree.iter().any(|(standing, _, _)| standing == &path), "durable drop removed the log file");
    }
}

#[test]
fn two_tasks_missed_in_one_run_are_fetched_before_one_answer() {
    let (mut disk, mut store) = setup();
    let range = fixtures::read(
        ALICE,
        reads::Scope::ErasureKeys {
            keys: Box::new([Key::Tasks(tasks::Key::Ended(CHILD)), Key::Tasks(tasks::Key::Ended(GOAL))]),
        },
    );
    let before = answer(disk.request(&mut store, Request::Read { owner: Token::new(7), range: range.clone() }));
    moved(&mut disk, &mut store, CHILD);
    moved(&mut disk, &mut store, GOAL);
    let (mut above, mut below) = begin(&mut store, range);
    let starts = drive(&mut disk, &mut store, &mut above, &mut below);
    assert_eq!(starts.iter().filter(|&&at| at == 0).count(), 4);
    equal(&before, &answer(fixtures::events(&mut above)));
}
#[test]
fn a_fetched_owner_discovers_another_tasks_history() {
    let (mut disk, mut store) = setup();
    let range = fixtures::read(ALICE, reads::Scope::Task { task: CHILD });
    let before = answer(disk.request(&mut store, Request::Read { owner: Token::new(7), range: range.clone() }));
    moved(&mut disk, &mut store, CHILD);
    moved(&mut disk, &mut store, GOAL);
    let (mut above, mut below) = begin(&mut store, range);
    let starts = drive(&mut disk, &mut store, &mut above, &mut below);
    assert!(
        starts.iter().filter(|&&at| at == 0).count() >= 6,
        "ended owner, new terminal interval, and new ancestor task"
    );
    equal(&before, &answer(fixtures::events(&mut above)));
}
#[test]
fn an_ended_frame_is_read_directly_then_new_intervals_of_that_task_are_fetched() {
    let (mut disk, mut store) = setup();
    let number = store.stores.store.last().checked_add(1).unwrap();
    let events = disk.request(
        &mut store,
        fixtures::commit(
            number,
            Box::new([fixtures::header(number), fixtures::save(fixtures::notified_line(CHILD, 7, ALICE))]),
        ),
    );
    let [Event::Committed { .. }] = events.as_ref() else { panic!("reply saved: {events:?}") };
    let reply = fixtures::read(ALICE, reads::Scope::Reply { task: CHILD, activity: 7 });
    let reply_before = answer(disk.request(&mut store, Request::Read { owner: Token::new(7), range: reply.clone() }));
    let reads::Loaded::Page { rows, .. } = &reply_before else { panic!("reply page") };
    assert_eq!(rows.len(), 1);
    let range = fixtures::read(ALICE, reads::Scope::Task { task: CHILD });
    let before = answer(disk.request(&mut store, Request::Read { owner: Token::new(7), range: range.clone() }));
    let key =
        keys::encode(&Key::Tasks(tasks::Key::History { task: CHILD, revision: 1 }), store.limits.store.key).unwrap();
    let value = fixtures::get_kv(&mut store, &key).unwrap();
    let ops = append_operations(&mut store, CHILD, Box::new([(key, Some(value))]), true);
    commit_ops(&mut disk, &mut store, ops);
    let at = store.stores.store.log_end(logs::history(CHILD).unwrap());
    assert!(at > 0);
    moved(&mut disk, &mut store, CHILD);
    let (mut above, mut below) = begin(&mut store, range);
    let starts = drive(&mut disk, &mut store, &mut above, &mut below);
    assert_eq!(starts.first(), Some(&at));
    assert!(starts.contains(&0), "the terminal's new interval scans from the beginning");
    equal(&before, &answer(fixtures::events(&mut above)));
    let (mut above, mut below) = begin(&mut store, reply);
    let starts = drive(&mut disk, &mut store, &mut above, &mut below);
    assert_eq!(starts.iter().filter(|&&offset| offset == 0).count(), 2);
    equal(&reply_before, &answer(fixtures::events(&mut above)));
}
#[test]
fn a_commit_asked_during_a_fetch_answers_after_the_read() {
    let (mut disk, mut store) = setup();
    let range = fixtures::read(ALICE, reads::Scope::Task { task: CHILD });
    let before = answer(disk.request(&mut store, Request::Read { owner: Token::new(7), range: range.clone() }));
    moved(&mut disk, &mut store, CHILD);
    moved(&mut disk, &mut store, GOAL);
    let (mut above, mut below) = begin(&mut store, range);
    let number = store.stores.store.last().checked_add(1).unwrap();
    crate::down(
        &mut store,
        &fixtures::env(),
        fixtures::commit(
            number,
            Box::new([fixtures::header(number), crate::Write::Erase(Key::Tasks(tasks::Key::Ended(GOAL)))]),
        ),
        &mut above,
        &mut below,
    );
    assert!(above.is_empty());
    drive(&mut disk, &mut store, &mut above, &mut below);
    let mut events = fixtures::events(&mut above).into_iter();
    let Some(Event::Read { result, .. }) = events.next() else { panic!("read first") };
    equal(&before, &result);
    let Some(Event::Committed { number: found, .. }) = events.next() else { panic!("commit after read") };
    assert_eq!(found, number);
    assert!(events.next().is_none());
}
#[test]
fn an_unread_history_frame_fails_the_read_as_undecodable() {
    let (mut disk, mut store) = setup();
    moved(&mut disk, &mut store, CHILD);
    let (mut above, mut below) = begin(&mut store, fixtures::read(ALICE, reads::Scope::Task { task: CHILD }));
    let mut damaged = false;
    for _ in 0_u32..64 {
        let Some(request) = below.store.pop() else { break };
        let mut arrived = disk.file(0, request);
        if let file::Event::Read { bytes, .. } = &mut arrived.event {
            bytes.fill(0);
            damaged = true;
        }
        crate::up(&mut store, &fixtures::env(), Io::Store(arrived), &mut above, &mut below);
    }
    assert!(damaged);
    assert_eq!(store.phase, Phase::Stopped);
    let events = fixtures::events(&mut above);
    assert!(
        if let [Event::Corrupt { owner }] = events.as_ref() { *owner == Token::new(7) } else { false },
        "{events:?}"
    );
}
