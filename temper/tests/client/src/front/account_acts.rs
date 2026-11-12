//! Accounts joins use the root-only pump of front/tests.rs; the page renderer belongs to part c.
use super::{Joint, engine_limits};
use crate::{
    app::{Event, Root, Temper},
    fixtures,
};
use jig_client_kit::harness::{App, ClientId, Duty, Front, FrontFault};
use jig_domain_accounts as a;
use jig_domain_link::{Form, Scheme, SourceName};
use skein_lib::{Duration, Env, Queue, Wall};
use temper_engine_domain as engine;
use temper_engine_domain_world::{store::Store, walking};
const CLIENT: ClientId = ClientId(0);
pub const PROVIDER: u32 = 17;
pub const SMALL_ROWS: u32 = 2;
#[must_use]
pub fn fixture(seed: u64) -> (Joint, Root) {
    fixture_root(seed, Tools::Off, 2, None, false, WelcomeForge::None, false)
}
#[must_use]
pub fn plan_fixture(seed: u64) -> (Joint, Root) {
    fixture_root(seed, Tools::Off, 2, None, true, WelcomeForge::None, false)
}
/// A signed-in `ChatGPT` plan, through the client and engine with the issuer faked.
#[must_use]
pub fn signed_in_plan(seed: u64) -> (Joint, Root, u32) {
    use temper_client_domain::Address;
    let (mut front, mut root) = plan_fixture(seed);
    event(&mut front, &mut root, Temper::went(Address::ConnectModel { way: None }));
    let (_, mut view) = Temper::new(seed);
    Temper::build(&mut view, &root, &mut Queue::uncapped());
    let node = Temper::tree(&view)
        .nodes()
        .iter()
        .find(|node| node.role == skein_ui::Role::Button && node.name.as_deref() == Some(b"Continue with ChatGPT"))
        .expect("plan sign-in control")
        .id;
    let press = Temper::act(&mut view, node, skein_ui::Act::Press).expect("plan sign-in act");
    event(&mut front, &mut root, press);
    let source = signing(&front);
    front.engine.send(temper_engine_domain::Event::Account(a::Event::Visited {
        source,
        address: b"https://issuer/visit".as_slice().into(),
    }));
    settle(&mut front, &mut root);
    front.engine.send(temper_engine_domain::Event::Account(a::Event::SignedIn {
        source,
        subject: b"subject".as_slice().into(),
        display: b"alice@example.com".as_slice().into(),
        declined: false,
        said: a::Said::default(),
    }));
    settle(&mut front, &mut root);
    answer_trial(&mut front, &mut root);
    event(&mut front, &mut root, Temper::went(Address::Accounts));
    (front, root, source)
}
#[must_use]
pub fn tools_fixture(seed: u64) -> (Joint, Root) {
    fixture_root(seed, Tools::On, 2, None, false, WelcomeForge::None, false)
}
/// An owner's small initial pool and room to double the project's spend, configured before startup.
#[must_use]
pub fn tools_fixture_with_spend(seed: u64, spend: u64) -> (Joint, Root) {
    fixture_root(seed, Tools::Spend(spend), 2, None, false, WelcomeForge::None, false)
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Tools {
    Off,
    On,
    Spend(u64),
    Trial,
    TrialSelection,
    TwoSignIns,
}
#[must_use]
pub fn small_rows_fixture(seed: u64) -> (Joint, Root) {
    fixture_root(seed, Tools::Off, 3, Some(SMALL_ROWS), false, WelcomeForge::None, false)
}
#[must_use]
pub fn welcome_fixture(seed: u64) -> (Joint, Root) {
    fixture_root(seed, Tools::Off, 2, None, false, WelcomeForge::Connected, false)
}
/// A deployment with no kept forge; model usability is supplied by the engine fixture.
#[must_use]
pub fn fresh_fixture(seed: u64) -> (Joint, Root) {
    let (mut front, root) = fixture_root(seed, Tools::Off, 2, None, false, WelcomeForge::Empty, false);
    front.engine.forge = Some(temper_engine_domain_world::forge_stand::ForgeStand::with(&[(b"work", b"hello", 42)]));
    (front, root)
}
/// A box's welcome with only the deployment owner and no project's role.
#[must_use]
pub fn tui_first_run_fixture(seed: u64) -> (Joint, Root) {
    let (mut front, mut root) = fixture_root(seed, Tools::Off, 2, None, false, WelcomeForge::FirstRun, false);
    event(&mut front, &mut root, Temper::went(temper_client_domain::Address::Accounts));
    without_deployment_account((front, root))
}
/// An empty first run offers a real provider but holds no account yet.
#[must_use]
pub fn empty_welcome_fixture(seed: u64) -> (Joint, Root) {
    let (mut front, mut root) = fresh_fixture(seed);
    event(&mut front, &mut root, Temper::went(temper_client_domain::Address::Accounts));
    let (mut front, mut root) = without_deployment_account((front, root));
    event(
        &mut front,
        &mut root,
        Temper::went(temper_client_domain::Address::Welcome { step: temper_client_domain::Step::Welcome }),
    );
    (front, root)
}
/// The first-run forge is owned by the box; the test scripts its connection rows.
#[must_use]
pub fn forge_fixture(seed: u64) -> (Joint, Root) {
    let (mut front, root) = fixture_root(seed, Tools::Off, 2, None, false, WelcomeForge::Owned, false);
    // The scripted box's Create makes the first repository available to Adopt.
    front.engine.forge = Some(temper_engine_domain_world::forge_stand::ForgeStand::with(&[
        (b"acme", b"shop", 7),
        (b"acme", b"second", 8),
        (b"work", b"hello", 42),
    ]));
    (front, root)
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum WelcomeForge {
    None,
    Empty,
    FirstRun,
    Connected,
    Owned,
    Playground { running: bool, printed: bool },
}
#[must_use]
pub fn program_fixture(seed: u64) -> (Joint, Root) {
    fixture_root(seed, Tools::Off, 2, None, false, WelcomeForge::None, true)
}
/// A program offered without a pre-existing usable deployment account.
#[must_use]
pub fn empty_program_fixture(seed: u64) -> (Joint, Root) {
    without_deployment_account(program_fixture(seed))
}
/// Claude's welcome trial, with a connected forge and room for Create's project.
#[must_use]
pub fn trial_fixture(seed: u64) -> (Joint, Root) {
    trial_root(seed, Tools::Trial)
}
/// Two program models with distinct providers, in the chat's selection order.
#[must_use]
pub fn trial_selection_fixture(seed: u64) -> (Joint, Root) {
    trial_root(seed, Tools::TrialSelection)
}
fn trial_root(seed: u64, tools: Tools) -> (Joint, Root) {
    let (mut front, mut root) = fixture_root(seed, tools, 2, None, false, WelcomeForge::Connected, true);
    event(&mut front, &mut root, Temper::went(temper_client_domain::Address::Accounts));
    let (mut front, mut root) = without_deployment_account((front, root));
    front.deployment = (true, true);
    front.engine.send(engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::Connection {
        forge: 1,
        state: temper_engine_domain_forge::Connection::Connected { id: 1, login: b"temper".as_slice().into() },
    })));
    settle(&mut front, &mut root);
    (front, root)
}

