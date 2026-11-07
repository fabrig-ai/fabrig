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
                list: None,
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
                list: None,
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

fn document_row(name: u64, scope: notes::Scope) -> crate::reads::Row {
    stored_row(
        Record::Notes(notes::Record::Entry(notes::Entry {
            name,
            scope,
            description: Box::new(*b"draft"),
            body: Box::new(*b"# Draft\n"),
            references: List::with_capacity(0),
            author: notes::Author::Task { task: name, attempt: 1 },
            revision: 1,
            changed: Wall::EPOCH,
        })),
        None,
    )
}

fn project_documents(
    core: &mut Core,
    limits: &Limits,
    sign_in: u64,
    project: u32,
    rows: Box<[crate::reads::Row]>,
) -> Box<crate::reads::Page> {
    let Held::ReadLoad { load, range } =
        read_start(core, limits, sign_in, crate::reads::Read::Notes { project, from: None })
    else {
        panic!("project notes load")
    };
    assert_eq!(range.scope, crate::reads::Scope::Notes { project });
    let crate::reads::Answer::Page(page) =
        loaded_answer(core, limits, load, crate::reads::Loaded::Page { rows, next: None }).unwrap()
    else {
        panic!("project notes page")
    };
    page
}

#[test]
fn project_notes_hold_goal_scoped_rows_alone() {
    for project in [0, 1] {
        let (mut core, limits) = two_parties();
        let scope = crate::reads::Scope::Notes { project };
        let goal = notes::Scope::Goal { project, goal: 7 };
        let other = notes::Scope::Goal { project: 99, goal: 7 };
        let resources = notes::Scope::Resources {
            project,
            connector: 1,
            pattern: notes::Pattern { segments: Box::new([]), last: notes::Last::Open(Box::new([])) },
        };
        let scopes = [notes::Scope::Deployment, notes::Scope::Project { project }, resources, other, goal.clone()];
        for (name, owner) in (1..).zip(scopes) {
            assert_eq!(
                crate::reads::admits(1, &scope, &Key::Notes(notes::Key::Line { scope: owner.clone(), name })),
                owner == goal
            );
            let Held::ReadLoad { load, .. } =
                read_start(&mut core, &limits, 10, crate::reads::Read::Notes { project, from: None })
            else {
                panic!("notes load")
            };
            let answer = loaded_answer(
                &mut core,
                &limits,
                load,
                crate::reads::Loaded::Page { rows: Box::new([document_row(name, owner.clone())]), next: None },
            )
            .unwrap();
            if owner == goal {
                let crate::reads::Answer::Page(page) = answer else { panic!("goal notes page") };
                assert_eq!(page.rows.len(), 1);
                let crate::reads::PageRow::Note(note) = page.rows.get(0).unwrap() else { panic!("goal note") };
                assert_eq!(note.number, 5);
                assert_eq!(note.scope, NoteLocation::Goal { project, task: 7 });
                assert_eq!(page.next, None);
            } else {
                let crate::reads::Answer::Page(page) = answer else { panic!("filtered notes page") };
                assert!(page.rows.is_empty(), "out-of-scope entries yield no notes");
            }
            core.reclaim();
        }
    }
}

#[test]
fn project_notes_leave_out_each_body() {
    let (mut core, mut limits) = two_parties();
    limits.documents.lines = 1;
    let first = document_row(1, notes::Scope::Goal { project: 1, goal: 1 });
    let mut second = document_row(2, notes::Scope::Goal { project: 1, goal: 2 });
    let Record::Notes(notes::Record::Entry(entry)) = &mut second.value else { panic!("note entry") };
    entry.body = Box::new([]);
    entry.references = List::from_boxed(Box::new([1, 2]));
    let page = project_documents(&mut core, &limits, 10, 1, Box::new([first, second]));
    assert_eq!(page.rows.len(), 2);
    for (index, row) in page.rows.iter().enumerate() {
        let crate::reads::PageRow::Note(note) = row else { panic!("note row") };
        assert!(note.body.bytes.is_empty());
        assert!(note.body.cut, "even an empty source body is omitted");
        assert_eq!(note.cut, index == 1, "Note.cut still describes omitted references");
        assert_eq!(note.description.bytes.as_ref(), b"draft");
        assert_eq!(note.revision, 1);
    }
}

fn remove_observer(core: &mut Core, limits: &Limits) {
    let mut out = Queue::uncapped();
    people::step(
        &mut core.people,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people },
        people::Event::Roles {
            project: 1,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
        },
        &mut out,
    );
    assert_eq!(core.people.role(2, 1), None);
}

