//! Whole-task erasure decisions and store-page continuations.
use crate::{
    routing::task::erase::{Select, scan},
    tests::*,
};

fn env(limits: &Limits) -> Env<Limits> {
    Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits }
}

fn start(core: &mut Core, limits: &Limits, project: u32) -> Queue<Request> {
    let mut out = Queue::uncapped();
    scan(core, &env(limits), &mut out, Select::Project(project));
    out
}

fn load(out: &Queue<Request>) -> (Token, crate::reads::Range) {
    let mut found = None;
    for request in out {
        if let Request::Held(held) = request
            && let Held::ReadLoad { load, range } = held.as_ref()
        {
            assert!(found.is_none(), "one next page");
            found = Some((*load, range.as_ref().clone()));
        }
    }
    found.expect("one next page")
}

fn page(
    core: &mut Core,
    limits: &Limits,
    previous: &Queue<Request>,
    values: Box<[Record]>,
    next: Option<Key>,
) -> Queue<Request> {
    let (load, _) = load(previous);
    core.reclaim();
    let mut rows = List::with_capacity(u32::try_from(values.len()).unwrap());
    for value in values {
        rows.push(stored_row(value, None)).unwrap();
    }
    let next = match next {
        Some(key) => Some(crate::reads::Position::Owner(key)),
        None => None,
    };
    crate::reads::loaded(
        core,
        &env(limits),
        &mut Queue::uncapped(),
        load,
        crate::reads::Loaded::Page { rows: rows.into_boxed(), next },
    )
}

fn erases(out: &Queue<Request>) -> Box<[Key]> {
    let mut keys = List::with_capacity(out.len());
    for request in out {
        if let Request::Write(Write::Erase(key)) = request {
            keys.push(key.clone()).unwrap();
        }
    }
    keys.into_boxed()
}

fn unnamed(out: &Queue<Request>) -> Box<[(u16, u64)]> {
    let mut asks = List::with_capacity(out.len());
    for request in out {
        if let Request::Ask { connector, ask: Ask::Effect(connector::Ask::Unname { task }) } = request {
            asks.push((*connector, *task)).unwrap();
        }
    }
    asks.into_boxed()
}

fn ended(core: &mut Core, limits: &Limits, record: tasks::TaskRecord) -> Queue<Request> {
    let out = start(core, limits, record.project);
    let (_, range) = load(&out);
    assert_eq!(range.scope, crate::reads::Scope::EndedTasks);
    page(core, limits, &out, Box::new([Record::Tasks(tasks::Stored::Ended(Box::new(record)))]), None)
}

fn finish_rows(core: &mut Core, limits: &Limits, rows: &Queue<Request>) -> (Queue<Request>, Queue<Request>) {
    let cleanup = page(core, limits, rows, Box::new([]), None);
    for key in erases(&cleanup) {
        assert_ne!(key, Key::Tasks(tasks::Key::Ended(9)));
    }
    let record = page(core, limits, &cleanup, Box::new([]), None);
    (cleanup, record)
}

#[test]
fn every_connector_is_told_in_the_decision_that_erases_the_record() {
    let (mut core, limits) = make_tree();
    let record = fixtures::ended_report(9, tasks::Party::Person(1));
    let out = ended(&mut core, &limits, record);
    assert!(unnamed(&out).is_empty());
    let rows = page(&mut core, &limits, &out, Box::new([]), None);
    assert!(unnamed(&rows).is_empty());
    let (cleanup, record_out) = finish_rows(&mut core, &limits, &rows);
    assert!(unnamed(&cleanup).is_empty());
    assert_eq!(erases(&record_out).as_ref(), &[Key::Tasks(tasks::Key::Ended(9))]);
    assert_eq!(unnamed(&record_out).as_ref(), &[(1, 9), (2, 9)]);
}

