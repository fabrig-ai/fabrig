//! Project funding closes after its last live task, preserving spend in history
//! (jig/docs/design/domain/tasks.md, 7).
use crate::{Domain, Funder, Key, Request, Scope};
use skein_lib::{List, Queue};

pub(crate) fn close_project(domain: &mut Domain, project: u32, out: &mut Queue<Request>) {
    assert!(domain.ready(), "project closure follows restoration");
    for (number, _) in &domain.names {
        assert_ne!(
            crate::domain::record(domain, *number).expect("indexed live task").project,
            project,
            "project closure follows its last live task"
        );
    }
    let mut allotments = List::with_capacity(domain.funding.len());
    let mut periods = List::with_capacity(domain.funding.len());
    for (funder, _) in &domain.funding {
        match *funder {
            Funder::Pool { project: owner, .. } | Funder::Recurring { project: owner, .. } => {
                if owner == project {
                    allotments.push(*funder).expect("one entry per funder");
                }
            }
            Funder::Period { project: owner, .. } => {
                if owner == project {
                    periods.push(*funder).expect("one entry per funder");
                }
            }
            Funder::Task(_) => unreachable!("task funding lives in task rows"),
        }
    }
    for funder in allotments.into_boxed() {
        let row = *domain.funding.get(&funder).expect("selected allotment");
        crate::retire::close_allotment(domain, funder, row.parent.expect("allotment's period"), row.numbers, out);
    }
    for funder in periods.into_boxed() {
        assert_eq!(domain.funding.get(&funder).expect("selected period").numbers.reserved, 0, "period settled");
        crate::retire::close(domain, funder, out);
    }
    let mut holds = List::with_capacity(domain.placed.len());
    for (number, hold) in &domain.placed {
        if hold.project == project && hold.scope == Scope::Project {
            holds.push(*number).expect("one entry per placed hold");
        }
    }
    for number in holds.into_boxed() {
        domain.placed.remove(&number);
        out.push(Request::Erase { key: Key::PlacedHold(number) });
    }
}
