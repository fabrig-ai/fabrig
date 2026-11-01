//! Whole-task erasure releases precisely the task's historical receipts.
use super::*;

#[test]
fn an_erased_task_releases_turns_and_terminals_but_keeps_other_tasks_receipts() {
    let mut fake = Fake::new(Recovery::Keyed);
    let mut rows = snapshot();
    rows.tasks.get_mut(&1).expect("task").phase = Phase::Done;
    let erased = [Receipt::Task(1), Receipt::Ended(1), Receipt::Turn(1, 1, 1), Receipt::Terminal(1, 1)];
    rows.receipts.extend(erased.clone());
    rows.receipts.insert(Receipt::Task(2));
    fake.commit(1, rows).expect("durable ended task");
    fake.send(Observed::Released { requires: erased.into_iter().chain([Receipt::Task(2)]).collect() })
        .expect("outside receipts");
    fake.commit(2, Snapshot { receipts: BTreeSet::from([Receipt::Task(2)]), ..Snapshot::default() })
        .expect("owner erased");
    fake.send(Observed::Erased { task: 1 }).expect("whole task erased");
    let mut cold = fake.referee.clone();
    cold.observe(fake.now, Observed::ColdStore { receipts: BTreeSet::from([Receipt::Task(2)]) })
        .expect("erased receipts released");
    fake.caught(Observed::ColdStore { receipts: BTreeSet::new() }, Promise::Commit);
}

#[test]
fn erasing_a_live_task_is_rejected_and_forgetting_keeps_turn_receipts() {
    let mut fake = Fake::ready(Recovery::Keyed);
    fake.caught(Observed::Erased { task: 1 }, Promise::Commit);
    let mut fake = Fake::new(Recovery::Keyed);
    let mut rows = snapshot();
    rows.tasks.get_mut(&1).expect("task").phase = Phase::Done;
    rows.receipts.insert(Receipt::Turn(1, 1, 1));
    fake.commit(1, rows).expect("durable task");
    fake.send(Observed::Released { requires: vec![Receipt::Turn(1, 1, 1)] }).expect("turn seen");
    fake.commit(2, Snapshot::default()).expect("owner forgotten");
    fake.send(Observed::Forgotten { task: 1 }).expect("owner gone");
    fake.caught(Observed::ColdStore { receipts: BTreeSet::new() }, Promise::Commit);
}