#[test]
fn project_notes_are_the_projects_visibility() {
    let (mut core, limits) = two_parties();
    assert_eq!(core.people.role(2, 1), Some(people::Role::Observer));
    let page = project_documents(
        &mut core,
        &limits,
        11,
        1,
        Box::new([document_row(1, notes::Scope::Goal { project: 1, goal: 1 })]),
    );
    assert_eq!(page.rows.len(), 1, "an observer reads another party's chat");
    let Held::ReadLoad { load, .. } =
        read_start(&mut core, &limits, 11, crate::reads::Read::Notes { project: 1, from: None })
    else {
        panic!("observer load")
    };
    remove_observer(&mut core, &limits);
    assert_eq!(
        loaded_answer(
            &mut core,
            &limits,
            load,
            crate::reads::Loaded::Page {
                rows: Box::new([document_row(1, notes::Scope::Goal { project: 1, goal: 1 })]),
                next: None,
            }
        ),
        Some(crate::reads::Answer::None(crate::reads::Why::Unknown))
    );
    let Held::Read { answer, .. } =
        read_start(&mut core, &limits, 11, crate::reads::Read::Notes { project: 1, from: None })
    else {
        panic!("no project role admits no load")
    };
    assert_eq!(answer, crate::reads::Answer::None(crate::reads::Why::Unknown));
}

fn write_chat_document(core: &mut Core, limits: &Limits, task: u64, title: &[u8]) -> crate::reads::Row {
    core.proofs.insert(task, RunProof::claimed(task, 1, 1, None, fleet::HostKind::Engine)).unwrap();
    let mut out = watch_outputs(
        core,
        limits,
        Event::NamedAction {
            to: ReplyTo::new(Token::new(task)),
            key: CallKey { task, attempt: 1, completion: 1, position: 0 },
            action: NamedAction::Note {
                entry: notes::New {
                    name: 0,
                    scope: notes::Scope::Goal { project: 1, goal: task },
                    description: skein_lib::bytes::copy_of(title),
                    body: Box::new(*b"# Draft\n"),
                    references: List::with_capacity(0),
                    author: notes::Author::Party { party: 1 },
                },
                recalled: None,
            },
        },
    );
    // A first write loads its absent entry and then its empty scope's index.
    for _ in 0_u32..2 {
        let mut owner = None;
        for request in &out {
            if let Request::Held(value) = request
                && let Held::NotesLoad { owner: load, .. } = value.as_ref()
            {
                assert!(owner.is_none(), "one notes load at a time");
                owner = Some(*load);
            }
        }
        let owner = owner.expect("chat note load");
        out = watch_outputs(
            core,
            limits,
            Event::Notes(notes::Event::Loaded {
                owner,
                rows: notes::Rows { records: List::with_capacity(0) },
                more: false,
            }),
        );
    }
    let mut entry = None;
    for request in out {
        if let Request::Write(Write::Save(Record::Notes(notes::Record::Entry(value)))) = request {
            assert!(entry.is_none(), "one document written by this chat");
            entry = Some(value);
        }
    }
    let entry = entry.expect("chat wrote its document");
    assert_eq!(entry.author, notes::Author::Task { task, attempt: 1 });
    assert_eq!(entry.body.as_ref(), b"# Draft\n");
    stored_row(Record::Notes(notes::Record::Entry(entry)), None)
}

#[test]
fn a_party_lists_two_chats_documents() {
    let (people, _) = two_parties();
    let (mut core, limits) = tree_core();
    core.people = people.people;
    let mut rules = core.authority.rules().clone();
    rules.ceiling.notes = authority::Scopes::GOAL;
    let mut policy = core.authority.policy(1).unwrap().clone();
    policy.ceiling.notes = authority::Scopes::GOAL;
    policy.roles[0].authority.notes = authority::Scopes::GOAL;
    core.authority = authority::Domain::new(rules, limits.authority).unwrap();
    let mut out = Queue::uncapped();
    authority::step(&mut core.authority, authority::Event::Policy { project: 1, policy }, &mut out);
    make_task(&mut core, &limits, 1);
    make_task(&mut core, &limits, 2);
    for task in [1, 2] {
        assert!(core.tasks.task(task).unwrap().flags.conversation);
        assert_eq!(core.note_goal(core.tasks.task(task).unwrap()), Some(task));
    }
    let first = write_chat_document(&mut core, &limits, 1, b"First draft");
    let second = write_chat_document(&mut core, &limits, 2, b"Second draft");
    let page = project_documents(&mut core, &limits, 11, 1, Box::new([first, second]));
    assert_eq!(page.rows.len(), 2);
    for (row, (task, title)) in page.rows.iter().zip([(1, b"First draft".as_slice()), (2, b"Second draft".as_slice())])
    {
        let crate::reads::PageRow::Note(note) = row else { panic!("chat document") };
        assert_eq!(note.scope, NoteLocation::Goal { project: 1, task });
        assert_eq!(note.description.bytes.as_ref(), title);
        assert!(note.body.bytes.is_empty());
        assert!(note.body.cut);
        assert!(!note.cut);
    }
    assert_eq!(page.next, None, "one read lists both documents");
    remove_observer(&mut core, &limits);
    let Held::Read { answer, .. } =
        read_start(&mut core, &limits, 11, crate::reads::Read::Notes { project: 1, from: None })
    else {
        panic!("a party with no role reads none")
    };
    assert_eq!(answer, crate::reads::Answer::None(crate::reads::Why::Unknown));
}
