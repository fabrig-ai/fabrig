//! A run leaves funds for delegates (jig/docs/design/domain/authority-budgets.md, 1).
use super::run_choice::{account, granted, host};
use crate::tests::*;

const TASK: u64 = 1;

pub(in crate::tests) fn funded(budget: u64, delegates: bool) -> (Core, Limits) {
    let (mut core, limits) = tree_core();
    let mut row = new_row(&core, TASK, tasks::Funder::Period { project: 1, period: 1 }, None);
    row.authority.budget.spend = budget;
    row.numbers.budget = budget;
    if !delegates {
        row.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    }
    fixtures::make_batch(&mut core, &limits, tasks::Party::Person(1), Box::new([row]));
    assert!(core.tasks.task(TASK).is_some());
    account(&mut core, &limits, accounts::Event::Want);
    granted(&mut core, &limits, 1);
    host(&mut core, &limits, Token::new(7), &[b"scripted"]);
    (core, limits)
}

fn funded_with_minimum(budget: u64, minimum: u64) -> (Core, Limits) {
    let (mut core, limits) = funded(budget, true);
    let mut rules = core.authority.rules().clone();
    rules.minimum_run_spend = minimum;
    authority::step(&mut core.authority, authority::Event::Rules(Box::new(rules)), &mut Queue::uncapped());
    assert_eq!(core.authority.rules().minimum_run_spend, minimum);
    (core, limits)
}

pub(in crate::tests) fn claim(core: &mut Core, limits: &Limits) -> u64 {
    let context = core.tasks.run_context(TASK).expect("due task");
    let RunAdmission::Allow(allowance) = core.run_admission(&context, Wall::EPOCH, Box::new([])) else {
        panic!("funded run admitted")
    };
    send_task(core, limits, tasks::Event::Prepare { reply_to: ReplyTo::new(Token::new(1)), task: TASK });
    send_task(
        core,
        limits,
        tasks::Event::Claim {
            sources: None,
            fresh: None,
            session_end: None,
            offered: tasks::Reasons::ALL,
            reasons: tasks::Reasons::FIRST,
            reply_to: ReplyTo::new(Token::new(1)),
            task: TASK,
            attempt: 1,
            resumed: false,
            budget: allowance.spend,
            account: Some(allowance.choice.accounts.first().expect("main account").account),
            writes: Box::new([]),
        },
    );
    send_task(core, limits, tasks::Event::Started { task: TASK, attempt: 1 });
    assert_eq!(
        core.tasks.task(TASK).expect("claimed task").phase,
        tasks::Phase::Active(tasks::Active::Running { attempt: 1 })
    );
    allowance.spend
}

#[test]
fn a_delegating_run_reserves_half_the_task_funds_up_to_the_cap() {
    for (budget, expected) in [(100, 50), (300, 100)] {
        let (mut core, limits) = funded(budget, true);
        assert_eq!(claim(&mut core, &limits), expected);
        let row = core.tasks.task(TASK).expect("claimed task");
        assert_eq!(row.numbers.reserved, expected);
        assert_eq!(row.run_reserved, expected);
    }
}

#[test]
fn small_delegating_runs_reserve_the_whole_up_to_the_cap() {
    for (budget, expected) in [(99, 99), (100, 100), (101, 100)] {
        let (mut core, limits) = funded_with_minimum(budget, 50);
        assert_eq!(claim(&mut core, &limits), expected);
        let row = core.tasks.task(TASK).expect("claimed task");
        assert_eq!(row.numbers.reserved, expected);
        assert_eq!(row.run_reserved, expected);
    }
}

#[test]
fn a_delegating_run_whose_half_is_at_the_minimum_reserves_the_whole() {
    for budget in [2, 3] {
        let (mut core, limits) = funded(budget, true);
        assert_eq!(claim(&mut core, &limits), budget);
        let row = core.tasks.task(TASK).expect("claimed task");
        assert_eq!(row.numbers.reserved, budget);
        assert_eq!(row.run_reserved, budget);
    }
}

#[test]
fn a_run_that_cannot_delegate_keeps_its_full_task_funds() {
    let (mut core, limits) = funded(100, false);
    assert_eq!(claim(&mut core, &limits), 100);
    assert_eq!(core.tasks.task(TASK).expect("claimed task").numbers.reserved, 100);
}

#[test]
fn a_whole_at_or_below_the_minimum_or_a_half_short_of_the_reserve_is_held_for_budget() {
    for (budget, reserve_turns, minimum) in [(0, 1, 1), (1, 1, 1), (4, 2, 1), (50, 1, 50)] {
        let (mut core, limits) = funded_with_minimum(budget, minimum);
        core.current.test_charter_mut(1).policy.reserve_turns = reserve_turns;
        assert_eq!(core.run_reserve(TASK), u64::from(reserve_turns) + 1);
        let context = core.tasks.run_context(TASK).expect("due task");
        assert_eq!(core.run_admission(&context, Wall::EPOCH, Box::new([])), RunAdmission::Hold(tasks::Hold::Budget));
        send_task(&mut core, &limits, tasks::Event::Hold { task: TASK, why: tasks::Hold::Budget });
        let row = core.tasks.task(TASK).expect("held task");
        assert_eq!(
            row.phase,
            tasks::Phase::Held { was: tasks::Was::Active(tasks::Active::Due), why: tasks::Hold::Budget, source: None }
        );
        assert_eq!(row.numbers.reserved, 0);
        assert_eq!(row.run_reserved, 0);
    }
}

#[test]
fn a_run_needs_the_reserve_at_its_default_speed() {
    for (budget, allowed) in [(19, false), (20, true)] {
        let (mut core, _) = funded(budget, false);
        let policy = &mut core.current.test_charter_mut(1).policy;
        let model = &mut policy.model;
        model.speeds = Box::new([Speed {
            name: b"fast".as_slice().into(),
            also: Box::default(),
            input_price: 1_000,
            cached_price: 1_000,
            cache_write_price: None,
            output_price: 1_000,
        }]);
        model.speed = Some(b"fast".as_slice().into());
        // 8,192 input and 1,024 output tokens, rounded up at 1,000 / 1,000,000,
        // cost 10 units per completion; the run needs two completions.
        let context = core.tasks.run_context(TASK).expect("due task");
        let admission = core.run_admission(&context, Wall::EPOCH, Box::new([]));
        if allowed {
            let RunAdmission::Allow(allowance) = admission else { panic!("at the exact floor: {admission:?}") };
            assert_eq!(allowance.spend, budget);
        } else {
            assert_eq!(admission, RunAdmission::Hold(tasks::Hold::Budget));
        }
        core.current.test_charter_mut(1).policy.model.speed = None;
        let admission = core.run_admission(&context, Wall::EPOCH, Box::new([]));
        let RunAdmission::Allow(allowance) = admission else { panic!("standard fits: {admission:?}") };
        assert_eq!(allowance.spend, budget);
    }
}
