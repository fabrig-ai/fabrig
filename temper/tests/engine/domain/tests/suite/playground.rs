use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_people as people;
use skein_lib::{Duration, Env, Queue, ReplyTo, Token};
use temper_engine_domain::{self as engine, Delivery};
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::{direct::Driver, fixtures, forge_stand::ForgeStand, store::Store, walking};

const SEEDED_PROJECT: u32 = 1;
const PLAYGROUND_PROJECT: u32 = 2;
// The demo fixture's journal bound, plus the playground's project and uses writes.
const START_WRITES: u32 = 2_160;

fn fake() -> forge::Forge {
    use forge::record as r;
    forge::Forge {
        number: 0,
        kind: forge::Kind::Forgejo,
        api: r::Address::Host { name: b"forge.example".as_slice().into(), port: 80 },
        transport: r::Transport::Plaintext,
        git: Box::new([]),
        git_program: None,
        web: None,
        read_only: false,
        identity: r::Identity { number: 0, login: Box::new([]), name: Box::new([]), email: Box::new([]) },
        files: None,
        setup: Some(r::Setup {
            directory: b"playground".as_slice().into(),
            bot: r::Account { login: b"temper".as_slice().into(), email: Box::new([]) },
            orgs: Box::new([]),
            asked: true,
            mode: r::SetupMode::Fake(r::FakeEntry {
                listen: ([127, 0, 0, 1], 3001).into(),
                git: b"fake".as_slice().into(),
                owner: b"playground".as_slice().into(),
                repositories: Box::new([]),
            }),
        }),
        first: None,
        playground: true,
        removing: false,
    }
}
pub(super) fn config(real: bool) -> engine::Config {
    let mut config = walking::config(71);
    config.core.defaults.templates[0].budget = people::TemplateBudget::AsAllowed { cap: Some(200) };
    config.core.defaults.rules.ceiling.delegation.depth = 2;
    config.core.seed_projects[0].policy.policy.ceiling.delegation.depth = 2;
    config.core.seed_projects[0].policy.policy.roles[0].authority.delegation.depth = 2;
    let seed = &config.core.seed_projects[0].policy;
    config
        .core
        .defaults
        .policies
        .insert(
            1,
            core::ConfiguredPolicy {
                policy: seed.policy.clone(),
                permissions: Box::new([]),
                templates: seed.templates.clone(),
            },
        )
        .expect("stock policy");
    config.core.grants = Box::new([people::ActGrant {
        connector: config.core.connectors[0],
        may: Box::new([people::ActKind::Cancel, people::ActKind::RemoveProject, people::ActKind::RemoveAccount]),
    }]);
    config.identities = Box::new([]);
    config.forge.writers = Box::new([]);
    config.forges = Box::new([]);
    config.seed = if real {
        Box::new([forge::Forge { number: 2, playground: false, setup: None, ..fake() }])
    } else {
        Box::new([])
    };
    let mut adoption = fixtures::adoption(0);
    adoption.owner = b"playground".as_slice().into();
    adoption.name = b"widget".as_slice().into();
    config.demo =
        Some(forge::DemoConfig { forge: Some(fake()), provider: 1, adoption, deadline: Duration::from_secs(10) });
    config
}
fn driver(real: bool) -> (Driver, u64) {
    let mut limits = fixtures::forge_limits();
    limits.tasks.depth = 2;
    limits.accounts.accounts = 2;
    limits.fleet.grants = limits.accounts.credentials + limits.accounts.accounts;
    limits.people.act_grants = 1;
    limits.people.pending = 4;
    limits.people.requests = 8;
    limits.authority.projects = 2;
    limits.people.projects = 3;
    limits.journal.writes = START_WRITES;
    limits.journal.deliveries += 4;
    limits.journal.held += 12;
    let mut d = Driver::cold(Store::new(), config(real), &limits);
    d.forge = Some(ForgeStand::with(&[(b"playground", b"widget", 2)]));
    d.settle();
    d.sign_in();
    d.settle();
    let sign_in = d.session();
    d.delivered.clear();
    (d, sign_in)
}
fn start(d: &mut Driver, sign_in: u64, number: u8) {
    start_with_key(d, sign_in, number, true);
}
fn start_with_key(d: &mut Driver, sign_in: u64, number: u8, carry: bool) {
    d.send(engine::Event::Party(engine::PartyInput::Application {
        reply_to: ReplyTo::new(Token::new(u64::from(number))),
        sign_in,
        key: [number; 16],
        digest: [9; 32],
        scope: people::ApplicationScope::Deployment,
        request: engine::Application::Forge(forge::Ask::StartPlayground),
        carried: carry.then_some(core::Carried { number: u64::from(number), digest: [7; 32] }),
    }));
    d.settle();
}
fn connect(d: &mut Driver) {
    let number = d.root.playground().expect("playground start saved").forge.expect("made forge");
    d.send(engine::Event::Forge(Box::new(forge::Event::Connection {
        forge: number,
        state: forge::Connection::Connected { id: 1, login: b"temper".as_slice().into() },
    })));
    d.settle();
}
fn running() -> (Driver, u64) {
    let (mut d, sign_in) = driver(false);
    start(&mut d, sign_in, 81);
    connect(&mut d);
    (d, sign_in)
}
fn answers(d: &Driver) -> Vec<Option<engine::ApplicationAnswer>> {
    d.delivered
        .iter()
        .filter_map(
            |delivery| {
                if let Delivery::Application { outcome, .. } = delivery { Some(outcome.clone()) } else { None }
            },
        )
        .collect()
}
fn account(d: &Driver) -> (u32, Option<Box<[accounts::Use]>>) {
    d.store
        .rows
        .values()
        .find_map(|record| {
            if let engine::Record::Core(core::Record::Accounts(accounts::Stored::Account {
                number,
                holder: accounts::Holder::Deployment,
                provider: 1,
                uses,
                ..
            })) = record
            {
                Some((*number, uses.clone()))
            } else {
                None
            }
        })
        .expect("scripted deployment account")
}
fn keeps(d: &Driver) -> usize {
    d.protocol
        .accounts
        .iter()
        .filter(|request| matches!(request, accounts::Request::Signing(accounts::Signing::Keep { .. })))
        .count()
}
#[test]
fn start_playground_makes_its_project_account_and_adoption() {
    let (d, _) = running();
    assert_eq!(answers(&d), [Some(engine::ApplicationAnswer::StartPlayground(forge::StartAnswer::Running))]);
    let (account, uses) = account(&d);
    assert_eq!(uses.as_deref(), Some([accounts::Use::Project(PLAYGROUND_PROJECT)].as_slice()));
    assert_eq!(
        d.root.playground(),
        Some(&forge::Playground {
            state: forge::PlaygroundState::Running,
            forge: Some(1),
            project: Some(PLAYGROUND_PROJECT),
            account: Some(account)
        })
    );
    assert!(d.store.rows.values().any(|record| matches!(record,
        engine::Record::Core(core::Record::People(people::Stored::Policy { project: PLAYGROUND_PROJECT, name, .. })) if name.as_ref() == "Playground")));
    assert!(d.store.rows.values().any(|record| matches!(record,
        engine::Record::Forge { row } if matches!(row.as_ref(), forge::Stored::Repository(row)
        if row.project == PLAYGROUND_PROJECT && row.provider.forge == 1 && row.owner.as_ref() == b"playground" && row.name.as_ref() == b"widget"))));
}
#[test]
fn the_scripted_account_serves_no_other_project() {
    let (mut d, sign_in) = running();
    let ask = d.chat_ask(SEEDED_PROJECT, b"hello");
    d.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(90)),
        sign_in,
        key: [90; 16],
        ask,
        models: Box::new([]),
    }));
    d.pump();
    let tasks = d.root.tasks().view_tasks();
    let task = tasks.first().expect("other project's task");
    let payer = d.root.tasks().payer(task.number).expect("task payer");
    let limits = engine::core_limits(&d.env.limits);
    let mut core = core::Core::new(config(false).core, &limits).expect("configured core");
    let mut people_out = Queue::uncapped();
    let mut accounts_out = Queue::uncapped();
    for record in d.store.rows.values() {
        if let engine::Record::Core(core::Record::People(record)) = record {
            people::step(
                &mut core.people,
                &Env { now: d.env.now, wall: d.env.wall, limits: limits.people },
                people::Event::Restore { record: record.clone() },
                &mut people_out,
            );
        } else if let engine::Record::Core(core::Record::Accounts(record)) = record {
            accounts::step(
                &mut core.accounts,
                &Env { now: d.env.now, wall: d.env.wall, limits: limits.accounts },
                accounts::Event::Restore { record: record.clone() },
                &mut accounts_out,
            );
        }
    }
    let proofs = core.account_coverage(payer);
    assert!(!proofs.is_empty(), "the playground account participates in coverage");
    assert!(proofs.iter().all(|proof| !proof.serves));
}
#[test]
fn a_second_start_makes_nothing_again() {
    let (mut d, sign_in) = running();
    let row = *d.root.playground().expect("running playground");
    let calls = d.forge.as_ref().expect("forge").calls;
    let rows = d.store.rows.clone();
    d.delivered.clear();
    start(&mut d, sign_in, 82);
    assert_eq!(answers(&d), [Some(engine::ApplicationAnswer::StartPlayground(forge::StartAnswer::Running))]);
    assert_eq!(d.root.playground(), Some(&row));
    assert_eq!(keeps(&d), 1);
    assert_eq!(d.forge.as_ref().expect("forge").calls, calls);
    for (key, record) in rows {
        if matches!(record, engine::Record::Forge { .. }) {
            assert_eq!(d.store.rows.get(&key), Some(&record));
        }
    }
    assert!(d.delivered.iter().any(|delivery| matches!(delivery, Delivery::CarriedUnused { number: 82 })));
}
#[test]
fn a_refused_project_fails_the_start_naming_it() {
    let (mut d, sign_in) = driver(false);
    d.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(90)),
        sign_in,
        key: [90; 16],
        models: Box::new([]),
        ask: people::Ask::MakeProject { name: "Playground".into(), policy: 1 },
    }));
    d.settle();
    start(&mut d, sign_in, 81);
    assert_eq!(
        answers(&d),
        [Some(engine::ApplicationAnswer::StartPlayground(forge::StartAnswer::Failed(forge::StartFailure::Project)))]
    );
    assert_eq!(keeps(&d), 0);
}
#[test]
fn a_kept_real_forge_refuses_the_start() {
    let (mut d, sign_in) = driver(true);
    start(&mut d, sign_in, 81);
    assert_eq!(answers(&d), [Some(engine::ApplicationAnswer::StartPlayground(forge::StartAnswer::NotOffered))]);
    assert_eq!(d.root.playground(), None);
}

