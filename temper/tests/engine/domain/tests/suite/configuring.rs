//! Scripted stock configuring chats through the root and jig's store
//! (temper/docs/design/domain/testing.md, 6.4; temper/docs/design/domain/agent.md, 8.1).
mod fixture;
use fixture::*;
use jig_domain_core::{self as core, settings as s};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{ReplyTo, Token};
use temper_engine_domain::{self as engine, Delivery, Event, PartyInput};
use temper_engine_domain_world::host_tool::host_tool;

// People in this story: owner (forge 7), signed-in member (8), retained
// writer (1), another project's person (9), adopted collaborator (2), service.
// The other initial owner takes number 2; account setup takes number 3.
const OWNER_PERSON: u64 = 1;
const MEMBER_PERSON: u64 = 4;
const WRITER_PERSON: u64 = 5;
const OTHER_PERSON: u64 = 2;
const ADOPTED_PERSON: u64 = 7;

fn sign_in_person(world: &mut World, user: u64, login: &[u8], key: u8) {
    world.driver.send(Event::Party(PartyInput::SignedIn {
        held: None,
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        identity: people::Identity {
            key: temper_engine_domain_forge::user_key(1, user),
            login: login.into(),
            name: b"Private display name".as_slice().into(),
        },
    }));
    world.driver.settle();
}

fn project_people_setup(mode: temper_engine_domain_forge::Mode, login: &[u8]) -> (engine::Config, engine::Limits) {
    use temper_engine_domain_forge as forge;
    let mut config = member_config();
    let other = core::SeedProject {
        number: 2,
        name: "Other project".into(),
        policy: config.core.seed_projects[0].policy.clone(),
    };
    config.core.seed_projects = config.core.seed_projects.into_vec().into_iter().chain([other]).collect();
    let mut owners = config.core.owners.into_vec();
    owners.push(people::InitialOwner { project: 2, identity: temper_engine_domain_forge::user_key(1, 9_u64) });
    config.core.owners = owners.into();
    config.forges[0].mode = mode;
    let place = forge::Place {
        api: forge::Address { host: b"forge.example".as_slice().into(), port: 3847, tls: None },
        git: b"http://forge.example:3847".as_slice().into(),
        web: None,
    };
    config.machine = Some(forge::Machine {
        owned_person: login.into(),
        owned: Some(place.clone()),
        fake: None,
        github: place,
        github_git_program: None,
    });
    let mut bounds = limits();
    bounds.people.people = 6; // Five people and the deployment service.
    bounds.people.sign_ins = 4; // Owner, member, writer and other project's owner.
    bounds.people.identity_bytes = 128; // Logins and the private display name fit.
    bounds.people.projects = 3; // Two projects and the deployment.
    bounds.people.initial_owners = 3;
    bounds.people.holdings = 8; // Five project parties and two other owner holdings.
    bounds.authority.projects = 2;
    (config, bounds)
}

fn project_people_world(mode: temper_engine_domain_forge::Mode, login: &[u8]) -> (World, u64) {
    let (config, bounds) = project_people_setup(mode, login);
    let mut world = World::with_limits(config, &bounds);
    let member = world.member();
    sign_in_person(&mut world, 1, b"temper-writer", 13);
    sign_in_person(&mut world, 9, b"elsewhere", 14);
    world.driver.forge =
        Some(temper_engine_domain_world::forge_stand::ForgeStand::with(&[(b"org", b"repo", 2), (b"org", b"other", 3)]));
    adopt_repository(&mut world, PROJECT, b"repo", 28);
    // A second readable repository on the same host must not repeat its person.
    adopt_repository(&mut world, PROJECT, b"other", 29);
    world.ask(
        world.owner,
        15,
        people::Ask::MakeService {
            project: PROJECT,
            name: b"deployment-service".as_slice().into(),
            role: people::Role::Member,
        },
    );
    let people::Reply::Outcome(people::Outcome::ServiceMade { person: service }) = *world.reply(15) else {
        panic!("service made: {:?}", world.reply(15))
    };
    world.ask(
        world.owner,
        16,
        people::Ask::SetRoles {
            project: PROJECT,
            holdings: Box::new([
                people::Holding { person: OWNER_PERSON, role: people::Role::Owner },
                people::Holding { person: MEMBER_PERSON, role: people::Role::Member },
                people::Holding { person: WRITER_PERSON, role: people::Role::Member },
                people::Holding { person: ADOPTED_PERSON, role: people::Role::Member },
                people::Holding { person: service, role: people::Role::Member },
            ]),
        },
    );
    (world, member)
}

fn box_approval_world() -> World {
    use skein_fake_forge_domain::api::Permission;
    let (mut config, mut bounds) = project_people_setup(temper_engine_domain_forge::Mode::Owned, b"ada");
    for owner in &mut config.core.owners {
        owner.identity = people::IdentityKey { provider: 3, subject: b"box-owner".as_slice().into() };
    }
    bounds.people.requests = 16; // Account, adoption, chat, turns and decisions.
    let mut world = World::with_limits(config, &bounds);
    let mut forge = temper_engine_domain_world::forge_stand::ForgeStand::with(&[(b"org", b"repo", 2)]);
    for user in [2, 7] {
        forge.grant(b"org/repo", user, Permission::None);
    }
    world.driver.forge = Some(forge);
    adopt_repository(&mut world, PROJECT, b"repo", 28);
    world
}

fn approval_items(world: &mut World, run: &engine::Assignment, completion: u32, role: u32) -> serde_json::Value {
    use serde_json::json;
    let setting = json!({"kind":"landing-rule","number":1});
    let (text, _) = read_whole(&mut world.driver, run, completion, json!({"setting":setting}));
    let (_, text) = text.split_once('\n').expect("landing rule heading");
    let mut landing = serde_json::Deserializer::from_str(text)
        .into_iter::<serde_json::Value>()
        .next()
        .expect("rule before application status")
        .expect("landing rule JSON");
    landing["approvals"] = json!([{"role":role,"people":1,"freshness":"exact"}]);
    json!([{"setting":setting,"set":landing}])
}

fn refuse_owner_approval(world: &mut World, run: &engine::Assignment, proposal: u64, key: u8) {
    world.ask(
        world.owner,
        key,
        people::Ask::DecideProposal {
            project: people::DEPLOYMENT,
            proposer: people::Proposer::Task(run.task),
            proposal,
            decision: people::ProposalDecision::Accept,
            running: Some(people::Running::Stays),
        },
    );
    let people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Judged { why, .. })) = world.reply(key) else {
        panic!("unsatisfiable approval refused: {:?}", world.reply(key))
    };
    let why = std::str::from_utf8(why).expect("refusal UTF-8");
    assert!(
        why.contains(
            "project 1 role 0 on forge forge.example:3847 requires 1 distinct known forge people; currently 0 eligible"
        ),
        "{why}"
    );
    assert_eq!(world.head().number, 1, "refused acceptance changes nothing");
}

fn approval_context(world: &mut World, run: &engine::Assignment) {
    use serde_json::json;
    let (inventory, _) = read_whole(&mut world.driver, run, 1, json!({}));
    assert!(
        inventory.contains("person 1 person: role 0, requester, forge identity unknown: cannot count for approvals"),
        "{inventory}"
    );
    assert!(
        inventory.contains(
            "forge forge.example:3847 configured owned-forge account ada; project membership unverified by this row"
        ),
        "{inventory}"
    );
    let policy = read_value(&mut world.driver, run, 10, json!({"setting":{"kind":"project-policy","project":PROJECT}}));
    assert!(
        policy["permissions"]
            .as_array()
            .expect("permission mapping")
            .iter()
            .any(|row| row == &json!({"connector":1,"permission":3,"role":1})),
        "{policy}"
    );
    approval_documentation(world, run);
    let words = std::str::from_utf8(&run.run.policy.instructions).expect("stock instructions");
    for rule in [
        "requester is the project party",
        "configured owned-forge account is separate",
        "existing verified project role",
        "read the current project-policy permission mapping",
        "admin seeds maintainer (1), never owner (0)",
        "Never promise admin fixes a role-0 proposal",
        "adopt again, reread inventory and propose afresh",
    ] {
        assert!(words.contains(rule), "missing stock rule: {rule}");
    }
}

fn approval_documentation(world: &mut World, run: &engine::Assignment) {
    use serde_json::json;
    let mut from = 0;
    let mut documentation = String::new();
    for completion in 20..30 {
        let (body, error) = host_tool(
            &mut world.driver,
            run,
            completion,
            b"read_docs",
            json!({"topic":"landing-rule","from":from}).to_string().as_bytes(),
        );
        assert!(!error);
        let page = std::str::from_utf8(&body).expect("documentation UTF-8");
        documentation.push_str(page);
        if let Some((_, next)) = page.rsplit_once("next: ") {
            from = next.trim().parse::<u32>().expect("documentation offset");
        } else {
            break;
        }
    }
    assert!(documentation.contains("Stock maps admin to maintainer (1)"), "{documentation}");
}

#[test]
fn a_box_inventory_distinguishes_the_requester_from_the_forge_approver() {
    let mut world = box_approval_world();
    let run = world.chat_words(CONFIGURING, world.owner, 20, b"Keep CI and agent review. Ask who else should approve.");
    world.ask(
        world.owner,
        21,
        people::Ask::Say {
            project: PROJECT,
            task: run.task,
            words: b"A person too, please: me for now.".as_slice().into(),
        },
    );
    let run = world.next(&run);
    heard_approval_turn(&mut world, &run, b"A person too, please: me for now.");
    approval_context(&mut world, &run);
    let items = approval_items(&mut world, &run, 40, 0);
    let proposal = propose(&mut world.driver, &run, 41, 1, &items);
    refuse_owner_approval(&mut world, &run, proposal, 22);
    world.ask(
        world.owner,
        23,
        people::Ask::Say {
            project: PROJECT,
            task: run.task,
            words: b"Acceptance was refused: role 0 has 0 eligible people.".as_slice().into(),
        },
    );
    adopted_approval(&mut world, &run, proposal);
}

