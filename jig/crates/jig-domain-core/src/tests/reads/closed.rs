//! Closed at once (jig/docs/design/domain/people.md, 5.2).
use crate::tests::*;
use jig_domain_people::ApplicationScope;

fn close(core: &mut Core) {
    core.projects.clear();
    assert!(!core.open_project(1));
    assert!(core.people.role(1, 1).is_some(), "roles remain until their deletion step");
}

fn two_parties() -> (Core, Limits) {
    let (mut core, limits) = watch_core();
    let first = core.people.identity(1).unwrap().clone();
    let mut second = first.clone();
    second.key.subject = Box::new(*b"second");
    core.people = people::Domain::new(&limits.people, Box::new([]), core.settings.deployment_provider, Box::new([]));
    core.people.start_restore(Wall::EPOCH);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people };
    let mut out = Queue::uncapped();
    for record in [
        people::Stored::Person { number: 1, identity: first },
        people::Stored::Person { number: 2, identity: second },
        people::Stored::SignIn { number: 10, person: 1, expires: Wall::from_nanos(Duration::from_secs(60).as_nanos()) },
        people::Stored::SignIn { number: 11, person: 2, expires: Wall::from_nanos(Duration::from_secs(60).as_nanos()) },
        people::Stored::Roles {
            project: 1,
            holdings: Box::new([
                people::Holding { person: 1, role: people::Role::Owner },
                people::Holding { person: 2, role: people::Role::Observer },
            ]),
        },
        people::Stored::Roles {
            project: people::DEPLOYMENT,
            holdings: Box::new([
                people::Holding { person: 1, role: people::Role::Observer },
                people::Holding { person: 2, role: people::Role::Observer },
            ]),
        },
    ] {
        people::step(&mut core.people, &env, people::Event::Restore { record }, &mut out);
        while out.pop().is_some() {}
    }
    finish_people(&mut core.people, &env);
    assert_eq!(core.people.person(11, Time::ZERO, Wall::EPOCH), Some(2));
    (core, limits)
}

fn application(project: u32) -> Event {
    Event::Application {
        reply_to: ReplyTo::new(Token::new(30)),
        sign_in: 10,
        key: [7; 16],
        scope: ApplicationScope::Project(project),
        connector: 1,
        digest: [8; 32],
        carried: None,
        owner: Token::new(500),
    }
}

#[test]
fn an_application_ask_on_a_closed_project_is_refused_as_on_one_never_made() {
    let (mut core, limits) = watch_core();
    close(&mut core);
    let closed = watch_outputs(&mut core, &limits, application(1));
    let unknown = watch_outputs(&mut core, &limits, application(99));
    let [Request::Held(unknown_reply), Request::Ask { ask: Ask::DropApplication { .. }, .. }] = unknown.as_ref() else {
        panic!("unknown project refuses without a save: {unknown:?}");
    };
    for (_, flight) in &core.flights {
        if let flight::Flight::Application { .. } = flight {
            panic!("refusal retains no application continuation");
        }
    }
    assert!(core.application_input.is_none());
    let [Request::Held(reply), Request::Ask { ask: Ask::DropApplication { .. }, .. }] = closed.as_ref() else {
        panic!("refusal has no flight or save: {closed:?}");
    };
    let Held::PeopleReply { reply: closed_reply, .. } = reply.as_ref() else {
        panic!("closed reply");
    };
    let Held::PeopleReply { reply: unknown_reply, .. } = unknown_reply.as_ref() else {
        panic!("unknown reply");
    };
    assert_eq!(closed_reply, unknown_reply);
    assert_eq!(*closed_reply, people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Role { held: None })));
    for project in [1, 99] {
        let out = watch_outputs(
            &mut core,
            &limits,
            Event::Read {
                reply: ReplyTo::new(Token::new(31)),
                sign_in: 10,
                read: crate::reads::Read::Application {
                    scope: ApplicationScope::Project(project),
                    connector: 1,
                    read: Token::new(501),
                    window: window(),
                },
            },
        );
        let [Request::Ask { ask: Ask::DropDocument { .. }, .. }, Request::Held(value)] = out.as_ref() else {
            panic!("read refuses without a handoff or save: {out:?}");
        };
        let Held::Read { answer, .. } = value.as_ref() else {
            panic!("read refusal");
        };
        assert_eq!(*answer, crate::reads::Answer::None(crate::reads::Why::Unknown));
    }
}