#[test]
fn an_ended_task_is_erased_whole_and_its_record_last() {
    let (mut core, limits) = make_tree();
    let mut record = fixtures::ended_report(9, tasks::Party::Person(1));
    record.flags.listed = true;
    let out = ended(&mut core, &limits, record.clone());
    assert_eq!(load(&out).1.scope, crate::reads::Scope::TaskRows { task: 9 });
    let values = Box::new([
        Record::Tasks(tasks::Stored::History(tasks::History {
            task: 9,
            revision: 1,
            by: tasks::Party::Person(1),
            change: tasks::Change::Cancelled,
            reason: Box::new([]),
            proposal: None,
            at: Wall::EPOCH,
            amendment: None,
        })),
        Record::Tasks(tasks::Stored::Milestone(tasks::Milestone {
            task: 9,
            position: 1,
            phase: record.phase.clone(),
            at: Wall::EPOCH,
        })),
        Record::Core(CoreRecord::Terminal(TerminalRecord {
            task: 9,
            attempt: 1,
            cumulative: 0,
            end: tasks::End::Parked,
        })),
        Record::People(people::Stored::EndedRead { task: 9, party: 1, through: 2 }),
    ]);
    let mut keys = List::with_capacity(4);
    for value in values.as_ref() {
        keys.push(value.key()).unwrap();
    }
    let keys = keys.into_boxed();
    let rows = page(&mut core, &limits, &out, values, None);
    assert_eq!(erases(&rows), keys);
    let (cleanup, record_out) = finish_rows(&mut core, &limits, &rows);
    let expected = crate::reads::forget::record_keys(&crate::reads::forget::Owner::of_row(&record).unwrap());
    for key in &expected {
        if *key != tasks::Key::Ended(9) {
            assert!(erases(&cleanup).contains(&Key::Tasks(*key)));
        }
    }
    let mut dropped = false;
    for request in &cleanup {
        if let Request::Write(Write::DropTranscript { task: 9 }) = request {
            dropped = true;
        }
    }
    assert!(dropped);
    assert_eq!(erases(&record_out).as_ref(), &[Key::Tasks(tasks::Key::Ended(9))]);
    assert_eq!(load(&record_out).1.scope, crate::reads::Scope::EndedTasks);
}

#[test]
fn a_reply_lines_update_goes_with_it() {
    for notified in [false, true] {
        let (mut core, limits) = make_tree();
        let record = fixtures::ended_report(9, tasks::Party::Person(1));
        let out = ended(&mut core, &limits, record);
        let at = Wall::from_nanos(77);
        let line = if notified {
            tasks::Line::Notified {
                recipient: tasks::Party::Person(2),
                word: tasks::Word {
                    number: 42,
                    from: tasks::Party::Task(9),
                    kind: tasks::MessageKind::Words,
                    words: Box::new([]),
                    at,
                    hits: 1,
                    eligible: false,
                },
            }
        } else {
            tasks::Line::Turn { reply: 42, attempt: 1, turn: 1, read: None }
        };
        let value = Record::Tasks(tasks::Stored::Activity(tasks::ActivityLine { task: 9, position: 3, at, line }));
        let rows = page(&mut core, &limits, &out, Box::new([value.clone()]), None);
        let update = tasks::Update {
            party: if notified { 2 } else { 1 },
            at,
            task: 9,
            object: tasks::UpdateObject::Reply { message: tasks::Newest::of(42), activity: tasks::Newest::of(3) },
        };
        assert_eq!(erases(&rows).as_ref(), &[Key::Tasks(update.key()), value.key()]);
    }
}

#[test]
fn the_walk_erases_only_the_selected_projects_tasks() {
    let (mut core, limits) = make_tree();
    let mut other = fixtures::ended_report(8, tasks::Party::Person(1));
    other.project = 2;
    let selected = fixtures::ended_report(9, tasks::Party::Task(1));
    let out = start(&mut core, &limits, 1);
    let skipped = page(
        &mut core,
        &limits,
        &out,
        Box::new([Record::Tasks(tasks::Stored::Ended(Box::new(other)))]),
        Some(Key::Tasks(tasks::Key::Ended(8))),
    );
    assert!(erases(&skipped).is_empty());
    assert_eq!(load(&skipped).1.after, Some(crate::reads::Position::Owner(Key::Tasks(tasks::Key::Ended(8)))));
    let selected =
        page(&mut core, &limits, &skipped, Box::new([Record::Tasks(tasks::Stored::Ended(Box::new(selected)))]), None);
    let rows = page(&mut core, &limits, &selected, Box::new([]), None);
    let (cleanup, record) = finish_rows(&mut core, &limits, &rows);
    assert_eq!(
        erases(&cleanup).as_ref(),
        &[
            Key::Tasks(tasks::Key::EndedDelegate { requester: 1, task: 9 }),
            Key::Tasks(tasks::Key::Delegate { requester: 1, task: 9 }),
        ]
    );
    assert_eq!(erases(&record).as_ref(), &[Key::Tasks(tasks::Key::Ended(9))]);
}

