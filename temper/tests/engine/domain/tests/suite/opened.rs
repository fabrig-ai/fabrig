//! A person's opening is their own Create batch, not the deployment's
//! (temper/docs/design/domain/forge/change/opened.md, 2 and 3).
use jig_domain_authority as authority;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{ReplyTo, Token};
use temper_engine_domain as root;
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::{direct::Driver, fixtures, forge_stand::ForgeStand, store::Store, walking};

const NO_TASKS: [&tasks::TaskRecord; 0] = [];

fn config() -> root::Config {
    let mut config = walking::config(71);
    let charter = config.core.defaults.charters[0].clone();
    let mut review = charter.clone();
    review.number = 4;
    config.core.defaults.charters = Box::new([charter, review]);
    let mut ceiling = config.core.defaults.rules.ceiling.clone();
    ceiling.delegation = authority::Delegation {
        kinds: Box::new([
            authority::Executor::Charter(1),
            authority::Executor::Charter(4),
            authority::Executor::Procedure(2),
            authority::Executor::Role(1),
            authority::Executor::Job(forge::MERGE_KIND),
        ]),
        tasks: 30,
        depth: 4,
    };
    ceiling.grants = [1, 2, 3, 4, 10, 11, 12, 17]
        .into_iter()
        .map(|kind| authority::Grant {
            connector: 0,
            kind,
            pattern: authority::Pattern {
                segments: Box::new([b"forge".as_slice().into()]),
                last: authority::Last::Open(Box::new([])),
            },
        })
        .collect();
    config.core.defaults.rules.ceiling = ceiling.clone();
    let policy = &mut config.core.seed_projects[0].policy.policy;
    policy.ceiling = ceiling.clone();
    ceiling.budget.spend = 500;
    policy.roles[0].authority = ceiling;
    config
}
fn limits() -> root::Limits {
    let mut limits = fixtures::forge_limits();
    limits.tasks.charters = 4;
    limits.tasks.tree_tasks = 20;
    limits.tasks.depth = 4;
    limits.tasks.parameters = 5;
    limits.tasks.spec_bytes = 512;
    limits.tasks.authority_grants = 16;
    limits.tasks.authority_segments = 8;
    limits.tasks.authority_bytes = 4096;
    limits.tasks.executor_kinds = 5;
    limits.people.spec_bytes = 512;
    limits.people.amendment_bytes = 8192;
    limits.authority.grants = 16;
    limits.authority.executors = 5;
    limits.authority.segments = 8;
    limits.documents.grants = 16;
    limits.documents.executors = 5;
    limits.notes.scopes = 19;
    limits.journal.transcript_bytes = 65_536;
    limits.journal.run_bytes = 16_384;
    assert!(root::valid(&limits), "opening fixture limits");
    limits
}
fn setup(configurations: [root::Config; 2], limits: &root::Limits, repositories: u32) -> Driver {
    let mut driver = Driver::configured(Store::with_limits(limits), configurations, limits);
    driver.forge = Some(ForgeStand::with(&[(b"org", b"repo", 2), (b"org", b"other", 3)]));
    driver.connect_forges();
    driver.settle();
    driver.sign_in();
    driver.settle();
    for number in 0..repositories {
        let mut adoption = fixtures::adoption(1);
        if number == 1 {
            adoption.name = b"other".as_slice().into();
            adoption.home = false;
        }
        application(
            &mut driver,
            root::Application::Forge(forge::Ask::Adopt(adoption)),
            u8::try_from(90 + number).expect("opening fixture"),
        );
        assert!(matches!(answer(&driver), forge::ApplicationOutcome::Adopted(_)));
    }
    driver
}
fn application(driver: &mut Driver, request: root::Application, key: u8) {
    driver
        .delivered
        .retain(|delivery| matches!(delivery, root::Delivery::WebReply { reply: people::Reply::SignedIn { .. }, .. }));
    // The root must use the opening's project, not the caller's envelope.
    let scope = if matches!(request, root::Application::OpenChange(_)) {
        people::ApplicationScope::Own
    } else {
        people::ApplicationScope::Project(1)
    };
    driver.send(root::Event::Party(root::PartyInput::Application {
        scope,
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: driver.session(),
        key: [key; 16],
        digest: [key; 32],
        carried: None,
        request,
    }));
    driver.settle();
}
fn answer(driver: &Driver) -> forge::ApplicationOutcome {
    driver
        .delivered
        .iter()
        .find_map(|delivery| {
            if let root::Delivery::Application { outcome: Some(root::ApplicationAnswer::Adopt(outcome)), .. } = delivery
            {
                Some(outcome.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("application answer: {:?}", driver.delivered))
}
fn call() -> forge::OpenChange {
    forge::OpenChange {
        project: 1,
        repository: None,
        branch: b"docs/x".as_slice().into(),
        at: None,
        into: None,
        title: b"Fix the docs".as_slice().into(),
        body: None,
        budget: forge::OpenBudget::Amount(500),
    }
}
// Assert on admission's saved row, independently of taking starting and archiving it.
fn task_rows(driver: &Driver) -> Vec<&tasks::TaskRecord> {
    let mut created = std::collections::BTreeMap::new();
    for transaction in &driver.transactions {
        for write in transaction {
            if let root::Write::Save(root::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) =
                write
            {
                created.entry(row.number).or_insert(row.as_ref());
            }
        }
    }
    created.into_values().collect()
}
fn open(driver: &mut Driver, call: forge::OpenChange) -> u64 {
    application(driver, root::Application::OpenChange(call), 101);
    let forge::ApplicationOutcome::Opened { task } = answer(driver) else { panic!("opened: {:?}", answer(driver)) };
    task
}

#[test]
fn a_persons_opening_makes_one_change_with_the_shared_builders_authority() {
    let mut driver = setup([config(), config()], &limits(), 1);
    let task = open(&mut driver, call());
    let rows = task_rows(&driver);
    let [row] = rows.as_slice() else { panic!("one change: {rows:?}") };
    assert_eq!(row.number, task);
    assert_eq!(row.requester, tasks::Party::Person(1));
    assert!(matches!(row.funder, tasks::Funder::Pool { project: 1, person: 1, .. }));
    assert_eq!(row.authority.budget.spend, 500);
    assert_eq!(row.executor, tasks::Executor::Procedure { connector: 0, code: 2 });
    assert_eq!(
        row.authority.delegation.kinds.as_ref(),
        &[
            tasks::AuthorityExecutor::Charter(1),
            tasks::AuthorityExecutor::Charter(4),
            tasks::AuthorityExecutor::Job(forge::MERGE_KIND),
        ]
    );
    assert_eq!(row.authority.delegation.tasks, 19.min(driver.env.limits.forge.change_policy.tasks));
    assert_eq!(row.authority.delegation.depth, 2);
    assert_eq!(row.authority.notes, tasks::Scopes(0));
    assert!(row.authority.note_resources.is_empty());
    assert_eq!(row.spec.words.as_ref(), b"Opened by person 1 from docs/x at its head into main.");
    assert_eq!(
        row.spec.parameters.as_ref(),
        &[
            tasks::Parameter::Resource { name: 1, connector: 0, resource: forge::tag(1, 2) },
            tasks::Parameter::Bytes { name: 2, value: b"main".as_slice().into() },
            tasks::Parameter::Bytes { name: 10, value: b"Fix the docs".as_slice().into() },
            tasks::Parameter::Bytes { name: 19, value: b"docs/x".as_slice().into() },
        ]
    );
    assert_eq!(row.authority.grants.len(), 9);
    for grant in &row.authority.grants {
        assert_eq!(grant.connector, 0);
        assert!(forge::CHANGE_KINDS.contains(&grant.kind) || forge::READ_KINDS.contains(&grant.kind));
        assert_eq!(grant.pattern.segments[0].as_ref(), b"forge");
        assert!(grant.pattern.segments.iter().any(|segment| segment.as_ref() == b"org"));
    }
}

#[test]
fn a_named_head_body_landing_and_as_allowed_survive_the_expansion() {
    for width in [40, 64] {
        let mut driver = setup([config(), config()], &limits(), 1);
        let mut request = call();
        request.repository =
            Some(forge::DraftRepository { owner: b"org".as_slice().into(), name: b"repo".as_slice().into() });
        request.at = Some(vec![b'a'; width].into_boxed_slice());
        request.into = Some(b"release".as_slice().into());
        request.body = Some(b"Keep the examples.".as_slice().into());
        request.budget = forge::OpenBudget::AsAllowed;
        let task = open(&mut driver, request);
        let rows = task_rows(&driver);
        let row = rows.iter().find(|row| row.number == task).expect("opening fixture");
        assert_eq!(row.authority.budget.spend, 500);
        assert_eq!(
            row.spec.words.as_ref(),
            format!("Opened by person 1 from docs/x at {} into release.\n\nKeep the examples.", "a".repeat(width))
                .as_bytes()
        );
        assert!(row.spec.parameters.contains(&tasks::Parameter::Bytes {
            name: 19,
            value: format!("docs/x:{}", "a".repeat(width)).into_bytes().into_boxed_slice()
        }));
        assert!(
            row.spec.parameters.contains(&tasks::Parameter::Bytes { name: 2, value: b"release".as_slice().into() })
        );
    }
}

fn gated_config() -> root::Config {
    let mut config = config();
    config.kinds_stock = root::landing_stock(
        &[root::LandingRule {
            connector: 0,
            kind: 6,
            pattern: authority::Pattern {
                segments: Box::new([
                    b"forge".as_slice().into(),
                    b"forge.example".as_slice().into(),
                    b"org".as_slice().into(),
                    b"repo".as_slice().into(),
                    b"branch".as_slice().into(),
                ]),
                last: authority::Last::Exact(b"main".as_slice().into()),
            },
            enforce: true,
            ci: false,
            up_to_date: false,
            gates: Box::new([]),
            approvals: Box::new([root::Approval { role: 1, people: 1, freshness: root::Freshness::Exact }]),
            sets: Box::new([]),
        }],
        &skein_lib::Map::with_capacity(0),
        1,
        4,
    );
    let mut stock = config.kinds_stock.into_vec();
    stock.push(forge::settings::Stock {
        kind: 6,
        scope: 1,
        number: 1,
        value: forge::settings::SizedValue {
            value: forge::settings::Value::Questions(temper_engine_domain_forge_change::Route::Up),
            size: 1,
        },
    });
    config.kinds_stock = stock.into_boxed_slice();
    config
}

#[test]
fn a_covering_person_gate_adds_its_role_and_up_is_encoded_only_when_configured() {
    for into in [b"main".as_slice(), b"release".as_slice()] {
        let mut driver = setup([gated_config(), gated_config()], &limits(), 1);
        let mut request = call();
        request.into = Some(into.into());
        let task = open(&mut driver, request);
        let rows = task_rows(&driver);
        let row = rows.iter().find(|row| row.number == task).expect("opening fixture");
        assert_eq!(row.authority.delegation.kinds.contains(&tasks::AuthorityExecutor::Role(1)), into == b"main");
        assert!(row.spec.parameters.contains(&tasks::Parameter::Number { name: 14, value: 2 }));
        assert!(!row.spec.parameters.iter().any(|parameter| matches!(
            parameter,
            tasks::Parameter::Bytes { name: 3 | 4 | 11, .. } | tasks::Parameter::Number { name: 11, .. }
        )));
    }
}

#[test]
fn repository_resolution_refuses_without_making_any_task() {
    for count in [0, 2] {
        let mut driver = setup([config(), config()], &limits(), count);
        application(&mut driver, root::Application::OpenChange(call()), 101);
        if count == 0 {
            assert_eq!(answer(&driver), forge::ApplicationOutcome::OpenRefused(forge::OpenRefusal::NoRepository));
        } else if count == 2 {
            let forge::ApplicationOutcome::OpenRefused(forge::OpenRefusal::Several(names)) = answer(&driver) else {
                panic!("several repositories")
            };
            assert_eq!(
                names.as_ref(),
                &[
                    forge::DraftRepository { owner: b"org".as_slice().into(), name: b"repo".as_slice().into() },
                    forge::DraftRepository { owner: b"org".as_slice().into(), name: b"other".as_slice().into() },
                ]
            );
        } else {
            continue;
        }
        assert_eq!(task_rows(&driver), NO_TASKS);
    }
    for case in 0..4 {
        let mut driver = setup([config(), config()], &limits(), 1);
        let mut request = call();
        let reason = match case {
            0 => {
                request.repository = Some(forge::DraftRepository {
                    owner: b"elsewhere".as_slice().into(),
                    name: b"repo".as_slice().into(),
                });
                forge::OpenRefusal::NotARepository
            }
            1 => {
                request.into = Some(b"temper/7/g8".as_slice().into());
                forge::OpenRefusal::UnderPrefix {
                    owner: b"org".as_slice().into(),
                    name: b"repo".as_slice().into(),
                    prefix: b"temper/".as_slice().into(),
                }
            }
            2 => {
                request.branch =
                    vec![b'x'; usize::try_from(driver.env.limits.forge.name_bytes).expect("opening fixture") + 1]
                        .into_boxed_slice();
                forge::OpenRefusal::NamePast { bound: driver.env.limits.forge.name_bytes }
            }
            3 => {
                request.into = Some(
                    vec![b'x'; usize::try_from(driver.env.limits.forge.name_bytes).expect("opening fixture") + 1]
                        .into_boxed_slice(),
                );
                forge::OpenRefusal::NamePast { bound: driver.env.limits.forge.name_bytes }
            }
            _ => unreachable!(),
        };
        application(&mut driver, root::Application::OpenChange(request), 101);
        assert_eq!(answer(&driver), forge::ApplicationOutcome::OpenRefused(reason));
        assert_eq!(task_rows(&driver), NO_TASKS);
    }
}

#[test]
fn opening_refusals_keep_the_cores_authority_facts_and_make_nothing() {
    for case in 0..7 {
        let mut configurations = [config(), config()];
        for configuration in &mut configurations {
            let role = &mut configuration.core.seed_projects[0].policy.policy.roles[0].authority;
            match case {
                0 => {
                    role.delegation.kinds = role
                        .delegation
                        .kinds
                        .iter()
                        .copied()
                        .filter(|kind| *kind != authority::Executor::Charter(4))
                        .collect();
                }
                2 => role.delegation.tasks = 1,
                3 => role.delegation.depth = 1,
                _ => {}
            }
        }
        let mut limits = limits();
        let mut request = call();
        let lacking = match case {
            0 => people::Lack::TaskKind { executor: None },
            1 => {
                request.budget = forge::OpenBudget::Amount(1);
                people::Lack::Budget { needs: 2, left: 1 }
            }
            2 => people::Lack::Tasks { left: 1 },
            3 => people::Lack::Depth { most: 0 },
            4 => {
                request.budget = forge::OpenBudget::Amount(501);
                people::Lack::Budget { needs: 501, left: 500 }
            }
            5 => {
                limits.tasks.parameters = 3;
                people::Lack::Reference
            }
            6 => {
                request.budget = forge::OpenBudget::Amount(0);
                people::Lack::Budget { needs: 2, left: 0 }
            }
            _ => unreachable!(),
        };
        let mut driver = setup(configurations, &limits, 1);
        application(&mut driver, root::Application::OpenChange(request), 101);
        let forge::ApplicationOutcome::CoreRefused(refused) = answer(&driver) else {
            panic!("core refusal, case {case}: {:?}", answer(&driver))
        };
        if case == 5 {
            assert_eq!(refused, people::Refusal::Limit { which: people::LimitKind::Spec });
        } else if let people::Refusal::Authority { lacking: actual, .. } = refused {
            assert_eq!(actual, lacking, "case {case}");
        } else {
            panic!("authority facts: {refused:?}");
        }
        assert_eq!(task_rows(&driver), NO_TASKS, "case {case}");
    }
}

#[test]
fn an_opening_resent_after_its_answer_made_room_is_unknown_and_makes_nothing() {
    let mut limits = limits();
    limits.forge.applications = 3;
    limits.people.requests = 5; // The adoption, the open and three answers displacing it.
    assert!(limits.forge.applications > limits.forge.adoptions);
    let mut driver = setup([config(), config()], &limits, 1);
    let task = open(&mut driver, call());
    assert_eq!(task_rows(&driver).len(), 1);
    for key in 102..=104 {
        let mut request = call();
        request.into = Some(b"temper/7/g8".as_slice().into());
        application(&mut driver, root::Application::OpenChange(request), key);
        assert!(matches!(
            answer(&driver),
            forge::ApplicationOutcome::OpenRefused(forge::OpenRefusal::UnderPrefix { .. })
        ));
    }
    assert!(!driver.store.rows.values().any(|row| matches!(row, root::Record::Forge { row } if matches!(row.as_ref(), forge::Stored::Application { outcome: forge::ApplicationOutcome::Opened { task: held }, .. } if *held == task))));
    let before = task_rows(&driver).len();
    application(&mut driver, root::Application::OpenChange(call()), 101);
    // The world exposes the root's absent outcome; the service encodes it Unknown.
    assert!(
        driver.delivered.iter().any(|delivery| matches!(delivery, root::Delivery::Application { outcome: None, .. })),
        "unknown answer: {:?}",
        driver.delivered
    );
    assert_eq!(task_rows(&driver).len(), before);
}

#[test]
fn an_opening_replays_its_durable_answer_before_and_after_restart() {
    let mut driver = setup([config(), config()], &limits(), 1);
    let task = open(&mut driver, call());
    assert_eq!(open(&mut driver, call()), task);
    assert_eq!(task_rows(&driver).len(), 1);
    assert!(driver.store.rows.values().any(|row| matches!(row, root::Record::Forge { row } if matches!(row.as_ref(), forge::Stored::Application { outcome: forge::ApplicationOutcome::Opened { task: held }, .. } if *held == task))));
    driver.reopen(config());
    driver.settle();
    driver.sign_in();
    driver.settle();
    assert_eq!(open(&mut driver, call()), task);
    assert_eq!(task_rows(&driver).len(), 1);
}
