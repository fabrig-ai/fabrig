//! Project deletion handoffs cross the root and store without deciding deletion.
use jig_domain_core as core;
use jig_test_domain as root;
use jig_test_store::{StoreOut, Up};
use jig_test_store_files::Store;
use skein_lib::{Env, Journal, Queue, Wall};

fn route(request: core::Request) -> (Journal<root::Write, root::Delivery>, Queue<core::Event>) {
    let limits = jig_core_world::world::limits();
    let env = Env { now: skein_lib::Time::ZERO, wall: Wall::EPOCH, limits };
    let mut domain = root::Domain::new(jig_core_world::world::config(711), &env.limits);
    let mut journal = Journal::uncapped(1, 0);
    let mut decision = journal.decide();
    let mut requests = Queue::uncapped();
    requests.push(request);
    let replies = root::route_requests(&mut domain, &env, &mut decision, core::Requests::Out(requests));
    journal.accept(decision);
    (journal, replies)
}

#[test]
fn a_release_ask_is_answered_at_once_by_the_test_root() {
    let (mut journal, mut replies) =
        route(core::Request::Ask { connector: 1, ask: core::Ask::ReleaseProject { project: 1 } });
    let Some(core::Event::ProjectReleased { project: 1, connector: 1 }) = replies.pop() else {
        panic!("release answered synchronously with project and connector");
    };
    assert!(replies.is_empty());
    assert!(journal.commit().is_none());
    let mut held = Queue::uncapped();
    let _released = journal.release(&mut held);
    assert!(held.is_empty());
}

#[test]
fn a_dropped_transcript_reaches_the_store() {
    const TASK: u64 = 21;
    let mut store = Store::new(&jig_core_world::world::limits(), &[]);
    store.submit(
        1,
        Box::new([
            header(1),
            root::Write::Save(root::Record::Core(core::Record::Core(core::CoreRecord::Turn(core::TurnRecord {
                task: TASK,
                attempt: 1,
                turn: 1,
                opens_window: true,
                spent: 0,
                read: None,
                at: Wall::EPOCH,
                transcript: Box::from(b"kept turn".as_slice()),
            })))),
        ]),
        0,
    );
    assert_eq!(store.tick(false), Ok(Some(1)));
    assert_eq!(transcript(&mut store, TASK), [Box::from(b"kept turn".as_slice())]);
    let (mut journal, replies) = route(core::Request::Write(core::Write::DropTranscript { task: TASK }));
    assert!(replies.is_empty());
    let mut commit = journal.commit().expect("transcript drop is a store write");
    let mut writes = Vec::new();
    while let Some(write) = commit.writes.pop() {
        writes.push(write);
    }
    writes.push(header(2));
    store.submit(2, writes.into_boxed_slice(), 0);
    assert_eq!(store.tick(false), Ok(Some(2)));
    assert!(transcript(&mut store, TASK).is_empty());
    assert!(
        !store.rows.values().any(|row| matches!(row,
            root::Record::Core(core::Record::Core(core::CoreRecord::Turn(turn))) if turn.task == TASK
        )),
        "typed mirror drops the task's transcript turns too"
    );
}

fn header(commits: u64) -> root::Write {
    root::Write::Save(root::Record::Core(core::Record::Core(core::CoreRecord::Deployment(core::Deployment {
        id: [18; 16],
        tasks: 21,
        people: 0,
        sign_ins: 0,
        messages: 0,
        runs: 1,
        calls: 0,
        connector_rows: 0,
        projects: 0,
        applications: 0,
        commits,
    }))))
}

fn transcript(store: &mut Store, task: u64) -> Vec<Box<[u8]>> {
    store.ask(StoreOut::Now(core::Now::StartPreparation { task, transcript_waiter: None, from: Some((1, 1)) }));
    let mut turns = Vec::new();
    for answer in store.answers() {
        if let Up::Root(root::Event::TranscriptLoaded { rows, .. }) = answer {
            turns.extend(rows.into_vec().into_iter().map(|(row, _)| row.transcript));
        }
    }
    turns
}
