use jig_domain_tasks::{
    self as tasks, Cause, Contract, Control, End, Event, Executor, Funder, Key, Mode, New, Numbers, Owner, Party,
    PersonAction, PersonProposal, PersonProposalState, PlacedHold, RecurringOverlap, RecurringTemplate, Request, Scope,
    Stored, TaskResult,
};
use jig_tasks_world::{LIMITS, Reply, World, task};
use skein_lib::{Env, List, Queue, Wall};
use std::collections::BTreeMap;

struct Snapshot {
    domain: tasks::Domain,
    env: Env<tasks::Limits>,
    records: BTreeMap<Key, Stored>,
}

impl Snapshot {
    fn new(world: World) -> Self {
        let mut snapshot =
            Self { domain: jig_tasks_world::domain(&LIMITS, 9014), env: world.env, records: world.records };
        snapshot.restart();
        snapshot
    }

    fn restart(&mut self) {
        self.domain = jig_tasks_world::domain(&LIMITS, 9014);
        self.domain.start_restore();
        let mut out = Queue::uncapped();
        for record in self.records.values().filter(|record| record.key().live()) {
            tasks::step(&mut self.domain, &self.env, Event::Restore { record: record.clone() }, &mut out);
        }
        tasks::step(&mut self.domain, &self.env, Event::Check, &mut out);
        assert!(self.domain.checked());
        tasks::step(&mut self.domain, &self.env, Event::Restored, &mut out);
        assert!(!out.iter().any(|request| matches!(request, Request::RestoreRefused { .. })));
    }

    fn send(&mut self, event: Event) -> Vec<Request> {
        let mut out = Queue::uncapped();
        tasks::step(&mut self.domain, &self.env, event, &mut out);
        let mut requests = Vec::new();
        while let Some(request) = out.pop() {
            if let Request::Save { record } = &request {
                self.records.insert(record.key(), record.clone());
            } else if let Request::Erase { key } = &request {
                self.records.remove(key);
            }
            requests.push(request);
        }
        requests
    }
}

fn period(project: u32) -> Funder {
    Funder::Period { project, period: 1 }
}

fn open(world: &mut World, project: u32) {
    let reply_to = world.to();
    world.send(Event::OpenPeriod { reply_to, project, period: 1, budget: 10_000 });
}

fn member(number: u64, project: u32, funder: Funder) -> New {
    let mut row = task(number, &[]);
    row.project = project;
    row.funder = funder;
    row
}

fn recurring(number: u64) -> New {
    let mut row = member(number, 1, period(1));
    row.executor = Executor::Procedure { connector: 0, code: 1 };
    row.numbers = Numbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 };
    row.contract = Contract::Report { words: 0 };
    row.recurring =
        Some(Box::new(RecurringTemplate { key: 1, batch: Box::new([task(1, &[])]), overlap: RecurringOverlap::Skip }));
    row
}

fn finish(world: &mut World, number: u64, spent: u64) {
    world.claim(number, number);
    world.terminal_cause(
        number,
        End::Finished { result: TaskResult::Report { words: Box::new([1]) }, cancel_delegates: false },
        Cause::Priced { cumulative: spent },
    );
    world.settle(number);
}

fn funded_project() -> World {
    let mut world = World::new(9010, LIMITS);
    open(&mut world, 1);
    open(&mut world, 2);
    world.carve_pool(1, 300);
    let pool = Funder::Pool { project: 1, person: 9, period: 1 };
    assert_eq!(world.make(Party::Person(9), vec![member(1, 1, pool)]), Reply::Made(vec![1]));
    finish(&mut world, 1, 11);
    assert_eq!(world.make(Party::Person(9), vec![member(2, 1, period(1))]), Reply::Made(vec![2]));
    finish(&mut world, 2, 13);
    assert_eq!(world.make(Party::Deployment { project: 1 }, vec![recurring(3)]), Reply::Made(vec![3]));
    world.send(Event::TickRecurring { task: 3, period: 1 });
    world.send(Event::RecurringBatch { task: 3, period: 1, numbers: Box::new([4]) });
    assert!(world.records.contains_key(&Key::Live(4)));
    finish(&mut world, 4, 17);
    let reply_to = world.to();
    world.send(Event::Control {
        reply_to,
        by: Party::Person(9),
        task: 3,
        control: Control::Cancel { reason: Box::new([]), grace: None, message: None },
    });
    world.settle(3);
    assert_eq!(world.live(), 0);
    world
}

