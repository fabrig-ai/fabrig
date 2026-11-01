//! Whole-task erasure keeps the ended owner until every sweep page is committed.
use super::*;
use crate::erase::{erase_task, erasing};
use crate::reads::Sweep;
use alloc::collections::{BTreeMap, BTreeSet};

const TASK: u64 = 1;
const PARTY: u64 = 1;
const OTHER_PARTY: u64 = 2;
const PARENT: u64 = 2;
const PROJECT: u32 = 1;
const RESULT: u64 = 7; // ended_report's result position.
const REPLY: u64 = 3;
const ACTIVITY: u64 = 4;
const REPLY_AT: Wall = Wall::from_nanos(11);
const ENDED_AT: tasks::Newest = tasks::Newest::of(1); // ended_report's ended_at.

fn fixture() -> (Core, Env<Limits>, Box<tasks::TaskRecord>) {
    let (core, limits) = core();
    let row = Box::new(fixtures::ended_report(TASK, tasks::Party::Person(PARTY)));
    (core, Env { now: Time::ZERO, wall: Wall::EPOCH, limits }, row)
}

fn begin(core: &mut Core, env: &Env<Limits>, row: Box<tasks::TaskRecord>) -> Queue<Request> {
    let mut work = Queue::uncapped();
    let mut out = Queue::uncapped();
    erase_task(core, env, &mut work, &mut out, row);
    assert!(work.is_empty());
    out
}

fn sweep_owner(out: &Queue<Request>) -> Token {
    for request in out {
        if let Request::Held(held) = request
            && let Held::Sweep { owner, sweep: Sweep::Task(TASK) } = held.as_ref()
        {
            return *owner;
        }
    }
    panic!("task sweep requested");
}

fn erased(out: &Queue<Request>) -> BTreeSet<Key> {
    let mut keys = BTreeSet::new();
    for request in out {
        if let Request::Write(Write::Erase(key)) = request {
            keys.insert(key.clone());
        }
    }
    keys
}

fn turn() -> tasks::ActivityLine {
    tasks::ActivityLine {
        task: TASK,
        position: ACTIVITY,
        at: REPLY_AT,
        line: tasks::Line::Turn { reply: REPLY, attempt: 1, turn: 1, read: None },
    }
}

fn sweep_rows() -> Box<[Record]> {
    Box::new([
        Record::Tasks(tasks::Stored::History(tasks::History {
            task: TASK,
            revision: 1,
            by: tasks::Party::Person(PARTY),
            change: tasks::Change::Cancelled,
            reason: Box::new([]),
            proposal: None,
            at: REPLY_AT,
            amendment: None,
        })),
        Record::Tasks(tasks::Stored::Milestone(tasks::Milestone {
            task: TASK,
            position: 1,
            phase: tasks::Phase::Waiting,
            at: REPLY_AT,
        })),
        Record::Tasks(tasks::Stored::Activity(turn())),
        Record::Core(CoreRecord::Terminal(TerminalRecord {
            task: TASK,
            attempt: 1,
            cumulative: 0,
            end: tasks::End::Parked,
        })),
        Record::Core(CoreRecord::EscalationDecision(EscalationDecisionRecord {
            project: PROJECT,
            requester: PARTY,
            task: TASK,
            revision: 1,
            by: PARTY,
            at: REPLY_AT,
            decision: people::EscalationDecision::Release,
        })),
        Record::People(people::Stored::EndedRead { task: TASK, party: PARTY, through: REPLY }),
        Record::People(people::Stored::EndedRead { task: TASK, party: OTHER_PARTY, through: REPLY }),
    ])
}

