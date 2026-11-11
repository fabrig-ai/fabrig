//! Standing agents are funded anew each period (tasks/standing.md, section 4).
use super::standing_lifecycle::{cancel, read, standing, wake, world};
use jig_domain_tasks::{
    self as tasks, Active, Cause, End, Event, Funder, Hold, Key, Numbers, Party, Phase, Stored, Subscription,
    SubscriptionKind, TaskResult, Tries,
};
use jig_tasks_world::{LIMITS, PERSON, PERSON_PARTY, Reply, World, task};

pub(super) fn ledger(world: &World, source: Funder) -> tasks::FundingRecord {
    match world.records.get(&Key::Ledger(source)).or_else(|| world.records.get(&Key::ClosedLedger(source))) {
        Some(Stored::Ledger(row) | Stored::ClosedLedger(row)) => *row,
        row => panic!("missing ledger {source:?}: {row:?}"),
    }
}

fn fresh(period: u64) -> Funder {
    Funder::Period { project: 1, period }
}
fn archived(period: u64) -> Funder {
    Funder::Recurring { project: 1, task: 1, period }
}

#[test]
fn standing_renews_its_budget_and_allotment_but_not_its_live_delegates_funding() {
    for subscribed in [false, true] {
        let mut world = World::new(867, LIMITS);
        world.carve_pool(0, 100);
        let mut member = standing();
        let pool = Funder::Pool { project: 1, person: PERSON, period: 0 };
        member.funder = pool;
        assert_eq!(world.make(PERSON_PARTY, vec![member]), Reply::Made(vec![1]));
        if subscribed {
            let reply_to = world.to();
            world.send(Event::SubscribeTopic {
                reply_to,
                task: 1,
                subscription: Subscription { number: 100, kind: SubscriptionKind::Topic { connector: 1, topic: 1 } },
            });
        }
        let mut child = task(2, &[]);
        child.funder = Funder::Task(1);
        child.numbers.budget = 40;
        child.authority.budget.spend = 40;
        assert_eq!(world.make(Party::Task(1), vec![child]), Reply::Made(vec![2]));
        world.claim(1, 1);
        world.terminal_cause(1, End::Parked, Cause::Priced { cumulative: 30 });
        assert_eq!(world.record(1).made, 2);
        world.open_period(1, 100);
        world.renew_standing(1);
        assert_eq!(world.record(1).funder, fresh(1));
        assert_eq!(world.record(1).numbers, Numbers { budget: 100, spent: 0, spent_below: 0, reserved: 0 });
        assert_eq!(world.record(1).made, 1);
        assert_eq!(ledger(&world, fresh(1)).numbers.reserved, 100);
        assert_eq!(world.record(2).funder, archived(0));
        assert_eq!(
            ledger(&world, archived(0)).numbers,
            Numbers { budget: 100, spent: 0, spent_below: 30, reserved: 40 }
        );
        assert_eq!(ledger(&world, pool).numbers.budget, 0);
        let unchanged = world.records.clone();
        world.renew_standing(1);
        assert_eq!(world.records, unchanged, "one carving per period");
        world.restart();
        world.claim(2, 2);
        world.terminal_cause(
            2,
            End::Finished { result: TaskResult::Report { words: Box::new([]) }, cancel_delegates: false },
            Cause::Priced { cumulative: 7 },
        );
        world.settle(2);
        assert_eq!(ledger(&world, fresh(0)).numbers.spent_below, 37, "old spend counted once");
        assert_eq!(world.record(1).numbers.spent_below, 0);
        assert_eq!(world.record(1).made, 1);
        cancel(&mut world);
    }
}