#[test]
fn a_watch_on_a_closed_project_is_refused_and_a_live_one_ends() {
    let (mut core, limits) = watch_core();
    let watcher = Token::new(30);
    watch_outputs(
        &mut core,
        &limits,
        Event::Watch { watcher, sign_in: 10, subject: Watched::Task { task: 1 }, window: window() },
    );
    watch_outputs(&mut core, &limits, Event::Delivered { watcher });
    let application_watcher = Token::new(31);
    watch_outputs(
        &mut core,
        &limits,
        Event::Watch {
            watcher: application_watcher,
            sign_in: 10,
            subject: Watched::Application {
                scope: ApplicationScope::Project(1),
                connector: 1,
                subject: Token::new(500),
            },
            window: window(),
        },
    );
    let internal = core.watch_token(application_watcher).expect("admitted application watch");
    watch_outputs(
        &mut core,
        &limits,
        Event::View(views::Event::Made { watcher: internal, outcome: views::Made::Ready }),
    );
    close(&mut core);
    let mut out =
        crate::routing::view::routes::close_views(&mut core, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits }, 1);
    let Some(Request::Held(value)) = out.pop() else {
        panic!("held watch ending");
    };
    let Held::WatchEnded { watcher: ended, why } = value.as_ref() else {
        panic!("watch ending");
    };
    assert_eq!((*ended, *why), (watcher, views::End::Gone));
    let out = watch_outputs(&mut core, &limits, Event::Delivered { watcher: application_watcher });
    let Request::Held(value) = &out[0] else {
        panic!("held application watch ending");
    };
    let Held::ApplicationWatchEnded { watcher: ended, why, .. } = value.as_ref() else {
        panic!("application watch ending");
    };
    assert_eq!((*ended, *why), (application_watcher, views::End::Gone));
    assert_eq!(core.watch_flights(), 0);
    let repeated =
        crate::routing::view::routes::close_views(&mut core, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits }, 1);
    assert!(repeated.is_empty(), "retired subjects need no second ending");
    for project in [1, 99] {
        let out = watch_outputs(
            &mut core,
            &limits,
            Event::Watch {
                watcher,
                sign_in: 10,
                subject: Watched::Application {
                    scope: ApplicationScope::Project(project),
                    connector: 1,
                    subject: Token::new(500),
                },
                window: window(),
            },
        );
        assert_eq!(out.len(), 2);
        assert_watch_refusal(&out[1], WatchRefusal::NotAllowed);
        assert_eq!(
            make(&core, &limits, 1, &Subject::Project { project }, &window(), Wall::EPOCH),
            Err(Missing::NotAllowed)
        );
    }
    for subject in [Watched::Task { task: 1 }, Watched::Tree { task: 1 }, Watched::Run { task: 1 }] {
        let out = watch_outputs(&mut core, &limits, Event::Watch { watcher, sign_in: 10, subject, window: window() });
        assert_watch_refusal(&out[0], WatchRefusal::NotAllowed);
        assert_eq!(out.len(), 1, "no retained load");
    }
}

#[test]
fn a_closed_project_gets_no_repair_and_no_connector_resource() {
    let (mut core, _) = watch_core();
    assert!(core.connector_repair_authority(1).is_some());
    assert!(core.connector_project(1, 1));
    close(&mut core);
    for project in [1, 99] {
        assert_eq!(core.connector_repair_authority(project), None);
        assert!(core.repair_seed(project).is_none());
        assert!(!core.connector_project(1, project));
    }
}

#[test]
fn no_party_reads_a_closed_projects_scope() {
    let (mut core, limits) = two_parties();
    let Held::ReadLoad { load, .. } =
        read_start(&mut core, &limits, 10, crate::reads::Read::Notes { project: 1, from: None })
    else {
        panic!("an open project can load notes");
    };
    close(&mut core);
    assert_eq!(
        loaded_answer(&mut core, &limits, load, crate::reads::Loaded::Page { rows: Box::new([]), next: None }),
        Some(crate::reads::Answer::None(crate::reads::Why::Unknown))
    );
    for sign_in in [10, 11] {
        for project in [1, 99] {
            for read in [
                crate::reads::Read::Project { project },
                crate::reads::Read::Notes { project, from: None },
                crate::reads::Read::Spend { project, from: None },
                crate::reads::Read::ScopedNotes {
                    scope: notes::Scope::Goal { project, goal: 1 },
                    description: None,
                    from: None,
                },
                crate::reads::Read::Activity { task: 1, before: None },
            ] {
                let Held::Read { answer, .. } = read_start(&mut core, &limits, sign_in, read) else {
                    panic!("a closed scope admits no load");
                };
                assert_eq!(answer, crate::reads::Answer::None(crate::reads::Why::Unknown));
            }
        }
    }
}