#[test]
fn an_erasure_drops_the_transcript_and_sweeps_the_tasks_rows_first() {
    let (mut core, env, row) = fixture();
    let mut out = begin(&mut core, &env, row);
    assert!(erasing(&core, TASK));
    assert!(!erasing(&core, PARENT));
    assert_eq!(core.erasures.capacity(), env.limits.authority.projects);
    let Some(Request::Write(Write::DropTranscript(task))) = out.pop() else { panic!("transcript drop first") };
    assert_eq!(task, TASK);
    let Some(Request::Held(held)) = out.pop() else { panic!("held sweep after transcript drop") };
    let Held::Sweep { sweep, .. } = *held else { panic!("task sweep") };
    assert_eq!(sweep, Sweep::Task(TASK));
    assert!(out.is_empty(), "the ended record survives the first decision");
}

#[test]
fn a_full_sweep_page_sweeps_again_and_an_empty_one_finishes() {
    let (mut core, env, row) = fixture();
    let first = begin(&mut core, &env, row);
    let owner = sweep_owner(&first);
    let Requests::Out(page) = step(&mut core, &env, Event::Swept { owner, rows: sweep_rows(), more: true });
    let expected = BTreeSet::from([
        Key::Tasks(tasks::Key::History { task: TASK, revision: 1 }),
        Key::Tasks(tasks::Key::Milestone { task: TASK, position: 1 }),
        Key::Tasks(tasks::Key::Activity { task: TASK, position: ACTIVITY }),
        Key::Core(CoreKey::Terminal { task: TASK, attempt: 1 }),
        Key::Core(CoreKey::EscalationDecision { task: TASK, revision: 1 }),
        Key::People(people::Key::EndedRead { task: TASK, party: PARTY }),
        Key::People(people::Key::EndedRead { task: TASK, party: OTHER_PARTY }),
        Key::Tasks(tasks::Key::Update {
            party: PARTY,
            at: tasks::Newest::of(11),
            task: tasks::Newest::of(TASK),
            object: tasks::UpdateObject::Reply {
                message: tasks::Newest::of(REPLY),
                activity: tasks::Newest::of(ACTIVITY),
            },
        }),
    ]);
    assert_eq!(erased(&page), expected);
    assert_eq!(sweep_owner(&page), owner);
    assert!(erasing(&core, TASK));
    let Requests::Out(last) = step(&mut core, &env, Event::Swept { owner, rows: Box::new([]), more: false });
    assert!(erased(&last).contains(&Key::Tasks(tasks::Key::Ended(TASK))));
    assert!(!erasing(&core, TASK));
    for request in &last {
        if let Request::Held(held) = request
            && let Held::Sweep { .. } = held.as_ref()
        {
            panic!("the final page requests no further sweep");
        }
    }
}

fn person_keys(listed: bool) -> BTreeSet<Key> {
    let mut keys = BTreeSet::from([
        Key::Tasks(tasks::Key::Ended(TASK)),
        Key::Tasks(tasks::Key::EndedByParty { party: PARTY, label: 0, ended_at: ENDED_AT, task: TASK }),
        Key::Tasks(tasks::Key::Result { party: PARTY, position: RESULT }),
        Key::Tasks(tasks::Key::Update {
            party: PARTY,
            at: ENDED_AT,
            task: tasks::Newest::of(TASK),
            object: tasks::UpdateObject::Result,
        }),
        Key::Tasks(tasks::Key::Ledger(tasks::Funder::Task(TASK))),
        Key::Tasks(tasks::Key::ClosedLedger(tasks::Funder::Task(TASK))),
        Key::Tasks(tasks::Key::Stub(TASK)),
    ]);
    if listed {
        keys.insert(Key::Tasks(tasks::Key::EndedByProject {
            project: PROJECT,
            label: 0,
            ended_at: ENDED_AT,
            task: TASK,
        }));
    }
    keys
}

