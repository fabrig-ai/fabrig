//! A made repository's connector act (temper/docs/design/domain/forge.md, 4.5).
use jig_domain_core as core;
use jig_domain_people as people;
use skein_lib::{ReplyTo, Token};
use temper_engine_domain as engine;
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::{direct::Driver, fixtures, store::Store, walking};

fn config(granted: bool) -> engine::Config {
    let mut config = walking::config(701);
    config.forges[0].mode = forge::Mode::Owned;
    if granted {
        config.core.grants = Box::new([people::ActGrant {
            connector: config.core.connectors[0],
            may: Box::new([people::ActKind::Adopt]),
        }]);
    }
    config
}
fn driver(granted: bool) -> Driver {
    driver_with_requests(granted, fixtures::forge_limits().people.requests)
}
fn driver_with_requests(granted: bool, requests: u32) -> Driver {
    let mut limits = fixtures::forge_limits();
    limits.people.act_grants = 1;
    limits.people.requests = requests;
    let mut driver = Driver::with_repositories(
        Store::new(),
        std::array::from_fn(|_| config(granted)),
        &limits,
        &[(b"org", b"repo", 2)],
    );
    driver.settle();
    driver.store.seed(
        Box::new([
            engine::Record::Forge {
                row: Box::new(forge::Stored::Forge(forge::ForgeRecord {
                    git_program: None,
                    number: 1,
                    kind: forge::Kind::Forgejo,
                    mode: forge::Mode::Owned,
                    state: forge::ForgeState::Kept,
                    place: forge::Place {
                        api: forge::Address { host: b"forge.example".as_slice().into(), port: 3000, tls: None },
                        git: b"http://forge.example:3000".as_slice().into(),
                        web: None,
                    },
                    read_only: false,
                    bot: forge::Bot {
                        login: b"temper".as_slice().into(),
                        name: b"Temper".as_slice().into(),
                        email: Box::new([]),
                    },
                    asked: true,
                    first: None,
                })),
            },
            engine::Record::Forge { row: Box::new(forge::Stored::ForgeNumber { last: 1 }) },
        ]),
        Box::new([]),
    );
    driver.reopen(config(granted));
    driver.settle();
    driver.sign_in();
    driver.settle();
    driver
}
fn key() -> forge::MakingKey {
    forge::MakingKey { forge: 1, org: b"org".as_slice().into(), name: b"repo".as_slice().into() }
}
fn into(project: u32) -> forge::MakeInto {
    forge::MakeInto { project, home: true, host: b"forge.example".as_slice().into() }
}
fn event(driver: &mut Driver, event: forge::Event) {
    driver.send(engine::Event::Forge(Box::new(event)));
    driver.settle();
}
fn make(driver: &mut Driver, into: Option<forge::MakeInto>, scope: people::ApplicationScope) {
    driver.send(engine::Event::Party(engine::PartyInput::Application {
        scope,
        reply_to: ReplyTo::new(Token::new(51)),
        sign_in: driver.session(),
        key: [51; 16],
        digest: [51; 32],
        carried: None,
        request: engine::Application::Forge(forge::Ask::MakeRepository {
            forge: 1,
            org: key().org,
            name: key().name,
            from: forge::MakeFrom::Empty,
            into,
        }),
    }));
    driver.settle();
}
fn kept(driver: &Driver) -> bool {
    driver.delivered.iter().any(|delivery| {
        matches!(
            delivery,
            engine::Delivery::Application {
                outcome: Some(engine::ApplicationAnswer::Make(forge::MakeAnswer::Kept)),
                ..
            }
        )
    })
}
fn row(driver: &Driver) -> &forge::RepositoryMaking {
    let engine::Record::Forge { row } = &driver.store.rows[&engine::Key::Forge(forge::Key::Making(key()))] else {
        panic!("making record")
    };
    let forge::Stored::Making(row) = row.as_ref() else { panic!("making row") };
    row
}
#[test]
fn make_scope_is_the_project_into_names() {
    let mut d = driver(true);
    make(&mut d, Some(into(2)), people::ApplicationScope::Deployment);
    assert!(!kept(&d), "a deployment owner cannot bypass the named project's admission");
    assert!(!d.store.rows.contains_key(&engine::Key::Forge(forge::Key::Making(key()))));
    assert!(
        d.delivered.iter().any(|delivery| matches!(
            delivery,
            engine::Delivery::WebReply {
                reply: people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Role { held: None })),
                ..
            }
        )),
        "{:?}",
        d.delivered
    );
}
#[test]
fn make_scope_is_the_deployment_without_into() {
    let mut d = driver(true);
    make(&mut d, None, people::ApplicationScope::Project(2));
    assert!(kept(&d), "the client's project scope must not replace deployment admission: {:?}", d.delivered);
    assert_eq!(row(&d).into, None);
}
#[test]
fn a_kept_making_is_answered_and_marks_forges() {
    let mut d = driver(true);
    let watcher = Token::new(900);
    d.send(engine::Event::Party(engine::PartyInput::Watch {
        watcher,
        sign_in: d.session(),
        subject: core::documents::Watched::Application {
            scope: people::ApplicationScope::Deployment,
            connector: config(true).core.connectors[0],
            subject: Token::new(0),
        },
        window: core::documents::Window { rows: 4, after: None },
    }));
    d.settle();
    d.send(engine::Event::Party(engine::PartyInput::Delivered { watcher }));
    d.settle();
    let before = d.delivered.len();
    make(&mut d, Some(into(1)), people::ApplicationScope::Own);
    assert!(kept(&d), "{:?}", d.delivered);
    assert_eq!(row(&d).state, forge::MakingState::Waiting);
    assert!(d.delivered[before..].iter().any(|delivery| matches!(delivery,
        engine::Delivery::Forges { watcher: actual, forges } if *actual == watcher && forges.rows[0].makings.len() == 1
    )), "the forges watch must see the kept making");
}