/// Cold restart on the account world's configuration and committed store.
pub fn restart_trial(front: &mut Joint, root: &mut Root) {
    let logins: Box<[_]> = temper_client_domain::pages::accounts::document(root.person().client())
        .expect("account fixture document")
        .accounts
        .iter()
        .flat_map(|account| account.sources.iter())
        .filter(|source| source.kind == jig_client::contract::AccountKind::Program)
        .map(|source| jig_domain_link::Login {
            name: jig_domain_link::SourceName::new(source.name.as_deref().expect("program name")).expect("source name"),
            state: jig_domain_link::LoginState::Usable,
            email: None,
        })
        .collect();
    let config = front.trial_config.take().expect("trial fixture configuration");
    front.trial_config = Some(temper_engine_domain_world::oracle::config::copy(&config));
    // The standalone worker dies with its engine.
    front.hosting.clear();
    let limits = front.engine.env.limits;
    let mut hello = Joint::worker_hello(&config, &limits, limits.fleet.slots, false);
    let channel = skein_lib::Token::new(8);
    if let engine::Event::Worker(engine::WorkerInput::Link { channel: at, .. }) = &mut hello {
        *at = channel;
    }
    let store = front.engine.take_store();
    front.engine = temper_engine_domain_world::direct::Driver::with_repositories(
        store,
        [temper_engine_domain_world::oracle::config::copy(&config), config],
        &limits,
        fixtures::REPOSITORIES,
    );
    front.engine.pump();
    front.engine.send(hello);
    // The direct world's default worker has no logins. This story's restarted
    // worker answers each source inventory with the private logins it still holds.
    for _ in 0..200 {
        for event in &mut front.engine.events {
            if let engine::Event::Worker(engine::WorkerInput::Link {
                link: engine::Link::Up { up: jig_domain_host::LinkUp::Logins { logins: observed }, .. },
                ..
            }) = event
            {
                observed.clone_from(&logins);
            }
        }
        front.engine.advance(true);
    }
    front.engine.send(engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::Connection {
        forge: 1,
        state: temper_engine_domain_forge::Connection::Connected { id: 1, login: b"temper".as_slice().into() },
    })));
    front.settle();
    front.reads = 0;
    front.read_loads.clear();
    front.drop_watches();
    settle(front, root);
    elapse(front, root, Duration::from_millis(100));
}