#[test]
fn live_roots_are_the_projects_trees() {
    let mut world = World::new(9011, LIMITS);
    for project in [1, 2] {
        open(&mut world, project);
        let root = u64::from(project) * 10;
        let mut conversation = member(root, project, period(project));
        conversation.flags.conversation = true;
        conversation.flags.ended_by = tasks::EndedBy::Requester;
        assert_eq!(world.make(Party::Person(9), vec![conversation]), Reply::Made(vec![root]));
        assert_eq!(
            world.make(Party::Task(root), vec![member(root + 1, project, period(project))]),
            Reply::Made(vec![root + 1])
        );
        assert_eq!(
            world.make(Party::Deployment { project }, vec![member(root + 2, project, period(project))]),
            Reply::Made(vec![root + 2])
        );
    }
    let world = Snapshot::new(world);
    let mut roots = List::with_capacity(LIMITS.tasks);
    assert_eq!(world.domain.live_roots(1, &mut roots), Some(()));
    assert_eq!(roots.as_slice(), &[10, 12]);
    assert!(world.domain.live_in(1));
    assert!(world.domain.live_in(2));
    assert!(!world.domain.live_in(3));
}

#[test]
fn dropping_a_project_closes_its_ledgers_to_history() {
    let mut world = Snapshot::new(funded_project());
    let before = world.domain.funding_records().iter().map(|(source, row)| (*source, *row)).collect::<Vec<_>>();
    let requests = world.send(Event::DropProject { project: 1 });
    for (source, old) in &before {
        if *source == period(2) {
            assert_eq!(world.domain.funding(*source), Some(old));
            continue;
        }
        assert!(world.domain.funding(*source).is_none());
        assert!(requests.contains(&Request::Erase { key: Key::Ledger(*source) }));
    }
    for (source, old) in &before {
        if *source == period(2) {
            assert_eq!(world.records.get(&Key::Ledger(*source)), Some(&Stored::Ledger(*old)));
            continue;
        }
        assert!(!world.records.contains_key(&Key::Ledger(*source)));
        let Some(Stored::ClosedLedger(closed)) = world.records.get(&Key::ClosedLedger(*source)) else {
            panic!("project ledger stays in history: {source:?}")
        };
        let mut expected = *old;
        expected.closed = true;
        expected.numbers.reserved = 0;
        if *source == period(1) {
            expected.numbers.spent_below = 41;
        } else {
            assert_eq!(old.numbers.reserved, 0, "no task reservation remains");
        }
        assert_eq!(*closed, expected);
    }
    assert_eq!(world.domain.funding_records().len(), 1);
}

fn holds_and_proposals(world: &mut World) {
    for project in [1, 2] {
        let number = u64::from(project) * 100;
        let proposer = 8 + u64::from(project);
        // Load/fit/link retains the stored judgment of a project-scoped hold.
        world.records.insert(
            Key::PlacedHold(number),
            Stored::PlacedHold(Box::new(PlacedHold {
                number,
                project,
                owner: Owner::Party { person: 9 },
                mode: Mode::Stop,
                scope: Scope::Project,
                reason: Box::new([]),
                placed_at: 0,
                revision: 1,
                exempt: Box::new([]),
                lifted: false,
            })),
        );
        let reply_to = world.to();
        world.send(Event::ProposePerson {
            reply_to,
            proposal: PersonProposal {
                number: number + 1,
                proposer,
                project,
                action: PersonAction::Batch(Box::new([member(
                    number + 2,
                    project,
                    Funder::Pool { project, person: proposer, period: 1 },
                )])),
                reason: Box::new([]),
                state: PersonProposalState::Pending { since: world.env.wall },
            },
        });
        assert!(world.records.contains_key(&Key::PlacedHold(number)));
        assert!(world.records.contains_key(&Key::PersonProposal(number + 1)));
    }
}