#[test]
fn standing_live_at_a_period_start_renews_in_the_decision_ending_its_run() {
    for end in [
        End::Parked,
        End::Finished { result: TaskResult::Report { words: b"done".as_slice().into() }, cancel_delegates: false },
        End::Failed(tasks::Class::Transient),
    ] {
        let mut world = world(868);
        world.claim(1, 1);
        world.open_period(1, 100);
        let before = world.records.clone();
        world.renew_standing(1);
        assert_eq!(world.records, before, "a live run keeps its reservation");
        world.open_period(3, 100);
        world.renew_standing(3);
        assert_eq!(world.record(1).funder, fresh(0));
        let reply_to = world.to();
        world.stage(Event::Activation {
            reply_to,
            task: 1,
            attempt: 1,
            end,
            saved: None,
            cause: Cause::Priced { cumulative: 30 },
        });
        assert!(
            world.staged_requests().iter().any(|request| matches!(request,
                tasks::Request::Save { record: Stored::Live(row) } if row.number == 1 && row.funder == fresh(3)
            )),
            "renewal is in the terminal decision, not a later event"
        );
        world.durable();
        world.deliver();
        assert_eq!(world.record(1).funder, fresh(3));
        assert_eq!(world.record(1).numbers, Numbers { budget: 100, spent: 0, spent_below: 0, reserved: 0 });
        assert_eq!(ledger(&world, fresh(0)).numbers.spent_below, 30);
        world.restart();
        cancel(&mut world);
    }
}

#[test]
fn standing_budget_renewal_releases_only_the_budget_hold_and_withdraws_its_escalation() {
    for why in [Hold::Budget, Hold::Stopped] {
        let mut world = world(869);
        world.claim(1, 1);
        world.terminal_cause(1, End::Parked, Cause::Priced { cumulative: 100 });
        world.send(Event::Hold { task: 1, why });
        let revision = world.record(1).revisions.hold;
        world.send(Event::RoutedEscalation {
            task: 1,
            revision,
            holder: tasks::EscalationHolder::Person(1),
            entry: 100,
        });
        assert!(matches!(world.record(1).escalation, tasks::Escalation::Waiting { .. }));
        world.open_period(1, 100);
        world.stage(Event::RenewStanding { task: 1, period: 1 });
        if why == Hold::Budget {
            assert!(world.staged_requests().iter().any(|request| matches!(request,
                tasks::Request::Save { record: Stored::Live(row) }
                    if row.number == 1 && row.funder == fresh(1)
                    && row.phase == Phase::Active(Active::Due)
                    && matches!(row.escalation, tasks::Escalation::Unheld { .. })
            )));
        }
        world.durable();
        world.deliver();
        if why == Hold::Budget {
            assert_eq!(world.record(1).phase, Phase::Active(Active::Due));
            assert!(matches!(world.record(1).escalation, tasks::Escalation::Unheld { .. }));
            assert_eq!(world.record(1).tries, Tries::NONE);
            assert!(world.activations.contains(&1));
        } else {
            assert!(matches!(world.record(1).phase, Phase::Held { why: Hold::Stopped, .. }));
            assert!(matches!(world.record(1).escalation, tasks::Escalation::Waiting { .. }));
        }
        world.restart();
        cancel(&mut world);
    }
}

#[test]
fn standing_unfunded_renewal_is_unchanged_and_retried_at_a_later_run_end() {
    let mut world = world(870);
    world.claim(1, 1);
    world.terminal_cause(1, End::Parked, Cause::Priced { cumulative: 30 });
    world.open_period(1, 100);
    let mut blocker = task(9, &[]);
    blocker.funder = fresh(1);
    blocker.authority.budget.spend = 1;
    blocker.numbers.budget = 1;
    assert_eq!(world.make(Party::Person(1), vec![blocker]), Reply::Made(vec![9]));
    let before = world.records.clone();
    world.renew_standing(1);
    assert_eq!(world.records, before);
    wake(&mut world);
    world.claim(1, 2);
    read(&mut world);
    world.terminal_cause(1, End::Parked, Cause::Priced { cumulative: 10 });
    assert_eq!(world.record(1).funder, fresh(0));
    assert_eq!(world.record(1).numbers.spent, 40);
    world.finish_task(9);
    world.restart();
    wake(&mut world);
    world.claim(1, 3);
    read(&mut world);
    world.terminal_cause(1, End::Parked, Cause::Priced { cumulative: 5 });
    assert_eq!(world.record(1).funder, fresh(1));
    assert_eq!(world.record(1).numbers.spent, 0);
    assert_eq!(ledger(&world, fresh(0)).numbers.spent_below, 45);
    cancel(&mut world);
}
