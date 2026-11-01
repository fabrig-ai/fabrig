//! Project funding cleanup at the tasks child's boundary.
use jig_domain_tasks::{
    self as tasks, Cause, Domain, End, Event, Funder, FundingRecord, Key, Party, Request, Stored, TaskResult,
};
use jig_tasks_world::{LIMITS, World, domain, task};
use skein_lib::{Env, Queue, Wall};
use std::collections::BTreeMap;

const PROJECT: u32 = 1;
const OTHER: u32 = 2;
const PERSON: u64 = 9;
const SEED: u64 = 851;
const PERIOD: Funder = Funder::Period { project: PROJECT, period: 1 };
const POOL: Funder = Funder::Pool { project: PROJECT, person: PERSON, period: 1 };
const OTHER_PERIOD: Funder = Funder::Period { project: OTHER, period: 1 };

fn step(domain: &mut Domain, env: &Env<tasks::Limits>, event: Event) -> Vec<Request> {
    let mut out = Queue::uncapped();
    tasks::step(domain, env, event, &mut out);
    let mut requests = Vec::new();
    while let Some(request) = out.pop() {
        requests.push(request);
    }
    requests
}

fn restore(records: &BTreeMap<Key, Stored>, env: &Env<tasks::Limits>) -> Domain {
    let mut cold = domain(&env.limits, SEED);
    cold.start_restore();
    for row in records.values().filter(|row| row.key().live()) {
        assert!(step(&mut cold, env, Event::Restore { record: row.clone() }).is_empty());
    }
    assert!(step(&mut cold, env, Event::Check).is_empty());
    assert!(cold.checked());
    step(&mut cold, env, Event::Restored);
    cold
}

fn commit(records: &mut BTreeMap<Key, Stored>, requests: &[Request]) {
    for request in requests {
        if let Request::Erase { key } = request {
            assert!(records.remove(key).is_some(), "erased row exists: {key:?}");
        } else if let Request::Save { record } = request {
            records.insert(record.key(), record.clone());
        } else {
            assert!(
                matches!(request, Request::SpendForgotten { spent, .. } if *spent != 0),
                "unexpected cleanup output: {request:?}"
            );
        }
    }
}

fn totals(requests: &[Request]) -> Vec<(Option<u64>, u64)> {
    requests
        .iter()
        .filter_map(
            |request| {
                if let Request::SpendForgotten { party, spent } = request { Some((*party, *spent)) } else { None }
            },
        )
        .collect()
}

fn spent_world() -> World {
    let mut world = World::new(SEED, LIMITS);
    world.open_period(1, 500);
    world.carve_pool(1, 200);
    for (number, funder, spent) in [(1, PERIOD, 11), (2, POOL, 17)] {
        let mut member = task(number, &[]);
        member.funder = funder;
        world.make(Party::Person(PERSON), vec![member]);
        world.claim(number, number);
        world.terminal_cause(
            number,
            End::Finished { result: TaskResult::Report { words: b"done".as_slice().into() }, cancel_delegates: false },
            Cause::Priced { cumulative: spent },
        );
        world.settle(number);
    }
    let reply_to = world.to();
    world.send(Event::OpenPeriod { reply_to, project: OTHER, period: 1, budget: 100 });
    world.send(Event::PeriodNext { project: PROJECT, period: 1, next: Some(Wall::from_nanos(100)) });
    world
}

#[test]
fn forget_project_erases_current_funding_and_preserves_spend_across_restart() {
    let mut world = spent_world();
    let mut child = restore(&world.records, &world.env);
    let other = *child.funding(OTHER_PERIOD).expect("other project");
    let requests = step(&mut child, &world.env, Event::ForgetProject { project: PROJECT });
    assert_eq!(totals(&requests), [(None, 28), (Some(PERSON), 17)]);
    for funder in [PERIOD, POOL] {
        assert!(requests.contains(&Request::Erase { key: Key::Ledger(funder) }));
        assert!(child.funding(funder).is_none());
    }
    assert_eq!(child.funding(OTHER_PERIOD), Some(&other));
    commit(&mut world.records, &requests);
    let mut cold = restore(&world.records, &world.env);
    assert_eq!(cold.funding_records().len(), 1);
    assert_eq!(cold.funding(OTHER_PERIOD), Some(&other));
    assert!(step(&mut cold, &world.env, Event::ForgetProject { project: PROJECT }).is_empty());
    assert!(step(&mut cold, &world.env, Event::PeriodNext { project: PROJECT, period: 1, next: None }).is_empty());
}

