//! Both ends of a writer slot remain linkable through release and restart.
use super::*;
use jig_domain_tasks as tasks;

const HOLDER: u64 = 7;
const DELEGATE: u64 = 9;

fn tasks(live: &[u64]) -> tasks::Domain {
    let mut world = jig_tasks_kit::World::new(71, jig_tasks_kit::LIMITS);
    assert_eq!(
        world.make(jig_tasks_kit::PERSON_PARTY, vec![jig_tasks_kit::task(HOLDER, &[])]),
        jig_tasks_kit::Reply::Made(vec![HOLDER])
    );
    assert_eq!(
        world.make(tasks::Party::Task(HOLDER), vec![jig_tasks_kit::task(DELEGATE, &[])]),
        jig_tasks_kit::Reply::Made(vec![DELEGATE])
    );
    let mut domain = jig_tasks_kit::domain(&world.env.limits, 71);
    for number in live {
        tasks::step(
            &mut domain,
            &world.env,
            tasks::Event::Restore { record: tasks::Stored::Live(Box::new(world.record(*number).clone())) },
            &mut Queue::uncapped(),
        );
    }
    domain
}

fn restored(hold: Hold, release: Option<ReleaseRow>) -> Domain {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restore { record: Stored::Hold(hold) });
    if let Some(row) = release {
        outputs(&mut d, Event::Restore { record: Stored::Release(row) });
    }
    d
}

fn linkable(d: &Domain, live: &[u64]) {
    let copy = restored(d.hold(&run_branch(b"7")).unwrap().clone(), d.releases.get(&DELEGATE).cloned());
    assert_eq!(copy.missing_link(&tasks(live)), None);
}

#[test]
fn a_delegates_release_waits_for_its_writer_reread_even_after_restart() {
    for (lost, restart, refused) in
        [(true, false, false), (true, true, false), (false, false, false), (true, false, true)]
    {
        let mut d = domain();
        outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
        outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        outputs(&mut d, Event::CatchUp);
        let name = run_branch(b"7");
        outputs(&mut d, Event::Hold { entry: 0, task: HOLDER, resource: name.clone(), from: None });
        linkable(&d, &[HOLDER, DELEGATE]);
        outputs(
            &mut d,
            Event::Claim {
                designs: Box::new([]),
                task: DELEGATE,
                attempt: 1,
                writes: Box::new([name.clone()]),
                holders: Box::new([HOLDER]),
            },
        );
        linkable(&d, &[HOLDER, DELEGATE]);
        if lost {
            outputs(&mut d, Event::Lost { task: DELEGATE, attempt: 1 });
            linkable(&d, &[HOLDER, DELEGATE]);
        }
        let closing = outputs(
            &mut d,
            Event::Release { task: DELEGATE, root: HOLDER, ending: ReleaseEnding::Done, reason: None, entry: 20 },
        );
        assert!(!closing.contains(&Request::Released { task: DELEGATE }), "the delegate still owns its writer slot");
        assert!(!release_failed(&closing, DELEGATE));
        linkable(&d, &[HOLDER, DELEGATE]);
        if restart {
            d = restored(d.hold(&name).unwrap().clone(), d.releases.get(&DELEGATE).cloned());
            assert_eq!(d.start_links(&tasks(&[HOLDER, DELEGATE]), &env(), &mut Queue::uncapped()), Ok(()));
            outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
            outputs(&mut d, Event::CatchUp);
            linkable(&d, &[HOLDER, DELEGATE]);
        }
        let error = if refused { skein_forge::api::Error::Refused } else { skein_forge::api::Error::Missing };
        let answered = fixtures::answer_next(&mut d, Err(error));
        linkable(&d, &[HOLDER, DELEGATE]);
        if refused {
            assert!(release_failed(&answered, DELEGATE));
            assert!(!answered.contains(&Request::ContinueRelease { task: DELEGATE }));
            assert_eq!(d.hold(&name).unwrap().writer, Some(Writer::Lost { task: DELEGATE, attempt: 1 }));
        } else {
            assert!(answered.contains(&Request::ContinueRelease { task: DELEGATE }));
            assert_eq!(d.hold(&name).unwrap().writer, None);
            let released = outputs(&mut d, Event::ContinueRelease { task: DELEGATE, entry: 21 });
            assert!(released.contains(&Request::Released { task: DELEGATE }));
            linkable(&d, &[HOLDER]);
            assert_eq!(d.hold(&name).unwrap().task, HOLDER, "a delegate does not release its holder's branch");
        }
    }
}

