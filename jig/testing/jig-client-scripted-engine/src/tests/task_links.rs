use super::*;

#[test]
fn task_list_filters_do_not_alias_person_task_and_deployment_requesters_with_equal_numbers() {
    for (requester, asked_by_person) in [
        (Requester::Person(REQUESTER), true),
        (Requester::Task(1), false),
        (Requester::Deployment { project: PROJECT }, false),
    ] {
        for asked_by_me in [false, true] {
            let mut setup = script();
            let mut row = task();
            row.requester = requester;
            setup.truth.objects.insert(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
            let mut engine = Engine::new(setup, 5).expect("valid script");
            engine
                .open(
                    Time::ZERO,
                    REQUESTER_SIGN_IN,
                    Token::new(3),
                    Subject::Tasks {
                        query: TaskQuery {
                            requester: asked_by_me.then_some(Requester::Person(REQUESTER)),
                            ..query(PROJECT)
                        },
                        window: 2,
                    },
                )
                .expect("typed watch fits");
            let up = engine.poll(Time::ZERO);
            let deliveries: Vec<_> = up
                .iter()
                .filter_map(|event| match event {
                    Up::Watch { event: WatchEvent::Delivered(Delivery::Tasks(rows)), .. } => Some(rows),
                    Up::Answer { .. }
                    | Up::Read { .. }
                    | Up::Watch { .. }
                    | Up::ApplicationAnswer { .. }
                    | Up::ApplicationRead { .. }
                    | Up::App { .. }
                    | Up::Ended { .. } => None,
                })
                .collect();
            assert_eq!(deliveries.len(), 1);
            let rows = &deliveries[0].rows;
            if asked_by_me && !asked_by_person {
                assert!(rows.is_empty());
            } else {
                assert_eq!(rows.len(), 1);
                assert_eq!(rows.get(0).expect("delivered task row").requester, requester);
                assert_eq!(
                    rows.get(0).expect("delivered task row").requester == Requester::Person(REQUESTER),
                    asked_by_person
                );
            }
        }
    }
}