#[test]
fn forget_project_closed_ledgers_fold_once_even_when_loaded_in_separate_pages() {
    let mut world = spent_world();
    world.open_period(2, 500);
    let rows: Vec<FundingRecord> = world
        .records
        .values()
        .filter_map(|row| if let Stored::ClosedLedger(ledger) = row { Some(*ledger) } else { None })
        .collect();
    assert!(rows.iter().any(|row| row.funder == PERIOD));
    assert!(rows.iter().any(|row| row.funder == POOL));
    let mut child = restore(&world.records, &world.env);
    let before = format!("{child:?}");
    let mut emitted = BTreeMap::<Option<u64>, u64>::new();
    for row in rows {
        let requests =
            step(&mut child, &world.env, Event::ForgetLedgers { project: PROJECT, ledgers: Box::new([row]) });
        assert!(requests.contains(&Request::Erase { key: Key::ClosedLedger(row.funder) }));
        for (party, spent) in totals(&requests) {
            *emitted.entry(party).or_default() += spent;
        }
        commit(&mut world.records, &requests);
    }
    assert_eq!(emitted, BTreeMap::from([(None, 28), (Some(PERSON), 17)]));
    assert_eq!(format!("{child:?}"), before, "closed history holds no memory");
    let requests = step(&mut child, &world.env, Event::ForgetProject { project: PROJECT });
    assert!(totals(&requests).is_empty(), "the fresh period spent nothing");
    commit(&mut world.records, &requests);
    let cold = restore(&world.records, &world.env);
    assert_eq!(cold.funding_records().len(), 1);
    assert_eq!(world.records.values().filter(|row| matches!(row, Stored::ClosedLedger(_))).count(), 0);
}

#[test]
#[should_panic(expected = "forget project only after its last live task ends")]
fn forget_project_asserts_no_live_task_remains() {
    let mut world = World::new(SEED, LIMITS);
    world.make(Party::Person(PERSON), vec![task(1, &[])]);
    let mut child = restore(&world.records, &world.env);
    step(&mut child, &world.env, Event::ForgetProject { project: PROJECT });
}

#[test]
#[should_panic(expected = "forgotten closed ledger belongs to the project")]
fn forget_project_asserts_closed_ledgers_belong_to_it() {
    let world = spent_world();
    let mut child = restore(&world.records, &world.env);
    let mut ledger = *child.funding(OTHER_PERIOD).expect("other ledger");
    ledger.closed = true;
    step(&mut child, &world.env, Event::ForgetLedgers { project: PROJECT, ledgers: Box::new([ledger]) });
}