#[test]
fn an_interrupted_account_step_resumes_the_saved_project() {
    let (mut d, sign_in) = driver(false);
    start_with_key(&mut d, sign_in, 81, false);
    assert_eq!(
        answers(&d),
        [Some(engine::ApplicationAnswer::StartPlayground(forge::StartAnswer::Failed(forge::StartFailure::Account)))]
    );
    let project = d.root.playground().expect("saved progress").project;
    d.delivered.clear();
    start(&mut d, sign_in, 82);
    connect(&mut d);
    assert_eq!(answers(&d), [Some(engine::ApplicationAnswer::StartPlayground(forge::StartAnswer::Running))]);
    assert_eq!(d.root.playground().expect("running playground").project, project);
    assert_eq!(keeps(&d), 1);
}

#[test]
fn an_existing_deployment_account_is_reused_with_project_uses() {
    let (mut d, sign_in) = driver(false);
    d.send(engine::Event::Party(engine::PartyInput::Key {
        reply_to: ReplyTo::new(Token::new(90)),
        sign_in,
        key: [90; 16],
        ask: Box::new(people::Ask::AccountChange {
            holder: people::AccountHolder::Deployment,
            change: people::AccountChange::AddKey { provider: 1, join: None, digest: [8; 32] },
        }),
        carried: core::Carried { number: 90, digest: [8; 32] },
    }));
    d.settle();
    start(&mut d, sign_in, 81);
    connect(&mut d);
    assert_eq!(answers(&d), [Some(engine::ApplicationAnswer::StartPlayground(forge::StartAnswer::Running))]);
    assert_eq!(keeps(&d), 1);
    assert_eq!(account(&d).1.as_deref(), Some([accounts::Use::Project(PLAYGROUND_PROJECT)].as_slice()));
    assert!(d.delivered.iter().any(|delivery| matches!(delivery, Delivery::CarriedUnused { number: 81 })));
}