/// Advance the world's clock and deliver the client's due continuations.
pub fn elapse(front: &mut Joint, root: &mut Root, span: Duration) {
    front.advance(front.now.saturating_add(span));
    let env = Env { now: front.now, wall: front.engine.env.wall, limits: front.limits };
    let mut out = Queue::uncapped();
    Temper::fire(root, &env, &mut out);
    flush(front, root, &env, &mut out, &mut Vec::new());
    settle(front, root);
}

/// Supply the configured worker's observation of the explicit check it received.
pub fn program_checked(front: &mut Joint, root: &mut Root, usable: bool) {
    use jig_domain_host::LinkUp;
    use jig_domain_link::{Login, LoginState};
    let pending =
        root.person().pages().connect_model().expect("model page").request.pending.as_ref().expect("check request");
    let jig_client::contract::Ask::AccountChange {
        change: jig_client::contract::AccountChange::CheckProgram { source },
        ..
    } = pending.ask
    else {
        panic!("explicit program check");
    };
    let doc = temper_client_domain::pages::accounts::document(root.person().client()).expect("accounts observed");
    let name = doc
        .accounts
        .iter()
        .flat_map(|a| a.sources.iter())
        .find(|s| s.number == source)
        .and_then(|s| s.name.as_deref())
        .expect("declared login name");
    let name = SourceName::new(name).expect("valid declaration");
    let channel = fixtures::QUESTION_WORKER;
    front.engine.send(temper_engine_domain_world::protocol::up(
        channel,
        LinkUp::Logins {
            logins: Box::new([Login {
                name,
                state: if usable { LoginState::Usable } else { LoginState::NotSignedIn },
                email: None,
            }]),
        },
        engine::Parts::none(),
    ));
    front.engine.settle();
    settle(front, root);
}

/// Answer a program sign-out through the configured worker's link.
pub fn program_signed_out(front: &mut Joint, root: &mut Root, name: &[u8]) {
    front.engine.send(temper_engine_domain_world::protocol::up(
        fixtures::QUESTION_WORKER,
        jig_domain_host::LinkUp::SignedOut {
            name: SourceName::new(name).expect("declared login name"),
            result: jig_domain_link::SignOutResult::SignedOut,
        },
        engine::Parts::none(),
    ));
    front.engine.settle();
    settle(front, root);
}

