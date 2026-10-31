//! Playground acts and restart reset (temper/docs/design/draft/playground-reset.md, 6.1–6.2).
use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_people as people;
use skein_lib::{Duration, Map, ReplyTo, Token};
use temper_engine_domain::{self as engine, Delivery};
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::{direct::Driver, fixtures, forge_stand::ForgeStand, store::Store, walking};

fn place() -> forge::Place {
    forge::Place {
        api: forge::Address { host: b"forge.example".as_slice().into(), port: 3000, tls: None },
        git: b"http://forge.example:3000".as_slice().into(),
        web: None,
    }
}
fn config(granted: bool) -> engine::Config {
    let mut config = walking::config(71);
    config.forges = Box::new([]);
    config.machine = Some(forge::Machine { owned: None, fake: Some(place()), github: place() });
    config.playground = Some(forge::PlaygroundConfig { provider: 1, deadline: Duration::from_secs(10) });
    config.core.seed_projects[0].policy.permissions = Box::new([]);
    let seed = &config.core.seed_projects[0].policy;
    config.core.defaults.policies = Map::with_capacity(1);
    config
        .core
        .defaults
        .policies
        .insert(
            1,
            core::ConfiguredPolicy {
                policy: seed.policy.clone(),
                permissions: seed.permissions.clone(),
                templates: seed.templates.clone(),
            },
        )
        .expect("playground fixture");
    if granted {
        config.core.grants = Box::new([people::ActGrant {
            connector: config.core.connectors[0],
            may: Box::new([people::ActKind::RemoveProject, people::ActKind::RemoveAccount]),
        }]);
    }
    config
}
fn driver(granted: bool) -> (Driver, u64) {
    let mut limits = fixtures::forge_limits();
    limits.authority.projects = 2;
    limits.people.projects = 3;
    limits.people.act_grants = 1;
    limits.people.pending = 8;
    limits.people.requests = 8;
    limits.journal.writes = 2140;
    limits.journal.deliveries += 4;
    limits.journal.held += 12;
    let mut bootstrap = Driver::cold(Store::new(), config(granted), &limits);
    bootstrap.period_sent = true;
    bootstrap.pump();
    assert!(!bootstrap.stopped, "bootstrap: {:?}", bootstrap.root.restore_failure());
    let store = bootstrap.store;
    let mut d = Driver::cold(store, config(granted), &limits);
    d.forge = Some(ForgeStand::with(&[(b"playground", b"widget", 2)]));
    d.pump();
    assert!(!d.stopped, "restore: {:?}", d.root.restore_failure());
    d.sign_in();
    d.settle();
    let sign_in = d.session();
    d.delivered.clear();
    (d, sign_in)
}
fn application(d: &mut Driver, sign_in: u64, number: u8, ask: forge::Ask, carry: bool) {
    d.send(engine::Event::Party(engine::PartyInput::Application {
        reply_to: ReplyTo::new(Token::new(u64::from(number))),
        sign_in,
        key: [number; 16],
        digest: [number; 32],
        scope: people::ApplicationScope::Deployment,
        request: engine::Application::Forge(ask),
        carried: carry.then_some(core::Carried { number: u64::from(number), digest: [7; 32] }),
    }));
}
fn start(d: &mut Driver, sign_in: u64) {
    application(d, sign_in, 81, forge::Ask::StartPlayground, true);
    d.settle();
    d.send(engine::Event::Forge(Box::new(forge::Event::Connection {
        forge: 1,
        state: forge::Connection::Connected { id: 1, login: b"temper".as_slice().into() },
    })));
    d.settle();
}
fn row(d: &Driver) -> Option<forge::Playground> {
    d.store.rows.values().find_map(|record| {
        if let engine::Record::Forge { row } = record
            && let forge::Stored::Playground(row) = row.as_ref()
        {
            Some(*row)
        } else {
            None
        }
    })
}
fn account_rows(d: &Driver) -> Vec<&accounts::Stored> {
    d.store
        .rows
        .values()
        .filter_map(|record| {
            if let engine::Record::Core(core::Record::Accounts(row @ accounts::Stored::Account { .. })) = record {
                Some(row)
            } else {
                None
            }
        })
        .collect()
}
fn restart(d: &mut Driver, granted: bool) {
    d.root = engine::Domain::new(config(granted), &d.env.limits).expect("playground fixture");
    d.events.clear();
    d.delivered.clear();
    d.send(engine::Event::Restart { pending: false });
    d.settle();
}
fn finish_forge_removal(d: &mut Driver) {
    d.send(engine::Event::Forge(Box::new(forge::Event::SecretsErased { forge: 1 })));
    d.send(engine::Event::Forge(Box::new(forge::Event::BoxRemoved { forge: 1 })));
    d.settle();
}
#[test]
fn a_start_adds_the_account_makes_the_project_sets_its_uses_and_adopts() {
    let (mut d, sign_in) = driver(true);
    start(&mut d, sign_in);
    let row = row(&d).expect("playground fixture");
    assert_eq!(row.state, forge::PlaygroundState::Running);
    assert_eq!(row.project, Some(2));
    assert!(row.account.is_some());
    assert!(d.delivered.iter().any(|delivery| matches!(
        delivery,
        Delivery::Application {
            outcome: Some(engine::ApplicationAnswer::StartPlayground(forge::StartAnswer::Running)),
            ..
        }
    )));
    assert!(d.store.rows.values().any(|record| matches!(record, engine::Record::Forge { row }
        if matches!(row.as_ref(), forge::Stored::Repository(row) if row.project == 2 && row.owner.as_ref() == b"playground" && row.name.as_ref() == b"widget" && row.prefix.as_ref() == b"temper/" && row.home && row.ci))));
}
#[test]
fn the_scripted_account_uses_name_the_playground_project_alone() {
    let (mut d, sign_in) = driver(true);
    start(&mut d, sign_in);
    assert!(account_rows(&d).iter().any(|row| matches!(row, accounts::Stored::Account {
        holder: accounts::Holder::Deployment, uses: Some(uses), ..
    } if uses.as_ref() == [accounts::Use::Project(2)])));
}
#[test]
fn a_held_deployment_account_is_not_added_again() {
    let (mut d, sign_in) = driver(true);
    d.send(engine::Event::Party(engine::PartyInput::Key {
        reply_to: ReplyTo::new(Token::new(90)),
        sign_in,
        key: [90; 16],
        ask: Box::new(people::Ask::AccountChange {
            holder: people::AccountHolder::Deployment,
            change: people::AccountChange::AddKey { provider: 1, join: None, digest: [7; 32] },
        }),
        carried: core::Carried { number: 90, digest: [7; 32] },
    }));
    d.settle();
    start(&mut d, sign_in);
    assert_eq!(row(&d).expect("playground fixture").state, forge::PlaygroundState::Running);
    assert_eq!(account_rows(&d).len(), 1);
    assert!(d.delivered.iter().any(|delivery| matches!(delivery, Delivery::CarriedUnused { number: 81 })));
}
#[test]
fn the_reset_removes_the_project_then_the_forge_then_the_account() {
    let (mut d, sign_in) = driver(true);
    start(&mut d, sign_in);
    application(&mut d, sign_in, 82, forge::Ask::EndPlayground, false);
    d.settle();
    assert_eq!(row(&d).expect("playground fixture").state, forge::PlaygroundState::Resetting(forge::ResetStep::Forge));
    assert_eq!(account_rows(&d).len(), 1);
    finish_forge_removal(&mut d);
    assert!(row(&d).is_none());
    assert!(account_rows(&d).is_empty());
    assert!(d.delivered.iter().any(|delivery| matches!(
        delivery,
        Delivery::Application { outcome: Some(engine::ApplicationAnswer::EndPlayground(forge::EndAnswer::Reset)), .. }
    )));
}
#[test]
fn an_ungranted_remove_project_is_kept_with_its_diagnostic() {
    let (mut d, sign_in) = driver(false);
    start(&mut d, sign_in);
    application(&mut d, sign_in, 82, forge::Ask::EndPlayground, false);
    d.settle();
    assert_eq!(
        row(&d).expect("playground fixture").state,
        forge::PlaygroundState::Resetting(forge::ResetStep::Project)
    );
    assert!(d.delivered.iter().any(|delivery| matches!(delivery, Delivery::DemoNote { line }
        if line.windows(b"remove_project".len()).any(|part| part == b"remove_project"))));
}
#[test]
fn a_start_cut_by_a_restart_resets_the_playground() {
    for completed in 0..4 {
        let (mut d, sign_in) = driver(true);
        application(&mut d, sign_in, 81, forge::Ask::StartPlayground, true);
        d.settle();
        d.send(engine::Event::Forge(Box::new(forge::Event::Connection {
            forge: 1,
            state: forge::Connection::Connected { id: 1, login: b"temper".as_slice().into() },
        })));
        for _ in 0..200 {
            let ready = match (completed, row(&d)) {
                (0, _) => true,
                (1, Some(row)) => row.account.is_some(),
                (2, Some(row)) => row.project.is_some(),
                (3, Some(row)) => {
                    row.project.is_some()
                        && d.transactions.iter().flatten().any(|write| {
                            matches!(
                                write,
                                engine::Write::Save(engine::Record::Core(core::Record::Accounts(
                                    accounts::Stored::Account { uses: Some(_), .. }
                                )))
                            )
                        })
                }
                _ => false,
            };
            if ready {
                break;
            }
            d.advance(true);
        }
        while !d.store.pending.is_empty() {
            d.store.apply();
        }
        restart(&mut d, true);
        finish_forge_removal(&mut d);
        assert!(row(&d).is_none(), "cut {completed}");
    }
}
#[test]
fn a_reset_cut_after_each_act_ends_whole_at_the_next_start() {
    for step in [forge::ResetStep::Project, forge::ResetStep::Forge, forge::ResetStep::Account] {
        let (mut d, sign_in) = driver(true);
        start(&mut d, sign_in);
        application(&mut d, sign_in, 82, forge::Ask::EndPlayground, false);
        match step {
            forge::ResetStep::Project => {}
            forge::ResetStep::Forge => {
                d.settle();
                assert_eq!(
                    row(&d).expect("reset row").state,
                    forge::PlaygroundState::Resetting(forge::ResetStep::Forge)
                );
                d.send(engine::Event::Forge(Box::new(forge::Event::SecretsErased { forge: 1 })));
                d.settle();
            }
            forge::ResetStep::Account => {
                d.settle();
                assert_eq!(
                    row(&d).expect("reset row").state,
                    forge::PlaygroundState::Resetting(forge::ResetStep::Forge)
                );
                d.send(engine::Event::Forge(Box::new(forge::Event::SecretsErased { forge: 1 })));
                d.settle();
                d.send(engine::Event::Forge(Box::new(forge::Event::BoxRemoved { forge: 1 })));
            }
        }
        while !d.store.pending.is_empty() {
            d.store.apply();
        }
        restart(&mut d, true);
        finish_forge_removal(&mut d);
        assert!(row(&d).is_none());
        assert!(account_rows(&d).is_empty());
    }
}