#[test]
fn simultaneous_starts_share_the_project_account_and_adoption() {
    let (mut d, sign_in) = driver(false);
    start(&mut d, sign_in, 81);
    start(&mut d, sign_in, 82);
    connect(&mut d);
    assert_eq!(
        answers(&d),
        [
            Some(engine::ApplicationAnswer::StartPlayground(forge::StartAnswer::Running)),
            Some(engine::ApplicationAnswer::StartPlayground(forge::StartAnswer::Running)),
        ]
    );
    assert_eq!(keeps(&d), 1);
    assert!(d.delivered.iter().any(|delivery| matches!(delivery, Delivery::CarriedUnused { number: 82 })));
}

#[test]
fn a_start_waits_for_connection_until_its_deadline() {
    let (mut d, sign_in) = driver(false);
    start(&mut d, sign_in, 81);
    assert!(answers(&d).is_empty());
    d.fire_at(Duration::from_secs(10));
    d.settle();
    assert_eq!(
        answers(&d),
        [Some(engine::ApplicationAnswer::StartPlayground(forge::StartAnswer::Failed(
            forge::StartFailure::NotConnected
        )))]
    );
}

#[test]
fn a_failed_connection_names_the_groups_reason() {
    let (mut d, sign_in) = driver(false);
    start(&mut d, sign_in, 81);
    d.send(engine::Event::Forge(Box::new(forge::Event::Connection {
        forge: 1,
        state: forge::Connection::SetUpFailed { why: b"group failed".as_slice().into() },
    })));
    d.settle();
    assert_eq!(
        answers(&d),
        [Some(engine::ApplicationAnswer::StartPlayground(forge::StartAnswer::Failed(forge::StartFailure::Group {
            why: b"group failed".as_slice().into()
        })))]
    );
}