#[test]
fn the_last_step_erases_the_record_and_every_row_it_names() {
    for listed in [false, true] {
        let (mut core, env, mut row) = fixture();
        row.flags.listed = listed;
        row.funder = tasks::Funder::Recurring { project: PROJECT, task: TASK, period: 1 };
        let first = begin(&mut core, &env, row);
        let owner = sweep_owner(&first);
        let mut work = Queue::uncapped();
        let Requests::Out(out) = crate::erase::swept(&mut core, &env, &mut work, owner, Box::new([]), false);
        assert_eq!(erased(&out), person_keys(listed));
        let Some(Request::Write(Write::Erase(key))) = out.iter().last() else { panic!("ended row erased last") };
        assert_eq!(*key, Key::Tasks(tasks::Key::Ended(TASK)));
        let Some(Event::People(people::Event::TaskErased { task, project })) = work.pop() else {
            panic!("people's task erasure follows the final write");
        };
        assert_eq!((task, project), (TASK, PROJECT));
        assert!(work.is_empty());
        assert!(!erasing(&core, TASK));
    }
}

#[test]
fn a_task_requesters_ended_delegate_edge_is_erased() {
    let (mut core, env, mut row) = fixture();
    row.requester = tasks::Party::Task(PARENT);
    let first = begin(&mut core, &env, row);
    let owner = sweep_owner(&first);
    let Requests::Out(out) = step(&mut core, &env, Event::Swept { owner, rows: Box::new([]), more: false });
    let expected = BTreeSet::from([
        Key::Tasks(tasks::Key::Ended(TASK)),
        Key::Tasks(tasks::Key::EndedDelegate { requester: PARENT, task: TASK }),
        Key::Tasks(tasks::Key::Ledger(tasks::Funder::Task(TASK))),
        Key::Tasks(tasks::Key::ClosedLedger(tasks::Funder::Task(TASK))),
        Key::Tasks(tasks::Key::Stub(TASK)),
        Key::People(people::Key::EndedRead { task: TASK, party: PARTY }),
    ]);
    assert_eq!(erased(&out), expected);
}

#[test]
fn a_reply_lines_update_is_erased_with_it() {
    let notified = tasks::ActivityLine {
        line: tasks::Line::Notified {
            recipient: tasks::Party::Person(OTHER_PARTY),
            word: tasks::Word {
                number: REPLY,
                from: tasks::Party::Task(TASK),
                kind: tasks::MessageKind::Words,
                words: b"notice".as_slice().into(),
                at: Wall::EPOCH,
                hits: 1,
                eligible: true,
            },
        },
        ..turn()
    };
    for (line, party) in [(turn(), PARTY), (notified, OTHER_PARTY)] {
        let (mut core, env, row) = fixture();
        let first = begin(&mut core, &env, row);
        let owner = sweep_owner(&first);
        let rows = Box::new([Record::Tasks(tasks::Stored::Activity(line))]);
        let Requests::Out(out) = step(&mut core, &env, Event::Swept { owner, rows, more: true });
        let expected = BTreeSet::from([
            Key::Tasks(tasks::Key::Activity { task: TASK, position: ACTIVITY }),
            Key::Tasks(tasks::Key::Update {
                party,
                at: tasks::Newest::of(11),
                task: tasks::Newest::of(TASK),
                object: tasks::UpdateObject::Reply {
                    message: tasks::Newest::of(REPLY),
                    activity: tasks::Newest::of(ACTIVITY),
                },
            }),
        ]);
        assert_eq!(erased(&out), expected);
    }
}

#[test]
fn a_stub_named_by_live_work_stays() {
    let (mut core, env, row) = fixture();
    let mut child = Queue::uncapped();
    tasks::step(
        &mut core.tasks,
        &Env { now: env.now, wall: env.wall, limits: env.limits.tasks },
        tasks::Event::Restore {
            record: tasks::Stored::Stub(tasks::Stub {
                task: TASK,
                phase: tasks::Status::Done,
                result: Token::new(TASK),
            }),
        },
        &mut child,
    );
    let first = begin(&mut core, &env, row);
    let owner = sweep_owner(&first);
    let mut work = Queue::uncapped();
    let Requests::Out(out) = crate::erase::swept(&mut core, &env, &mut work, owner, Box::new([]), false);
    let mut expected = person_keys(false);
    expected.remove(&Key::Tasks(tasks::Key::Stub(TASK)));
    assert_eq!(erased(&out), expected);
    assert!(core.tasks.stub(TASK).is_some());
}