fn heard_approval_turn(world: &mut World, run: &engine::Assignment, expected: &[u8]) {
    assert_heard(run, expected);
    let read = run.messages.iter().map(|word| word.number).max();
    let turn = world.driver.task(run.task).expect("live configuring chat").turn + 1;
    world.driver.send(temper_engine_domain_world::protocol::turn(
        CHANNEL,
        run.task,
        run.attempt,
        turn,
        0,
        read,
        b"Read the person's reply",
    ));
    world.driver.settle();
}

fn adopted_approval(world: &mut World, run: &engine::Assignment, proposal: u64) {
    use serde_json::json;
    world.driver.forge.as_mut().expect("fake forge").grant(
        b"org/repo",
        2,
        skein_fake_forge_domain::api::Permission::Admin,
    );
    adopt_repository(world, PROJECT, b"repo", 29);
    world.ask(
        world.owner,
        24,
        people::Ask::Say {
            project: PROJECT,
            task: run.task,
            words: b"Done: I am a collaborator on the forge with admin permission and adopted again.".as_slice().into(),
        },
    );
    let run = world.next(run);
    heard_approval_turn(
        world,
        &run,
        b"Done: I am a collaborator on the forge with admin permission and adopted again.",
    );
    let (inventory, _) = read_whole(&mut world.driver, &run, 50, json!({}));
    assert!(inventory.contains("person 1 person: role 0, requester, forge identity unknown"), "{inventory}");
    assert!(inventory.contains("person 3: role 1, forge forge.example:3847 user 2"), "{inventory}");
    refuse_owner_approval(world, &run, proposal, 25);
    world.decide(
        &run,
        proposal,
        26,
        people::ProposalDecision::Reject { reason: b"Use the verified maintainer role.".as_slice().into() },
        people::Running::Stays,
    );
    let items = approval_items(world, &run, 60, 1);
    let proposal = propose(&mut world.driver, &run, 61, 1, &items);
    world.decide(&run, proposal, 27, people::ProposalDecision::Accept, people::Running::Stays);
    assert_eq!(world.head().number, 2, "fresh maintainer proposal accepted");
}

