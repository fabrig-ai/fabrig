//! A run leaves funds for delegates (jig/docs/design/domain/authority.md, 7).
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
fn a_run_that_cannot_delegate_keeps_its_full_task_funds() {
    let (mut core, limits) = funded(100, false);
    assert_eq!(claim(&mut core, &limits), 100);
    assert_eq!(core.tasks.task(TASK).expect("claimed task").numbers.reserved, 100);
}

#[test]
fn a_half_at_the_minimum_or_short_of_the_reserve_is_held_for_budget() {
    // At the minimum, or above it but short of a three-completion reserve floor.
    for (budget, reserve_turns) in [(2, 1), (3, 1), (4, 2)] {
        let (mut core, limits) = funded(budget, true);
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
    }
}