fn made(d: &mut Driver) {
    make(d, Some(into(1)), people::ApplicationScope::Own);
    assert!(kept(d), "making admitted");
    event(d, forge::Event::MakingStarted { key: key() });
    event(d, forge::Event::MakingMade { key: key(), id: 2, branch: b"main".as_slice().into() });
}
fn acted_adoptions(d: &Driver) -> usize {
    d.store
        .rows
        .values()
        .filter(|row| {
            matches!(
                row,
                engine::Record::Core(core::Record::People(people::Stored::Acted {
                    act: people::Act::Adopt { project: 1, .. },
                    outcome: people::Outcome::Adopted { project: 1 },
                    ..
                }))
            )
        })
        .count()
}
fn adopted(d: &Driver) {
    assert_eq!(acted_adoptions(d), 1, "jig must retain the connector act's adoption answer");
    let engine::Record::Forge { row } = &d.store.rows
        [&engine::Key::Forge(forge::Key::Repository(skein_forge::api::Repository { forge: 1, repository: 2 }))]
    else {
        panic!("adopted repository")
    };
    let forge::Stored::Repository(row) = row.as_ref() else { panic!("repository row") };
    assert_eq!(row.project, 1);
    assert_eq!(row.role, forge::Role::Owned);
    assert!(row.home);
    assert_eq!(row.owner.as_ref(), b"org");
    assert_eq!(row.name.as_ref(), b"repo");
    assert!(
        d.store.rows.values().any(|record| matches!(record,
            engine::Record::Core(core::Record::People(people::Stored::Acted {
                act: people::Act::Adopt { adoption, .. }, ..
            })) if adoption.options[5].as_ref() == b"main"
        )),
        "the act lands on the branch the making returned"
    );
    assert_eq!(row.prefix.as_ref(), b"temper/");
    assert!(row.ci);
    assert!(row.checks.is_empty());
    assert!(!d.store.rows.contains_key(&engine::Key::Forge(forge::Key::Making(key()))), "adoption erases the making");
}
#[test]
fn made_into_a_project_is_adopted_by_the_connectors_act() {
    let mut d = driver(true);
    made(&mut d);
    event(&mut d, forge::Event::MakingAdopt { key: key() });
    adopted(&d);
    let roles = &d.store.rows[&engine::Key::Core(core::Key::People(people::Key::Roles(1)))];
    assert!(
        matches!(roles, engine::Record::Core(core::Record::People(people::Stored::Roles { holdings, .. }))
        if holdings.iter().any(|holding| holding.role == people::Role::Member)),
        "adoption seeds the human write collaborator"
    );
    assert!(
        !d.store.rows.values().any(|record| matches!(record,
            engine::Record::Core(core::Record::People(people::Stored::Person { identity, .. }))
            if identity.key == temper_engine_domain_forge::user_key(1, 1_u64)
        )),
        "temper's writer is not seeded as a person"
    );
}
#[test]
fn a_restart_during_the_act_joins_it_and_adopts_once() {
    let mut d = driver(true);
    made(&mut d);
    d.send(engine::Event::Forge(Box::new(forge::Event::MakingAdopt { key: key() })));
    let mut cut = None;
    for _ in 0..200 {
        d.advance(false);
        cut = d.store.pending.iter().flat_map(|(_, writes)| writes).find_map(|write| {
            if let engine::Write::Save(engine::Record::Core(core::Record::People(people::Stored::Acted {
                key,
                act: people::Act::Adopt { .. },
                ..
            }))) = write
            {
                Some(*key)
            } else {
                None
            }
        });
        if cut.is_some() {
            break;
        }
        if !d.store.pending.is_empty() {
            let number = d.store.apply();
            d.send(engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
        }
    }
    let cut = cut.expect("cut before the act answer and making erasure commit together");
    assert!(matches!(row(&d).state, forge::MakingState::Adopting { .. }));
    d.reopen(config(true));
    d.forge = Some(temper_engine_domain_world::forge_stand::ForgeStand::with(&[(b"org", b"repo", 2)]));
    d.settle();
    d.send(engine::Event::Forge(Box::new(forge::Event::MakingAdopt { key: key() })));
    d.send(engine::Event::Forge(Box::new(forge::Event::MakingAdopt { key: key() })));
    d.settle();
    adopted(&d);
    assert!(
        d.store.rows.contains_key(&engine::Key::Core(core::Key::People(people::Key::Acted(cut)))),
        "the resumed act retains the same connector key"
    );
    let calls = d.forge.as_ref().expect("forge stand").calls;
    d.reopen(config(true));
    d.settle();
    event(&mut d, forge::Event::MakingAdopt { key: key() });
    adopted(&d);
    assert_eq!(d.forge.as_ref().expect("forge stand").calls, calls, "a kept act cannot adopt again after restart");
}
#[test]
fn a_document_with_no_grant_says_so_in_the_row() {
    let mut d = driver(false);
    made(&mut d);
    event(&mut d, forge::Event::MakingAdopt { key: key() });
    let forge::MakingState::Failed(failed) = &row(&d).state else { panic!("missing grant fails the row") };
    assert_eq!(failed.step, forge::MakeStep::Adoption);
    assert_eq!(failed.reason, forge::MakeFailure::NotAllowed);
    let words = std::str::from_utf8(&failed.words).expect("refusal is text");
    assert!(words.contains("adopt") && words.contains("engine.acts"), "{words}");
}
#[test]
fn jig_busy_is_asked_again() {
    let mut d = driver_with_requests(true, 1);
    made(&mut d);
    event(&mut d, forge::Event::MakingAdopt { key: key() });
    assert!(matches!(row(&d).state, forge::MakingState::Adopting { .. }), "Again must leave the making retryable");
    d.fire_at(skein_lib::Duration::from_secs(121));
    d.settle();
    event(&mut d, forge::Event::MakingAdopt { key: key() });
    adopted(&d);
}
#[test]
fn a_deleted_project_fails_the_row() {
    let mut d = driver(true);
    made(&mut d);
    d.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(52)),
        sign_in: d.session(),
        key: [52; 16],
        ask: people::Ask::RemoveProject { project: 1, grace: None },
    }));
    d.settle();
    event(&mut d, forge::Event::MakingAdopt { key: key() });
    let forge::MakingState::Failed(failed) = &row(&d).state else { panic!("deleted project fails the making") };
    assert_eq!(failed.step, forge::MakeStep::Adoption);
    assert_eq!(failed.reason, forge::MakeFailure::NotAllowed);
    assert!(std::str::from_utf8(&failed.words).expect("refusal is text").contains("project"));
    assert_eq!(acted_adoptions(&d), 0);
}

#[test]
fn a_copy_mark_is_returned_to_the_service() {
    let mut d = driver(true);
    make(&mut d, None, people::ApplicationScope::Own);
    event(&mut d, forge::Event::MakingCopying { key: key() });
    assert!(row(&d).copying);
    assert!(
        d.immediate.iter().any(|request| matches!(request,
            engine::Request::MakingCopied { key: actual } if *actual == key()
        )),
        "the service must hear the connector's copy mark"
    );
}