#[test]
fn retired_run_and_lost_writers_open_writerless_and_read_before_claiming() {
    for writer in [Writer::Run { task: DELEGATE, attempt: 1 }, Writer::Lost { task: DELEGATE, attempt: 1 }] {
        let name = run_branch(b"7");
        let hold = Hold {
            name: name.clone(),
            task: HOLDER,
            writer: Some(writer),
            beside: None,
            design_writers: Box::new([]),
            drift: None,
        };
        let mut d = restored(hold.clone(), None);
        let tasks = tasks(&[HOLDER]);
        assert!(d.missing_link(&tasks).is_some(), "the old record names a retired task");
        let mut out = Queue::uncapped();
        assert_eq!(d.start_links(&tasks, &env(), &mut out), Ok(()));
        assert_eq!(d.hold(&name).unwrap().writer, None);
        assert_eq!(d.missing_link(&tasks), None);
        assert!(out.is_empty(), "the old durable writer keeps the fresh read due across a crash in Link");
        // Another cold start before the read sees the same old record and repairs it again.
        d = restored(hold, None);
        assert_eq!(d.start_links(&tasks, &env(), &mut out), Ok(()));
        outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        outputs(&mut d, Event::CatchUp);
        let blocked = outputs(
            &mut d,
            Event::Claim {
                designs: Box::new([]),
                task: HOLDER,
                attempt: 2,
                writes: Box::new([name.clone()]),
                holders: Box::new([]),
            },
        );
        assert!(blocked.is_empty(), "the claim waits for the repository's fresh read: {blocked:?}");
        assert_eq!(d.hold(&name).unwrap().writer, None);
        let (call, op) = fixtures::next_call(&mut d);
        assert_eq!(
            op,
            skein_forge::api::Op::Read(skein_forge::api::Read::Branch { branch: b"temper/7".as_slice().into() })
        );
        fixtures::answer(&mut d, call, Err(skein_forge::api::Error::Refused));
        let blocked = outputs(
            &mut d,
            Event::Claim {
                designs: Box::new([]),
                task: HOLDER,
                attempt: 2,
                writes: Box::new([name.clone()]),
                holders: Box::new([]),
            },
        );
        assert!(blocked.is_empty(), "the claim waits for the repository's fresh read: {blocked:?}");
        assert_eq!(d.hold(&name).unwrap().writer, None);
        outputs(
            &mut d,
            Event::Connection { forge: 1, state: Connection::Connected { id: 7, login: b"bot".as_slice().into() } },
        );
        ready(&mut d);
        let (call, retried) = fixtures::next_call(&mut d);
        assert_eq!(retried, op);
        let read = fixtures::answer(&mut d, call, Ok(skein_forge::api::Answer::Commit([2; 32])));
        let mut saved = false;
        for request in &read {
            if let Request::Save { record: Stored::Hold(row), .. } = request {
                saved |= row.writer.is_none();
            }
        }
        assert!(saved, "the fresh answer saves the repaired hold");
        assert_eq!(d.branch_head(&name), Some([2; 32]));
        assert_eq!(d.missing_link(&tasks), None);
        let claimed = outputs(
            &mut d,
            Event::Claim {
                designs: Box::new([]),
                task: HOLDER,
                attempt: 2,
                writes: Box::new([name.clone()]),
                holders: Box::new([]),
            },
        );
        assert!(!claimed.contains(&Request::Taken { task: HOLDER, resource: name.clone(), by: HOLDER }));
        assert_eq!(d.hold(&name).unwrap().writer, Some(Writer::Run { task: HOLDER, attempt: 2 }));
    }
}

