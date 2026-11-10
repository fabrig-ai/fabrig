//! Closer reclamation (temper/docs/design/domain/forge.md, 11.2).
use super::*;

fn closer(task: u64, goal: u64, done: bool) -> CloserRow {
    CloserRow {
        task,
        goal,
        closer: temper_engine_domain_forge_closer::Closer::new(),
        making: None,
        close_changes: Box::new([]),
        landing_changes: Box::new([]),
        done,
    }
}

#[test]
fn a_forgotten_goal_reclaims_its_completed_closer_rows() {
    let mut d = domain();
    let live = closer(1, 7, false);
    let other = closer(2, 8, true);
    outputs(&mut d, Event::Closer { row: live.clone() });
    outputs(&mut d, Event::Closer { row: other.clone() });
    for task in 100..100 + u64::from(LIMITS.tasks) * 2 {
        outputs(&mut d, Event::Closer { row: closer(task, 7, true) });
        let erased = outputs(&mut d, Event::ForgetProjection { goal: 7 });
        assert!(d.closer(task).is_none(), "completed closer {task} retained");
        assert!(erased.contains(&Request::Erase { key: Key::Closer(task) }));
        assert_eq!(d.closer(1), Some(&live));
        assert_eq!(d.closer(2), Some(&other));
    }
}

#[test]
fn a_finished_closer_discards_its_containment_check() {
    let mut d = domain();
    outputs(&mut d, Event::Closer { row: closer(3, 7, false) });
    outputs(
        &mut d,
        Event::CheckCloser {
            task: 3,
            step: 1,
            repository: REPO,
            branch: b"g".as_slice().into(),
            base: b"main".as_slice().into(),
        },
    );
    assert!(!d.closing.reads.is_empty() || d.closer_checked(3, 1).is_some());
    outputs(&mut d, Event::Closer { row: closer(3, 7, true) });
    assert!(d.closing.reads.is_empty());
    assert_eq!(d.closer_checked(3, 1), None);
}