#[test]
fn the_last_page_drops_the_core_and_people_projections() {
    use crate::routing::result_reads::{ReadFor, ResultRead};
    const READ: Token = Token::new(500);
    const MARK: Token = Token::new(501);
    const CLOSE: Token = Token::new(502);
    let (mut core, env, row) = fixture();
    core.people.remember_result(
        &env.limits.people,
        PARTY,
        people::ResultRef {
            task: TASK,
            project: PROJECT,
            at: row.ended_at.unwrap(),
            position: RESULT,
            forgettable: None,
        },
    );
    core.cache_archived_chat(&env.limits, &row, None);
    core.census_position = Some((TASK, PARTY, REPLY));
    core.question_activity.insert((TASK, ACTIVITY), turn()).unwrap();
    core.ended_marks
        .insert(
            MARK,
            ended_marks::Loaded { task: TASK, party: PARTY, project: PROJECT, last_reply: REPLY, owner: None },
        )
        .unwrap();
    core.result_reads
        .insert(
            CLOSE,
            ResultRead {
                why: ReadFor::DeploymentClose { request: CLOSE, project: PROJECT },
                ids: Box::new([TASK]),
                at: 0,
                results: List::with_capacity(0),
            },
        )
        .unwrap();
    core.result_reads
        .insert(
            READ,
            ResultRead {
                why: ReadFor::Preparation { task: TASK, last_message: REPLY },
                ids: Box::new([PARENT]),
                at: 0,
                results: List::with_capacity(0),
            },
        )
        .unwrap();
    let first = begin(&mut core, &env, row);
    let owner = sweep_owner(&first);
    let _out = step(&mut core, &env, Event::Swept { owner, rows: Box::new([]), more: false });
    assert!(core.archived_chats.is_empty());
    assert!(core.census_position.is_none());
    assert!(core.question_activity.is_empty());
    assert!(core.ended_marks.is_empty());
    assert!(core.result_reads.is_empty());
    assert!(core.people.cached_results(PARTY).unwrap().is_empty());
    assert!(core.people.cached_updates(&env.limits.people, PARTY).unwrap().is_empty());
}

fn commit(store: &mut BTreeMap<Key, Record>, out: &Queue<Request>) {
    for key in erased(out) {
        store.remove(&key);
    }
}

#[test]
fn an_erasure_begun_again_from_the_same_record_finishes() {
    let (mut core, env, row) = fixture();
    let mut store = BTreeMap::new();
    for record in sweep_rows() {
        store.insert(record.key(), record);
    }
    store.insert(Key::Tasks(tasks::Key::Ended(TASK)), Record::Tasks(tasks::Stored::Ended(row.clone())));
    let first = begin(&mut core, &env, row.clone());
    let owner = sweep_owner(&first);
    let rows = Box::new([Record::Tasks(tasks::Stored::Activity(turn()))]);
    let Requests::Out(page) = step(&mut core, &env, Event::Swept { owner, rows, more: true });
    commit(&mut store, &page);
    assert!(store.contains_key(&Key::Tasks(tasks::Key::Ended(TASK))));

    // A fresh core loses the volatile owner; only the durable ended record resumes it.
    let (mut restarted, _, _) = fixture();
    let first = begin(&mut restarted, &env, row);
    let Some(Request::Write(Write::DropTranscript(task))) = first.iter().next() else {
        panic!("restart drops the transcript idempotently");
    };
    assert_eq!(*task, TASK);
    let owner = sweep_owner(&first);
    let mut rows = List::with_capacity(u32::try_from(store.len()).unwrap());
    for row in store.values() {
        if Sweep::Task(TASK).admits(&row.key()) {
            rows.push(row.clone()).unwrap();
        }
    }
    let Requests::Out(last) = step(&mut restarted, &env, Event::Swept { owner, rows: rows.into_boxed(), more: false });
    commit(&mut store, &last);
    assert!(store.is_empty());
    assert!(!erasing(&restarted, TASK));
}
