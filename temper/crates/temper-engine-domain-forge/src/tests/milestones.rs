//! Milestone retention and reads (temper/docs/design/domain/milestones.md, 8).
use super::*;

fn milestone(task: u64, at: Option<u64>) -> MilestoneRow {
    MilestoneRow {
        project: 5,
        task,
        name: b"Store shape".as_slice().into(),
        last_landing: None,
        ended: match at {
            Some(at) => Some(MilestoneEnd {
                at: Wall::from_nanos(at),
                outcome: MilestoneOutcome::Complete,
                done: 1,
                failed: 0,
                closed: 0,
                spent: 7,
            }),
            None => None,
        },
    }
}

#[test]
fn a_full_project_drops_its_earliest_end_but_never_a_live_row() {
    let mut d = domain();
    for task in 1..=256 {
        outputs(&mut d, Event::Milestone { row: milestone(task, None) });
    }
    outputs(&mut d, Event::Milestone { row: milestone(2, Some(30)) });
    outputs(&mut d, Event::Milestone { row: milestone(3, Some(10)) });
    let mut other = milestone(1000, Some(1));
    other.project = 6;
    outputs(&mut d, Event::Milestone { row: other.clone() });
    let out = outputs(&mut d, Event::Milestone { row: milestone(257, None) });
    assert!(out.contains(&Request::Erase { key: Key::Milestone(3) }));
    assert_eq!(d.milestone(3), None);
    assert_eq!(d.milestone(1), Some(&milestone(1, None)));
    assert_eq!(d.milestone(2), Some(&milestone(2, Some(30))));
    assert_eq!(d.milestone(1000), Some(&other));
    assert_eq!(d.milestones.len(), 257, "256 of this project and one of another");
}

#[test]
fn live_rows_can_pass_256_and_restore_then_fall_back_as_they_end() {
    let mut d = domain();
    for task in 1..=258 {
        let out = outputs(&mut d, Event::Milestone { row: milestone(task, None) });
        for request in &out {
            assert!(!matches!(request, Request::Erase { .. }));
        }
    }
    assert_eq!(d.milestones.len(), 258, "every live milestone keeps its row");
    let mut restored = domain();
    for (_, row) in &d.milestones {
        outputs(&mut restored, Event::Restore { record: Stored::Milestone(row.clone()) });
    }
    assert_eq!(restored.fit_restored(&LIMITS), Ok(()));
    assert!(restored.milestones.iter().eq(d.milestones.iter()), "all live rows load back");
    for task in 1..=2 {
        outputs(&mut restored, Event::Milestone { row: milestone(task, Some(task)) });
        assert!(restored.milestone(task).is_none(), "no ended row while live count exceeds 256");
    }
    assert_eq!(restored.milestones.len(), 256);
    outputs(&mut restored, Event::Milestone { row: milestone(3, Some(3)) });
    assert_eq!(restored.milestone(3), Some(&milestone(3, Some(3))));
    assert_eq!(restored.milestones.len(), 256);
}

#[test]
fn milestones_changes_and_project_facts_load_back_together() {
    let mut d = domain();
    let mut row = milestone(40, Some(50));
    row.last_landing = Some(MilestoneLanding { at: Wall::from_nanos(30), goal: 41, change: fixtures::CHANGE });
    let mut change = fixtures::opening_row();
    change.goal = Some(41);
    change.milestone = Some(40);
    let (cold, hot) = change.clone().split();
    let Stored::ProjectFacts(facts) = restore_fit::row(RecordKind::ProjectFacts, 5) else { panic!("facts fixture") };
    for record in [
        Stored::ProjectFacts(facts.clone()),
        Stored::ChangeCold(Box::new(cold)),
        Stored::ChangeHot(Box::new(hot)),
        Stored::Milestone(row.clone()),
    ] {
        outputs(&mut d, Event::Restore { record });
    }
    assert_eq!(d.fit_restored(&LIMITS), Ok(()));
    assert_eq!(d.milestone(40), Some(&row));
    assert_eq!(d.change(fixtures::CHANGE), Some(&change));
    assert_eq!(d.project_facts.get(&facts.project), Some(&facts));
    assert_eq!(d.ended_milestones(5, Wall::from_nanos(50), 1).as_slice(), &[row]);
}

#[test]
fn ended_milestones_are_project_scoped_inclusive_newest_first_and_counted() {
    let mut d = domain();
    for row in [milestone(1, Some(10)), milestone(2, Some(30)), milestone(3, Some(20)), milestone(4, None)] {
        outputs(&mut d, Event::Milestone { row });
    }
    let mut other = milestone(5, Some(40));
    other.project = 6;
    outputs(&mut d, Event::Milestone { row: other });
    assert_eq!(
        d.ended_milestones(5, Wall::from_nanos(20), 10).as_slice(),
        &[milestone(2, Some(30)), milestone(3, Some(20))]
    );
    assert_eq!(d.ended_milestones(5, Wall::EPOCH, 1).as_slice(), &[milestone(2, Some(30))]);
    assert!(d.ended_milestones(5, Wall::EPOCH, 0).is_empty());
    assert!(d.ended_milestones(5, Wall::from_nanos(31), 10).is_empty());
}
