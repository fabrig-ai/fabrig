use super::*;

fn configured(under_box: bool) -> Domain {
    let mut d = domain();
    d.under_box = under_box;
    for number in [1, 2] {
        d.configured
            .insert(
                number,
                Configured {
                    admin_connect: false,
                    bot: Box::new([]),
                    owned: None,
                    git: Box::new([]),
                    web: None,
                    read_only: false,
                    number,
                    kind: Kind::Forgejo,
                    mode: Mode::Fake,
                    asked: true,
                },
            )
            .unwrap();
    }
    d.connection_limits(3, 1);
    d.configure_demo(Some(DemoConfig {
        project_name: "Playground".into(),
        project_policy: 1,
        provider: 1,
        adoption: fixtures::adoption(),
        deadline: Duration::from_secs(10),
    }));
    d
}

#[test]
fn demo_acts_are_saved_before_send_restored_kept_and_erased_on_answer() {
    let rows = Box::new([
        DemoAct { key: [1; 16], act: DemoActKind::Close { project: 5, task: 7 } },
        DemoAct { key: [2; 16], act: DemoActKind::RemoveAccount { account: 3 } },
    ]);
    let mut d = configured(false);
    let sent = outputs(&mut d, Event::DemoActs { acts: rows.clone() });
    for (index, row) in rows.iter().enumerate() {
        assert_eq!(sent[index], Request::Save { read_afresh: None, release: None, record: Stored::DemoAct(*row) });
        assert_eq!(sent[index + 2], Request::SendDemoAct(*row));
    }
    let mut restored = configured(false);
    for row in rows.as_ref() {
        assert!(outputs(&mut restored, Event::Restore { record: Stored::DemoAct(*row) }).is_empty());
    }
    assert_eq!(
        outputs(&mut restored, Event::SendDemoActs).as_ref(),
        [Request::SendDemoAct(rows[0]), Request::SendDemoAct(rows[1])]
    );
    assert!(outputs(&mut restored, Event::DemoActAnswered { key: rows[0].key, keep: true }).is_empty());
    for row in rows.as_ref() {
        assert_eq!(
            outputs(&mut restored, Event::DemoActAnswered { key: row.key, keep: false }).as_ref(),
            [Request::Erase { key: Key::DemoAct(row.key) }]
        );
    }
    assert!(outputs(&mut restored, Event::SendDemoActs).is_empty());
}

#[test]
fn demo_act_room_is_tasks_plus_one_and_reclaimed_on_answer() {
    let mut d = configured(false);
    let bound = demo_act_bound(&LIMITS).unwrap();
    assert_eq!(bound, LIMITS.tasks + 1);
    for number in 0..bound {
        let key = [u8::try_from(number).expect("small act bound"); 16];
        outputs(
            &mut d,
            Event::Restore {
                record: Stored::DemoAct(DemoAct {
                    key,
                    act: DemoActKind::Close { project: 5, task: u64::from(number) },
                }),
            },
        );
    }
    assert_eq!(outputs(&mut d, Event::SendDemoActs).len(), usize::try_from(bound).unwrap());
    outputs(&mut d, Event::DemoActAnswered { key: [0; 16], keep: false });
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::DemoAct(DemoAct { key: [255; 16], act: DemoActKind::RemoveAccount { account: 3 } }),
        },
    );
    assert_eq!(outputs(&mut d, Event::SendDemoActs).len(), usize::try_from(bound).unwrap());
    let mut overflowing = LIMITS;
    overflowing.tasks = u32::MAX;
    assert_eq!(demo_act_bound(&overflowing), None);
    assert!(!valid(&overflowing));
}

#[test]
fn restoring_more_than_tasks_plus_one_demo_acts_is_refused() {
    let mut d = configured(false);
    for number in 0..=demo_act_bound(&LIMITS).unwrap() {
        let key = [u8::try_from(number).expect("small act bound"); 16];
        outputs(
            &mut d,
            Event::Restore {
                record: Stored::DemoAct(DemoAct {
                    key,
                    act: DemoActKind::Close { project: 5, task: u64::from(number) },
                }),
            },
        );
    }
    assert_eq!(d.restore_fit_failure().unwrap().set, RestoreSet::Rows(RecordKind::DemoAct));
}
