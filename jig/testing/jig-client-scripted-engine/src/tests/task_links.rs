use super::*;

#[test]
fn task_list_filters_do_not_alias_person_task_and_deployment_requesters_with_equal_numbers() {
    for (requester, asked_by_person) in [
        (OwnerRequester::Person(1), true),
        (OwnerRequester::Task(1), false),
        (OwnerRequester::Deployment { project: 1 }, false),
    ] {
        for asked_by_me in [false, true] {
            let mut setup = script();
            let mut row = task();
            row.requester = requester;
            setup.truth.objects.insert(ObjectKey::Task(8), ReadValue::Task(Box::new(row)));
            let mut engine = Engine::new(setup, 5).unwrap();
            engine
                .open(
                    Time::ZERO,
                    11,
                    Token::new(3),
                    Subject::Tasks {
                        query: TaskQuery { requester: asked_by_me.then_some(OwnerRequester::Person(1)), ..query(1) },
                        window: 2,
                    },
                )
                .unwrap();
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
                assert_eq!(rows.get(0).unwrap().requester, requester);
                assert_eq!(rows.get(0).unwrap().requester == OwnerRequester::Person(1), asked_by_person);
            }
        }
    }
}