#[test]
fn a_refused_adoption_fails_the_start_naming_it() {
    let (mut d, sign_in) = driver(false);
    d.forge = Some(ForgeStand::with(&[(b"playground", b"another", 2)]));
    start(&mut d, sign_in, 81);
    connect(&mut d);
    assert_eq!(
        answers(&d),
        [Some(engine::ApplicationAnswer::StartPlayground(forge::StartAnswer::Failed(forge::StartFailure::Adoption)))]
    );
}

#[test]
fn a_refused_adoption_admission_fails_the_start_naming_it() {
    let (mut d, sign_in) = driver(false);
    start(&mut d, sign_in, 81);
    d.send(engine::Event::Party(engine::PartyInput::SignOut { reply_to: ReplyTo::new(Token::new(90)), sign_in }));
    d.settle();
    connect(&mut d);
    assert_eq!(
        answers(&d),
        [Some(engine::ApplicationAnswer::StartPlayground(forge::StartAnswer::Failed(forge::StartFailure::Adoption)))]
    );
}

#[test]
fn another_forge_connecting_ends_nothing() {
    let (mut d, _) = running();
    let row = *d.root.playground().expect("running playground");
    d.send(engine::Event::Forge(Box::new(forge::Event::Restore {
        record: forge::Stored::Forge(forge::Forge {
            number: 2,
            identity: forge::record::Identity { number: 2, ..fake().identity },
            playground: false,
            setup: None,
            ..fake()
        }),
    })));
    d.settle();
    d.send(engine::Event::Forge(Box::new(forge::Event::Connection {
        forge: 2,
        state: forge::Connection::Connected { id: 2, login: b"real".as_slice().into() },
    })));
    d.settle();
    assert_eq!(d.root.forge_connection(2), forge::Connection::Connected { id: 2, login: b"real".as_slice().into() });
    assert_eq!(row.state, forge::PlaygroundState::Running);
    assert_eq!(d.root.playground(), Some(&row));
    assert!(!d.store.rows.values().any(|record| matches!(record,
        engine::Record::Forge { row } if matches!(row.as_ref(), forge::Stored::Kept(_)))));
}
