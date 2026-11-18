//! A claimed run leaves a delegate funded while it runs (domain/authority.md, 7).
use crate::tests::runs::spend::{claim, funded};
use crate::tests::*;

#[test]
fn a_live_run_leaves_room_for_a_forty_unit_delegate_in_a_hundred_unit_task() {
    let (mut core, limits) = funded(100, true);
    claim(&mut core, &limits);
    let mut child = new_row(&core, 2, tasks::Funder::Task(1), None);
    child.numbers.budget = 40;
    child.authority.budget.spend = 40;
    child.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let context = core.tasks.delegation(1).expect("running creator");
    let mut findings = Queue::uncapped();
    let checked = core.connector_batch_admit(
        context.project,
        &context.authority,
        context.numbers,
        context.tasks_left,
        Box::new([authority::Delegate {
            executor: authority::Executor::Charter(1),
            authority: tasks_authority(&child.authority),
            symbolic: Box::new([]),
        }]),
        &mut findings,
    );
    assert_eq!(checked.answer, authority::Answer::Allow, "{findings:?}");
    fixtures::make_batch(&mut core, &limits, tasks::Party::Task(1), Box::new([child]));
    let delegate = core.tasks.task(2).expect("delegate admitted during the run");
    assert_eq!(delegate.requester, tasks::Party::Task(1));
    assert_eq!(delegate.numbers.budget, 40);
    let row = core.tasks.task(1).expect("running creator");
    assert_eq!(row.run_reserved, 50);
    assert_eq!(row.numbers.reserved, 90);
    assert!(row.numbers.spent + row.numbers.spent_below + row.numbers.reserved <= 100);
}