#[test]
fn an_inventory_names_its_own_projects_people() {
    use serde_json::json;
    let (mut world, _) = project_people_world(temper_engine_domain_forge::Mode::Owned, b"box-person");
    let run = world.chat(CONFIGURING, world.owner, 20);
    let (inventory, _) = read_whole(&mut world.driver, &run, 1, json!({}));
    assert!(
        !inventory.contains(&format!("person {OTHER_PERSON}")),
        "another project's person stays private: {inventory}"
    );
    let (_, people) = inventory.split_once("project people:\n").expect("project people block");
    let (people, _) = people.split_once("models you can offer:").expect("offer follows people");
    assert_eq!(
        people,
        "  person 1 person: role 0, requester, forge forge.example:3847 user 7\n  person 4 member: role 2, forge forge.example:3847 user 8\n  person 5 temper-writer: role 2, forge forge.example:3847 user 1, temper's own forge writer: never counts as a person\n  person 7: role 2, forge forge.example:3847 user 2\nforge people:\n  forge forge.example:3847 configured owned-forge account box-person; project membership unverified by this row\n",
        "{inventory}"
    );
    for forbidden in ["elsewhere", "deployment-service", "Private display name", "Person", "@"] {
        assert!(!inventory.contains(forbidden), "private identity or service {forbidden}: {inventory}");
    }
    let repository_end = inventory.find("project people:").expect("people heading");
    assert!(inventory[..repository_end].contains("project repositories:\n"));
    for (completion, input) in [
        (10, json!({"setting":{"kind":"charter","number":CHAT}})),
        (11, json!({"revision":1})),
        (12, json!({"history":{}})),
    ] {
        let (text, _) = read_whole(&mut world.driver, &run, completion, input);
        assert!(!text.contains("project people:") && !text.contains("forge people:"), "{text}");
    }
    let (body, error) =
        host_tool(&mut world.driver, &run, 13, b"settings", br#"{"setting":{"kind":"charter","number":999}}"#);
    assert!(error);
    let text = std::str::from_utf8(&body).expect("not-read text");
    assert!(!text.contains("project people:") && !text.contains("forge people:"), "{text}");
}

// Keep the scripted run live while changing the accounts its inventory observes.
fn remove_scripted_account(world: &mut World) {
    let account = world
        .driver
        .store
        .rows
        .values()
        .find_map(|record| {
            if let engine::Record::Core(core::Record::Accounts(jig_domain_accounts::Stored::Account {
                number,
                provider: 1,
                ..
            })) = record
            {
                Some(*number)
            } else {
                None
            }
        })
        .expect("scripted account");
    world.ask(
        world.owner,
        31,
        people::Ask::AccountChange {
            holder: people::AccountHolder::Deployment,
            change: people::AccountChange::Remove { account },
        },
    );
    assert!(
        matches!(world.reply(31), people::Reply::Outcome(people::Outcome::AccountRemoved { .. })),
        "{:?}",
        world.reply(31)
    );
}

fn claude_offer_setup() -> (engine::Config, engine::Limits) {
    use jig_domain_accounts as accounts;
    use temper_engine_domain_world::walking;
    let (mut config, mut bounds) = project_people_setup(temper_engine_domain_forge::Mode::Owned, b"ada");
    bounds.accounts.accounts = 2; // Scripted key and the payer's Claude login.
    bounds.accounts.holder_accounts = 2;
    bounds.fleet.grants = bounds.accounts.credentials + bounds.accounts.accounts;
    let mut models = config.core.settings.models.clone().into_vec();
    // Cheap scripted models exercise the shipped names without a real provider or program.
    for name in ["claude-opus-5-5", "claude-sonnet-5-5", "claude-fable-5-1", "gpt-6.1-sol"] {
        let mut model = walking::models()[0].clone();
        model.name = name.as_bytes().into();
        model.provider = if name.starts_with("claude") { 2 } else { 3 };
        model.program = Some(b"claude_code".as_slice().into());
        model.endpoint = 2;
        model.efforts = Box::new([b"low".as_slice().into(), b"high".as_slice().into()]);
        model.effort = Some(b"high".as_slice().into());
        models.push(model);
    }
    let mut providers = config.core.settings.llm_providers.providers.clone().into_vec();
    for (number, name, _names) in [
        (2, "anthropic", vec!["claude-opus-5-5", "claude-sonnet-5-5", "claude-fable-5-1"]),
        (3, "openai", vec!["gpt-6.1-sol"]),
    ] {
        providers.push(accounts::Provider {
            number,
            name: name.as_bytes().into(),
            signed_in_plan: true,
            manage_usage: None,
            offers: accounts::Offers {
                sign_in: false,
                key: false,
                programs: Box::new([b"claude_code".as_slice().into()]),
            },
        });
    }
    // A setting keeps every configured alternative, even those the offer omits.
    for charter in &mut config.core.defaults.charters {
        charter.policy.alternatives = models[1..].into();
    }
    config.core.settings.endpoints = Box::new([
        jig_domain_core::EndpointFacts {
            providers: Box::new([]),
            number: 1,
            dialect: models[0].dialect,
            program: None,
        },
        jig_domain_core::EndpointFacts {
            providers: Box::new([]),
            number: 2,
            dialect: models[0].dialect,
            program: Some(b"claude_code".as_slice().into()),
        },
    ]);
    let stock = config.core.defaults.models[0].clone();
    config.core.defaults.models = models
        .iter()
        .enumerate()
        .map(|(at, model)| {
            let mut entry = stock.clone();
            entry.number = u32::try_from(at).expect("bounded fixture models") + 1;
            entry.model = model.clone();
            entry
        })
        .collect();
    config.core.settings.models = models.into();
    config.core.settings.llm_providers.providers = providers.into();
    (config, bounds)
}

fn claude_offer_world() -> World {
    use temper_engine_domain_world::protocol;
    let (config, bounds) = claude_offer_setup();
    let mut capabilities: Vec<_> = config
        .core
        .settings
        .models
        .iter()
        .map(|model| jig_domain_link::EndpointBound {
            endpoint: model.endpoint,
            window: model.window,
            output: model.output,
        })
        .collect();
    capabilities.sort_by_key(|endpoint| endpoint.endpoint);
    capabilities.dedup_by_key(|endpoint| endpoint.endpoint);
    let mut world = World::with_limits(config, &bounds);
    world.ask(
        world.owner,
        30,
        people::Ask::AccountChange {
            holder: people::AccountHolder::Own,
            change: people::AccountChange::DeclareProgram {
                provider: 2,
                join: None,
                name: b"claude-login".as_slice().into(),
            },
        },
    );
    assert!(
        matches!(world.reply(30), people::Reply::Outcome(people::Outcome::AccountProgramDeclared { .. })),
        "{:?}",
        world.reply(30)
    );
    world.driver.send(protocol::lost(CHANNEL));
    world.driver.settle();
    let mut greeting = protocol::hello_slots(CHANNEL, 3, Box::new([]));
    let Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut greeting else {
        panic!("hello")
    };
    hello.bounds.endpoints = capabilities.into_boxed_slice();
    hello.bounds.program_endpoints = Box::new([2]);
    hello.bounds.time = skein_lib::Duration::from_secs(28_800);
    hello.bounds.turns = 800;
    hello.bounds.message_bytes = bounds.tasks.message_bytes;
    world.driver.send(greeting);
    // The shared protocol queues an empty inventory; this worker answers its declared login.
    let mut answered = false;
    for _ in 0..200 {
        // The direct world's settlement step bound.
        for event in &mut world.driver.events {
            if let Event::Worker(engine::WorkerInput::Link {
                channel,
                link: engine::Link::Up { up: jig_domain_host::LinkUp::Logins { logins }, .. },
            }) = event
            {
                assert_eq!(*channel, CHANNEL);
                *logins = Box::new([jig_domain_link::Login {
                    name: jig_domain_link::SourceName::new(b"claude-login").expect("fake login name"),
                    state: jig_domain_link::LoginState::Usable,
                    email: None,
                }]);
                answered = true;
            }
        }
        if answered {
            break;
        }
        world.driver.advance(true);
    }
    assert!(answered, "the worker is asked for its program login inventory");
    world.driver.settle();
    world
}

#[test]
fn an_inventory_names_the_payers_usable_claude_offer() {
    use serde_json::json;
    let mut world = claude_offer_world();
    let run = world.chat(CONFIGURING, world.owner, 20);
    remove_scripted_account(&mut world);
    let (inventory, _) = read_whole(&mut world.driver, &run, 1, json!({}));
    let (_, offer) = inventory.split_once("models you can offer:\n").expect("offer heading");
    assert_eq!(
        offer,
        "  claude-opus-5-5: anthropic; efforts low, high; default high\n  claude-sonnet-5-5: anthropic; efforts low, high; default high\n  claude-fable-5-1: anthropic; efforts low, high; default high\n",
        "{inventory}"
    );
    assert!(!inventory.contains("gpt-") && !inventory.contains("openai"), "{inventory}");
}

#[test]
fn an_inventory_says_when_no_account_can_run_a_model() {
    use serde_json::json;
    let mut world = box_approval_world();
    let run = world.chat(CONFIGURING, world.owner, 20);
    remove_scripted_account(&mut world);
    let (inventory, _) = read_whole(&mut world.driver, &run, 1, json!({}));
    assert!(inventory.ends_with("models you can offer: none; no account can run a model yet\n"), "{inventory}");
}

#[test]
fn settings_and_revision_reads_carry_no_model_offer() {
    use serde_json::json;
    let mut world = claude_offer_world();
    let run = world.chat(CONFIGURING, world.owner, 20);
    remove_scripted_account(&mut world);
    for (completion, input) in [
        (1, json!({"setting":{"kind":"charter","number":CONFIGURING}})),
        (100, json!({"revision":1})),
        (110, json!({"history":{}})),
    ] {
        let (text, _) = read_whole(&mut world.driver, &run, completion, input);
        assert!(
            !text.lines().any(|line| line.starts_with("models you can offer:")) && !text.contains("project people:"),
            "{text}"
        );
        if completion == 1 {
            assert!(text.contains("gpt-6.1-sol"), "a setting keeps its complete model list: {text}");
        }
    }
}

#[test]
fn the_configuring_agent_speaks_from_the_inventory_offer() {
    let mut world = box_approval_world();
    let run = world.chat(CONFIGURING, world.owner, 20);
    let instructions = std::str::from_utf8(&run.run.policy.instructions).expect("stock instructions");
    assert!(instructions.contains("Inventory: models you can offer: all that accounts can run now. Propose and mention only those, even where a charter names others; none, or asked for a model no account runs: say so; connect one in browser Settings, Accounts, one line."), "{instructions}");
}

#[test]
fn a_members_chat_reads_the_same_people() {
    use serde_json::json;
    let (mut world, _) = project_people_world(temper_engine_domain_forge::Mode::Owned, b"box-person");
    let owner_run = world.chat(CONFIGURING, world.owner, 20);
    let (owner, _) = read_whole(&mut world.driver, &owner_run, 1, json!({}));
    let (mut world, member) = project_people_world(temper_engine_domain_forge::Mode::Owned, b"box-person");
    let member_run = world.chat(CONFIGURING, member, 20);
    let (member, _) = read_whole(&mut world.driver, &member_run, 1, json!({}));
    let (_, owner) = owner.split_once("project people:\n").expect("owner's people block");
    let (_, member) = member.split_once("project people:\n").expect("member's people block");
    assert!(owner.contains("person 1 person: role 0, requester, forge forge.example:3847 user 7"), "{owner}");
    assert!(member.contains("person 4 member: role 2, requester, forge forge.example:3847 user 8"), "{member}");
    assert_eq!(owner.replace(", requester", ""), member.replace(", requester", ""));
}

#[test]
fn a_forge_person_requires_owned_mode_and_a_login() {
    use temper_engine_domain_forge::Mode;
    for (mode, login) in [
        (Mode::Connected, b"box-person".as_slice()),
        (Mode::Managed, b"box-person"),
        (Mode::Fake, b"box-person"),
        (Mode::Owned, b""),
    ] {
        let (mut world, _) = project_people_world(mode, login);
        let run = world.chat(CONFIGURING, world.owner, 20);
        let (inventory, _) = read_whole(&mut world.driver, &run, 1, serde_json::json!({}));
        assert!(!inventory.contains("forge people:"), "{mode:?}: {inventory}");
    }
}

#[test]
fn a_configuring_chat_discovers_the_forge_host_for_charter_documents() {
    use serde_json::json;
    let mut world = repository_projects();
    configuring_repository_text(&mut world, &["rules\n".into(), "words\n".into()]);
    adopt_repository(&mut world, 2, b"other", 28);
    let run = world.chat(CONFIGURING, world.owner, 20);
    let (inventory, _) = read_whole(&mut world.driver, &run, 1, json!({}));
    let row = inventory
        .lines()
        .find_map(|line| line.strip_prefix("  forge "))
        .expect("settings inventory exposes the canonical forge host and repository");
    let (host, repository) = row.split_once(" repository ").expect("repository identity");
    assert_eq!(host, "forge.example:3847", "the canonical host keeps its port");
    assert_eq!(repository, "org/repo");
    assert!(!inventory.contains("org/other"), "another project's repository stays private");
    let mut value = read_charter(&mut world.driver, &run, 10, CHAT);
    value["documents"] = json!([{
        "forge":host,"repository":repository,"branch":"main","path":"docs/direction.md"
    }]);
    let items = json!([{"setting":{"kind":"charter","number":CHAT},"set":value}]);
    let proposal = propose(&mut world.driver, &run, 20, 1, &items);
    world.decide(&run, proposal, 21, people::ProposalDecision::Accept, people::Running::Stays);
    assert_eq!(world.head().number, 2);
    let chat = world.chat(CHAT, world.owner, 22);
    let section = chat
        .sections
        .iter()
        .find(|section| {
            matches!(&section.kind,
        engine::BriefKind::Forge(engine::ForgeBriefKind::Document(name))
            if name.as_ref() == b"org/repo docs/direction.md")
        })
        .expect("accepted charter document gathered through the fake forge");
    let engine::BriefBody::Text(text) = &section.body else { panic!("document text: {:?}", section.body) };
    let text = std::str::from_utf8(text).expect("document UTF-8");
    assert!(text.starts_with("At main's head "), "{text}");
    assert!(text.contains("rules\n"), "{text}");
}

#[test]
fn an_owners_configuring_chat_proposes_and_hears_it_accepted() {
    let mut world = World::stock();
    let run = world.chat(CONFIGURING, world.owner, 20);
    assert_eq!(run.charter, CONFIGURING);
    assert_eq!(
        run.run.policy.instructions.as_ref(),
        include_bytes!("../../../../../crates/temper-service/src/stock/configuring.txt")
    );
    let offered = temper_engine_smith::charter(&run);
    for name in [b"settings".as_slice(), b"propose_settings"] {
        assert!(offered.grants.host_tools.iter().any(|tool| tool.name.as_ref() == name));
    }
    let (inventory, error) = host_tool(&mut world.driver, &run, 1, b"settings", br"{}");
    assert!(!error);
    assert!(inventory.starts_with(b"newest revision 1\n"));
    let value = read_charter(&mut world.driver, &run, 2, CHAT);
    let items = charter_item(value, b"Push early and report the result.");
    let proposal = propose(&mut world.driver, &run, 3, 1, &items);
    let review = world.review(&run, proposal);
    assert_eq!(review.reason.bytes.as_ref(), REASON.as_bytes());
    assert_eq!(world.head().number, 1, "a proposal alone changes no settings");
    world.decide(&run, proposal, 21, people::ProposalDecision::Accept, people::Running::Stays);
    assert_eq!(world.head().number, 2);
    assert!(!world.head().moved);
    assert_eq!(
        world.driver.task(run.task).expect("configuring task").settings,
        tasks::SettingsRevision { task: 1, run: Some(1) },
        "accepting with running work staying keeps the current task and run on revision 1"
    );
    assert_eq!(
        world.head().proposal,
        Some((people::Proposer::Task(run.task), u32::try_from(proposal).expect("proposal number")))
    );
    let next = world.next(&run);
    assert_heard(&next, b"accepted: revision 2 made");
}

#[test]
fn a_proposal_selects_the_template_it_installs() {
    use serde_json::json;
    for reverse in [false, true] {
        let mut world = World::stock();
        let run = world.chat(CONFIGURING, world.owner, 20);
        let mut value = read_value(&mut world.driver, &run, 1, json!({"setting":{"kind":"template","number":3}}));
        value["number"] = json!(4);
        value["tracked"] = json!(1);
        let ways = json!({"drafts":true,"goals":true,"changes":true,"away":true,"way":4});
        let mut items = vec![
            json!({"setting":{"kind":"template","number":4},"set":value}),
            json!({"setting":{"kind":"project-ways","project":PROJECT},"set":ways}),
        ];
        if reverse {
            items.reverse();
        }
        let proposal = propose(&mut world.driver, &run, 10, 1, &json!(items));
        let review = world.review(&run, proposal);
        let core::documents::ProposalAction::Settings { review: Some(settings), .. } = &review.action else {
            panic!("settings review")
        };
        assert_eq!(settings.items.len(), 2);
        assert_eq!(world.head().number, 1, "proposing publishes neither item");
        let (_, error) =
            host_tool(&mut world.driver, &run, 12, b"settings", br#"{"setting":{"kind":"template","number":4}}"#);
        assert!(error, "the proposed template is not published");
        let (text, _) =
            read_whole(&mut world.driver, &run, 13, json!({"setting":{"kind":"project-ways","project":PROJECT}}));
        assert!(text.contains("\"way\":0"), "proposing keeps the stock way: {text}");
        world.decide(&run, proposal, 21, people::ProposalDecision::Accept, people::Running::Stays);
        assert_eq!(world.head().number, 2, "one acceptance makes one revision");
        let installed = read_value(&mut world.driver, &run, 30, json!({"setting":{"kind":"template","number":4}}));
        assert_eq!(installed, value);
        let (text, _) =
            read_whole(&mut world.driver, &run, 40, json!({"setting":{"kind":"project-ways","project":PROJECT}}));
        let body = text.lines().nth(1).expect("ways value");
        assert_eq!(serde_json::from_str::<serde_json::Value>(body).expect("ways JSON"), ways);
        assert!(!world.settings_rows().iter().any(|row| matches!(row, s::Record::Proposal { .. })), "proposal ends");
        let next = world.next(&run);
        assert_heard(&next, b"accepted: revision 2 made");
    }
}

#[test]
fn direct_ways_are_checked_against_the_templates_the_decision_leaves() {
    use temper_engine_domain_forge::settings::{ProjectWays, Value};
    use temper_engine_domain_world::kinds;
    let template = config()
        .core
        .defaults
        .templates
        .into_vec()
        .into_iter()
        .find(|value| value.number == 3)
        .expect("configuring template");
    let added = s::Item {
        setting: s::Setting::Template(4),
        edit: s::Edit::Set(s::Value::Template(Box::new(core::Template { number: 4, ..template }))),
    };
    for (name, item, way, accepted) in [
        ("zero", added.clone(), 0, true),
        ("current", added.clone(), 3, true),
        ("installed", added.clone(), 4, true),
        ("absent", added, 5, false),
        ("removed", s::Item { setting: s::Setting::Template(3), edit: s::Edit::Removed }, 3, false),
        ("reset addition", s::Item { setting: s::Setting::Template(4), edit: s::Edit::Reset }, 4, false),
        ("reset default", s::Item { setting: s::Setting::Template(3), edit: s::Edit::Reset }, 3, true),
    ] {
        let mut world = World::stock();
        if name == "reset default" {
            world.change(1, 21, Box::new([s::Item { setting: s::Setting::Template(3), edit: s::Edit::Removed }]));
        }
        let before = world.settings_rows();
        let against = world.head().number;
        let ways = kinds::item(
            7,
            PROJECT,
            1,
            &Value::Ways(ProjectWays { drafts: true, goals: true, changes: true, away: true, way }),
        );
        world.driver.send(kinds::change(world.owner, against, 22, Box::new([ways, item])));
        world.driver.settle();
        if accepted {
            assert_eq!(world.head().number, against + 1, "{name}");
        } else {
            let people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Judged { at, setting, why })) =
                world.reply(22)
            else {
                panic!("{name}: {:?}", world.reply(22))
            };
            assert_eq!(*at, 0, "{name}");
            assert_eq!(*setting, s::Setting::Application { kind: 7, scope: PROJECT, number: 1 });
            assert_eq!(
                why.as_ref(),
                format!("way {way} is not a template the project offers after this decision").as_bytes()
            );
            assert_eq!(world.settings_rows(), before, "{name} commits neither item");
        }
    }
}

