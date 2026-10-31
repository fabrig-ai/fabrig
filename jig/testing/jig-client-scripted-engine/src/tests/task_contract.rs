use super::*;

#[test]
fn task_windows_filter_the_supplied_served_goal_without_reconstructing_it_from_lineage() {
    for supplied in [None, Some(0), Some(8), Some(u64::MAX)] {
        for wanted in [None, Some(0), Some(8), Some(9), Some(u64::MAX)] {
            let mut setup = script();
            let mut row = task();
            row.goal = supplied;
            row.lineage = list(supplied.into_iter().collect());
            let mut ancestor = task();
            ancestor.chip.task = 9;
            ancestor.project = 2;
            ancestor.goal = Some(9);
            setup.truth.objects.insert(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
            setup.truth.objects.insert(ObjectKey::Task(9), ReadValue::Task(Box::new(ancestor)));
            let mut engine = Engine::new(setup, 5).unwrap();
            engine
                .open(
                    Time::ZERO,
                    11,
                    Token::new(3),
                    Subject::Tasks { query: TaskQuery { root: wanted, ..query(1) }, window: 2 },
                )
                .unwrap();
            let up = engine.poll(Time::ZERO);
            let rows: Vec<_> = up
                .iter()
                .filter_map(|event| match event {
                    Up::Watch { event: WatchEvent::Delivered(Delivery::Tasks(value)), .. } => Some(&value.rows),
                    Up::Answer { .. }
                    | Up::Read { .. }
                    | Up::Watch { .. }
                    | Up::ApplicationAnswer { .. }
                    | Up::ApplicationRead { .. }
                    | Up::App { .. }
                    | Up::Ended { .. } => None,
                })
                .collect();
            assert_eq!(rows.len(), 1);
            if wanted.is_none() || wanted == Some(supplied.unwrap_or(8)) {
                assert_eq!(rows[0].len(), 1);
                assert_eq!(rows[0].get(0).unwrap().chip.task, 8);
            } else {
                assert!(rows[0].is_empty());
            }
        }
    }
}
