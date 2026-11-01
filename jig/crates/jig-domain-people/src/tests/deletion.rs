use super::*;

#[test]
fn a_deletion_row_is_saved_restored_and_erased() {
    let mut test = Test::new(LIMITS);
    assert_eq!(test.d.deletion(3), None);
    for stage in [Stage::Ending, Stage::Erasing, Stage::Releasing] {
        let row = Stored::Deletion { project: 3, stage };
        assert_eq!(
            test.send(Event::Deletion { project: 3, stage: Some(stage) }),
            [Request::Save { record: row.clone() }]
        );
        assert_eq!(test.d.deletion(3), Some(stage));
        let mut restored = Test::new(LIMITS);
        restored.d = Domain::new(&LIMITS, Box::new([]), 2, Box::new([]));
        assert!(restored.send(Event::Restore { record: row }).is_empty());
        assert!(restored.send(Event::Restored).is_empty());
        assert_eq!(restored.d.deletion(3), Some(stage));
        assert_eq!(restored.d.deletions().len(), 1);
        assert_eq!(
            restored.send(Event::Deletion { project: 3, stage: None }),
            [Request::Erase { key: Key::Deletion(3) }]
        );
        assert!(restored.d.deletions().is_empty());
        assert!(restored.send(Event::Deletion { project: 3, stage: None }).is_empty());
        assert!(restored.send(Event::EraseTask { project: 3, task: 7 }).is_empty());
        assert!(restored.send(Event::DropProject { project: 3 }).is_empty());
    }
}

#[test]
fn a_deployment_act_needs_its_grant_for_both_new_kinds() {
    for (kind, act) in [
        (ActKind::RemoveProject, Act::RemoveProject { project: 3 }),
        (ActKind::Cancel, Act::Cancel { project: 3, task: 7, reason: Box::new([1]) }),
    ] {
        for granted in [false, true] {
            let mut test = Test::new(LIMITS);
            let may: Box<[ActKind]> = if granted { Box::new([kind]) } else { Box::new([]) };
            test.d = Domain::new(
                &LIMITS,
                Box::new([InitialOwner { project: 0, identity: identity(0, 1).key }]),
                2,
                Box::new([ActGrant { connector: 1, may }]),
            );
            test.send(Event::Restored);
            let reply_to = test.to();
            let out = test.send(Event::Act { reply_to, connector: 1, key: [1; 16], act: Box::new(act.clone()) });
            if granted {
                assert_eq!(out.len(), 1);
                let [Request::Act { connector, act: routed, .. }] = &out[..] else { panic!("one deployment route") };
                assert_eq!(*connector, 1);
                assert_eq!(**routed, act);
            } else {
                let [Request::Acted { outcome, .. }] = &out[..] else { panic!("one ungranted act refusal") };
                assert_eq!(*outcome, Outcome::Refused(Refusal::NotOffered));
            }
        }
    }
}

#[test]
fn an_add_key_with_too_many_uses_is_refused() {
    for uses in [
        (1..=USES + 1).map(Use::Project).collect::<Vec<_>>().into_boxed_slice(),
        Box::new([Use::Project(3), Use::Project(3)]),
    ] {
        let mut test = Test::new(LIMITS);
        let reply_to = test.to();
        let out = test.send(Event::Act {
            reply_to,
            connector: 1,
            key: [1; 16],
            act: Box::new(Act::AddKey { provider: 1, join: None, digest: [2; 32], uses: Some(uses) }),
        });
        let [Request::Acted { outcome, .. }] = &out[..] else { panic!("one act use limit refusal") };
        assert_eq!(*outcome, Outcome::Refused(Refusal::Limit { which: LimitKind::Resource }));
    }
}

#[test]
fn deletion_rows_fit_the_selected_start_and_survive_its_sweep() {
    let mut test = Test::new(LIMITS);
    test.d = Domain::new(&LIMITS, Box::new([]), 2, Box::new([]));
    test.d.start_restore(Wall::EPOCH);
    for project in 1..=LIMITS.projects {
        assert!(test.send(Event::Restore { record: Stored::Deletion { project, stage: Stage::Erasing } }).is_empty());
    }
    assert!(test.d.start_link(&test.env));
    test.d.start_sweep();
    let mut out = Queue::uncapped();
    test.d.start_sweep_row(&test.env, &Stored::Deletion { project: 1, stage: Stage::Erasing }, &mut out);
    assert!(out.is_empty());
    assert!(test.d.start_complete(&test.env));
    assert_eq!(test.d.deletions().len(), LIMITS.projects);
    assert_eq!(test.d.deletion(1), Some(Stage::Erasing));
}

