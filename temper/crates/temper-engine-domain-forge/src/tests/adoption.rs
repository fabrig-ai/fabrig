//! Core admission and the bounded retained application outcome.
use super::*;
use jig_domain_people as people;
use skein_lib::List;

#[test]
fn writer_readoption_needs_collaborator_access_and_retries_as_admin() {
    use skein_forge::api;
    let mut d = domain();
    let writer = adopted();
    outputs(&mut d, Event::Restore { record: Stored::Forge(records::record(1)) });
    outputs(&mut d, Event::Restore { record: Stored::Repository(writer.clone()) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    d.writers.insert(1, 17).expect("fixture writer differs from the link's user");
    let human = api::Collaborator { user: 42, permission: api::Permission::Admin };
    let bot = api::Collaborator { user: 17, permission: api::Permission::Write };
    for (key, link, permission, protection, collaborators) in [
        (4, None, api::Permission::Write, Err(api::Error::Forbidden), Err(api::Error::Forbidden)),
        (
            5,
            None,
            api::Permission::Admin,
            Ok(api::Answer::Protection(None)),
            Ok(api::Answer::Collaborators { collaborators: Box::new([human.clone(), bot.clone()]), more: false }),
        ),
        (
            6,
            Some(Link { user: 7, person: 31 }),
            api::Permission::Admin,
            Ok(api::Answer::Protection(None)),
            Ok(api::Answer::Collaborators {
                collaborators: Box::new([
                    human.clone(),
                    bot.clone(),
                    api::Collaborator { user: 7, permission: api::Permission::Admin },
                ]),
                more: false,
            }),
        ),
    ] {
        d.records.get_mut(&1).expect("restored forge").person = link;
        fixtures::begin_adoption(&mut d, Token::new(key), fixtures::adoption());
        for answer in [
            Ok(api::Answer::Permission(api::UserPermission { user: 1, login: Box::new([]), permission })),
            Ok(api::Answer::Branches(Box::new([]))),
            Ok(api::Answer::Settings(writer.settings.clone())),
            protection,
        ] {
            fixtures::answer_next(&mut d, answer);
        }
        let requests = fixtures::answer_next(&mut d, collaborators);
        if permission == api::Permission::Write {
            let Some(Request::Adopted { result: Err(api::Error::Forbidden), .. }) = requests.last() else {
                panic!("forbidden collaborators must refuse re-adoption even after write permission");
            };
            assert_eq!(d.repository(REPO), Some(&writer), "failed re-adoption keeps the writer");
            assert_eq!(d.home(writer.project), Some(&writer), "failed re-adoption keeps the home");
            for request in &requests {
                if let Request::Save { record: Stored::Repository(_), .. } = request {
                    panic!("failed re-adoption must not save a repository");
                }
            }
        } else {
            let Some(Request::Adopted { result: Ok(value), .. }) = requests.last() else {
                panic!("repository admin can import the human on re-adoption");
            };
            assert_eq!(value.collaborators.as_ref(), core::slice::from_ref(&human), "adoption {key}");
            assert_eq!(&value.repository, &writer);
            fixtures::workflows_absent(&mut d);
        }
        let outcome = if permission == api::Permission::Write {
            people::Outcome::Refused(people::Refusal::Unknown)
        } else {
            people::Outcome::Adopted { project: writer.project }
        };
        outputs(&mut d, Event::ApplicationAnswered { request: Token::new(key), outcome });
    }
}

#[test]
fn context_beside_a_writer_preserves_its_home_and_readoption() {
    use skein_forge::api;
    let mut d = domain();
    let writer = adopted();
    outputs(&mut d, Event::Restore { record: Stored::Repository(writer.clone()) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    outputs(&mut d, Event::SettleOutbox);
    let mut context = fixtures::adoption();
    context.project += 1;
    context.role = Role::Context;
    context.home = false;
    fixtures::begin_adoption(&mut d, Token::new(4), context);
    for answer in [
        api::Answer::Permission(api::UserPermission {
            user: 1,
            login: Box::from(b"".as_slice()),
            permission: api::Permission::Read,
        }),
        api::Answer::Settings(writer.settings.clone()),
        api::Answer::Protection(None),
    ] {
        fixtures::answer_next(&mut d, Ok(answer));
    }
    let requests =
        fixtures::answer_next(&mut d, Ok(api::Answer::Collaborators { collaborators: Box::new([]), more: false }));
    let [.., Request::Adopted { result: Ok(value), .. }] = requests.as_ref() else { panic!("context adopted") };
    assert_eq!(value.repository.role, Role::Context);
    assert_eq!(value.repository.project, writer.project + 1);
    assert!(value.repository.kinds.read && !value.repository.kinds.push && !value.repository.home);
    let mut shared = writer.clone();
    shared.contexts = Box::new([writer.project + 1]);
    assert!(requests.contains(&Request::Save {
        record: Stored::Repository(shared.clone()),
        release: None,
        read_afresh: None,
    }));
    assert_eq!(d.repository(REPO), Some(&shared));
    assert_eq!(d.home(writer.project), Some(&shared));
    assert_context_view(&d, writer.project + 1);
    assert_eq!(d.readable_repositories(writer.project).as_ref(), core::slice::from_ref(&shared));
    assert_eq!(d.repository_tag(writer.project, tag(1, 2)), Some(shared.clone()));
    assert!(d.context_repositories(writer.project, 2).unwrap().is_empty());
    let mut restarted = domain();
    outputs(&mut restarted, Event::Restore { record: Stored::Repository(shared) });
    outputs(&mut restarted, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut restarted, Event::ReadAfresh);
    outputs(&mut restarted, Event::SettleOutbox);
    assert_context_view(&restarted, writer.project + 1);
    let again = adopt_context(&mut d, writer.project + 1, 6);
    for request in &again {
        assert!(!matches!(request, Request::Save { record: Stored::Repository(_), .. }));
    }
    assert_eq!(d.repository(REPO).unwrap().contexts.as_ref(), &[writer.project + 1]);

    fixtures::begin_adoption(&mut d, Token::new(5), fixtures::adoption());
    for answer in [
        api::Answer::Permission(api::UserPermission {
            user: 1,
            login: Box::from(b"".as_slice()),
            permission: api::Permission::Write,
        }),
        api::Answer::Branches(Box::new([])),
        api::Answer::Settings(writer.settings),
        api::Answer::Protection(None),
    ] {
        fixtures::answer_next(&mut d, Ok(answer));
    }
    let requests =
        fixtures::answer_next(&mut d, Ok(api::Answer::Collaborators { collaborators: Box::new([]), more: false }));
    let Some(Request::Adopted { result: Ok(_), .. }) = requests.last() else { panic!("writer re-adopts") };
    let home = d.home(writer.project).expect("writer keeps its home");
    assert_eq!(home.role, Role::Owned);
    assert!(home.kinds.push);
    assert_context_view(&d, writer.project + 1);
}

fn assert_context_view(d: &Domain, project: u32) {
    let row = d.repository_tag(project, tag(1, 2)).expect("context resolves a repository tag");
    assert_eq!(row.project, project);
    assert_eq!(row.role, Role::Context);
    assert!(!row.home);
    assert_eq!(
        row.kinds,
        Kinds {
            read: true,
            push: false,
            open: false,
            land: false,
            review: false,
            status: false,
            comment: false,
            issue: false,
            branch: false,
        }
    );
    assert_eq!(d.readable_repositories(project).as_ref(), core::slice::from_ref(&row));
    assert_eq!(d.context_repositories(project, 2).unwrap().as_ref(), &[REPO]);
    assert!(d.writable_repositories(project).is_empty());
    assert!(d.home(project).is_none());
    let item = d
        .workspace_item(items::Seed {
            project,
            modify: true,
            task: 9,
            parent: None,
            repository: REPO,
            run_branch: b"temper/9/r9".as_slice().into(),
            saved_branch: None,
        })
        .expect("context checkout");
    assert!(item.push.is_none() && item.landing.is_none() && !item.delivers);
    assert_eq!(item.start, items::Start::Base(b"main".as_slice().into()));
}

fn adopt_context(d: &mut Domain, project: u32, key: u64) -> Box<[Request]> {
    let mut request = fixtures::adoption();
    request.project = project;
    request.role = Role::Context;
    request.home = false;
    let settings = adopted().settings;
    let mut out = Queue::uncapped();
    crate::adoption::making_asked(d, &env(), Token::new(key), request, REPO, &mut out);
    for answer in [
        skein_forge::api::Answer::Permission(skein_forge::api::UserPermission {
            user: 1,
            login: Box::new([]),
            permission: skein_forge::api::Permission::Read,
        }),
        skein_forge::api::Answer::Settings(settings),
        skein_forge::api::Answer::Protection(None),
    ] {
        fixtures::answer_next(d, Ok(answer));
    }
    let requests = fixtures::answer_next(
        d,
        Ok(skein_forge::api::Answer::Collaborators { collaborators: Box::new([]), more: false }),
    );
    if d.repository(REPO).unwrap().project == project {
        fixtures::workflows_absent(d);
    }
    requests
}

#[test]
fn context_projects_are_bounded_including_the_first_context_and_readoption_is_free() {
    for writer in [true, false] {
        let mut d = domain();
        let mut row = adopted();
        if !writer {
            row.role = Role::Context;
            row.home = false;
        }
        outputs(&mut d, Event::Restore { record: Stored::Repository(row.clone()) });
        outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        outputs(&mut d, Event::ReadAfresh);
        outputs(&mut d, Event::SettleOutbox);
        let first = if writer { 6 } else { 5 };
        for project in (first..).take(usize::try_from(LIMITS.judge_projects).unwrap()) {
            let requests = adopt_context(&mut d, project, u64::from(project) + 10);
            assert!(matches!(requests.last(), Some(Request::Adopted { result: Ok(_), .. })));
        }
        let requests = adopt_context(&mut d, first, 40);
        assert!(matches!(requests.last(), Some(Request::Adopted { result: Ok(_), .. })));
        let mut request = fixtures::adoption();
        request.project = first + LIMITS.judge_projects;
        request.role = Role::Context;
        request.home = false;
        let refused = fixtures::begin_adoption(&mut d, Token::new(41), request);
        assert!(
            matches!(refused.last(), Some(Request::Adopted { result: Err(skein_forge::api::Error::TooLarge(excess)), .. })
            if *excess == skein_forge::api::Excess::exact(skein_forge::api::Bound::Forge,
                u64::from(LIMITS.judge_projects), u64::from(LIMITS.judge_projects + 1)))
        );
        outputs(
            &mut d,
            Event::ApplicationAnswered {
                request: Token::new(41),
                outcome: people::Outcome::Refused(people::Refusal::Unknown),
            },
        );
        assert_eq!(
            d.application_outcome(Token::new(41)),
            Some(&ApplicationOutcome::ForgeRefused(AdoptionRefusal::TooLarge))
        );
        assert!(ready(&mut d).is_empty(), "one past the bound reaches no permission read");
        assert_eq!(d.repository(REPO).unwrap().project, row.project);
    }
}

#[test]
fn releasing_a_context_project_saves_the_writer_without_its_context() {
    let mut d = domain();
    let mut row = adopted();
    row.contexts = Box::new([6, 7]);
    outputs(&mut d, Event::Restore { record: Stored::Repository(row) });
    let mut out = Queue::uncapped();
    crate::adoption::release(&mut d, 6, Token::new(9), &mut out);
    let mut expected = adopted();
    expected.contexts = Box::new([7]);
    assert_eq!(d.repository(REPO), Some(&expected));
    assert_eq!(d.repository_tag(6, tag(1, 2)), None);
    assert_context_view(&d, 7);
    let mut saved = false;
    for request in &out {
        saved |= *request
            == Request::Save { record: Stored::Repository(expected.clone()), release: None, read_afresh: None };
        assert!(!matches!(request, Request::Erase { key: Key::Repository(_) | Key::Client(_), .. }));
    }
    assert!(saved);
    crate::adoption::release(&mut d, 5, Token::new(10), &mut out);
    assert!(d.repository(REPO).is_none(), "the writer's release drops its contexts too");
    let mut context_only = adopted();
    context_only.role = Role::Context;
    context_only.home = false;
    context_only.contexts = Box::new([7]);
    outputs(&mut d, Event::Restore { record: Stored::Repository(context_only) });
    crate::adoption::release(&mut d, 5, Token::new(11), &mut out);
    assert_eq!(d.repository(REPO).unwrap().project, 7, "a context-only row stays for the remaining adopter");
    assert_context_view(&d, 7);
}

#[test]
fn ending_a_forge_drops_contexts_but_keeps_the_named_writer_row() {
    let mut d = domain();
    let mut shared = adopted();
    shared.contexts = Box::new([6]);
    outputs(&mut d, Event::Restore { record: Stored::Forge(records::record(1)) });
    outputs(&mut d, Event::Restore { record: Stored::Repository(shared) });
    outputs(&mut d, Event::Restore { record: Stored::Names { task: 7, resources: Box::new([branch()]) } });
    let mut out = Queue::uncapped();
    assert!(crate::removal::begin(&mut d, 1, &mut out));
    assert_eq!(d.repository(REPO), Some(&adopted()));
    assert_eq!(d.repository_tag(6, tag(1, 2)), None);
    assert_eq!(d.repository_tag(5, tag(1, 2)), Some(adopted()));
    assert!(d.readable_repositories(6).is_empty());
    assert!(d.context_repositories(6, 2).unwrap().is_empty());
    let mut saved = false;
    for request in &out {
        saved |= matches!(request, Request::Save { record: Stored::Repository(row), .. } if row.contexts.is_empty());
    }
    assert!(saved);
}

#[test]
fn restored_context_projects_fit_the_configured_bound_including_the_first_context() {
    for first_context in [false, true] {
        for extra in [0, 1] {
            let mut d = domain();
            let mut row = adopted();
            if first_context {
                row.role = Role::Context;
                row.home = false;
            }
            let count = LIMITS.judge_projects + extra - u32::from(first_context);
            let mut contexts = List::with_capacity(count);
            for project in 6..6 + count {
                contexts.push(project).unwrap();
            }
            row.contexts = contexts.into_boxed();
            outputs(&mut d, Event::Restore { record: Stored::Repository(row.clone()) });
            if extra == 0 {
                assert_eq!(d.restore_fit_failure(), None);
                assert_eq!(d.repository(REPO), Some(&row));
            } else {
                assert_eq!(
                    d.restore_fit_failure(),
                    Some(&RestoreFit {
                        family: Some(RecordKind::Repository),
                        set: RestoreSet::ContextProjects,
                        held: u64::from(LIMITS.judge_projects + 1),
                        additional: 0,
                        limit: u64::from(LIMITS.judge_projects),
                    })
                );
                assert!(d.repository(REPO).is_none());
            }
        }
    }
}

#[test]
fn an_adoption_refuses_a_prefix_whose_branches_exceed_authority_segments_before_a_call() {
    let mut d = domain();
    let mut adoption = fixtures::adoption();
    adoption.prefix = Box::from(b"temper/live-x/".as_slice());
    outputs(
        &mut d,
        Event::Application {
            role: None,
            carried: None,
            request: Token::new(4),
            token: Token::new(9),
            retention: Duration::from_secs(60),
            ask: Ask::Adopt(adoption),
        },
    );
    assert_eq!(
        d.application_outcome(Token::new(9)),
        Some(&ApplicationOutcome::ForgeRefused(AdoptionRefusal::TooLarge))
    );
    assert!(ready(&mut d).is_empty(), "no forge read for a prefix that cannot fit");
}

#[test]
fn an_adoption_accepts_a_prefix_whose_branches_fit_authority_segments() {
    use skein_forge::api;
    let mut d = domain();
    fixtures::begin_adoption(&mut d, Token::new(4), fixtures::adoption());
    for answer in [
        api::Answer::Permission(api::UserPermission {
            user: 1,
            login: Box::from(b"".as_slice()),
            permission: api::Permission::Write,
        }),
        api::Answer::Branches(Box::new([])),
        api::Answer::Settings(adopted().settings),
        api::Answer::Protection(None),
    ] {
        fixtures::answer_next(&mut d, Ok(answer));
    }
    let requests =
        fixtures::answer_next(&mut d, Ok(api::Answer::Collaborators { collaborators: Box::new([]), more: false }));
    let [.., Request::Adopted { result: Ok(value), .. }] = requests.as_ref() else { panic!("adoption accepted") };
    assert_eq!(value.repository.prefix.as_ref(), b"temper/");
}

#[test]
fn an_adoption_locates_its_repository_before_any_other_read() {
    let mut d = domain();
    d.configured.get_mut(&1).unwrap().mode = Mode::Connected;
    outputs(
        &mut d,
        Event::Application {
            role: None,
            carried: None,
            request: Token::new(4),
            token: Token::new(9),
            retention: Duration::from_secs(60),
            ask: Ask::Adopt(fixtures::adoption()),
        },
    );
    let sent = ready(&mut d);
    let [Request::Call { repository, op, .. }] = sent.as_ref() else { panic!("one locate call") };
    assert_eq!(*repository, skein_forge::api::Repository { forge: 1, repository: 0 });
    assert_eq!(
        *op,
        skein_forge::api::Op::Read(skein_forge::api::Read::Locate {
            owner: Box::from(&b"org"[..]),
            name: Box::from(&b"repo"[..]),
        })
    );
}

#[test]
fn an_adoption_refuses_fields_outside_its_bounds_before_a_call() {
    for field in 0_u8..5 {
        for bytes in [&[][..], &[b'a'; 129][..]] {
            let mut d = domain();
            let mut request = fixtures::adoption();
            match field {
                0 => request.host = bytes.into(),
                1 => request.owner = bytes.into(),
                2 => request.name = bytes.into(),
                3 => request.prefix = bytes.into(),
                4 => request.landing = bytes.into(),
                _ => panic!("field"),
            }
            outputs(
                &mut d,
                Event::Application {
                    role: None,
                    carried: None,
                    request: Token::new(4),
                    token: Token::new(9),
                    retention: Duration::from_secs(60),
                    ask: Ask::Adopt(request),
                },
            );
            assert_eq!(
                d.application_outcome(Token::new(9)),
                Some(&ApplicationOutcome::ForgeRefused(AdoptionRefusal::TooLarge)),
                "field {field}"
            );
            assert!(ready(&mut d).is_empty());
        }
    }
}

#[test]
fn a_core_refusal_is_kept_as_jigs_refusal() {
    let mut d = domain();
    outputs(
        &mut d,
        Event::Application {
            role: None,
            carried: None,
            request: Token::new(4),
            token: Token::new(9),
            retention: Duration::from_secs(60),
            ask: Ask::Adopt(fixtures::adoption()),
        },
    );
    let requests = outputs(
        &mut d,
        Event::ApplicationAnswered {
            request: Token::new(4),
            outcome: people::Outcome::Refused(people::Refusal::Unknown),
        },
    );
    assert!(requests.contains(&Request::ApplicationDone { request: Token::new(4), token: Token::new(9) }));
    assert_eq!(d.application_outcome(Token::new(9)), Some(&ApplicationOutcome::CoreRefused(people::Refusal::Unknown)));
}

#[test]
fn a_full_outcome_table_answers_the_application_again_busy() {
    let mut d = domain();
    for index in 1..=LIMITS.applications {
        let request = Token::new(u64::from(index));
        outputs(
            &mut d,
            Event::Application {
                role: None,
                carried: None,
                request,
                token: request,
                retention: Duration::from_secs(60),
                ask: Ask::Adopt(fixtures::adoption()),
            },
        );
        outputs(
            &mut d,
            Event::ApplicationAnswered { request, outcome: people::Outcome::Refused(people::Refusal::Unknown) },
        );
    }
    let requests = outputs(
        &mut d,
        Event::Application {
            role: None,
            carried: None,
            request: Token::new(99),
            token: Token::new(99),
            retention: Duration::from_secs(60),
            ask: Ask::Adopt(fixtures::adoption()),
        },
    );
    assert_eq!(requests.as_ref(), &[Request::ApplicationAgain { request: Token::new(99), why: people::Again::Busy }]);
    assert_eq!(d.application_outcome(Token::new(99)), None);
}

#[test]
fn a_kept_outcome_is_saved_with_the_end_of_its_retention() {
    let mut d = domain();
    let mut adoption = fixtures::adoption();
    adoption.forge = 9;
    let requests = outputs(
        &mut d,
        Event::Application {
            role: None,
            carried: None,
            request: Token::new(4),
            token: Token::new(9),
            retention: Duration::from_secs(60),
            ask: Ask::Adopt(adoption),
        },
    );
    let row = Stored::Application {
        token: 9,
        until: Wall::from_nanos(160_000_000_000),
        outcome: ApplicationOutcome::ForgeRefused(AdoptionRefusal::NoSuchForge),
    };
    assert!(requests.contains(&Request::Save { record: row, release: None, read_afresh: None }));
    assert!(requests.contains(&Request::ApplicationDone { request: Token::new(4), token: Token::new(9) }));
}

#[test]
fn an_outcome_past_its_retention_is_erased_at_the_next_application() {
    let mut d = domain();
    let outcome = ApplicationOutcome::CoreRefused(people::Refusal::Unknown);
    outputs(&mut d, Event::Restore { record: Stored::Application { token: 9, until: env().wall, outcome } });
    let requests = outputs(
        &mut d,
        Event::Application {
            role: None,
            carried: None,
            request: Token::new(4),
            token: Token::new(10),
            retention: Duration::from_secs(60),
            ask: Ask::Adopt(fixtures::adoption()),
        },
    );
    assert!(requests.contains(&Request::Erase { key: Key::Application(9) }));
    assert_eq!(d.application_outcome(Token::new(9)), None);
}

#[test]
fn not_installed_is_a_terminal_application_refusal_rather_than_a_retry() {
    let mut d = domain();
    d.configured.get_mut(&1).unwrap().mode = Mode::Connected;
    outputs(
        &mut d,
        Event::Application {
            role: None,
            carried: None,
            request: Token::new(4),
            token: Token::new(9),
            retention: Duration::from_secs(60),
            ask: Ask::Adopt(fixtures::adoption()),
        },
    );
    let sent = ready(&mut d);
    let [Request::Call { call, .. }] = sent.as_ref() else { panic!("locate call") };
    let requests = outputs(
        &mut d,
        Event::Client(skein_forge_client::Event::Answered {
            call: *call,
            cost: 0,
            free: 0,
            rate: None,
            result: Err(skein_forge::api::Error::NotInstalled),
        }),
    );
    assert!(requests.contains(&Request::ApplicationDone { request: Token::new(4), token: Token::new(9) }));
    assert_eq!(
        d.application_outcome(Token::new(9)),
        Some(&ApplicationOutcome::ForgeRefused(AdoptionRefusal::NotInstalled))
    );
}

#[test]
fn a_configured_forge_without_a_writer_refuses_adoption_before_any_read() {
    let mut d = Domain::new(
        &LIMITS,
        4,
        Config { push: None, namespace: Box::from(b"test".as_slice()), writers: Box::new([]) },
        &[GitIdentity { forge: 1, credential: 1 }],
        &configured(),
        false,
    )
    .unwrap();
    outputs(
        &mut d,
        Event::Application {
            role: None,
            carried: None,
            request: Token::new(4),
            token: Token::new(9),
            retention: Duration::from_secs(60),
            ask: Ask::Adopt(fixtures::adoption()),
        },
    );
    assert_eq!(
        d.application_outcome(Token::new(9)),
        Some(&ApplicationOutcome::ForgeRefused(AdoptionRefusal::Unreachable))
    );
    assert!(ready(&mut d).is_empty(), "no Locate before Connected");
}
#[test]
fn a_stopped_forge_keeps_its_writer_but_refuses_adoption_before_a_read() {
    let mut d = domain();
    outputs(
        &mut d,
        Event::Connection { forge: 1, state: Connection::Stopped { reason: Box::from(b"key refused".as_slice()) } },
    );
    outputs(
        &mut d,
        Event::Application {
            role: None,
            carried: None,
            request: Token::new(4),
            token: Token::new(9),
            retention: Duration::from_secs(60),
            ask: Ask::Adopt(fixtures::adoption()),
        },
    );
    assert_eq!(
        d.application_outcome(Token::new(9)),
        Some(&ApplicationOutcome::ForgeRefused(AdoptionRefusal::Unreachable))
    );
    assert_eq!(d.writers.get(&1), Some(&7));
    assert!(ready(&mut d).is_empty());
}

#[test]
fn each_admission_conflict_keeps_its_own_reason() {
    for why in [AdoptionRefusal::AnotherProject, AdoptionRefusal::HomeHeld, AdoptionRefusal::PrefixOccupied] {
        let mut d = domain();
        if why != AdoptionRefusal::PrefixOccupied {
            let mut row = adopted();
            if why == AdoptionRefusal::AnotherProject {
                row.project += 1;
            } else {
                row.provider.repository += 1;
            }
            outputs(&mut d, Event::Restore { record: Stored::Repository(row) });
            outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
            outputs(&mut d, Event::ReadAfresh);
            outputs(&mut d, Event::SettleOutbox);
        }
        let request = Token::new(4);
        let mut requests = fixtures::begin_adoption(&mut d, request, fixtures::adoption());
        if why == AdoptionRefusal::PrefixOccupied {
            fixtures::answer_next(
                &mut d,
                Ok(skein_forge::api::Answer::Permission(skein_forge::api::UserPermission {
                    user: 1,
                    login: Box::from(b"".as_slice()),
                    permission: skein_forge::api::Permission::Write,
                })),
            );
            requests = fixtures::answer_next(
                &mut d,
                Ok(skein_forge::api::Answer::Branches(Box::new([Box::from(b"temper/foreign".as_slice())]))),
            );
        }
        assert!(
            requests.contains(&Request::Adopted { reply_to: request, result: Err(skein_forge::api::Error::Refused) }),
            "{why:?}"
        );
        let requests = outputs(
            &mut d,
            Event::ApplicationAnswered { request, outcome: people::Outcome::Refused(people::Refusal::Unknown) },
        );
        let outcome = ApplicationOutcome::ForgeRefused(why);
        assert_eq!(d.application_outcome(request), Some(&outcome));
        assert!(requests.contains(&Request::Save {
            record: Stored::Application { token: request.raw(), until: Wall::from_nanos(160_000_000_000), outcome },
            release: None,
            read_afresh: None
        }));
        assert!(requests.contains(&Request::ApplicationDone { request, token: request }));
    }
}

#[test]
fn thirty_two_adoptions_are_answered_during_retention() {
    let limits = Limits { applications: 32, ..LIMITS };
    let env = Env { limits, ..env() };
    let mut d = Domain::new(
        &limits,
        4,
        Config {
            push: None,
            namespace: Box::from(b"test".as_slice()),
            writers: Box::new([skein_forge_client::Writer { forge: 1, author: 7 }]),
        },
        &[GitIdentity { forge: 1, credential: 1 }],
        &configured(),
        false,
    )
    .expect("valid connector");
    let mut out = Queue::uncapped();
    for number in 1..=32 {
        let request = Token::new(number);
        let mut adoption = fixtures::adoption();
        adoption.forge = 9;
        step(
            &mut d,
            &env,
            Event::Application {
                role: None,
                carried: None,
                request,
                token: request,
                retention: Duration::from_secs(86400),
                ask: Ask::Adopt(adoption),
            },
            &mut out,
        );
        let mut done = false;
        while let Some(output) = out.pop() {
            if output == (Request::ApplicationDone { request, token: request }) {
                done = true;
            }
        }
        assert!(done, "adoption {number}, including fifth and 32nd, must answer");
        assert_eq!(
            d.application_outcome(request),
            Some(&ApplicationOutcome::ForgeRefused(AdoptionRefusal::NoSuchForge))
        );
    }
    assert_eq!(
        d.application_outcome(Token::new(1)),
        Some(&ApplicationOutcome::ForgeRefused(AdoptionRefusal::NoSuchForge)),
        "earlier answers remain retained"
    );
    let request = Token::new(33);
    step(
        &mut d,
        &env,
        Event::Application {
            role: None,
            carried: None,
            request,
            token: request,
            retention: Duration::from_secs(86400),
            ask: Ask::Adopt(fixtures::adoption()),
        },
        &mut out,
    );
    assert_eq!(out.pop(), Some(Request::ApplicationAgain { request, why: people::Again::Busy }));
    assert!(out.is_empty());
}

fn start_access(d: &mut Domain) -> Box<[Request]> {
    outputs(
        d,
        Event::Application {
            role: None,
            carried: None,
            request: Token::new(4),
            token: Token::new(9),
            retention: Duration::from_secs(60),
            ask: Ask::Adopt(fixtures::adoption()),
        },
    )
}
fn asks_access(mode: Mode) {
    let mut d = domain();
    d.configured.get_mut(&1).unwrap().mode = mode;
    let requests = start_access(&mut d);
    assert_eq!(
        requests.as_ref(),
        &[Request::AdoptAccess {
            forge: 1,
            request: Token::new(4),
            owner: b"org".as_slice().into(),
            name: b"repo".as_slice().into()
        }]
    );
    assert!(ready(&mut d).is_empty(), "no bot read before access");
    outputs(&mut d, Event::AccessAnswered { forge: 1, request: Token::new(4), answer: Ok(()) });
    let sent = ready(&mut d);
    let [Request::Call { op, .. }] = sent.as_ref() else { panic!("one Locate after access") };
    assert_eq!(
        *op,
        skein_forge::api::Op::Read(skein_forge::api::Read::Locate {
            owner: b"org".as_slice().into(),
            name: b"repo".as_slice().into()
        })
    );
    assert!(
        outputs(&mut d, Event::AccessAnswered { forge: 1, request: Token::new(4), answer: Ok(()) }).is_empty(),
        "a duplicate cannot start another read"
    );
}
#[test]
fn an_adoption_on_a_managed_forge_asks_access_before_any_read() {
    asks_access(Mode::Managed);
}
#[test]
fn an_adoption_on_an_owned_forge_asks_access_before_any_read() {
    asks_access(Mode::Owned);
}
#[test]
fn a_refused_access_refuses_the_adoption_with_no_read() {
    for why in [
        AdoptionRefusal::NoAccess,
        AdoptionRefusal::NotFound,
        AdoptionRefusal::Unreachable,
        AdoptionRefusal::InFlight,
        AdoptionRefusal::InvalidAnswer,
    ] {
        let mut d = domain();
        start_access(&mut d);
        let requests = outputs(&mut d, Event::AccessAnswered { forge: 1, request: Token::new(4), answer: Err(why) });
        assert!(requests.contains(&Request::ApplicationDone { request: Token::new(4), token: Token::new(9) }));
        assert_eq!(d.application_outcome(Token::new(9)), Some(&ApplicationOutcome::ForgeRefused(why)));
        assert!(ready(&mut d).is_empty(), "no read after {why:?}");
        assert!(d.adoptions.is_empty());
        assert!(outputs(&mut d, Event::AccessAnswered { forge: 1, request: Token::new(4), answer: Ok(()) }).is_empty());
    }
}
#[test]
fn a_connected_forge_asks_no_access() {
    for (mode, kind) in [(Mode::Connected, Kind::Forgejo), (Mode::Fake, Kind::Forgejo), (Mode::Managed, Kind::GitHub)] {
        let mut d = domain();
        let config = d.configured.get_mut(&1).unwrap();
        config.mode = mode;
        config.kind = kind;
        assert!(start_access(&mut d).is_empty());
        let sent = ready(&mut d);
        let [Request::Call { op, .. }] = sent.as_ref() else { panic!("one Locate without setup") };
        assert_eq!(
            *op,
            skein_forge::api::Op::Read(skein_forge::api::Read::Locate {
                owner: b"org".as_slice().into(),
                name: b"repo".as_slice().into(),
            })
        );
    }
}