#[test]
fn a_walk_with_nothing_selected_ends_at_once() {
    let (mut core, limits) = make_tree();
    let out = start(&mut core, &limits, 2);
    let other = fixtures::ended_report(9, tasks::Party::Person(1));
    let out = page(&mut core, &limits, &out, Box::new([Record::Tasks(tasks::Stored::Ended(Box::new(other)))]), None);
    assert!(erases(&out).is_empty());
    assert!(out.is_empty(), "the empty walk finishes in its page's decision");
    let out = start(&mut core, &limits, 1);
    assert_eq!(load(&out).1.scope, crate::reads::Scope::EndedTasks, "completion releases the next walk");
}

#[test]
fn a_busy_page_is_asked_again() {
    let (mut core, limits) = make_tree();
    let out = start(&mut core, &limits, 1);
    let (token, range) = load(&out);
    let out = crate::reads::loaded(&mut core, &env(&limits), &mut Queue::uncapped(), token, crate::reads::Loaded::Busy);
    let (new, retried) = load(&out);
    assert_ne!(new, token);
    assert_eq!(retried, range);
    assert!(erases(&out).is_empty());
    let mut row = fixtures::ended_report(9, tasks::Party::Person(1));
    row.project = 1;
    let out = page(&mut core, &limits, &out, Box::new([Record::Tasks(tasks::Stored::Ended(Box::new(row)))]), None);
    let (token, range) = load(&out);
    let out = crate::reads::loaded(&mut core, &env(&limits), &mut Queue::uncapped(), token, crate::reads::Loaded::Busy);
    assert_eq!(load(&out).1, range, "a busy task page also retries");
}

#[test]
fn each_page_is_its_own_decision_within_the_read_bound() {
    let (mut core, mut limits) = make_tree();
    limits.documents.rows = 1;
    let record = fixtures::ended_report(9, tasks::Party::Person(1));
    let mut out = ended(&mut core, &limits, record);
    for position in 1..=3 {
        let key = Key::Tasks(tasks::Key::Activity { task: 9, position });
        let row = Record::Tasks(tasks::Stored::Activity(tasks::ActivityLine {
            task: 9,
            position,
            at: Wall::EPOCH,
            line: tasks::Line::Turn { reply: position, attempt: 1, turn: 1, read: None },
        }));
        let (_, range) = load(&out);
        assert_eq!(range.rows, 1);
        out = page(&mut core, &limits, &out, Box::new([row]), Some(key.clone()));
        assert_eq!(erases(&out).len(), 2, "at most one row erase and one reply update per admitted row");
        assert_eq!(out.len(), 3, "two erases and the next held page share one decision");
        assert_eq!(load(&out).1.after, Some(crate::reads::Position::Owner(key)));
    }
}

#[test]
fn a_second_selection_waits_for_the_active_walk() {
    let (mut core, limits) = make_tree();
    let first = start(&mut core, &limits, 1);
    assert!(start(&mut core, &limits, 2).is_empty());
    let second = page(&mut core, &limits, &first, Box::new([]), None);
    assert_eq!(load(&second).1.scope, crate::reads::Scope::EndedTasks);
    let ended = page(&mut core, &limits, &second, Box::new([]), None);
    assert!(ended.is_empty());
}

#[test]
fn an_erasure_can_load_and_retry_while_client_read_slots_are_full() {
    let (mut core, limits) = watch_core();
    for number in 0_u64..u64::from(limits.load_slots) {
        let watcher = Token::new(30_u64.checked_add(number).unwrap());
        let task = 90_u64.checked_add(number).unwrap();
        let out = watch_outputs(
            &mut core,
            &limits,
            Event::Watch { watcher, sign_in: 10, subject: Watched::Task { task }, window: window() },
        );
        let Request::Held(held) = &out[0] else {
            panic!("client load");
        };
        let Held::ReadLoad { .. } = held.as_ref() else {
            panic!("retained task load");
        };
    }
    let out = start(&mut core, &limits, 1);
    let (token, range) = load(&out);
    let retry =
        crate::reads::loaded(&mut core, &env(&limits), &mut Queue::uncapped(), token, crate::reads::Loaded::Busy);
    assert_eq!(load(&retry).1, range);
}