#[test]
fn forget_project_erases_pending_rows_of_only_its_project() {
    let world = World::new(SEED, LIMITS);
    let mut child = domain(&LIMITS, SEED);
    child.start_restore();
    for project in [PROJECT, OTHER] {
        let mut member = task(100, &[]);
        member.project = project;
        member.funder = Funder::Pool { project, person: PERSON, period: 1 };
        let proposal = tasks::PersonProposal {
            number: u64::from(project),
            proposer: PERSON,
            project,
            action: tasks::PersonAction::Batch(Box::new([member])),
            reason: Box::new([]),
            state: tasks::PersonProposalState::Pending { since: world.env.wall },
        };
        let hold = tasks::PlacedHold {
            number: u64::from(project),
            project,
            owner: tasks::Owner::Party { person: PERSON },
            mode: tasks::Mode::Stop,
            scope: tasks::Scope::Subtree { task: 100 },
            reason: Box::new([]),
            placed_at: 0,
            revision: 1,
            exempt: Box::new([]),
            lifted: false,
        };
        // The deletion erases tasks before its final pending-row cleanup.
        for record in [Stored::PersonProposal(Box::new(proposal)), Stored::PlacedHold(Box::new(hold))] {
            assert!(step(&mut child, &world.env, Event::Restore { record }).is_empty());
        }
    }
    let requests = step(&mut child, &world.env, Event::ForgetProject { project: PROJECT });
    assert_eq!(
        requests,
        [
            Request::Erase { key: Key::PersonProposal(u64::from(PROJECT)) },
            Request::Erase { key: Key::PlacedHold(u64::from(PROJECT)) },
        ]
    );
    assert!(child.person_proposal(PERSON, u64::from(PROJECT)).is_none());
    assert!(child.placed_hold(u64::from(PROJECT)).is_none());
    assert!(child.person_proposal(PERSON, u64::from(OTHER)).is_some());
    assert!(child.placed_hold(u64::from(OTHER)).is_some());
}

#[test]
fn forget_project_folds_recurring_spend_and_combines_a_persons_periods() {
    let world = World::new(SEED, LIMITS);
    let mut rows = BTreeMap::new();
    for period in [1, 2] {
        let parent = Funder::Period { project: PROJECT, period };
        for (funder, direct, below) in [
            (parent, 2, 3),
            (Funder::Pool { project: PROJECT, person: PERSON, period }, 5, 7),
            (Funder::Pool { project: PROJECT, person: 10, period }, 0, 0),
            (Funder::Recurring { project: PROJECT, task: 100, period }, 11, 13),
        ] {
            let row = FundingRecord {
                funder,
                parent: (funder != parent).then_some(parent),
                period: (funder == parent).then_some(tasks::PeriodTimes { began: Wall::EPOCH, next: None }),
                numbers: tasks::Numbers {
                    budget: if funder == parent { 2000 } else { 500 },
                    spent: direct,
                    spent_below: below,
                    reserved: if funder == parent { 1500 } else { 0 },
                },
                made: 0,
                closed: false,
            };
            rows.insert(Key::Ledger(funder), Stored::Ledger(row));
        }
    }
    let mut child = restore(&rows, &world.env);
    let requests = step(&mut child, &world.env, Event::ForgetProject { project: PROJECT });
    assert_eq!(totals(&requests), [(None, 82), (Some(PERSON), 24)]);
    commit(&mut rows, &requests);
    assert!(rows.is_empty());
    assert!(child.funding_records().is_empty());

    let closed: Box<[_]> = [1, 2]
        .into_iter()
        .flat_map(|period| {
            [
                FundingRecord {
                    funder: Funder::Period { project: PROJECT, period },
                    parent: None,
                    period: Some(tasks::PeriodTimes { began: Wall::EPOCH, next: None }),
                    numbers: tasks::Numbers { budget: 100, spent: 2, spent_below: 39, reserved: 0 },
                    made: 0,
                    closed: true,
                },
                FundingRecord {
                    funder: Funder::Pool { project: PROJECT, person: PERSON, period },
                    parent: Some(Funder::Period { project: PROJECT, period }),
                    period: None,
                    numbers: tasks::Numbers { budget: 100, spent: 5, spent_below: 7, reserved: 0 },
                    made: 0,
                    closed: true,
                },
                FundingRecord {
                    funder: Funder::Recurring { project: PROJECT, task: 100, period },
                    parent: Some(Funder::Period { project: PROJECT, period }),
                    period: None,
                    numbers: tasks::Numbers { budget: 100, spent: 11, spent_below: 13, reserved: 0 },
                    made: 0,
                    closed: true,
                },
            ]
        })
        .collect();
    let requests = step(&mut child, &world.env, Event::ForgetLedgers { project: PROJECT, ledgers: closed });
    assert_eq!(totals(&requests), [(None, 82), (Some(PERSON), 24)]);
    assert!(child.funding_records().is_empty());
}