/// Settle the welcome's trial before a story scripts its own worker attempt
/// (temper/docs/design/domain/trial.md, 2; temper/docs/design/testing/checks.md, 3).
pub fn answer_trial(front: &mut Joint, root: &mut Root) {
    settle(front, root);
    let tasks: Vec<_> = root
        .person()
        .forge()
        .trials()
        .iter()
        .filter(|row| row.state == temper_client_domain::forge::trials::State::Running)
        .filter_map(|row| row.task)
        .collect();
    for task in tasks {
        let attempt = front.hosted_attempt(task).expect("trial reached its scripted worker");
        let read = front.engine.task(task).expect("trial task").inbox.last().map(|message| message.number);
        front.finish_reply(task, attempt, read, Some(b"Yes"));
        settle(front, root);
    }
}
#[must_use]
pub fn empty_fixture(seed: u64) -> (Joint, Root) {
    without_deployment_account(fixture(seed))
}
/// An empty first run offering a `ChatGPT` sign-in, with no kept forge or usable model.
#[must_use]
pub fn empty_plan_fixture(seed: u64) -> (Joint, Root) {
    let (mut front, mut root) = fixture_root(seed, Tools::Off, 2, None, true, WelcomeForge::Empty, false);
    event(&mut front, &mut root, Temper::went(temper_client_domain::Address::Accounts));
    without_deployment_account((front, root))
}
/// `empty_plan_fixture`, with a second provider offering a sign-in: `playground_providers`' `other`.
#[must_use]
pub fn two_sign_ins_fixture(seed: u64) -> (Joint, Root) {
    let (mut front, mut root) = fixture_root(seed, Tools::TwoSignIns, 2, None, true, WelcomeForge::Empty, false);
    event(&mut front, &mut root, Temper::went(temper_client_domain::Address::Accounts));
    without_deployment_account((front, root))
}
fn without_deployment_account((mut front, mut root): (Joint, Root)) -> (Joint, Root) {
    use temper_client_domain::pages::{
        Act as PageAct,
        accounts::{Act, Operation, Row},
    };
    let domain = root.person();
    let account = domain
        .pages()
        .accounts()
        .expect("account fixture")
        .deployment(domain.client())
        .expect("owner deployment")
        .accounts
        .get(0)
        .expect("seeded deployment account")
        .number;
    for act in [
        Act::Submit {
            name: [67; 16],
            row: Row { holder: jig_client::contract::AskAccountHolder::Deployment, account },
            operation: Operation::RemoveAccount,
        },
        Act::Confirm,
    ] {
        event(
            &mut front,
            &mut root,
            temper_client_domain::Event::Act { action: temper_client_domain::Action::Pages(PageAct::Accounts(act)) }
                .into(),
        );
    }
    (front, root)
}
fn playground_providers(config: &mut engine::Config) {
    // Both providers use this world's restored fake key sources, so Remove
    // goes through the real account policy. Only their usability is scripted.
    let playground = config.core.settings.llm_providers.providers[0].clone();
    let mut other = playground.clone();
    other.number = PROVIDER + 1;
    other.name = Box::from(b"Other fake".as_slice());
    other.offers = a::Offers { sign_in: true, key: true, programs: Box::new([]) };
    config.core.settings.llm_providers.providers = Box::new([playground, other]);
}
#[expect(clippy::too_many_lines, reason = "one complete client fixture with its configured forge fields")]
fn fixture_root(
    seed: u64,
    tools: Tools,
    own_accounts: u32,
    rows: Option<u32>,
    plan: bool,
    forge: WelcomeForge,
    program: bool,
) -> (Joint, Root) {
    let spend = match tools {
        Tools::Spend(spend) => Some(spend),
        Tools::Off | Tools::On | Tools::Trial | Tools::TrialSelection | Tools::TwoSignIns => None,
    };
    let two_sign_ins = tools == Tools::TwoSignIns;
    let selection_story = tools == Tools::TrialSelection;
    let trial_story = matches!(tools, Tools::Trial | Tools::TrialSelection);
    let tools = matches!(tools, Tools::On | Tools::Spend(_));
    let welcome = forge != WelcomeForge::None;
    let forge_setup = matches!(forge, WelcomeForge::Owned | WelcomeForge::Playground { .. });
    let mut limits = if tools { temper_client_tools::limits::client() } else { fixtures::domain_limits() };
    if let Some(rows) = rows {
        limits.client.domain.rows = rows;
    }
    let [(bootstrap, engine_limits), (config, _)] = std::array::from_fn(|index| {
        let mut config = walking::config(seed);
        if trial_story {
            config.core.workers_in_process = true;
        }
        // The joined world's worker scripts trials through the same model as its chats (domain/trial.md, 2).
        let mut trial = walking::charter();
        trial.number = 10;
        trial.policy.turns = 2;
        trial.policy.inspect = false;
        trial.policy.modify = false;
        trial.policy.shell = false;
        trial.policy.agents = false;
        trial.policy.tools = jig_domain_tasks::Tools(0);
        config.core.defaults.charters = Box::new([walking::charter(), trial]);
        for model in &mut config.core.settings.models {
            model.provider = PROVIDER;
        }
        for charter in &mut config.core.defaults.charters {
            charter.policy.model.provider = PROVIDER;
        }
        config.core.settings.llm_providers = a::Providers {
            providers: Box::new([a::Provider {
                list: None,
                number: PROVIDER,
                name: Box::from(if trial_story {
                    b"anthropic".as_slice()
                } else if plan {
                    b"openai".as_slice()
                } else {
                    b"Scripted".as_slice()
                }),
                signed_in_plan: plan,
                manage_usage: None,
                offers: a::Offers { sign_in: true, key: true, programs: Box::new([]) },
            }]),
        };
        // The runtime offers the playground and the real provider.
        let runtime = index == 1;
        if two_sign_ins && runtime {
            playground_providers(&mut config);
            // Two synthetic brands keep both sign-ins offered without the named plan cards.
            config.core.settings.llm_providers.providers[0].name = b"Welcome fake".as_slice().into();
        }
        if let WelcomeForge::Playground { .. } = forge
            && runtime
        {
            playground_providers(&mut config);
        }
        if tools || program {
            config.core.settings.llm_providers.providers.get_mut(0).expect("fixture provider").offers.programs =
                Box::new([Box::from(b"claude_code".as_slice())]);
        }
        if tools {
            let primary = config.core.settings.models[0].clone();
            let mut model = primary.clone();
            model.name = Box::from(b"effort-model".as_slice());
            model.efforts = Box::new([Box::from(b"high".as_slice())]);
            model.effort = None;
            config.core.settings.models = Box::new([primary, model.clone()]);
            config.core.defaults.charters.get_mut(0).expect("fixture charter").policy.alternatives = Box::new([model]);
        }
        if program {
            for charter in &mut config.core.defaults.charters {
                charter.policy.model.program = Some(b"claude_code".as_slice().into());
            }
            for model in &mut config.core.settings.models {
                model.program = Some(b"claude_code".as_slice().into());
            }
            config.core.settings.endpoints[0].program = Some(b"claude_code".as_slice().into());
        }
        if selection_story {
            config.core.settings.models[0].program = None;
            for charter in &mut config.core.defaults.charters {
                charter.policy.model.program = None;
            }
            let mut model = config.core.settings.models[0].clone();
            model.name = b"second-scripted".as_slice().into();
            model.provider = PROVIDER + 1;
            model.program = Some(b"claude_code".as_slice().into());
            model.endpoint = 2;
            config.core.settings.endpoints = Box::new([
                jig_domain_core::EndpointFacts {
                    direct: Box::new([]),
                    providers: Box::new([]),
                    number: 1,
                    dialect: model.dialect,
                    program: None,
                },
                jig_domain_core::EndpointFacts {
                    direct: Box::new([]),
                    providers: Box::new([]),
                    number: 2,
                    dialect: model.dialect,
                    program: model.program.clone(),
                },
            ]);
            config.core.settings.models = Box::new([config.core.settings.models[0].clone(), model.clone()]);
            config.core.defaults.charters[1].policy.alternatives = Box::new([model]);
            let mut provider = config.core.settings.llm_providers.providers[0].clone();
            provider.number = PROVIDER + 1;
            provider.name = b"Second".as_slice().into();
            config.core.settings.llm_providers.providers =
                Box::new([config.core.settings.llm_providers.providers[0].clone(), provider]);
        }
        if let Some(spend) = spend {
            let policy = &mut config.core.seed_projects[0].policy.policy;
            policy.roles[0].period_spend = spend;
            config.core.defaults.rules.period_spend = policy.period_spend.checked_mul(2).expect("fixture spend fits");
        }
        config.under_box |= forge_setup || matches!(forge, WelcomeForge::Empty | WelcomeForge::FirstRun);
        if forge == WelcomeForge::FirstRun {
            for owner in &mut config.core.owners {
                owner.project = 0;
            }
        }
        let place = temper_engine_domain_forge::Place {
            api: temper_engine_domain_forge::Address { host: b"forge.test".as_slice().into(), port: 443, tls: None },
            git: b"https://forge.test".as_slice().into(),
            web: None,
        };
        config.machine = Some(temper_engine_domain_forge::Machine {
            owned_person: Box::new([]),
            github_git_program: None,
            owned: Some(place.clone()),
            fake: Some(place.clone()),
            github: place,
        });
        config.core.connectors = Box::new([fixtures::FORGE]);
        for permission in &mut config.core.seed_projects[0].policy.permissions {
            permission.connector = fixtures::FORGE;
        }
        if welcome {
            config.forges = Box::new([temper_engine_domain_forge::Configured {
                admin_connect: false,
                bot: Box::new([]),
                owned: None,
                git: Box::new([]),
                web: None,
                read_only: false,
                number: 1,
                kind: temper_engine_domain_forge::Kind::Forgejo,
                mode: if forge_setup {
                    temper_engine_domain_forge::Mode::Owned
                } else {
                    temper_engine_domain_forge::Mode::Connected
                },
                asked: forge_setup,
            }]);
        }
        if matches!(forge, WelcomeForge::Playground { .. } | WelcomeForge::Empty | WelcomeForge::FirstRun) {
            config.forges = Box::new([]);
        }
        let mut engine_limits = engine_limits();
        engine_limits.tasks.charters = 2;
        // Restore one deployment account and leave the requested room for party accounts.
        engine_limits.documents.lines = engine_limits.documents.lines.max(own_accounts);
        engine_limits.accounts.accounts = own_accounts + 1;
        engine_limits.accounts.holder_accounts = own_accounts;
        engine_limits.accounts.credentials = own_accounts + 1;
        engine_limits.authority.accounts = 4;
        engine_limits.fleet.grants = (own_accounts + 1) * 2;
        // Each extra grant adds one entry to each of the four fleet routing cohorts.
        engine_limits.journal.writes =
            engine_limits.journal.writes.checked_add((engine_limits.fleet.grants - 2) * 4).expect("grant cohorts fit");
        if trial_story || matches!(forge, WelcomeForge::Owned | WelcomeForge::Empty | WelcomeForge::FirstRun) {
            super::first_run_projects(&mut config, &mut engine_limits);
        }
        if trial_story {
            // A minute's trial outlives the small world's one-minute sign-in.
            // Keep the person's session beyond this story's horizon.
            engine_limits.people.sign_in_lifetime = Duration::from_secs(86_400);
            // Declaration, verification, trial, Create and Home's keyed acts,
            // with room for the retry before their retention ends.
            engine_limits.people.requests = 12;
        }
        (config, engine_limits)
    });
    let restart_config = trial_story.then(|| temper_engine_domain_world::oracle::config::copy(&config));
    let mut front =
        Joint::with_engine(seed, &limits, Store::with_limits(&engine_limits), [bootstrap, config], &engine_limits, 1);
    front.trial_config = restart_config;
    // A running trial keeps its deadline armed while the story plays the worker.
    front.settles_with_a_deadline_armed = true;
    if forge_setup && !matches!(forge, WelcomeForge::Playground { .. }) {
        front.engine.send(engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::Connection {
            forge: 1,
            state: temper_engine_domain_forge::Connection::NotSetUp,
        })));
        front.engine.settle();
    }
    if let WelcomeForge::Playground { running: true, .. } = forge {
        let running = true;
        let playground = playground_answer(running).into_parts();
        front.playground = Some(
            temper_client_protocol_wire::v1::PlaygroundAnswer::new(
                &temper_client_protocol_wire::v1::CEILINGS,
                playground,
            )
            .expect("playground fixture"),
        );
    }
    if matches!(forge, WelcomeForge::Playground { printed: true, .. }) {
        front.offer_printed_link(limits);
    }
    front.bind(CLIENT, fixtures::PARTY);
    let mut root = if tools {
        crate::agent::new(seed).0
    } else {
        Root::Person(Box::new(temper_client_domain::unlinked(&limits, seed)))
    };
    event(
        &mut front,
        &mut root,
        Temper::start(
            if matches!(forge, WelcomeForge::Playground { running: true, .. }) {
                fixtures::chats_page()
            } else if welcome {
                temper_client_domain::Address::Welcome { step: temper_client_domain::Step::Welcome }
            } else {
                temper_client_domain::Address::Accounts
            },
            Vec::new(),
            0,
        ),
    );
    (front, root)
}
pub fn event(front: &mut Joint, root: &mut Root, event: Event) {
    event_traced(front, root, event);
}
/// Carry one press without delivering any engine callbacks.
pub fn press_turn(front: &mut Joint, root: &mut Root, event: Event) {
    let env = Env { now: front.now, wall: Wall::EPOCH, limits: front.limits };
    let mut out = Queue::uncapped();
    Temper::step(root, &env, event, &mut out);
    flush(front, root, &env, &mut out, &mut Vec::new());
}
/// Account requests in transport order, including sends chained from an answer.
pub fn event_traced(front: &mut Joint, root: &mut Root, event: Event) -> Vec<jig_client::contract::Ask> {
    let env = Env { now: front.now, wall: Wall::EPOCH, limits: front.limits };
    let mut out = Queue::uncapped();
    let mut sent = Vec::new();
    Temper::step(root, &env, event, &mut out);
    flush(front, root, &env, &mut out, &mut sent);
    settle_traced(front, root, &mut sent);
    sent
}
fn flush(
    front: &mut Joint,
    root: &mut Root,
    env: &Env<temper_client_domain::Limits>,
    out: &mut Queue<temper_client_domain::Request>,
    sent: &mut Vec<jig_client::contract::Ask>,
) {
    Temper::take(root, env, out);
    while let Some(request) = out.pop() {
        if let temper_client_domain::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Send { request: jig_client::Ask::Jig(ask), .. },
                },
        } = &request
        {
            sent.push(*ask.clone());
        }
        if let temper_client_domain::Request::Tab { tab } = request {
            front.tabs.push(tab);
            continue;
        }
        if let temper_client_domain::Request::Leave { to, .. } = request {
            front.leaves.push(to);
            continue;
        }
        match Temper::duty(request) {
            Duty::Wire(request) => front.wire(CLIENT, env.now, env.wall, request),
            Duty::Show { address, replace } => {
                front.shows.push((address.clone(), replace));
                Temper::step(root, env, Temper::went(address), out);
            }
            Duty::Store(_) | Duty::SignIn { .. } | Duty::SignOut | Duty::Fact => {}
        }
    }
}
/// Drain immediate callbacks at the current fake time; stories advance time explicitly.
pub fn settle(front: &mut Joint, root: &mut Root) {
    settle_traced(front, root, &mut Vec::new());
}
fn settle_traced(front: &mut Joint, root: &mut Root, sent: &mut Vec<jig_client::contract::Ask>) {
    let env = Env { now: front.now, wall: Wall::EPOCH, limits: front.limits };
    let mut out = Queue::uncapped();
    for _ in 0..32 {
        let callbacks = front.poll(env.now, env.wall);
        if callbacks.is_empty() {
            return;
        }
        for (who, event) in callbacks {
            assert_eq!(who, CLIENT);
            front.received(CLIENT, &event);
            Temper::step(root, &env, event, &mut out);
            flush(front, root, &env, &mut out, sent);
        }
    }
    panic!(
        "account root/front did not settle: page {:?}, trials {:?}",
        root.person().pages().address(),
        root.person().forge().trials()
    );
}
pub fn lose_next(front: &mut Joint) {
    front.fault(front.now, FrontFault::NoAnswer);
}
/// The root-only fixture plays the link's whole-answer loss under its actual attempt.
pub fn lost(front: &mut Joint, root: &mut Root) {
    let (who, attempt) = front.lost_answer.take().expect("a whole answer was lost");
    assert_eq!(who, CLIENT);
    event(
        front,
        root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Lost,
                    answer: None,
                    why: None,
                },
            },
        }
        .into(),
    );
}
/// Refuse the fake transport's sign-in answer before the client reads it.
pub fn refused_answer(front: &mut Joint, root: &mut Root) {
    let callbacks = front.poll(front.now, Wall::EPOCH);
    let mut replaced = 0;
    for (who, mut event) in callbacks {
        assert_eq!(who, CLIENT);
        if let Event::Root(temper_client_domain::Event::Client {
            event: jig_client::Event::Up { up: jig_client::Up::Answer { answer, .. } },
        }) = &mut event
            && matches!(
                answer,
                Some(jig_client::Answered::Jig(jig_client::contract::Answer::Done(
                    jig_client::contract::Outcome::Document(
                        jig_client::contract::DocumentOutcome::AccountSignInStarted { .. }
                    )
                )))
            )
        {
            *answer = Some(jig_client::Answered::Jig(jig_client::contract::Answer::Refused {
                refusal: jig_client::contract::Refusal::NotOffered,
                at: None,
            }));
            replaced += 1;
        }
        press_turn(front, root, event);
    }
    assert_eq!(replaced, 1, "one final sign-in answer refused");
    settle(front, root);
}
/// A durable answer lost until its retention ends becomes unknown.
pub fn expire_answer(front: &mut Joint, root: &mut Root) {
    let retention = temper_client_domain::pages::accounts::current_me(root.person().client()).expect("me").retention;
    front.now = front.now.saturating_add(retention);
    let env = Env { now: front.now, wall: Wall::EPOCH, limits: front.limits };
    let mut out = Queue::uncapped();
    Temper::fire(root, &env, &mut out);
    flush(front, root, &env, &mut out, &mut Vec::new());
}
/// The fake secrets store acknowledges the key under its source after moving the carrier.
pub fn kept(front: &mut Joint) -> u32 {
    let (source, carried) = front
        .engine
        .protocol
        .accounts
        .iter()
        .rev()
        .find_map(|request| {
            if let a::Request::Signing(a::Signing::Keep { source, carried, .. }) = request {
                Some((*source, *carried))
            } else {
                None
            }
        })
        .expect("key reached its component");
    assert_eq!(front.component.carried[&carried].expose(), b"test-private-key");
    front.component.carried.remove(&carried);
    front.engine.send(engine::Event::Account(a::Event::Component(a::FromComponent::Granted {
        account: source,
        generation: 1,
        valid: Duration::from_secs(3600),
        form: Form::Credential(Scheme::Bearer),
    })));
    front.engine.settle();
    source
}
#[must_use]
pub fn signing(front: &Joint) -> u32 {
    front
        .engine
        .protocol
        .accounts
        .iter()
        .rev()
        .find_map(|request| {
            if let a::Request::Signing(a::Signing::SignIn { source, .. }) = request { Some(*source) } else { None }
        })
        .expect("sign-in reached its component")
}
pub fn assert_no_carried(front: &Joint) {
    assert!(front.component.carried.is_empty(), "adopted and unused carriers were released");
}