#[test]
fn no_document_lists_a_closed_project_or_its_tasks() {
    let (mut core, mut limits) = two_parties();
    limits.accounts.accounts = 2;
    core.accounts = accounts::Domain::new(&limits.accounts, core.settings.llm_providers.clone());
    let mut out = Queue::uncapped();
    accounts::step(
        &mut core.accounts,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.accounts },
        accounts::Event::Restore {
            record: accounts::Stored::Account {
                number: 0,
                provider: 1,
                holder: accounts::Holder::Party(1),
                place: 0,
                uses: Some(Box::new([Use::Project(1)])),
                subject: Box::new([]),
                display: Box::new([]),
            },
        },
        &mut out,
    );
    accounts::step(
        &mut core.accounts,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.accounts },
        accounts::Event::Restore {
            record: accounts::Stored::Account {
                number: 1,
                provider: 1,
                holder: accounts::Holder::Deployment,
                place: 0,
                uses: Some(Box::new([Use::Project(1)])),
                subject: Box::new([]),
                display: Box::new([]),
            },
        },
        &mut out,
    );
    for (account, number) in [(0, 2), (1, 3)] {
        accounts::step(
            &mut core.accounts,
            &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.accounts },
            accounts::Event::Restore {
                record: accounts::Stored::Source {
                    account,
                    number,
                    place: 0,
                    kind: accounts::Kind::Key,
                    name: Box::new([]),
                    held: None,
                },
            },
            &mut out,
        );
    }
    accounts::step(
        &mut core.accounts,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.accounts },
        accounts::Event::Want,
        &mut out,
    );
    for party in [1, 2] {
        let Document::Frame(frame) =
            make(&core, &limits, party, &Subject::Frame { party }, &window(), Wall::EPOCH).unwrap()
        else {
            panic!("open project frame");
        };
        assert_eq!(frame.projects.len(), 1);
    }
    let query = TaskQuery {
        project: Some(1),
        requester: None,
        root: None,
        parent: None,
        label: None,
        phase: None,
        executor: None,
        tracked: None,
        order: QueryOrder::Number,
    };
    let Document::Lines(lines) = make(&core, &limits, 1, &Subject::Query { query }, &window(), Wall::EPOCH).unwrap()
    else {
        panic!("open project query");
    };
    assert_eq!(lines.lines.len(), 1);
    close(&mut core);
    let Document::Deployment(deployment) =
        make(&core, &limits, 1, &Subject::Deployment, &window(), Wall::EPOCH).unwrap()
    else {
        panic!("deployment document");
    };
    assert!(deployment.projects.is_empty());
    assert_eq!(deployment.accounts.get(0).unwrap().uses.as_ref().unwrap().len(), 0);
    for party in [1, 2] {
        let Document::Frame(frame) =
            make(&core, &limits, party, &Subject::Frame { party }, &window(), Wall::EPOCH).unwrap()
        else {
            panic!("party frame");
        };
        assert!(frame.projects.is_empty());
        assert!(!frame.cut);
        let Document::Accounts(accounts) =
            make(&core, &limits, party, &Subject::Accounts { party }, &window(), Wall::EPOCH).unwrap()
        else {
            panic!("party accounts");
        };
        for row in &accounts.readiness {
            assert_ne!(row.payer, Use::Project(1));
        }
        if party == 1 {
            assert!(accounts.accounts.get(0).unwrap().uses.as_ref().unwrap().is_empty());
        }
        let query = TaskQuery {
            project: None,
            requester: None,
            root: None,
            parent: None,
            label: None,
            phase: None,
            executor: None,
            tracked: None,
            order: QueryOrder::Number,
        };
        let Document::Lines(lines) =
            make(&core, &limits, party, &Subject::Query { query }, &window(), Wall::EPOCH).unwrap()
        else {
            panic!("task query");
        };
        assert!(lines.lines.is_empty());
        let Document::Inbox(inbox) =
            make(&core, &limits, party, &Subject::Inbox { party }, &window(), Wall::EPOCH).unwrap()
        else {
            panic!("party inbox");
        };
        assert!(inbox.needs.is_empty());
        assert!(inbox.updates.is_empty());
    }
}
