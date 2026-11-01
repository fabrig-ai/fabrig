//! Accounts joins use the root-only pump of front/tests.rs; the page renderer belongs to part c.
use super::{Joint, engine_limits};
use crate::{
    app::{Event, Root, Temper},
    fixtures,
};
use jig_client_world::harness::{App, ClientId, Duty, Front, FrontFault};
use jig_domain_accounts as a;
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
}
#[must_use]
pub fn small_rows_fixture(seed: u64) -> (Joint, Root) {
    fixture_root(seed, Tools::Off, 3, Some(SMALL_ROWS), false, WelcomeForge::None, false)
}
#[must_use]
pub fn welcome_fixture(seed: u64) -> (Joint, Root) {
    fixture_root(seed, Tools::Off, 2, None, false, WelcomeForge::Fresh, false)
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
/// A story explicitly starts with the owned forge's kept record.
#[must_use]
pub fn seeded_forge_fixture(seed: u64) -> (Joint, Root) {
    let (mut front, root) = fixture_root(seed, Tools::Off, 2, None, false, WelcomeForge::SeededOwned, false);
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
    Fresh,
    Owned,
    SeededOwned,
    Playground,
    PlaygroundAccount,
}
#[must_use]
pub fn program_fixture(seed: u64) -> (Joint, Root) {
    fixture_root(seed, Tools::Off, 2, None, false, WelcomeForge::None, true)
}
#[must_use]
pub fn empty_fixture(seed: u64) -> (Joint, Root) {
    without_deployment_account(fixture(seed))
}
/// A `ChatGPT` sign-in with no configured model already usable.
#[must_use]
pub fn empty_plan_fixture(seed: u64) -> (Joint, Root) {
    without_deployment_account(plan_fixture(seed))
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
            row: Row { holder: jig_client::contract::AccountHolder::Deployment, account },
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
    // Keep the playground's provider distinct from the person's real model.
    let demo = config.core.settings.llm_providers.providers[0].clone();
    let mut other = demo.clone();
    other.number = PROVIDER + 1;
    other.name = Box::from(b"Other fake".as_slice());
    other.models = Box::new([Box::from(b"other-scripted".as_slice())]);
    other.offers = a::Offers { sign_in: true, key: true, programs: Box::new([]) };
    config.core.settings.llm_providers.providers = Box::new([demo, other]);
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
    let real_account = forge == WelcomeForge::PlaygroundAccount;
    let forge = if real_account { WelcomeForge::Playground } else { forge };
    let spend = match tools {
        Tools::Spend(spend) => Some(spend),
        Tools::Off | Tools::On => None,
    };
    let tools = tools != Tools::Off;
    let welcome = forge != WelcomeForge::None;
    let mut limits = if tools { temper_client_tools::limits::client() } else { fixtures::domain_limits() };
    if let Some(rows) = rows {
        limits.client.domain.rows = rows;
    }
    let [(bootstrap, engine_limits), (config, _)] = std::array::from_fn(|_| {
        let mut config = walking::config(seed);
        config.core.settings.llm_providers = a::Providers {
            providers: Box::new([a::Provider {
                number: PROVIDER,
                name: Box::from(if plan { b"openai".as_slice() } else { b"Scripted".as_slice() }),
                signed_in_plan: plan,
                manage_usage: None,
                models: Box::new([Box::from(b"scripted".as_slice())]),
                offers: a::Offers { sign_in: true, key: true, programs: Box::new([]) },
            }]),
        };
        if forge == WelcomeForge::Playground {
            playground_providers(&mut config);
        }
        if tools || program {
            config.core.settings.llm_providers.providers.get_mut(0).expect("fixture provider").offers.programs =
                Box::new([Box::from(b"claude_code".as_slice())]);
        }
        if tools {
            let mut model = walking::charter().policy.model;
            model.name = Box::from(b"effort-model".as_slice());
            model.efforts = Box::new([Box::from(b"high".as_slice())]);
            model.effort = None;
            config.core.settings.models = Box::new([walking::charter().policy.model, model.clone()]);
            config.core.defaults.charters.get_mut(0).expect("fixture charter").policy.alternatives = Box::new([model]);
            config.core.settings.llm_providers.providers.get_mut(0).expect("fixture provider").models =
                Box::new([Box::from(b"scripted".as_slice()), Box::from(b"effort-model".as_slice())]);
        }
        if let Some(spend) = spend {
            let policy = &mut config.core.seed_projects[0].policy.policy;
            policy.roles[0].period_spend = spend;
            config.core.defaults.rules.period_spend = policy.period_spend.checked_mul(2).expect("fixture spend fits");
        }
        config.under_box |= welcome;
        if welcome {
            config.core.connectors = Box::new([fixtures::FORGE]);
            config.machine = fixtures::machine();
            config.forge.writers = Box::new([]);
        }
        let mut engine_limits = engine_limits();
        if forge == WelcomeForge::Playground {
            super::first_run_projects(&mut config, &mut engine_limits);
            config.demo = Some(fixtures::playground_config());
            engine_limits.tasks.depth = 2;
            engine_limits.brief.sections = 5;
            config.core.defaults.rules.ceiling.delegation.depth = 2;
            let policy = &mut config.core.seed_projects[0].policy.policy;
            policy.ceiling.delegation.depth = 2;
            policy.roles[0].authority.delegation.depth = 2;
            let mut policy = config.core.defaults.policies.get(&1).expect("playground policy").clone();
            policy.policy.ceiling.delegation.depth = 2;
            policy.policy.roles[0].authority.delegation.depth = 2;
            config.core.defaults.policies.insert(1, policy).expect("configured policy");
            config.core.grants = Box::new([jig_domain_people::ActGrant {
                connector: fixtures::FORGE,
                may: Box::new([
                    jig_domain_people::ActKind::MakeProject,
                    jig_domain_people::ActKind::AddKey,
                    jig_domain_people::ActKind::Adopt,
                    jig_domain_people::ActKind::RemoveProject,
                    jig_domain_people::ActKind::RemoveAccount,
                ]),
            }]);
            engine_limits.people.act_grants = 1;
        }
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
        if matches!(forge, WelcomeForge::Owned | WelcomeForge::SeededOwned) {
            super::first_run_projects(&mut config, &mut engine_limits);
        }
        (config, engine_limits)
    });
    let mut store = if forge == WelcomeForge::Playground {
        temper_engine_domain_world::fixtures::first_start_over(Store::new(), bootstrap, &engine_limits)
    } else {
        temper_engine_domain_world::fixtures::restored_over(Store::new(), bootstrap, &engine_limits)
    };
    if real_account {
        let providers = a::Providers { providers: Box::new([config.core.settings.llm_providers.providers[1].clone()]) };
        let mut rows = temper_engine_domain_world::fixtures::account_rows(&providers);
        for record in &mut rows {
            if let engine::Record::Core(jig_domain_core::Record::Accounts(a::Stored::Account { holder, .. })) = record {
                *holder = a::Holder::Party(fixtures::PARTY);
            }
        }
        store.seed(rows, Box::new([]));
    }
    if forge == WelcomeForge::SeededOwned {
        let mut record = config.machine.owned.clone().expect("owned fixture");
        record.number = 1;
        store.seed(temper_engine_domain_world::forges::rows(&[record]), Box::new([]));
    }
    let mut front = Joint::with_engine(seed, &limits, store, [walking::config(seed), config], &engine_limits, 1);
    if forge == WelcomeForge::SeededOwned {
        front.engine.send(engine::Event::Forge(Box::new(temper_engine_domain_forge::Event::Connection {
            forge: 1,
            state: temper_engine_domain_forge::Connection::NotSetUp,
        })));
        front.engine.settle();
    }
    if real_account {
        front.engine.send(engine::Event::Account(a::Event::Component(a::FromComponent::Granted {
            account: 1,
            generation: 1,
            valid: Duration::from_secs(3600),
            form: a::Form::Credential(a::Scheme::Key),
        })));
        front.engine.settle();
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
            if welcome {
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
    let env = Env { now: front.now, wall: Wall::EPOCH, limits: front.limits };
    let mut out = Queue::uncapped();
    Temper::step(root, &env, event, &mut out);
    flush(front, root, &env, &mut out);
    settle(front, root);
}
fn flush(
    front: &mut Joint,
    root: &mut Root,
    env: &Env<temper_client_domain::Limits>,
    out: &mut Queue<temper_client_domain::Request>,
) {
    Temper::take(root, env, out);
    while let Some(request) = out.pop() {
        if let temper_client_domain::Request::Leave { to, .. } = request {
            front.leaves.push(to);
            continue;
        }
        match Temper::duty(request) {
            Duty::Wire(request) => front.wire(CLIENT, env.now, env.wall, request),
            Duty::Show { address, .. } => Temper::step(root, env, Temper::went(address), out),
            Duty::Store(_) | Duty::SignIn { .. } | Duty::SignOut | Duty::Fact => {}
        }
    }
}
pub fn settle(front: &mut Joint, root: &mut Root) {
    let mut env = Env { now: front.now, wall: Wall::EPOCH, limits: front.limits };
    let mut out = Queue::uncapped();
    for _ in 0..32 {
        let callbacks = front.poll(env.now, env.wall);
        if callbacks.is_empty() {
            return;
        }
        for (who, event) in callbacks {
            assert_eq!(who, CLIENT);
            env.now = env.now.saturating_add(Duration::from_millis(3));
            front.received(CLIENT, &event);
            Temper::step(root, &env, event, &mut out);
            flush(front, root, &env, &mut out);
        }
    }
    panic!("account root/front did not settle");
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
    assert_eq!(front.transport.carried()[&carried].expose(), b"test-private-key");
    front.transport.unused(carried);
    front.engine.send(engine::Event::Account(a::Event::Component(a::FromComponent::Granted {
        account: source,
        generation: 1,
        valid: Duration::from_secs(3600),
        form: a::Form::Credential(a::Scheme::Bearer),
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
    assert!(front.transport.carried().is_empty(), "adopted and unused carriers were released");
}

/// A chat explicitly funded by the account whose row the story exercises.
pub fn account_chat(front: &mut Joint, account: u32) -> u64 {
    use jig_domain_people as people;
    let mut ask = front.engine.chat_ask(1, b"Wait for my plan");
    let people::Ask::Create { batch, .. } = &mut ask else { panic!("chat template") };
    batch[0].llm = Some(people::LlmChoice {
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

/// The playground's configured model and fake forge, before its first start.
#[must_use]
pub fn playground_fixture(seed: u64) -> (Joint, Root) {
    fixture_root(seed, Tools::Off, 2, None, false, WelcomeForge::Playground, false)
}

/// A person's usable account precedes the playground's own account.
#[must_use]
pub fn playground_with_real_account_fixture(seed: u64) -> (Joint, Root) {
    fixture_root(seed, Tools::Off, 2, None, false, WelcomeForge::PlaygroundAccount, false)
}

/// Begin through the same client request as the welcome's playground button.
#[must_use]
pub fn running_playground_fixture(seed: u64) -> (Joint, Root) {
    let (mut front, mut root) = playground_fixture(seed);
    event(
        &mut front,
        &mut root,
        temper_client_domain::Event::Act {
            action: temper_client_domain::Action::Pages(temper_client_domain::pages::Act::Playground),
        }
        .into(),
    );
    assert!(front.playground().is_some());
    (front, root)
}

/// Fresh page on the same engine and person, with an explicit departure policy.
pub fn reopen(front: &mut Joint, root: &mut Root, address: temper_client_domain::Address, departing: bool) {
    let seed = fixtures::SEED.wrapping_add(1);
    // Close the previous client's watches before replacing its root.
    front.detach(CLIENT);
    front.hello(CLIENT);
    *root = Root::Person(Box::new(if departing {
        temper_client_domain::unlinked_departing(&front.limits, seed)
    } else {
        temper_client_domain::unlinked(&front.limits, seed)
    }));
    event(front, root, Temper::start(address, Vec::new(), 0));
}
