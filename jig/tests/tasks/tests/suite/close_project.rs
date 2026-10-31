//! Project deletion retains financial history (jig/docs/design/domain/tasks.md, 7).
use jig_domain_tasks::{
    self as tasks, Cause, Event, Funder, Key, Mode, Owner, Party, PlacedHold, RecurringOverlap, Scope, Stored,
};
use jig_tasks_world::{LIMITS, Reply, World, task};

fn finish(world: &mut World, number: u64, spent: u64) {
    world.claim(number, number);
    world.terminal_cause(
        number,
        tasks::End::Finished { result: tasks::TaskResult::Report { words: Box::new([1]) }, cancel_delegates: false },
        Cause::Priced { cumulative: spent },
    );
    world.settle(number);
}

fn ledger(world: &World, funder: Funder, closed: bool) -> tasks::FundingRecord {
    let key = if closed { Key::ClosedLedger(funder) } else { Key::Ledger(funder) };
    let row = world.records.get(&key).expect("durable ledger");
    let (Stored::Ledger(row) | Stored::ClosedLedger(row)) = row else { panic!("ledger family") };
    assert_eq!(row.closed, closed);
    *row
}

#[test]
fn closing_a_project_keeps_its_spend_in_closed_ledgers() {
    let mut world = World::new(1101, LIMITS);
    world.open_period(1, 1000);
    world.carve_pool(1, 200);
    let period = Funder::Period { project: 1, period: 1 };
    let pool = Funder::Pool { project: 1, person: 9, period: 1 };
    let recurring = Funder::Recurring { project: 1, task: 9, period: 1 };
    let mut member = task(1, &[]);
    member.funder = pool;
    assert_eq!(world.make(Party::Person(9), vec![member]), Reply::Made(vec![1]));
    finish(&mut world, 1, 17);
    let mut direct = task(2, &[]);
    direct.funder = period;
    assert_eq!(world.make(Party::Person(9), vec![direct]), Reply::Made(vec![2]));
    finish(&mut world, 2, 11);
    let mut master = super::recurring::recurring(9, RecurringOverlap::Skip);
    master.funder = period;
    assert_eq!(world.make(Party::Deployment { project: 1 }, vec![master]), Reply::Made(vec![9]));
    world.send(Event::TickRecurring { task: 9, period: 1 });
    world.send(Event::RecurringBatch { task: 9, period: 1, numbers: Box::new([10]) });
    finish(&mut world, 10, 7);
    let reply_to = world.to();
    world.send(Event::Control {
        reply_to,
        by: Party::Person(9),
        task: 9,
        control: tasks::Control::Cancel { reason: Box::new([]), grace: None, message: None },
    });
    world.complete_cancel();
    assert_eq!(world.live(), 0);
    let pool_before = ledger(&world, pool, false);
    let recurring_before = ledger(&world, recurring, false);
    world.send(Event::CloseProject { project: 1 });
    assert_eq!(ledger(&world, pool, true).numbers, pool_before.numbers);
    assert_eq!(ledger(&world, recurring, true).numbers, recurring_before.numbers);
    assert_eq!(ledger(&world, period, true).numbers.spent_below, 35);
    assert_eq!(ledger(&world, period, true).numbers.reserved, 0);
    assert!(!world.records.keys().any(|key| matches!(key, Key::Ledger(_))));
    let before = world.records.clone();
    world.send(Event::PeriodNext { project: 1, period: 2, next: None });
    world.send(Event::TickRecurring { task: 9, period: 2 });
    world.send(Event::RecurringBatch { task: 9, period: 2, numbers: Box::new([11]) });
    world.send(Event::RenewStanding { task: 9, period: 2 });
    assert_eq!(world.records, before);
    world.restart();
    assert_eq!(world.records, before);
}

#[test]
fn a_pools_spend_reaches_its_period_before_it_closes() {
    let mut world = World::new(1102, LIMITS);
    world.carve_pool(0, 100);
    let pool = Funder::Pool { project: 1, person: 9, period: 0 };
    let period = Funder::Period { project: 1, period: 0 };
    let mut member = task(1, &[]);
    member.funder = pool;
    assert_eq!(world.make(Party::Person(9), vec![member]), Reply::Made(vec![1]));
    finish(&mut world, 1, 23);
    assert_eq!(ledger(&world, period, false).numbers.spent_below, 0);
    world.send(Event::CloseProject { project: 1 });
    assert_eq!(ledger(&world, pool, true).numbers.spent_below, 23);
    assert_eq!(ledger(&world, period, true).numbers.spent_below, 23);
    assert_eq!(ledger(&world, period, true).numbers.reserved, 0);
}

#[test]
fn its_project_holds_go_and_other_projects_stay() {
    let mut world = World::new(1103, LIMITS);
    let reply_to = world.to();
    world.send(Event::OpenPeriod { reply_to, project: 2, period: 1, budget: 200 });
    let reply_to = world.to();
    world.send(Event::CarvePool { reply_to, project: 2, person: 8, period: 1, budget: 100 });
    let mut other = task(2, &[]);
    other.project = 2;
    other.funder = Funder::Pool { project: 2, person: 8, period: 1 };
    assert_eq!(world.make(Party::Person(8), vec![other]), Reply::Made(vec![2]));
    let reply_to = world.to();
    world.send(Event::Place {
        reply_to,
        hold: PlacedHold {
            number: 100,
            project: 2,
            owner: Owner::Party { person: 8 },
            mode: Mode::Stop,
            scope: Scope::Subtree { task: 2 },
            reason: Box::new([]),
            placed_at: 0,
            revision: 1,
            exempt: Box::new([]),
            lifted: false,
        },
    });
    let before: Vec<_> = world
        .records
        .iter()
        .filter(|(key, _)| **key != Key::Ledger(Funder::Period { project: 1, period: 0 }))
        .map(|(key, row)| (*key, row.clone()))
        .collect();
    world.send(Event::CloseProject { project: 1 });
    for (key, row) in before {
        assert_eq!(world.records.get(&key), Some(&row));
    }
    assert!(world.placed_hold(100).is_some());
}

#[test]
fn closing_twice_or_with_nothing_emits_nothing() {
    let mut world = World::new(1104, LIMITS);
    let before = world.records.clone();
    world.send(Event::CloseProject { project: 99 });
    assert_eq!(world.records, before);
    world.send(Event::CloseProject { project: 1 });
    let before = world.records.clone();
    world.send(Event::CloseProject { project: 1 });
    assert_eq!(world.records, before);
}