#[test]
fn dropping_a_project_erases_its_holds_and_proposals() {
    let mut world = World::new(9012, LIMITS);
    holds_and_proposals(&mut world);
    let mut world = Snapshot::new(world);
    let other_hold = world.records[&Key::PlacedHold(200)].clone();
    let other_proposal = world.records[&Key::PersonProposal(201)].clone();
    let requests = world.send(Event::DropProject { project: 1 });
    assert!(world.domain.placed_hold(100).is_none());
    assert!(world.domain.person_proposal(9, 101).is_none());
    for key in [Key::PlacedHold(100), Key::PersonProposal(101)] {
        assert!(requests.contains(&Request::Erase { key }));
    }
    assert!(!world.records.contains_key(&Key::PlacedHold(100)));
    assert!(!world.records.contains_key(&Key::PersonProposal(101)));
    assert_eq!(world.records[&Key::PlacedHold(200)], other_hold);
    assert_eq!(world.records[&Key::PersonProposal(201)], other_proposal);
    world.restart();
    assert!(world.domain.placed_hold(100).is_none());
    assert!(world.domain.person_proposal(9, 101).is_none());
    assert!(world.domain.placed_hold(200).is_some());
    assert!(world.domain.person_proposal(10, 201).is_some());
}

#[test]
fn a_dropped_project_opens_no_period() {
    let mut world = Snapshot::new(funded_project());
    world.send(Event::DropProject { project: 1 });
    let before = world.records.clone();
    for event in [
        Event::PeriodNext { project: 1, period: 1, next: Some(Wall::from_nanos(100)) },
        Event::TickRecurring { task: 3, period: 2 },
        Event::RecurringBatch { task: 3, period: 2, numbers: Box::new([5]) },
    ] {
        assert!(world.send(event).is_empty());
    }
    world.env.now = world.env.now.saturating_add(skein_lib::Duration::from_secs(100));
    let mut out = skein_lib::Queue::uncapped();
    tasks::fire(&mut world.domain, &world.env, &mut out);
    assert!(out.is_empty());
    assert_eq!(world.records, before);
    assert!(world.domain.recurring_tasks(1).is_empty());
    assert!(world.domain.task(5).is_none());
}

#[test]
fn dropping_a_project_with_a_live_task_is_refused() {
    let mut world = World::new(9013, LIMITS);
    assert_eq!(world.make(Party::Person(9), vec![task(1, &[])]), Reply::Made(vec![1]));
    holds_and_proposals(&mut world);
    let mut world = Snapshot::new(world);
    let before = world.records.clone();
    let task_before = world.domain.task(1).expect("live task").clone();
    let requests = world.send(Event::DropProject { project: 1 });
    assert_eq!(requests, [Request::DropRefused { project: 1 }]);
    assert_eq!(world.domain.task(1), Some(&task_before));
    assert_eq!(world.records, before);
}

#[test]
fn a_dropped_projects_ledgers_stay_closed_across_a_restart() {
    let mut world = funded_project();
    world.send(Event::DropProject { project: 1 });
    let before = world.records.clone();
    world.restart();
    let world = Snapshot::new(world);
    assert_eq!(world.records, before);
    assert_eq!(world.domain.funding_records().len(), 1);
    assert!(world.domain.funding(period(2)).is_some());
    assert!(!world.domain.live_in(1));
    assert_eq!(world.records.values().filter(|row| matches!(row, Stored::ClosedLedger(ledger)
        if matches!(ledger.funder, Funder::Pool { project: 1, .. } | Funder::Recurring { project: 1, .. } | Funder::Period { project: 1, .. }))).count(), 4);
}
