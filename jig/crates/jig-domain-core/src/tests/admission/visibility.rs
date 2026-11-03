//! Visibility decisions, projections and kept rows (jig/docs/design/domain/people.md, 13.2).
use crate::tests::*;

fn environment(limits: &Limits) -> Env<Limits> {
    Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits }
}

fn authenticated() -> (Core, Limits) {
    let (mut core, limits) = core();
    core.people = people::Domain::new(&limits.people, Box::new([]), core.settings.deployment_provider, Box::new([]));
    core.people.start_restore(Wall::EPOCH);
    let env = environment(&limits).with(limits.people);
    let mut out = Queue::uncapped();
    for person in [1_u64, 2] {
        for record in [
            people::Stored::Person {
                number: person,
                identity: people::Identity {
                    key: people::IdentityKey { provider: 0, subject: Box::new([u8::try_from(person).unwrap()]) },
                    login: Box::from(b"party".as_slice()),
                    name: Box::from(b"Party".as_slice()),
                },
            },
            people::Stored::SignIn {
                number: person.checked_mul(10).expect("fixture sign-in number"),
                person,
                expires: Wall::from_nanos(Duration::from_secs(60).as_nanos()),
            },
        ] {
            people::step(&mut core.people, &env, people::Event::Restore { record }, &mut out);
            assert!(out.is_empty());
        }
    }
    people::step(
        &mut core.people,
        &env,
        people::Event::Restore {
            record: people::Stored::Roles {
                project: 1,
                holdings: Box::new([
                    people::Holding { person: 1, role: people::Role::Owner },
                    people::Holding { person: 2, role: people::Role::Member },
                ]),
            },
        },
        &mut out,
    );
    assert!(out.is_empty());
    finish_people(&mut core.people, &env);
    let mut child = Queue::uncapped();
    tasks::step(&mut core.tasks, &environment(&limits).with(limits.tasks), tasks::Event::Check, &mut child);
    tasks::step(&mut core.tasks, &environment(&limits).with(limits.tasks), tasks::Event::Restored, &mut child);
    (core, limits)
}

fn ask(core: &mut Core, limits: &Limits, person: u64, key: u8, public: bool) -> Box<[Request]> {
    watch_outputs(
        core,
        limits,
        Event::People(people::Event::Ask {
            reply_to: ReplyTo::new(Token::new(90)),
            sign_in: person.checked_mul(10).expect("fixture sign-in number"),
            key: [key; 16],
            ask: people::Ask::SetVisibility { project: 1, public },
            models: Box::new([]),
        }),
    )
}

fn assert_answer(rows: &[Request], expected: people::Outcome) {
    let mut replies = 0_u32;
    let mut saved = 0_u32;
    for row in rows {
        if let Request::Held(held) = row
            && let Held::PeopleReply { reply: people::Reply::Outcome(outcome), .. } = held.as_ref()
        {
            assert_eq!(*outcome, expected);
            replies = replies.checked_add(1).unwrap();
        }
        if let Request::Write(Write::Save(Record::People(people::Stored::Answer { outcome, .. }))) = row {
            assert_eq!(*outcome, expected);
            saved = saved.checked_add(1).unwrap();
        }
    }
    assert_eq!((replies, saved), (1, 1), "{rows:?}");
}

fn public_write(row: &Request, public: bool) -> bool {
    match row {
        Request::Write(Write::Save(Record::People(people::Stored::Public { project }))) => public && *project == 1,
        Request::Write(Write::Erase(Key::People(people::Key::Public(project)))) => !public && *project == 1,
        Request::Write(_) | Request::Ask { .. } | Request::Held(_) | Request::Now(_) | Request::ProjectMade { .. } => {
            false
        }
    }
}

fn assert_frames(rows: &[Request], public: bool) {
    let mut parties = List::with_capacity(2);
    for row in rows {
        if let Request::Held(held) = row
            && let Held::Delivery { document, .. } = held.as_ref()
            && let Document::Frame(frame) = document.as_ref()
        {
            assert_eq!(frame.projects.len(), 1);
            assert_eq!(frame.projects.get(0).unwrap().project, 1);
            assert_eq!(frame.projects.get(0).unwrap().public, public);
            parties.push(frame.party).unwrap();
        }
    }
    assert_eq!(parties.len(), 2);
    assert!(parties.as_slice().contains(&1));
    assert!(parties.as_slice().contains(&2));
}