#[test]
fn a_project_with_no_ways_reads_at_their_stock() {
    use serde_json::json;
    let mut world = World::stock();
    let run = world.chat(CONFIGURING, world.owner, 20);
    for (completion, kind, value) in [
        (1, "project-ways", r#"{"drafts":true,"goals":false,"changes":false,"away":false,"way":0}"#),
        (2, "project-questions", r#""none""#),
    ] {
        let input = json!({"setting":{"kind":kind,"project":PROJECT}});
        let (body, error) = host_tool(&mut world.driver, &run, completion, b"settings", input.to_string().as_bytes());
        let text = std::str::from_utf8(&body).expect("settings UTF-8");
        assert!(!error, "{text}");
        let expected = format!("{kind} {PROJECT}, revision 1\n{value}\nthe stock\nend\n");
        assert_eq!(text, expected);
        let input = json!({"setting":{"kind":kind,"project":PROJECT},"revision":1});
        let (body, error) =
            host_tool(&mut world.driver, &run, completion + 2, b"settings", input.to_string().as_bytes());
        assert!(!error);
        assert_eq!(body.as_ref(), expected.as_bytes(), "the named newest revision also takes the stock");
    }
}

#[test]
fn a_projects_stored_ways_read_as_stored() {
    use serde_json::json;
    let mut world = World::stock();
    let run = world.chat(CONFIGURING, world.owner, 20);
    let setting = json!({"kind":"project-ways","project":PROJECT});
    let value = json!({"drafts":true,"goals":true,"changes":true,"away":true,"way":1});
    let items = json!([{"setting":setting,"set":value}]);
    let proposal = propose(&mut world.driver, &run, 1, 1, &items);
    world.decide(&run, proposal, 21, people::ProposalDecision::Accept, people::Running::Stays);
    assert_eq!(world.head().number, 2);
    let (text, _) = read_whole(&mut world.driver, &run, 2, json!({"setting":setting}));
    let body = text.strip_prefix("project-ways 1, revision 2\n").expect("newest setting heading");
    let (body, state) = body.split_once('\n').expect("value and application state");
    assert_eq!(serde_json::from_str::<serde_json::Value>(body).expect("ways JSON"), value);
    assert!(state.starts_with("changed at revision 2, "), "{text}");
    assert!(!text.contains("the stock"), "{text}");
}

#[test]
fn a_read_that_finds_nothing_says_it_was_not_read() {
    use serde_json::json;
    let mut world = World::stock();
    let run = world.chat(CONFIGURING, world.owner, 20);
    world.change(1, 21, Box::new([owner_charter(b"Keep commits small.")]));
    for (completion, input) in [
        (1, json!({"setting":{"kind":"project-ways","project":999}})),
        (2, json!({"setting":{"kind":"project-questions","project":999}})),
        (3, json!({"setting":{"kind":"charter","number":REMOVED}})),
        (4, json!({"setting":{"kind":"project-ways","project":PROJECT},"revision":1})),
        (5, json!({"setting":{"kind":"project-questions","project":PROJECT},"revision":1})),
    ] {
        let (body, error) = host_tool(&mut world.driver, &run, completion, b"settings", input.to_string().as_bytes());
        let text = std::str::from_utf8(&body).expect("settings UTF-8");
        assert!(error, "{input}: {text}");
        assert_eq!(
            text,
            "not read: no such setting at that revision. This says nothing about the settings: read the inventory and name what it lists; project-ways and project-questions take project and no number.\nend\n"
        );
    }
}

#[test]
fn a_members_proposal_waits_for_an_owner() {
    let mut world = World::new(member_config());
    let member = world.member();
    let run = world.chat(CONFIGURING, member, 20);
    let value = read_charter(&mut world.driver, &run, 1, CHAT);
    let items = charter_item(value, b"Push early and report the result.");
    let proposal = propose(&mut world.driver, &run, 2, 1, &items);
    assert_eq!(world.head().number, 1);
    let review = world.review(&run, proposal);
    assert_eq!(
        review.waiting.holder,
        tasks::ProposalHolder::Policy { project: people::DEPLOYMENT, kind: tasks::ProposalKind::Settings }
    );
    world.driver.send(Event::Party(PartyInput::Read {
        reply_to: ReplyTo::new(Token::new(91)),
        sign_in: world.owner,
        read: core::reads::Read::Inbox {
            group: core::reads::Group::Proposals,
            filter: core::reads::Filter { project: None, goal: None },
            from: None,
        },
    }));
    world.driver.settle();
    let inbox = world
        .driver
        .delivered
        .iter()
        .rev()
        .find_map(|delivery| {
            if let Delivery::Read { answer: core::reads::Answer::Page(page), .. } = delivery {
                Some(page)
            } else {
                None
            }
        })
        .expect("owner's proposal inbox");
    assert!(inbox.rows.iter().any(|row| matches!(row,
        core::reads::PageRow::Inbox(entry) if entry.object == core::documents::InboxObject::Proposal {
            proposer: people::Proposer::Task(run.task), number: proposal
        }
    )));
    world.ask(
        member,
        21,
        people::Ask::DecideProposal {
            project: people::DEPLOYMENT,
            proposer: people::Proposer::Task(run.task),
            proposal,
            decision: people::ProposalDecision::Accept,
            running: Some(people::Running::Stays),
        },
    );
    assert!(matches!(world.reply(21), people::Reply::Outcome(people::Outcome::Refused(_))));
    assert_eq!(world.head().number, 1, "a project role cannot accept the deployment's settings");
    assert_eq!(world.review(&run, proposal).waiting, review.waiting);
    world.decide(&run, proposal, 22, people::ProposalDecision::Accept, people::Running::Stays);
    assert_eq!(world.head().number, 2);
    assert_heard(&world.next(&run), b"accepted: revision 2 made");
}

#[test]
fn a_stale_proposal_is_withdrawn_read_again_and_proposed_afresh() {
    let mut world = World::stock();
    let run = world.chat(CONFIGURING, world.owner, 20);
    let value = read_charter(&mut world.driver, &run, 1, CHAT);
    let items = charter_item(value, b"Push early.");
    let stale = propose(&mut world.driver, &run, 2, 1, &items);
    world.change(1, 21, Box::new([owner_charter(b"Keep commits small.")]));
    let next = world.next(&run);
    assert_heard(&next, b"withdrawn as stale: the newest is 2");
    let (inventory, error) = host_tool(&mut world.driver, &next, 1, b"settings", br"{}");
    assert!(!error);
    assert!(inventory.starts_with(b"newest revision 2\n"));
    let value = read_charter(&mut world.driver, &next, 2, CHAT);
    assert_eq!(value["instructions"], "Keep commits small.");
    let items = charter_item(value, b"Keep commits small. Push early.");
    let fresh = propose(&mut world.driver, &next, 3, 2, &items);
    assert_ne!(fresh, stale);
    world.decide(&next, fresh, 22, people::ProposalDecision::Accept, people::Running::Stays);
    assert_eq!(world.head().number, 3);
    assert_heard(&world.next(&next), b"accepted: revision 3 made");
}

#[test]
fn a_rejected_proposal_is_revised() {
    let mut world = World::stock();
    let run = world.chat(CONFIGURING, world.owner, 20);
    let value = read_charter(&mut world.driver, &run, 1, CHAT);
    let items = charter_item(value, b"Push every edit.");
    let first = propose(&mut world.driver, &run, 2, 1, &items);
    world.decide(
        &run,
        first,
        21,
        people::ProposalDecision::Reject { reason: b"Push each complete change instead.".as_slice().into() },
        people::Running::Stays,
    );
    assert_eq!(world.head().number, 1, "rejecting changes no settings");
    let next = world.next(&run);
    assert_heard(&next, b"rejected: Push each complete change instead.");
    let value = read_charter(&mut world.driver, &next, 1, CHAT);
    let items = charter_item(value, b"Push each complete change.");
    let revised = propose(&mut world.driver, &next, 2, 1, &items);
    assert_ne!(first, revised);
    let review = world.review(&next, revised);
    let core::documents::ProposalAction::Settings { review: Some(review), .. } = review.action else {
        panic!("settings proposal review")
    };
    assert!(review.items.iter().any(|item| matches!(&item.would_be,
        Some(s::Value::Charter(charter)) if charter.policy.instructions.as_ref() == b"Push each complete change."
    )));
    world.decide(&next, revised, 22, people::ProposalDecision::Accept, people::Running::Stays);
    assert_eq!(world.head().number, 2);
    assert_heard(&world.next(&next), b"accepted: revision 2 made");
}

#[test]
fn an_accepted_proposal_moves_running_work() {
    let mut world = World::new(moving_config());
    let chat = world.chat(CHAT, world.owner, 20);
    let removed = world.chat(REMOVED, world.owner, 21);
    world.park(&chat);
    world.park(&removed);
    let run = world.chat(CONFIGURING, world.owner, 22);
    let value = read_charter(&mut world.driver, &run, 1, CHAT);
    let mut items = charter_item(value, b"Push each complete change.");
    let mut rules = read_value(&mut world.driver, &run, 10, serde_json::json!({"setting":{"kind":"rules"}}));
    omit_removable(&mut rules["ceiling"]);
    let mut policy =
        read_value(&mut world.driver, &run, 30, serde_json::json!({"setting":{"kind":"policy","number":1}}));
    omit_removable(&mut policy["ceiling"]);
    for role in policy["roles"].as_array_mut().expect("policy roles") {
        omit_removable(&mut role["authority"]);
    }
    // Removing a charter also removes its references, in the same proposal.
    items.as_array_mut().expect("items").extend([
        serde_json::json!({"setting":{"kind":"charter","number":REMOVED},"edit":"removed"}),
        serde_json::json!({"setting":{"kind":"template","number":REMOVED},"edit":"removed"}),
        serde_json::json!({"setting":{"kind":"rules"},"set":rules}),
        serde_json::json!({"setting":{"kind":"policy","number":1},"set":policy}),
        serde_json::json!({"setting":{"kind":"project-policy","project":PROJECT},"set":policy}),
    ]);
    let proposal = propose(&mut world.driver, &run, 50, 1, &items);
    world.decide(&run, proposal, 23, people::ProposalDecision::Accept, people::Running::Moves);
    assert_eq!(world.head().number, 2);
    assert!(world.head().moved);
    assert_eq!(
        world.driver.task(chat.task).expect("stock chat").settings,
        tasks::SettingsRevision { task: 2, run: None }
    );
    assert_eq!(
        world.driver.task(removed.task).expect("removed charter's task").settings,
        tasks::SettingsRevision { task: 1, run: None }
    );
    assert_eq!(
        world.driver.task(run.task).expect("configuring task").settings,
        tasks::SettingsRevision { task: 2, run: Some(1) },
        "the live run keeps its prepared revision"
    );
    let next = world.wake(&chat, 24);
    assert_eq!(next.run.revision, 2);
    assert_eq!(next.run.policy.instructions.as_ref(), b"Push each complete change.");
    let stayed = world.wake(&removed, 25);
    assert_eq!(stayed.run.revision, 1);
    assert_eq!(stayed.run.policy.instructions, removed.run.policy.instructions);
}

#[test]
fn the_configuring_agent_reads_each_kind() {
    use serde_json::json;
    let mut world = World::stock();
    let run = world.chat(CONFIGURING, world.owner, 20);
    let held = read_charter(&mut world.driver, &run, 1, CHAT);
    world.change(1, 21, Box::new([owner_charter(b"The newest chat instructions.")]));
    let before = world.driver.store.rows.clone();
    let mut completion = 10;
    for (input, heading, content) in [
        (json!({}), "newest revision 2\n", "held revisions:\n  revision 1,"),
        (
            json!({"setting":{"kind":"charter","number":CHAT}}),
            "charter 1, revision 2\n",
            "The newest chat instructions.",
        ),
        (json!({"setting":{"kind":"template","number":1}}), "template 1, revision 2\n", "\"bytes\":\"chat\""),
        (json!({"setting":{"kind":"policy","number":1}}), "policy 1, revision 2\n", "\"roles\":"),
        (json!({"setting":{"kind":"rules"}}), "rules, revision 2\n", "\"ceiling\":"),
        (json!({"setting":{"kind":"landing-rule","number":1}}), "landing-rule 1, revision 2\n", "\"enforce\":true"),
        (json!({"setting":{"kind":"forge-charter","number":1}}), "forge-charter 1, revision 2\n", "default, "),
        (json!({"revision":2}), "revision 2, against revision 1,", "The newest chat instructions."),
        (json!({"history":{}}), "revision 2, against revision 1,", "revision 1, against revision 0,"),
    ] {
        let (text, pages) = read_whole(&mut world.driver, &run, completion, input);
        assert!(text.starts_with(heading), "{text}");
        assert!(text.contains(content), "{text}");
        completion += pages;
    }
    let older = read_value(
        &mut world.driver,
        &run,
        completion,
        json!({"setting":{"kind":"charter","number":CHAT},"revision":1}),
    );
    assert_eq!(older, held, "a live run holds its older charter revision");
    let (text, pages) = read_whole(
        &mut world.driver,
        &run,
        completion + 10,
        json!({"setting":{"kind":"charter","number":CONFIGURING}}),
    );
    assert_eq!(pages, 1, "the host allowance holds the stock configuring instructions");
    let (_, value) = text.split_once('\n').expect("charter heading");
    let value: serde_json::Value = serde_json::from_str(value).expect("reassembled charter JSON");
    assert_eq!(value["instructions"], std::str::from_utf8(&run.run.policy.instructions).expect("stock instructions"));
    assert_eq!(world.driver.store.rows, before, "settings reads commit nothing");
}

#[test]
fn propose_settings_refuses_a_stale_revision_and_an_item_out_of_shape() {
    use serde_json::json;
    let mut world = World::stock();
    let run = world.chat(CONFIGURING, world.owner, 20);
    let charter = read_charter(&mut world.driver, &run, 1, CHAT);
    world.change(1, 21, Box::new([owner_charter(b"The owner's instructions.")]));
    let before = world.settings_rows();
    let valid = charter_item(charter.clone(), b"A proposed change.");
    let input = json!({"against":1,"reason":REASON,"items":valid});
    let (body, error) = host_tool(&mut world.driver, &run, 10, b"propose_settings", input.to_string().as_bytes());
    assert!(error);
    assert_eq!(body.as_ref(), b"stale; newest revision 2; read settings show again");
    assert_eq!(world.settings_rows(), before, "a stale call records no proposal or settings change");
    let mut unknown = charter.clone();
    unknown["unknown"] = json!(true);
    let mut missing = charter;
    missing.as_object_mut().expect("charter object").remove("instructions");
    for (completion, value, member) in [(11, unknown, "unknown"), (12, missing, "instructions")] {
        let input = json!({
            "against":2,"reason":REASON,
            "items":[{"setting":{"kind":"charter","number":CHAT},"set":value}]
        });
        let (body, error) =
            host_tool(&mut world.driver, &run, completion, b"propose_settings", input.to_string().as_bytes());
        assert!(error);
        let text = std::str::from_utf8(&body).expect("refusal text");
        assert!(text.contains("item 1"), "{text}");
        assert!(text.contains(member), "{text}");
        assert_eq!(world.settings_rows(), before, "an item out of shape records no proposal or settings change");
    }
    assert_eq!(world.head().number, 2);
}

#[test]
fn a_chat_is_offered_no_settings_tools() {
    let config = config();
    let policy = &config.core.defaults.policies.get(&1).expect("stock policy 1").policy;
    assert_ne!(policy.roles[0].authority.tools.0 & core::ToolFamilies::JIG.settings, 0);
    let mut world = World::new(config);
    let run = world.chat(CHAT, world.owner, 20);
    assert_eq!(run.charter, CHAT);
    let task = world.driver.task(run.task).expect("stock chat task");
    assert_ne!(task.authority.tools.0 & core::ToolFamilies::JIG.settings, 0);
    let offered = temper_engine_smith::charter(&run);
    assert!(!offered.grants.host_tools.is_empty(), "the chat is offered its other host tools");
    for name in [b"settings".as_slice(), b"propose_settings"] {
        assert!(!offered.grants.host_tools.iter().any(|tool| tool.name.as_ref() == name));
    }
    let mut ask = world.driver.chat_ask(PROJECT, b"Read without settings authority");
    let people::Ask::Create { batch, .. } = &mut ask else { panic!("chat creation") };
    batch[0].authority.tools &= !core::ToolFamilies::JIG.settings;
    batch[0].budget = people::Budget::AsAllowed { cap: Some(128) };
    world.ask(world.owner, 21, ask);
    let run = world.driver.last_assignment();
    assert_eq!(run.charter, CHAT);
    assert_eq!(run.run.authority.tools.0 & core::ToolFamilies::JIG.settings, 0);
    for (completion, input) in [
        (1, serde_json::json!({})),
        (2, serde_json::json!({"setting":{"kind":"project-ways","project":PROJECT}})),
        (3, serde_json::json!({"setting":{"kind":"project-questions","project":PROJECT}})),
    ] {
        let (_, error) = host_tool(&mut world.driver, &run, completion, b"settings", input.to_string().as_bytes());
        assert!(error, "the stock cannot bypass the settings family");
        let (_, answer) = world.driver.protocol.answers.last().expect("family denial");
        assert!(matches!(answer, engine::CallAnswer::ToolDenied { .. }));
    }
}

#[test]
fn settings_and_named_documents_use_the_host_answer_allowance() {
    use serde_json::json;
    let mut world = World::stock();
    let run = world.chat(CONFIGURING, world.owner, 20);
    let (inventory, error) = host_tool(&mut world.driver, &run, 1, b"settings", br"{}");
    assert!(!error);
    assert!(inventory.starts_with(b"newest revision 1\n"));
    let input = json!({"setting":{"kind":"policy","number":1}});
    let (whole, pages) = read_whole(&mut world.driver, &run, 2, input.clone());
    assert!(whole.len() > 7000, "stock policy exercises a long JSON value");
    assert_eq!(pages, 1, "the configured host allowance holds the whole stock policy");
    let mut small = limits();
    small.call_answer_bytes = 2048;
    small.authority.segments = 8;
    small.people.people = 4;
    small.people.holdings = 4;
    small.forge.client.answer_bytes = 65_536;
    small.journal.transcript_bytes = 65_536;
    let mut world = World::with_limits(config(), &small);
    configuring_repository(&mut world);
    let run = world.chat(CONFIGURING, world.owner, 20);
    let (first, error) = host_tool(&mut world.driver, &run, 9, b"settings", input.to_string().as_bytes());
    assert!(!error);
    assert!(first.len() > 1000, "a long value fills the page after its heading");
    let (continued, pages) = read_whole(&mut world.driver, &run, 10, input);
    assert!(pages > 1);
    assert_eq!(continued, whole, "continuations reconstruct every original byte");
    configuring_documents(&mut world, &run);
}

fn configuring_repository(world: &mut World) {
    configuring_repository_text(world, &["rules\n".repeat(5334), "words\n".repeat(2167)]);
}

fn configuring_repository_text(world: &mut World, contents: &[String; 2]) {
    use skein_fake_forge_domain::api as raw;
    use temper_engine_domain_world::forge_stand::ForgeStand;
    let paths = ["docs/direction.md", "docs/conventions.md"];
    world.driver.forge = Some(ForgeStand::with_setup(
        [(2, "org/repo"), (3, "org/other")]
            .map(|(id, name)| raw::Setup {
                workflows: Box::new([]),
                id,
                name: name.as_bytes().into(),
                default: b"main".as_slice().into(),
                tree: paths
                    .iter()
                    .zip(contents)
                    .map(|(path, content)| raw::File {
                        path: path.as_bytes().into(),
                        content: content.as_bytes().into(),
                    })
                    .collect(),
                labels: Box::new([]),
                protection: None,
                hooked: false,
                checks: raw::Checks {
                    cues: Box::new([]),
                    contexts: Box::new([]),
                    latency_min: skein_lib::Duration::ZERO,
                    latency_max: skein_lib::Duration::ZERO,
                    silent: 0,
                    passes: 1000,
                    reruns: 0,
                    cue: None,
                },
            })
            .into(),
    ));
    adopt_repository(world, PROJECT, b"repo", 29);
}

fn configuring_documents(world: &mut World, run: &engine::Assignment) {
    use serde_json::json;
    let paths = ["docs/direction.md", "docs/conventions.md"];
    let contents = ["rules\n".repeat(5334), "words\n".repeat(2167)];
    let (branch, error) = host_tool(&mut world.driver, run, 30, b"read_branch", br#"{"branch":"main"}"#);
    assert!(!error, "{}", String::from_utf8_lossy(&branch));
    let head = std::str::from_utf8(&branch).expect("head").trim();
    for (index, path) in paths.iter().enumerate() {
        for (offset, max_bytes) in [(0, None), (1, Some(64))] {
            let mut input = json!({"head":head,"path":path});
            if let Some(max_bytes) = max_bytes {
                input["max_bytes"] = json!(max_bytes);
            }
            let (body, error) = host_tool(
                &mut world.driver,
                run,
                31 + u32::try_from(index).expect("document index") * 2 + offset,
                b"read_file",
                input.to_string().as_bytes(),
            );
            assert!(!error, "{}", String::from_utf8_lossy(&body));
            assert!(body.len() <= 2048 - b"read_file".len());
            let text = std::str::from_utf8(&body).expect("file answer");
            assert!(text.contains(path));
            assert!(text.ends_with("[file truncated]"));
            let (_, prefix) = text.split_once(":\n").expect("file metadata");
            let prefix = prefix.strip_suffix("\n[file truncated]").expect("visible cut");
            let prefix = prefix.split_once("\nread_file rendered size ").map_or(prefix, |(content, _)| content);
            assert!(contents[index].starts_with(prefix) && !prefix.is_empty());
            if max_bytes.is_some() {
                assert_eq!(prefix.len(), 64);
            }
        }
    }
}

fn adopt_repository(world: &mut World, project: u32, name: &[u8], key: u8) {
    use temper_engine_domain_forge as forge;
    world.driver.send(Event::Party(PartyInput::Application {
        scope: people::ApplicationScope::Project(project),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: world.owner,
        key: [key; 16],
        digest: [key; 32],
        carried: None,
        request: engine::Application::Forge(forge::Ask::Adopt(forge::Adoption {
            project,
            home: false,
            forge: 1,
            host: b"forge.example:3847".as_slice().into(),
            owner: b"org".as_slice().into(),
            name: name.into(),
            checks: Box::new([]),
            ci: true,
            landing: b"main".as_slice().into(),
            prefix: b"temper/".as_slice().into(),
            role: forge::Role::Owned,
        })),
    }));
    world.driver.settle();
    let reply = world.driver.delivered.iter().rev().find(|delivery| {
        matches!(delivery,
        Delivery::Application { to, .. } | Delivery::WebReply { to, .. }
            if *to == ReplyTo::new(Token::new(u64::from(key))))
    });
    assert!(
        matches!(
            reply,
            Some(Delivery::Application {
                outcome: Some(engine::ApplicationAnswer::Adopt(forge::ApplicationOutcome::Adopted(_))),
                ..
            })
        ),
        "repository adopted in project {project}: {reply:?}"
    );
}

#[test]
fn a_configuring_chat_proposes_large_settings_from_a_note() {
    use serde_json::json;
    let mut world = note_world();
    let run = world.chat(CONFIGURING, world.owner, 20);
    let settings = reference_settings(&mut world, &run);
    let body = settings.to_string();
    assert!(body.len() > 23_000, "reference-sized settings: {}", body.len());
    let (note, revision) = settings_note(&mut world, &run, &body);
    assert_eq!(world.head().number, 1, "note assembly applies nothing");
    let input = json!({"against":1,"reason":REASON,"note":{"name":note,"revision":revision}});
    let (answer, error) = bounded_call(&mut world, &run, 100, b"propose_settings", &input);
    assert!(!error, "{}", String::from_utf8_lossy(&answer));
    let proposal = std::str::from_utf8(&answer)
        .expect("proposal answer")
        .strip_prefix(&format!("proposed as t{}/", run.task))
        .expect("proposal")
        .split_whitespace()
        .next()
        .expect("proposal number")
        .parse()
        .expect("number");
    let review = world.review(&run, proposal);
    let core::documents::ProposalAction::Settings { review: Some(review), .. } = review.action else {
        panic!("whole settings review")
    };
    assert_eq!(
        usize::try_from(review.items.len()).expect("review length"),
        settings["items"].as_array().expect("items").len()
    );
    assert_eq!(world.head().number, 1, "review applies nothing");
    world.decide(&run, proposal, 21, people::ProposalDecision::Accept, people::Running::Stays);
    assert_eq!(world.head().number, 2, "one acceptance makes one revision");
    assert_submitted_settings(&mut world, &run, &settings);
}

fn assert_submitted_settings(world: &mut World, run: &engine::Assignment, settings: &serde_json::Value) {
    use serde_json::json;
    let (record, _) = read_whole(&mut world.driver, run, 200, json!({"revision":2}));
    assert!(record.contains("charter"));
    let accepted = world
        .settings_rows()
        .into_iter()
        .find_map(|row| {
            if let s::Record::Revision { history: false, revision } = row {
                (revision.head.number == 2).then_some(revision)
            } else {
                None
            }
        })
        .expect("accepted revision");
    assert_eq!(accepted.items.len() + accepted.applications.len(), settings["items"].as_array().expect("items").len());
    for (at, item) in settings["items"].as_array().expect("items").iter().enumerate() {
        let (text, _) = read_whole(
            &mut world.driver,
            run,
            30_000 + u32::try_from(at).expect("item") * 100,
            json!({"setting":item["setting"]}),
        );
        let (_, text) = text.split_once('\n').expect("setting heading");
        let value = serde_json::Deserializer::from_str(text)
            .into_iter::<serde_json::Value>()
            .next()
            .expect("setting value")
            .expect("setting JSON before application status");
        assert_eq!(value, item["set"], "whole value: {}", item["setting"]);
    }
}

fn reference_settings(world: &mut World, run: &engine::Assignment) -> serde_json::Value {
    use serde_json::json;
    let reference: serde_json::Value =
        serde_json::from_str(include_str!("../../../../../docs/examples/reference/settings.json"))
            .expect("reference settings");
    let mut items = Vec::new();
    for (at, item) in reference["items"].as_array().expect("reference items").iter().enumerate() {
        if item["setting"]["kind"] == "charter" {
            let number = item["setting"]["number"].as_u64().expect("charter number");
            let mut value = read_charter(&mut world.driver, run, 1 + u32::try_from(at).expect("item") * 3, 2);
            value["number"] = json!(number);
            value["instructions"] = item["set"]["instructions"].clone();
            // Keep the fixture's model and authority; transport preserves the reference's words.
            items.push(json!({"setting":item["setting"],"set":value}));
        }
    }
    for (at, setting) in
        [json!({"kind":"template","number":2}), json!({"kind":"rules"}), json!({"kind":"policy","number":1})]
            .into_iter()
            .enumerate()
    {
        let mut value = read_value(
            &mut world.driver,
            run,
            5000 + u32::try_from(at).expect("item") * 100,
            json!({"setting":setting}),
        );
        if setting["kind"] == "template" {
            value["llm"] = json!({"account":null,"main":{"model":"scripted","effort":null},"subagents":[]});
        }
        items.push(json!({"setting":setting,"set":value}));
    }
    for item in reference["items"].as_array().expect("reference items") {
        if item["setting"]["kind"] == "landing-rule" {
            items.push(item.clone());
        }
    }
    // One setting exceeds a call even before its JSON framing and escaped quotes.
    items[0]["set"]["instructions"] = json!(format!(
        "{}{}",
        items[0]["set"]["instructions"].as_str().expect("instructions"),
        " Continue carefully.".repeat(200)
    ));
    items[1]["set"]["instructions"] = json!(format!(
        "{}{}",
        items[1]["set"]["instructions"].as_str().expect("instructions"),
        " Review carefully.".repeat(200)
    ));
    json!({"items":items})
}

fn bounded_call(
    world: &mut World,
    run: &engine::Assignment,
    completion: u32,
    tool: &[u8],
    input: &serde_json::Value,
) -> (Box<[u8]>, bool) {
    let input = input.to_string();
    // configuration.md, 13.8: init's output 8,192 leaves about 10,662 payload bytes.
    assert!(input.len() <= 10_662, "call exceeds configured payload ceiling: {}", input.len());
    host_tool(&mut world.driver, run, completion, tool, input.as_bytes())
}

fn settings_note(world: &mut World, run: &engine::Assignment, body: &str) -> (u64, u32) {
    use serde_json::json;
    let mut name = 0;
    let mut revision = 0;
    let chars: Vec<char> = body.chars().collect();
    for (at, chunk) in chars.chunks(2000).enumerate() {
        let mut input = json!({"scope":{"kind":"goal","project":PROJECT,"goal":run.task},"description":"work: settings","body":chunk.iter().collect::<String>()});
        if at > 0 {
            input["name"] = json!(name);
            input["recalled"] = json!(revision);
            input["first"] = json!(2);
            input["remove"] = json!(0);
            input["join"] = json!(true);
        }
        let (answer, error) = bounded_call(world, run, 1000 + u32::try_from(at).expect("chunk"), b"note", &input);
        assert!(!error, "{}", String::from_utf8_lossy(&answer));
        // The store supplies the same durable note identity and revision the write answered.
        let entry = world
            .driver
            .store
            .rows
            .values()
            .find_map(|row| {
                if let engine::Record::Core(core::Record::Notes(jig_domain_notes::Record::Entry(entry))) = row {
                    Some(entry)
                } else {
                    None
                }
            })
            .expect("saved note");
        name = entry.name;
        revision = entry.revision;
    }
    (name, revision)
}

fn note_world() -> World {
    let mut bounds = limits();
    bounds.notes.body_bytes = 65_536;
    bounds.journal.transcript_bytes = 131_072;
    bounds.loads.bytes = 131_072;
    bounds.loads.reply_bytes = 131_072;
    bounds.loads.transcript_bytes = 131_072;
    bounds.tasks.charters = 10;
    bounds.authority.segments = 8;
    bounds.call_records = 64;
    bounds.forge.named_calls = 64;
    World::with_limits(config(), &bounds)
}

#[test]
fn settings_note_sources_refuse_moved_missing_private_and_malformed_values() {
    use serde_json::json;
    for (case, expected) in
        [("moved", "revision moved"), ("missing", "missing"), ("private", "inaccessible"), ("json", "invalid JSON")]
    {
        let mut world = note_world();
        let run = world.chat(CONFIGURING, world.owner, 20);
        let writer = if case == "private" { world.chat(CONFIGURING, world.owner, 21) } else { run.clone() };
        let (note, revision) = settings_note(&mut world, &writer, "{");
        let input = json!({"against":1,"reason":REASON,"note":{
            "name":if case == "missing" {note + 1} else {note},
            "revision":if case == "moved" {revision + 1} else {revision}
        }});
        let (answer, error) = bounded_call(&mut world, &run, 100, b"propose_settings", &input);
        assert!(error, "{case}: {}", String::from_utf8_lossy(&answer));
        assert!(String::from_utf8_lossy(&answer).contains(expected), "{case}: {}", String::from_utf8_lossy(&answer));
        assert_eq!(world.head().number, 1);
        assert!(
            !world.settings_rows().iter().any(|row| matches!(row, s::Record::Proposal { .. })),
            "no partial proposal"
        );
    }
}

#[test]
fn a_decided_note_proposal_replays_after_its_note_moves() {
    use serde_json::json;
    let mut world = note_world();
    let run = world.chat(CONFIGURING, world.owner, 20);
    let value = read_charter(&mut world.driver, &run, 1, CHAT);
    let settings = json!({"items":charter_item(value, b"Keep commits small.")}).to_string();
    let (note, revision) = settings_note(&mut world, &run, &settings);
    let input = json!({"against":1,"reason":REASON,"note":{"name":note,"revision":revision}});
    let (answer, error) = bounded_call(&mut world, &run, 100, b"propose_settings", &input);
    assert!(!error);
    let (changed, error) = bounded_call(
        &mut world,
        &run,
        101,
        b"note",
        &json!({
            "scope":{"kind":"goal","project":PROJECT,"goal":run.task}, "description":"work: settings",
            "name":note,"recalled":revision,"body":"invalid JSON"
        }),
    );
    assert!(!error, "{}", String::from_utf8_lossy(&changed));
    let before = world.driver.store.applied;
    let loads = world.driver.loads.len();
    let body = engine::Call {
        completion: 100,
        position: 0,
        tool: engine::Tool::ProposeSettingsNote { against: 1, note, revision, reason: REASON.as_bytes().into() },
    };
    world.driver.call(CHANNEL, run.task, run.attempt, 900, body);
    world.driver.settle();
    assert_eq!(world.driver.store.applied, before, "replay makes no new proposal");
    assert_eq!(world.driver.loads.len(), loads, "a decided call never rereads its note");
    assert!(world.driver.delivered.iter().any(|delivery| matches!(delivery,
        Delivery::Link { down: jig_domain_host::LinkDown::Relayed { call, answer:jig_domain_host::Relayed::Answer {text,error:false},.. },.. }
        if call.as_ref() == 900_u64.to_be_bytes() && *text == answer)));
}

#[test]
fn a_chats_fresh_proposal_replaces_its_own_that_waits() {
    use serde_json::json;
    let mut bounds = limits();
    bounds.call_records = 4; // Read, two proposals and the refused withdrawal.
    let mut world = World::with_limits(config(), &bounds);
    let run = world.chat(CONFIGURING, world.owner, 20);
    let value = read_charter(&mut world.driver, &run, 1, CHAT);
    let items = charter_item(value.clone(), b"Push each complete change.");
    let first = propose(&mut world.driver, &run, 2, 1, &items);
    let head = world.head().clone();
    let items = charter_item(value, b"Push each complete change. Keep commits small.");
    let fresh = propose(&mut world.driver, &run, 3, 1, &items);
    assert_ne!(fresh, first);
    let (answer, error) =
        host_tool(&mut world.driver, &run, 4, b"withdraw", json!({"proposal":fresh}).to_string().as_bytes());
    assert!(error, "{}", String::from_utf8_lossy(&answer));
    assert!(answer.as_ref() == b"Authority refused: tool family decide", "{}", String::from_utf8_lossy(&answer));
    world.driver.send(Event::Party(PartyInput::Read {
        reply_to: ReplyTo::new(Token::new(90)),
        sign_in: world.owner,
        read: core::reads::Read::Proposal { proposer: people::Proposer::Task(run.task), proposal: first },
    }));
    world.driver.settle();
    assert!(
        world.driver.delivered.iter().any(|delivery| matches!(delivery,
            Delivery::Read { answer: core::reads::Answer::Ended(ended), .. }
            if matches!(&ended.value, core::reads::EndedValue::Proposal(review)
                if review.number == first && review.state == core::documents::ProposalHistoryState::Withdrawn)
        )),
        "the first proposal's review reads withdrawn"
    );
    assert_eq!(world.head(), &head, "replacement changes no settings");
    let review = world.review(&run, fresh);
    let rows = world.settings_rows();
    world.ask(
        world.owner,
        21,
        people::Ask::DecideProposal {
            project: people::DEPLOYMENT,
            proposer: people::Proposer::Task(run.task),
            proposal: first,
            decision: people::ProposalDecision::Accept,
            running: None,
        },
    );
    assert_eq!(world.reply(21), &people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Unknown)));
    assert_eq!(world.head(), &head, "a stale Accept changes no settings");
    assert_eq!(world.settings_rows(), rows);
    assert_eq!(world.review(&run, fresh), review);
    world.decide(&run, fresh, 22, people::ProposalDecision::Accept, people::Running::Stays);
    assert_eq!(world.head().number, 2);
    let next = world.next(&run);
    assert_heard(&next, b"accepted: revision 2 made");
    assert!(
        !next.messages.iter().any(|message| matches!(message.kind,
            tasks::MessageKind::SettingsDecision { proposal, .. } if proposal == first
        )),
        "a replacement sends no withdrawn message"
    );
    assert!(
        next.messages.iter().any(|message| matches!(message.kind,
            tasks::MessageKind::SettingsDecision { proposal, outcome: tasks::SettingsOutcome::Accepted { .. } }
                if proposal == fresh
        )),
        "the acceptance names the fresh proposal"
    );
}

#[test]
fn a_chats_refused_fresh_proposal_leaves_its_own_waiting() {
    use serde_json::json;
    let mut world = World::stock();
    let run = world.chat(CONFIGURING, world.owner, 20);
    let value = read_charter(&mut world.driver, &run, 1, CHAT);
    let items = charter_item(value, b"Push each complete change.");
    let first = propose(&mut world.driver, &run, 2, 1, &items);
    let review = world.review(&run, first);
    let rows = world.settings_rows();
    let input = json!({"against":2,"reason":REASON,"items":items});
    let (answer, error) = host_tool(&mut world.driver, &run, 3, b"propose_settings", input.to_string().as_bytes());
    assert!(error, "{}", String::from_utf8_lossy(&answer));
    assert_eq!(answer.as_ref(), b"stale; newest revision 1; read settings show again");
    assert_eq!(world.review(&run, first), review, "the pending review is unchanged");
    assert_eq!(world.settings_rows(), rows, "a refused replacement changes no settings rows");
    let pending = world.driver.task(run.task).expect("configuring task").proposal.as_deref().expect("still pending");
    assert_eq!(pending.number, first);
    assert!(matches!(pending.state, tasks::ProposalState::Pending { .. }));
}

fn says_no_empty_change(answer: &[u8]) {
    let answer = String::from_utf8_lossy(answer).to_lowercase();
    for forbidden in ["nothing has changed", "nothing changed", "changes nothing"] {
        assert!(!answer.contains(forbidden), "tool or message says {forbidden}: {answer}");
    }
}

#[test]
fn no_answer_a_configuring_run_hears_says_nothing_changed() {
    use serde_json::json;
    let mut world = World::stock();
    let run = world.chat(CONFIGURING, world.owner, 20);
    let (inventory, _) = read_whole(&mut world.driver, &run, 1, json!({}));
    says_no_empty_change(inventory.as_bytes());
    let (charter, _) = read_whole(&mut world.driver, &run, 10, json!({"setting":{"kind":"charter","number":2}}));
    says_no_empty_change(charter.as_bytes());
    let input = json!({"scope":{"kind":"goal","project":PROJECT,"goal":run.task},
        "description":"How this project works","body":"Push each complete change."});
    let (answer, error) = host_tool(&mut world.driver, &run, 20, b"note", input.to_string().as_bytes());
    assert!(!error, "{}", String::from_utf8_lossy(&answer));
    says_no_empty_change(&answer);
    let (_, value) = charter.split_once('\n').expect("charter heading");
    let mut value: serde_json::Value = serde_json::from_str(value.trim()).expect("charter JSON");
    value["instructions"] = json!("Push each complete change.");
    let items = json!([{ "setting":{"kind":"charter","number":2},"set":value }]);
    let input = json!({"against":1,"reason":REASON,"items":items});
    let (answer, error) = host_tool(&mut world.driver, &run, 21, b"propose_settings", input.to_string().as_bytes());
    assert!(!error, "{}", String::from_utf8_lossy(&answer));
    says_no_empty_change(&answer);
    let proposal = std::str::from_utf8(&answer)
        .expect("proposal UTF-8")
        .strip_prefix(&format!("proposed as t{}/", run.task))
        .expect("proposal")
        .split_once(' ')
        .expect("proposal holder")
        .0
        .parse()
        .expect("proposal number");
    world.decide(&run, proposal, 21, people::ProposalDecision::Accept, people::Running::Stays);
    let next = world.next(&run);
    assert_heard(&next, b"accepted: revision 2 made");
    for assignment in [&run, &next] {
        let smith_domain::Event::Start(start) = temper_engine_smith::start(
            assignment,
            None,
            None,
            ReplyTo::new(Token::new(99)),
            Token::new(assignment.task),
            smith_domain::Window { turns: 1, bytes: 100_000 },
            &smith_agent_world::LIMITS.run,
        ) else {
            panic!("smith start")
        };
        for message in start.messages {
            says_no_empty_change(&message.text);
        }
    }
}

#[test]
fn a_member_reads_and_proposes_a_whole_model_inline_or_from_a_note() {
    use serde_json::json;
    for from_note in [false, true] {
        let mut configuration = config();
        configuration.core.defaults.models[0].number = 4;
        configuration.core.defaults.models[0].shown = Some(b"Scripted model".as_slice().into());
        let mut bounds = limits();
        bounds.notes.body_bytes = 65_536;
        bounds.journal.transcript_bytes = 131_072;
        bounds.loads.bytes = 131_072;
        bounds.loads.reply_bytes = 131_072;
        bounds.loads.transcript_bytes = 131_072;
        bounds.tasks.charters = 10;
        bounds.authority.segments = 8;
        bounds.call_records = 64;
        bounds.forge.named_calls = 64;
        let mut world = World::with_limits(configuration, &bounds);
        let member = world.member();
        let run = world.chat(CONFIGURING, member, 20);
        let charter = temper_engine_smith::charter(&run);
        let tool = charter.grants.host_tools.iter().find(|tool| tool.name.as_ref() == b"propose_settings").unwrap();
        let schema: serde_json::Value = serde_json::from_slice(&tool.schema).unwrap();
        assert!(
            schema["properties"]["items"]["items"]["properties"]["setting"]["properties"]["kind"]["enum"]
                .as_array()
                .unwrap()
                .contains(&json!("model"))
        );
        let selector = json!({"kind":"model","number":4});
        let (text, _) = read_whole(&mut world.driver, &run, 1, json!({"setting":selector}));
        assert!(text.starts_with("model 4, revision 1\n"), "{text}");
        let (_, body) = text.split_once('\n').unwrap();
        let before: serde_json::Value = serde_json::from_str(body.trim()).unwrap();
        assert_eq!(before["shown"], "Scripted model");
        let mut after = before.clone();
        after["shown"] = json!("Careful model");
        after["model"]["output_price"] = json!(2);
        let items = json!([{"setting":selector,"set":after}]);
        let input = if from_note {
            let (name, revision) = settings_note(&mut world, &run, &json!({"items":items}).to_string());
            json!({"against":1,"reason":"New prices and shown name","note":{"name":name,"revision":revision}})
        } else {
            json!({"against":1,"reason":"New prices and shown name","items":items})
        };
        let (answer, error) = bounded_call(&mut world, &run, 100, b"propose_settings", &input);
        assert!(!error, "{}", String::from_utf8_lossy(&answer));
        let answer = std::str::from_utf8(&answer).unwrap();
        let proposal = answer
            .strip_prefix(&format!("proposed as t{}/", run.task))
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(world.head().number, 1, "a proposal waits for acceptance");
        world.decide(&run, proposal, 21, people::ProposalDecision::Accept, people::Running::Stays);
        assert_eq!(world.head().number, 2, "one acceptance makes one revision");
        let (text, _) = read_whole(&mut world.driver, &run, 200, json!({"setting":selector}));
        let (_, body) = text.split_once('\n').unwrap();
        assert_eq!(serde_json::from_str::<serde_json::Value>(body.trim()).unwrap(), after);
        let (history, _) = read_whole(&mut world.driver, &run, 300, json!({"revision":2}));
        assert!(history.contains("Careful model") && history.contains("was: default"), "{history}");
        let mut next = after.clone();
        next["shown"] = json!("Model for careful work");
        let proposal = propose(&mut world.driver, &run, 400, 2, &json!([{"setting":selector,"set":next}]));
        world.decide(&run, proposal, 22, people::ProposalDecision::Accept, people::Running::Stays);
        assert_eq!(world.head().number, 3);
        let (history, _) = read_whole(&mut world.driver, &run, 500, json!({"revision":3}));
        assert!(history.contains("Careful model") && history.contains("Model for careful work"), "{history}");
        let revision = world
            .settings_rows()
            .into_iter()
            .find_map(|row| match row {
                s::Record::Revision { history: false, revision } if revision.head.number == 3 => Some(revision),
                _ => None,
            })
            .unwrap();
        let item = revision.items.iter().find(|item| item.setting == s::Setting::Model(4)).unwrap();
        let s::Replaced::Value(s::Value::Model(replaced)) = &item.replaced else { panic!("previous model") };
        assert_eq!(replaced.shown.as_deref(), Some(b"Careful model".as_slice()));
        assert_eq!(replaced.model.output_price, 2);
    }
}
