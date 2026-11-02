//! Reset crash cuts (temper/docs/design/protocol/setup.md, 8.4; temper/docs/design/domain/forge.md, 18).
use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_people as people;
use skein_lib::{Duration, Map, ReplyTo, Token};
use temper_engine_domain as engine;
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
    config.machine =
        Some(forge::Machine { owned: None, fake: Some(place()), github: place(), github_git_program: None });
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
    assert!(!bootstrap.stopped, "bootstrap: {:?}", bootstrap.root.start_stop());
    let store = bootstrap.take_store();
    let mut d = Driver::cold(store, config(granted), &limits);
    d.forge = Some(ForgeStand::with(&[(b"playground", b"widget", 2)]));
    d.pump();
    assert!(!d.stopped, "restore: {:?}", d.root.start_stop());
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
fn restart(d: &mut Driver, granted: bool, seed: u64) {
    d.store.crash(seed);
    d.root = engine::Domain::new(config(granted), &d.env.limits).expect("playground fixture");
    d.events.clear();
    d.out.clear();
    d.immediate.clear();
    d.delivered.clear();
    d.send(engine::Event::Restart { pending: false });
    d.settle();
}
fn finish_forge_removal(d: &mut Driver) {
    d.send(engine::Event::Forge(Box::new(forge::Event::SecretsErased { forge: 1 })));
    d.send(engine::Event::Forge(Box::new(forge::Event::BoxRemoved { forge: 1 })));
    d.settle();
}

fn assert_whole(d: &Driver, project: u32, account: u32, forge: u16) {
    assert!(row(d).is_none(), "reset row: {:?}; deliveries: {:?}; immediate: {:?}", row(d), d.delivered, d.immediate);
    assert!(account_rows(d).is_empty(), "scripted account {account} erased");
    assert!(d.root.forges().rows.is_empty());
    assert!(!d.store.rows.values().any(|record| matches!(record, engine::Record::Forge { row }
        if matches!(row.as_ref(), forge::Stored::Forge(row) if row.number == forge)
            || matches!(row.as_ref(), forge::Stored::Repository(row) if row.project == project))));
    assert!(!d.store.rows.contains_key(&engine::Key::Core(core::Key::People(people::Key::Policy(project)))));
}

fn run(step: forge::ResetStep, seed: u64) -> std::collections::BTreeMap<engine::Key, engine::Record> {
    let (mut d, sign_in) = driver(true);
    start(&mut d, sign_in);
    let running = row(&d).expect("running playground");
    let project = running.project.expect("playground project");
    let account = running.account.expect("scripted account");
    application(&mut d, sign_in, 82, forge::Ask::EndPlayground, false);
    // Stop on the first durable row at the chosen step; its terminal is lost.
    let mut forge_answered = false;
    let mut cut = false;
    for _ in 0..400 {
        if let Some((_, writes)) = d.store.pending.front() {
            let cutting = writes.iter().any(|write| match step {
                forge::ResetStep::Project => matches!(write, engine::Write::Save(engine::Record::Forge { row })
                    if matches!(row.as_ref(), forge::Stored::Playground(row)
                        if row.state == forge::PlaygroundState::Resetting(forge::ResetStep::Forge))),
                forge::ResetStep::Forge => matches!(write,
                    engine::Write::Erase(engine::Key::Forge(forge::Key::Forge(number))) if *number == running.forge),
                forge::ResetStep::Account => {
                    matches!(write, engine::Write::Erase(engine::Key::Forge(forge::Key::Playground)))
                }
            });
            let number = d.store.apply();
            if cutting {
                cut = true;
                break;
            }
            d.send(engine::Event::Store(engine::StoreInput::Committed { number, fill: 0 }));
        } else if step != forge::ResetStep::Project
            && !forge_answered
            && row(&d).is_some_and(|row| row.state == forge::PlaygroundState::Resetting(forge::ResetStep::Forge))
        {
            d.send(engine::Event::Forge(Box::new(forge::Event::SecretsErased { forge: running.forge })));
            d.send(engine::Event::Forge(Box::new(forge::Event::BoxRemoved { forge: running.forge })));
            forge_answered = true;
        } else {
            d.advance(false);
        }
    }
    assert!(cut, "durable cut after {step:?}");
    // Removing the account can finish in the decision that finishes the forge.
    let expected = match step {
        forge::ResetStep::Project => Some(forge::PlaygroundState::Resetting(forge::ResetStep::Forge)),
        forge::ResetStep::Forge | forge::ResetStep::Account => None,
    };
    assert_eq!(row(&d).map(|row| row.state), expected, "cut after {step:?}");
    restart(&mut d, true, seed);
    finish_forge_removal(&mut d);
    assert_whole(&d, project, account, running.forge);
    d.store.rows.clone()
}
fn replay(step: forge::ResetStep) {
    for seed in [1, 7, 71] {
        assert_eq!(run(step, seed), run(step, seed), "step {step:?}, seed {seed}");
    }
}

#[test]
fn reset_cut_after_project_ends_whole() {
    replay(forge::ResetStep::Project);
}
#[test]
fn reset_cut_after_forge_ends_whole() {
    replay(forge::ResetStep::Forge);
}
#[test]
fn reset_cut_after_account_ends_whole() {
    replay(forge::ResetStep::Account);
}
