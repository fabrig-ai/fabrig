//! A party's amendment is judged by its whole authority and its funding need.
use super::*;

fn pool(person: u64) -> tasks::Funder {
    tasks::Funder::Pool { project: 1, person, period: 1 }
}

fn funded(own: bool, spent: u64, spent_below: u64, left: Option<u64>) -> (Core, Limits) {
    let (mut core, limits) = core();
    make_task(&mut core, &limits, 1);
    let mut row = core.tasks.task(1).unwrap().clone();
    row.funder = pool(if own { 1 } else { 2 });
    row.numbers.spent = spent;
    row.numbers.spent_below = spent_below;
    let mut period = *core.tasks.funding(tasks::Funder::Period { project: 1, period: 1 }).unwrap();
    period.numbers.reserved = 0;
    let mut ledgers = List::with_capacity(2);
    if let Some(left) = left {
        let reserved = if own { 100 } else { 0 };
        let budget = left.checked_add(reserved).unwrap();
        ledgers
            .push(tasks::FundingRecord {
                funder: pool(1),
                parent: Some(period.funder),
                numbers: tasks::Numbers { budget, spent: 0, spent_below: 0, reserved },
                period: None,
                made: 0,
                closed: false,
            })
            .unwrap();
        period.numbers.reserved = period.numbers.reserved.checked_add(budget).unwrap();
    }
    if !own {
        ledgers
            .push(tasks::FundingRecord {
                funder: pool(2),
                parent: Some(period.funder),
                numbers: tasks::Numbers { budget: 500, spent: 0, spent_below: 0, reserved: 100 },
                period: None,
                made: 0,
                closed: false,
            })
            .unwrap();
        period.numbers.reserved = period.numbers.reserved.checked_add(500).unwrap();
    }
    core.tasks = fixtures::fresh_tasks(&core, &limits);
    send_task(&mut core, &limits, tasks::Event::Restore { record: tasks::Stored::Ledger(period) });
    for ledger in ledgers.into_boxed() {
        send_task(&mut core, &limits, tasks::Event::Restore { record: tasks::Stored::Ledger(ledger) });
    }
    send_task(&mut core, &limits, tasks::Event::Restore { record: tasks::Stored::Live(Box::new(row)) });
    send_task(&mut core, &limits, tasks::Event::Check);
    send_task(&mut core, &limits, tasks::Event::Restored);
    (core, limits)
}

fn change(core: &Core, spend: u64) -> tasks::Amendment {
    let mut after = core.tasks.task(1).unwrap().authority.clone();
    after.budget.spend = spend;
    tasks::Amendment { authority: Some(after), ..fixtures::unchanged(b"funded widening") }
}

fn party() -> compound::PartyTask {
    compound::PartyTask { person: 1, role: Some(people::Role::Owner), project: 1, task: 1 }
}

// The client's scalar rule for drafts preserving all permissions and the deadline.
fn offered(core: &Core, limits: &Limits, spend: u64) -> Admission {
    let Admission::Bounded { within: Bounds::Amend { authority, ceiling, budget_left, .. }, beyond } =
        offers::admit(core, limits, 1, Act::Amend, &Subject::Task { task: 1 }, Wall::EPOCH)
    else {
        panic!("bounded amendment")
    };
    let before = core.tasks.task(1).unwrap().authority.budget.spend;
    if spend <= before {
        return Admission::Allowed;
    }
    if spend > ceiling.ceiling.spend {
        return Admission::Refused(people::Refusal::NotOffered);
    }
    if spend <= authority.ceiling.spend && spend.saturating_sub(before) <= budget_left {
        Admission::Allowed
    } else {
        match beyond {
            Beyond::Proposed { to } => Admission::Proposed { to },
            Beyond::Refused(why) => Admission::Refused(why),
        }
    }
}

fn check(core: &Core, limits: &Limits, spend: u64, expected: Admission) {
    assert_eq!(offered(core, limits, spend), expected);
    let actual = core.amend_admit(party(), &change(core, spend), limits.tasks.depth, &limits.tasks);
    let class: u8 = match &actual {
        Ok(_) => 0,
        Err(people::Refusal::Authority { proposable: Some(people::Accepter::Role(0)), .. }) => 1,
        Err(_) => 2,
    };
    let expected: u8 = match expected {
        Admission::Allowed => 0,
        Admission::Proposed { .. } => 1,
        Admission::Refused(_) => 2,
        Admission::Bounded { .. } => panic!("concrete judgement"),
    };
    assert_eq!(class, expected, "{actual:?}");
}

