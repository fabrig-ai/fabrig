//! Needs page over live state, independently of the updates cache.
use crate::reads::{self, Filter, Group, PageRow, Read};
use crate::tests::*;

fn fixture() -> (Core, Limits) {
    let (mut core, mut limits) = tree_core();
    limits.people.projects = 2;
    limits.people.inbox_entries = 2;
    limits.documents.rows = 3;
    let identity = core.people.identity(1).unwrap().clone();
    core.people = people::Domain::new(&limits.people, Box::new([]), core.settings.deployment_provider, Box::new([]));
    core.people.start_restore(Wall::EPOCH);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people };
    let mut out = Queue::uncapped();
    for record in [
        people::Stored::Person { number: 1, identity },
        people::Stored::SignIn { number: 10, person: 1, expires: Wall::from_nanos(Duration::from_secs(60).as_nanos()) },
        people::Stored::Roles {
            project: 1,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
        },
        people::Stored::Roles {
            project: 2,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
        },
    ] {
        people::step(&mut core.people, &env, people::Event::Restore { record }, &mut out);
    }
    finish_people(&mut core.people, &env);
    core.projects = List::with_capacity(2);
    core.projects.push(1).unwrap();
    core.projects.push(2).unwrap();
    core.names = Map::with_capacity(2);
    core.names.insert(1, Box::from("First project")).unwrap();
    core.names.insert(2, Box::from("Second project")).unwrap();
    send_task(
        &mut core,
        &limits,
        tasks::Event::OpenPeriod { reply_to: ReplyTo::new(Token::new(1)), project: 2, period: 1, budget: 1000 },
    );
    // Interleave projects and kinds, and feed each task's projection out of time order.
    projected(
        &mut core,
        &limits,
        1,
        1,
        &[
            (7, people::EntryKind::PersonTask),
            (1, people::EntryKind::Escalation { revision: 1 }),
            (4, people::EntryKind::Question { message: 1 }),
        ],
    );
    projected(
        &mut core,
        &limits,
        2,
        2,
        &[(5, people::EntryKind::PersonTask), (2, people::EntryKind::Escalation { revision: 1 })],
    );
    projected(
        &mut core,
        &limits,
        3,
        1,
        &[(6, people::EntryKind::PersonTask), (3, people::EntryKind::Escalation { revision: 1 })],
    );
    (core, limits)
}

fn projected(core: &mut Core, limits: &Limits, task: u64, project: u32, needs: &[(u64, people::EntryKind)]) {
    let mut row = new_row(core, task, tasks::Funder::Period { project, period: 1 }, None);
    row.project = project;
    fixtures::make_batch(core, limits, tasks::Party::Person(1), Box::new([row]));
    assert!(core.tasks.task(task).is_some());
    let mut entries = List::with_capacity(u32::try_from(needs.len()).unwrap());
    for &(at, kind) in needs {
        entries
            .push(people::Entry { task, project, whom: people::Whom::Person(1), kind, at: Wall::from_nanos(at) })
            .unwrap();
    }
    people::step(
        &mut core.people,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people },
        people::Event::Waiting { source: people::NeedSource::Task(task), entries: entries.into_boxed() },
        &mut Queue::uncapped(),
    );
}

fn page(core: &mut Core, limits: &Limits, filter: Filter, from: Option<reads::Cursor>) -> Box<reads::Page> {
    let held = read_start(core, limits, 10, Read::Inbox { group: Group::Needs, filter, from });
    let Held::Read { answer: reads::Answer::Page(page), .. } = held else {
        panic!("needs must be answered from live state: {held:?}")
    };
    assert!(page.rows.len() <= limits.documents.rows);
    match page.next {
        Some(reads::Cursor::Store { .. }) => panic!("needs never continue in the store"),
        Some(reads::Cursor::Live { .. }) | None => {}
    }
    page
}

fn all(core: &mut Core, limits: &Limits, project: Option<u32>) -> (Box<[InboxEntry]>, Box<[u32]>) {
    let mut found = List::with_capacity(8);
    let mut lengths = List::with_capacity(8);
    let mut from = None;
    for _ in 0_u32..8 {
        let page = page(core, limits, Filter { project, goal: None }, from);
        lengths.push(page.rows.len()).unwrap();
        for row in page.rows.into_boxed() {
            let PageRow::Inbox(entry) = row else { panic!("inbox row") };
            found.push(entry).unwrap();
        }
        from = page.next;
        if from.is_none() {
            return (found.into_boxed(), lengths.into_boxed());
        }
    }
    panic!("needs must reach a last page")
}

fn times(entries: &[InboxEntry]) -> Box<[u64]> {
    let mut times = List::with_capacity(u32::try_from(entries.len()).unwrap());
    for entry in entries {
        times.push(entry.at.unwrap().as_nanos()).unwrap();
    }
    times.into_boxed()
}

fn document(core: &Core, limits: &Limits) -> Inbox {
    let Document::Inbox(value) = make(
        core,
        limits,
        1,
        &Subject::Inbox { party: 1 },
        &Window { rows: limits.documents.rows, after: None },
        Wall::EPOCH,
    )
    .unwrap() else {
        panic!("inbox document")
    };
    value
}

