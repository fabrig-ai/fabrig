//! Stamping at the common creation path (jig/docs/design/domain/engine/project-charters.md, 3–4).
use super::revisions::{Harness, task, to};
use crate::{Event, Executor, Funder, Party, PersonAddress};

#[test]
fn task_records_the_projects_newest_set_not_its_parents() {
    let mut h = Harness::new();
    h.domain.project_charters.raise(1);
    h.make([task(1, &[])].into());
    assert_eq!(h.domain.task(1).expect("task without set").charters, 0);
    h.domain.set_project_charters(1, 7);
    let mut child = task(2, &[]);
    child.numbers.budget = 10;
    child.authority.budget.spend = 10;
    child.funder = Funder::Task(1);
    let out = h.send(Event::Make {
        reads_project: false,
        maker: None,
        places: Box::new([]),
        reply_to: to(),
        creator: Party::Task(1),
        batch: Box::new([child]),
    });
    assert_eq!(h.domain.task(1).expect("parent unchanged").charters, 0);
    let Some(child) = h.domain.task(2) else { panic!("child not made: {out:?}") };
    assert_eq!(child.charters, 7);
    assert!(h.domain.holds_project_charters(1, 7));
    h.send(Event::OpenPeriod { reply_to: to(), project: 0, period: 0, budget: 100 });
    let mut deployment = task(3, &[]);
    deployment.project = 0;
    deployment.funder = Funder::Period { project: 0, period: 0 };
    h.make([deployment].into());
    assert_eq!(h.domain.task(3).expect("task without a project").charters, 0);
}

#[test]
fn non_agents_record_but_do_not_hold_their_set() {
    let mut h = Harness::new();
    h.domain.project_charters.raise(1);
    h.domain.set_project_charters(1, 7);
    for (number, executor) in
        [(1, Executor::Person(PersonAddress::Person(5))), (2, Executor::Procedure { connector: 1, code: 1 })]
    {
        let mut member = task(number, &[]);
        member.executor = executor;
        h.make([member].into());
        assert_eq!(h.domain.task(number).expect("live non-agent task").charters, 7);
        assert!(!h.domain.holds_project_charters(1, 7));
    }
}