#[test]
fn a_deletion_restore_refuses_zero_duplicates_and_capacity() {
    for selected in [false, true] {
        let mut test = Test::new(LIMITS);
        test.d = Domain::new(&LIMITS, Box::new([]), 2, Box::new([]));
        if selected {
            test.d.start_restore(Wall::EPOCH);
        }
        assert_eq!(
            test.send(Event::Restore { record: Stored::Deletion { project: 0, stage: Stage::Ending } }),
            [Request::RestoreRefused { key: Key::Deletion(0), refusal: Refusal::Unknown }]
        );
        assert!(!test.d.ready());
    }
    let mut test = Test::new(LIMITS);
    test.d = Domain::new(&LIMITS, Box::new([]), 2, Box::new([]));
    let row = Stored::Deletion { project: 1, stage: Stage::Ending };
    assert!(test.send(Event::Restore { record: row.clone() }).is_empty());
    assert_eq!(
        test.send(Event::Restore { record: row }),
        [Request::RestoreRefused { key: Key::Deletion(1), refusal: Refusal::Limit { which: LimitKind::Restore } }]
    );
    for selected in [false, true] {
        let mut test = Test::new(LIMITS);
        test.d = Domain::new(&LIMITS, Box::new([]), 2, Box::new([]));
        if selected {
            test.d.start_restore(Wall::EPOCH);
        }
        for project in 1..=LIMITS.projects {
            assert!(
                test.send(Event::Restore { record: Stored::Deletion { project, stage: Stage::Ending } }).is_empty()
            );
        }
        assert_eq!(
            test.send(Event::Restore { record: Stored::Deletion { project: 4, stage: Stage::Ending } }),
            [Request::RestoreRefused { key: Key::Deletion(4), refusal: Refusal::Limit { which: LimitKind::Restore } }]
        );
        assert_eq!(test.d.deletions().len(), LIMITS.projects);
    }
}

#[test]
fn a_remove_project_request_routes_its_project_and_replays_its_answer() {
    let mut test = Test::new(LIMITS);
    test.signin(1, 10, identity(0, 1));
    test.send(Event::Roles { project: 3, holdings: Box::new([Holding { person: 1, role: Role::Owner }]) });
    let ask = Ask::RemoveProject { project: 3 };
    let routed = test.request(10, 1, ask.clone());
    let [Request::Route { project, role, .. }] = &routed[..] else { panic!("one project request route") };
    assert_eq!(*project, 3);
    assert_eq!(*role, Some(Role::Owner));
    let request = route(&routed);
    let outcome = Outcome::ProjectRemoved { project: 3 };
    assert_eq!(
        reply(&test.send(Event::Decided { request, outcome: outcome.clone() })),
        Reply::Outcome(outcome.clone())
    );
    assert_eq!(reply(&test.request(10, 1, ask)), Reply::Outcome(outcome));
}

#[test]
fn an_add_keys_uses_restore_at_the_fixed_use_bound() {
    let mut test = Test::new(LIMITS);
    test.d = Domain::new(&LIMITS, Box::new([]), 2, Box::new([]));
    test.d.start_restore(Wall::EPOCH);
    let uses = (1..=USES).map(Use::Project).collect::<Vec<_>>().into_boxed_slice();
    let row = Stored::Acted {
        key: ActKey { connector: 1, key: [1; 16] },
        act: Act::AddKey { provider: 1, join: None, digest: [1; 32], uses: Some(uses) },
        outcome: Outcome::Refused(Refusal::NotOffered),
        at: Wall::EPOCH,
    };
    assert!(test.send(Event::Restore { record: row }).is_empty());
    assert!(test.d.start_link(&test.env));
    test.d.start_sweep();
    assert!(test.d.start_complete(&test.env));
}