#[test]
fn every_live_need_pages_oldest_first_past_the_cache_and_stops_without_the_store() {
    let (mut core, mut limits) = fixture();
    let (entries, lengths) = all(&mut core, &limits, None);
    assert_eq!(times(&entries).as_ref(), [1, 2, 3, 4, 5, 6, 7]);
    assert_eq!(lengths.as_ref(), [3, 3, 1]);
    assert_eq!(document(&core, &limits).cached_needs, u32::try_from(entries.len()).unwrap());
    let Document::Frame(me) = make(&core, &limits, 1, &Subject::Frame { party: 1 }, &window(), Wall::EPOCH).unwrap()
    else {
        panic!("me")
    };
    assert_eq!(me.needs, 7);
    let (filtered, lengths) = all(&mut core, &limits, Some(2));
    assert_eq!(times(&filtered).as_ref(), [2, 5]);
    assert_eq!(lengths.as_ref(), [1, 1, 0], "the continuation precedes the project filter");
    for entry in &filtered {
        assert_eq!(entry.project, Some(2));
    }

    core.people.remember_result(
        &limits.people,
        1,
        people::ResultRef { task: 100, project: 1, at: Wall::from_nanos(8), position: 1, forgettable: None },
    );
    let first = document(&core, &limits);
    assert_eq!(first.needs.len(), 3);
    assert!(first.updates.is_empty());
    assert_eq!(first.next, Some(entries[2].cursor));
    limits.documents.rows = 8;
    let whole = document(&core, &limits);
    assert_eq!(whole.needs.len(), 7);
    assert_eq!(whole.updates.len(), 1);
    assert_eq!(whole.next, None, "all needs and updates fit");
}

#[test]
fn a_standing_roles_person_delegate_is_live_beyond_the_first_needs_page() {
    let (mut core, limits) = fixture();
    let standing = new_row(&core, 8, tasks::Funder::Period { project: 1, period: 1 }, None);
    fixtures::make_batch(&mut core, &limits, tasks::Party::Deployment { project: 1 }, Box::new([standing]));
    send_task(&mut core, &limits, tasks::Event::Prepare { reply_to: ReplyTo::new(Token::new(1)), task: 8 });
    send_task(
        &mut core,
        &limits,
        tasks::Event::Claim {
            session_end: None,
            offered: tasks::Reasons::ALL,
            reasons: tasks::Reasons::FIRST,
            account: None,
            reply_to: ReplyTo::new(Token::new(1)),
            task: 8,
            attempt: 1,
            resumed: false,
            budget: 10,
            writes: Box::new([]),
        },
    );
    send_task(&mut core, &limits, tasks::Event::Started { task: 8, attempt: 1 });
    assert_eq!(core.tasks.task(8).unwrap().tracked, None);
    assert_eq!(core.tasks.task(8).unwrap().phase, tasks::Phase::Active(tasks::Active::Running { attempt: 1 }));
    let mut delegate = new_row(&core, 9, tasks::Funder::Task(8), None);
    delegate.executor = tasks::Executor::Person(tasks::PersonAddress::Role(0));
    drop(step(
        &mut core,
        &Env { now: Time::ZERO, wall: Wall::from_nanos(8), limits },
        Event::Tasks(tasks::Event::Make {
            maker: None,
            places: Box::new([]),
            reply_to: ReplyTo::new(Token::new(1)),
            creator: tasks::Party::Task(8),
            batch: Box::new([delegate]),
        }),
    ));
    let task = core.tasks.task(9).expect("the standing run's delegate is live");
    assert_eq!(task.requester, tasks::Party::Task(8));
    assert_eq!(task.contract, tasks::Contract::Report { words: 64 });
    let (entries, lengths) = all(&mut core, &limits, None);
    assert_eq!(lengths.as_ref(), [3, 3, 2]);
    let entry = entries.last().unwrap();
    assert_eq!(entry.object, InboxObject::PersonTask);
    assert_eq!(entry.project, Some(1));
    let TaskLink::Live(chip) = &entry.task else { panic!("live delegate task link") };
    assert_eq!(chip.number, 9);
    assert_eq!(document(&core, &limits).cached_needs, 8);
}

#[test]
fn a_needs_store_position_continues_over_live_state() {
    let (mut core, limits) = fixture();
    let filter = Filter { project: None, goal: None };
    let first = page(&mut core, &limits, filter, None);
    let Some(reads::Cursor::Live { scope, store_after, .. }) = first.next else { panic!("live continuation") };
    let printed = reads::Cursor::Store { scope, after: store_after };
    let next = page(&mut core, &limits, filter, Some(printed));
    let mut times = List::with_capacity(limits.documents.rows);
    for row in next.rows.into_boxed() {
        let PageRow::Inbox(entry) = row else { panic!("inbox row") };
        times.push(entry.at.unwrap().as_nanos()).unwrap();
    }
    assert_eq!(times.as_slice(), [4, 5, 6]);
}