#[test]
fn an_owner_makes_a_project_public_then_private() {
    let (mut core, limits) = authenticated();
    let mut initial = List::uncapped();
    for person in [1_u64, 2] {
        let opened = watch_outputs(
            &mut core,
            &limits,
            Event::Watch {
                watcher: Token::new(person),
                sign_in: person.checked_mul(10).expect("fixture sign-in number"),
                subject: Watched::Frame,
                window: window(),
            },
        );
        for row in opened {
            initial.push(row).unwrap();
        }
    }
    for row in finish_watch_outputs(&mut core, &limits) {
        initial.push(row).unwrap();
    }
    assert_frames(&initial.into_boxed(), false);
    for (key, public) in [(1, true), (2, true), (3, false), (4, false)] {
        core.reclaim();
        for person in [1_u64, 2] {
            let _delivered = watch_outputs(&mut core, &limits, Event::Delivered { watcher: Token::new(person) });
        }
        let rows = ask(&mut core, &limits, 1, key, public);
        assert_answer(&rows, people::Outcome::VisibilitySet { project: 1, public });
        let mut writes = 0_u32;
        for row in &rows {
            if public_write(row, public) {
                writes = writes.checked_add(1).unwrap();
            }
        }
        assert_eq!(writes, 1);
        assert_eq!(core.public_project(1), public);
        let mut deliveries = List::uncapped();
        for row in rows {
            deliveries.push(row).unwrap();
        }
        for row in finish_watch_outputs(&mut core, &limits) {
            deliveries.push(row).unwrap();
        }
        assert_frames(&deliveries.into_boxed(), public);
    }
}

#[test]
fn a_member_setting_visibility_is_refused_and_nothing_is_written() {
    let (mut core, limits) = core();
    let rows = routed(
        &mut core,
        &limits,
        1,
        Some(people::Role::Member),
        people::Refusal::Role { held: Some(people::Role::Member) },
    );
    assert!(rows.is_empty());
    assert!(!core.public_project(1));
}

fn routed(
    core: &mut Core,
    limits: &Limits,
    project: u32,
    role: Option<people::Role>,
    expected: people::Refusal,
) -> Queue<Request> {
    let mut child = Queue::uncapped();
    child.push(people::Request::Route {
        request: Token::new(90),
        person: 1,
        project,
        role,
        ask: Box::new(people::Ask::SetVisibility { project, public: true }),
        models: Box::new([]),
    });
    let mut work = Queue::uncapped();
    let out = crate::routing::party::people::tag_people(core, &environment(limits), child, &mut work);
    let Some(Work::Event(Event::People(people::Event::Decided { outcome, .. }))) = work.pop() else {
        panic!("one decision")
    };
    assert_eq!(outcome, people::Outcome::Refused(expected));
    assert!(work.is_empty());
    out
}

#[test]
fn project_zero_an_unknown_project_and_a_deleting_one_are_refused_alike() {
    let (mut core, limits) = authenticated();
    let mut work = Queue::uncapped();
    let mut out = Queue::uncapped();
    deletion::begin(&mut core, &environment(&limits), &mut work, &mut out, Token::new(91), 1, None).unwrap();
    for project in [0, 99, 1] {
        assert!(routed(&mut core, &limits, project, Some(people::Role::Owner), people::Refusal::Unknown).is_empty());
        assert!(!core.public_project(project));
    }
}

fn restoring() -> (Core, Limits) {
    let limits = limits();
    let mut core = Core::new(config(41), &limits).unwrap();
    let _action = core.start_begin(false, Wall::EPOCH);
    core.counters.deployment.projects = 1;
    assert_eq!(
        core.restore(&environment(&limits), Record::Core(CoreRecord::Deployment(core.counters.deployment()))),
        Restore::Deployment { commits: 0 }
    );
    let value = policy::snapshot(&config(41).seed_projects[0].policy.policy, &[], &[]).unwrap();
    assert_eq!(
        core.restore(
            &environment(&limits),
            Record::People(people::Stored::Policy { project: 1, name: Box::from("Project"), value })
        ),
        Restore::Kept
    );
    (core, limits)
}

#[test]
fn a_restored_public_row_makes_the_project_public() {
    let (mut core, limits) = restoring();
    assert_eq!(
        core.restore(&environment(&limits), Record::People(people::Stored::Public { project: 1 })),
        Restore::Kept
    );
    assert!(core.public_project(1));
    assert!(!core.public_project(0));
}

#[test]
fn a_store_without_public_rows_restores_every_project_private() {
    let (core, _) = restoring();
    for &project in &core.projects {
        assert!(!core.public_project(project));
    }
}

#[test]
fn a_public_row_for_project_zero_or_an_unknown_project_refuses_restore() {
    for project in [0, 99] {
        let (mut core, limits) = restoring();
        assert_eq!(
            core.restore(&environment(&limits), Record::People(people::Stored::Public { project })),
            Restore::Refused
        );
        assert_eq!(core.start_failure(), Some(crate::start::Failure::ProjectPolicy { project }));
    }
}

#[test]
fn deletion_ends_publicity_at_its_decision_and_erases_the_row() {
    let (mut core, limits) = authenticated();
    assert_answer(&ask(&mut core, &limits, 1, 1, true), people::Outcome::VisibilitySet { project: 1, public: true });
    assert!(core.public_project(1));
    let mut work = Queue::uncapped();
    let mut out = Queue::uncapped();
    deletion::begin(&mut core, &environment(&limits), &mut work, &mut out, Token::new(91), 1, None).unwrap();
    assert!(!core.public_project(1));
    assert!(core.visibility.public.contains_key(&1));
    deletion::tasks_erased(&mut core, &environment(&limits), &mut out, 1);
    assert!(!core.visibility.public.contains_key(&1));
    let mut erased = 0_u32;
    while let Some(row) = out.pop() {
        if public_write(&row, false) {
            erased = erased.checked_add(1).unwrap();
        }
    }
    assert_eq!(erased, 1);
}