#[test]
fn a_widening_is_judged_by_what_its_giver_needs_not_its_whole_budget() {
    let proposed = Admission::Proposed { to: policy_holder(&[0]) };
    let (core, limits) = funded(true, 0, 0, Some(30));
    let Admission::Bounded { within: Bounds::Amend { authority, budget_left, ceiling, .. }, .. } =
        offers::admit(&core, &limits, 1, Act::Amend, &Subject::Task { task: 1 }, Wall::EPOCH)
    else {
        panic!("bounded amendment")
    };
    assert_eq!((authority.ceiling.spend, budget_left, ceiling.ceiling.spend), (130, 30, 1000));
    check(&core, &limits, 130, Admission::Allowed);
    check(&core, &limits, 131, proposed.clone());
    let (core, limits) = funded(false, 50, 10, Some(50));
    let Admission::Bounded { within: Bounds::Amend { authority, budget_left, ceiling, .. }, .. } =
        offers::admit(&core, &limits, 1, Act::Amend, &Subject::Task { task: 1 }, Wall::EPOCH)
    else {
        panic!("bounded amendment")
    };
    assert_eq!((authority.ceiling.spend, budget_left, ceiling.ceiling.spend), (110, 10, 1000));
    check(&core, &limits, 110, Admission::Allowed);
    check(&core, &limits, 111, proposed);
}

#[test]
fn credit_does_not_raise_the_roles_spend_or_clip_the_policy_ceiling() {
    let (mut core, limits) = funded(true, 0, 0, Some(600));
    policy(&mut core, true, 1000);
    check(&core, &limits, 501, Admission::Proposed { to: policy_holder(&[0]) });
    check(&core, &limits, 701, Admission::Proposed { to: policy_holder(&[0]) });
    check(&core, &limits, 1001, Admission::Refused(people::Refusal::NotOffered));
}

#[test]
fn an_unopened_amenders_pool_is_read_at_its_allotment() {
    let (core, limits) = funded(false, 50, 10, None);
    assert_eq!(core.person_pool(1, 1).1.budget, 500);
    check(&core, &limits, 500, Admission::Allowed);
    check(&core, &limits, 501, Admission::Proposed { to: policy_holder(&[0]) });
}

#[test]
fn a_period_preflight_still_refuses_widening_but_not_narrowing() {
    let (mut core, limits) = funded(true, 0, 0, Some(30));
    policy(&mut core, true, 100);
    check(&core, &limits, 101, Admission::Refused(people::Refusal::NotOffered));
    check(&core, &limits, 90, Admission::Allowed);
}

#[test]
fn a_compounds_amendment_reads_its_staged_credit_and_pool_balances() {
    let (core, limits) = funded(true, 0, 0, Some(30));
    let mut admission = tasks::CompoundAdmission::new(&core.tasks, 1);
    admission
        .check(
            &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.tasks },
            tasks::CompoundContext {
                by: tasks::Party::Person(1),
                project: 1,
                period: 1,
                pool_budget: 130,
                period_budget: 1000,
            },
            tasks::Operation::Amend(tasks::Amend {
                funder: None,
                task: 1,
                message: 1,
                stop_run: true,
                amendment: Box::new(change(&core, 80)),
            }),
        )
        .unwrap();
    assert_eq!(admission.task(1).unwrap().numbers.budget, 80);
    assert_eq!(admission.funding(pool(1)).unwrap().reserved, 80);
    assert_eq!(
        crate::acts::bounds::amendment_admit_in(
            &core,
            party(),
            &change(&core, 130),
            limits.tasks.depth,
            &limits.tasks,
            &admission
        ),
        Ok(false)
    );
    assert_eq!(
        crate::acts::bounds::amendment_admit_in(
            &core,
            party(),
            &change(&core, 131),
            limits.tasks.depth,
            &limits.tasks,
            &admission
        ),
        Err(people::Refusal::Authority {
            proposable: Some(people::Accepter::Role(0)),
            lacking: people::Lack::Budget { needs: 131, left: 130 }
        })
    );
}