/// A chat explicitly funded by the account whose row the story exercises.
pub fn account_chat(front: &mut Joint, account: u32) -> u64 {
    use jig_domain_people as people;
    let mut ask = front.engine.chat_ask(1, b"Wait for my plan");
    let people::Ask::Create { batch, .. } = &mut ask else { panic!("chat template") };
    batch[0].llm = Some(people::LlmChoice {
        later: None,
        account: Some(account),
        main: people::ModelEffort { model: 0, effort: None },
        subagents: Box::new([]),
    });
    front.engine.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(1001)),
        sign_in: front.transport.sign_in(CLIENT).expect("authenticated fixture client"),
        key: [101; 16],
        ask,
    }));
    // Waiting for this limited account deliberately keeps the fleet's deadline armed.
    front.settles_with_a_deadline_armed = true;
    front.settle();
    front
        .engine
        .delivered
        .iter()
        .rev()
        .find_map(|delivery| {
            if let engine::Delivery::WebReply {
                reply: people::Reply::Outcome(people::Outcome::Created { first, .. }),
                ..
            } = delivery
            {
                Some(*first)
            } else {
                None
            }
        })
        .expect("created account-funded task")
}

/// The playground, unchosen or running, beside the real root's account and forge facts.
#[must_use]
pub fn playground_fixture(seed: u64, running: bool) -> (Joint, Root) {
    playground_root(seed, running, false)
}
/// The same browser's playground with a local link for the TUI beside it.
#[must_use]
pub fn tui_playground_fixture(seed: u64) -> (Joint, Root) {
    playground_root(seed, false, true)
}
fn playground_root(seed: u64, running: bool, printed: bool) -> (Joint, Root) {
    let (mut front, mut root) =
        fixture_root(seed, Tools::Off, 2, None, false, WelcomeForge::Playground { running, printed }, false);
    if !running {
        event(&mut front, &mut root, Temper::went(temper_client_domain::Address::Accounts));
        (front, root) = without_deployment_account((front, root));
        event(
            &mut front,
            &mut root,
            Temper::went(temper_client_domain::Address::Welcome { step: temper_client_domain::Step::Welcome }),
        );
    }
    (front, root)
}

pub(super) fn playground_answer(running: bool) -> temper_client_protocol_wire::v1::PlaygroundAnswer {
    use temper_client_protocol_wire::v1 as w;
    let mut cues = skein_lib::List::with_capacity(1);
    cues.push(b"Change the title".as_slice().into()).expect("one cue");
    w::PlaygroundAnswer::new(
        &w::CEILINGS,
        w::PlaygroundAnswerParts {
            project: Some(1),
            forge: 2,
            provider: PROVIDER,
            owner: b"acme".as_slice().into(),
            name: b"shop".as_slice().into(),
            state: if running { w::PlaygroundState::Running } else { w::PlaygroundState::Starting },
            cues,
        },
    )
    .expect("playground fixture")
}

/// Fresh page on the same engine and person.
pub fn reopen(front: &mut Joint, root: &mut Root, address: temper_client_domain::Address) {
    let seed = fixtures::SEED.wrapping_add(1);
    // Close the previous client's watches before replacing its root.
    front.detach(CLIENT);
    front.hello(CLIENT);
    *root = Root::Person(Box::new(temper_client_domain::unlinked(&front.limits, seed)));
    event(front, root, Temper::start(address, Vec::new(), 0));
}