#[test]
fn an_ending_writer_waits_for_its_foreign_hold_to_let_go() {
    use skein_forge::api::{Answer, Error, Op, Read};

    const HOLDER: u64 = 3;
    const WRITER: u64 = 4;
    const ATTEMPT: u64 = 2;

    for (design, result) in [
        (false, Err(Error::Missing)),
        (true, Ok(Answer::Commit([7; 32]))),
        (false, Err(Error::Refused)),
        (true, Err(Error::Refused)),
    ] {
        let mut d = fixtures::connected_domain();
        outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
        outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        outputs(&mut d, Event::CatchUp);
        let name = Name {
            forge: 1,
            repository: 2,
            what: What::Branch(Box::new([
                Box::from(b"temper".as_slice()),
                Box::from(b"1".as_slice()),
                Box::from(if design { b"g3".as_slice() } else { b"c3".as_slice() }),
            ])),
        };
        outputs(
            &mut d,
            Event::Restore {
                record: Stored::Hold(Hold {
                    name: name.clone(),
                    task: HOLDER,
                    writer: if design { None } else { Some(Writer::Run { task: WRITER, attempt: ATTEMPT }) },
                    design_writers: if design {
                        Box::new([DesignWriter { task: WRITER, attempt: ATTEMPT, rereading: false }])
                    } else {
                        Box::new([])
                    },
                    beside: None,
                    drift: None,
                }),
            },
        );
        outputs(&mut d, Event::Lost { task: WRITER, attempt: ATTEMPT });
        let (call, op) = fixtures::next_call(&mut d);
        assert!(matches!(op, Op::Read(Read::Branch { .. })));
        let release = outputs(
            &mut d,
            Event::Release { task: WRITER, root: HOLDER, ending: ReleaseEnding::Done, reason: None, entry: 20 },
        );
        assert!(
            !release.contains(&Request::Released { task: WRITER }),
            "writer cannot end before its read: {release:?}"
        );
        assert!(d.releases.contains_key(&WRITER), "the durable release survives a crash with its writer");
        let failed = result == Err(Error::Refused);
        let answered = fixtures::answer(&mut d, call, result);
        if failed {
            let branch = if design { "g3" } else { "c3" };
            let target = format!("resource org/repo branch temper/1/{branch}");
            release_account(
                &answered,
                WRITER,
                None,
                &[
                    "read branch: ",
                    "the reread after run 2 of task 4 failed: forge policy refused the operation",
                    &target,
                ],
            );
            assert!(!answered.contains(&Request::Released { task: WRITER }));
            assert!(d.releases.contains_key(&WRITER));
            continue;
        }
        assert!(
            answered.contains(&Request::ContinueRelease { task: WRITER }),
            "the read wakes its writer's release: {answered:?}"
        );
        let hold = d.hold(&name).expect("the parent still holds its branch");
        assert!(hold.writer.is_none() && hold.design_writers.is_empty());
        assert!(crate::reconcile::references_resolve(&d, &LIMITS, &Stored::Hold(hold.clone()), &[HOLDER]));
        let released = outputs(&mut d, Event::ContinueRelease { task: WRITER, entry: 21 });
        assert!(released.contains(&Request::Released { task: WRITER }));
        for request in &released {
            assert!(
                !matches!(request, Request::Save { record: Stored::Entry { .. }, .. }),
                "no deletion of the parent's branch"
            );
        }
    }
}

#[test]
fn a_release_waiting_for_its_writer_reread_fails_at_its_step_figure() {
    use skein_forge::api::Error;
    const BRANCH: &[u8] = b"temper/1/c3";
    const ATTEMPT: u64 = 1;
    const ENTRY: u64 = 20;
    let mut d = fixtures::connected_domain();
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::CatchUp);
    let name = goal_branch::name(BRANCH);
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::Hold(Hold {
                name: name.clone(),
                task: HOLDER,
                writer: Some(Writer::Run { task: DELEGATE, attempt: ATTEMPT }),
                drift: None,
                beside: None,
                design_writers: Box::new([]),
            }),
        },
    );
    outputs(&mut d, Event::Lost { task: DELEGATE, attempt: ATTEMPT });
    let (call, _) = fixtures::next_call(&mut d);
    outputs(
        &mut d,
        Event::Release { task: DELEGATE, root: HOLDER, ending: ReleaseEnding::Done, reason: None, entry: ENTRY },
    );
    let mut clock = env();
    clock.limits.client.read_attempts = 1;
    fixtures::answer_at(&mut d, &clock, call, Err(Error::Unavailable));
    let failed = fire_at(&mut d, &clock, advanced_wall(clock.wall, clock.limits.waits.step));
    let expected = Request::ReleaseFailed { task: DELEGATE, entry: None, words: b"no progress at release for 30 minutes: waiting for a writer's reread of branch temper/1/c3; the last answer: forge unavailable".as_slice().into() };
    assert!(failed.contains(&expected), "{failed:?}");
    assert_eq!(
        d.hold(&name).expect("lost writer hold retained").writer,
        Some(Writer::Lost { task: DELEGATE, attempt: ATTEMPT })
    );
}
