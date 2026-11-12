//! Scripted Ask temper and settings-family chats through the root and jig's store
//! (temper/docs/design/testing/checks.md, 4; temper/docs/design/domain/agent/ask-temper.md).
mod asking_forge;
pub(crate) mod asking_person;
mod asking_proposal;
mod fixture;
mod remembering;
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

fn box_approval_world(second_owner: bool) -> World {
    use skein_fake_forge_domain::api::Permission;
    let mut world = box_approval_fixture(second_owner);
    let mut forge = temper_engine_domain_world::forge_stand::ForgeStand::with(&[(b"org", b"repo", 2)]);
    for user in [2, 7] {
        forge.grant(b"org/repo", user, Permission::None);
    }
    world.driver.forge = Some(forge);
    adopt_repository(&mut world, PROJECT, b"repo", 28);
    world
}

fn approval_items(
    world: &mut World,
    run: &engine::Assignment,
    completion: u32,
    role: u32,
    people: u32,
) -> serde_json::Value {
    use serde_json::json;
    let setting = json!({"kind":"landing-rule","project":PROJECT,"number":1});
    let (text, _) = see_whole(&mut world.driver, run, completion, "settings landing-rule 1 project");
    let (_, text) = text.split_once('\n').expect("landing rule heading");
    let mut landing = serde_json::Deserializer::from_str(text)
        .into_iter::<serde_json::Value>()
        .next()
        .expect("rule before application status")
        .expect("landing rule JSON");
    landing["approvals"] = json!([{"role":role,"people":people,"freshness":"exact"}]);
    json!([{"setting":setting,"set":landing}])
}

fn refuse_owner_approval(
    world: &mut World,
    run: &engine::Assignment,
    completion: u32,
    against: u32,
    items: &serde_json::Value,
    counts: (u32, u32),
) {
    let before = world.settings_rows();
    let (text, error) = parts_call(world, run, completion, against, items, false);
    assert!(error, "{text}");
    let (required, eligible) = counts;
    assert!(text.starts_with("propose_settings: not proposed, as acceptance would refuse it: "), "{text}");
    assert!(text.contains(&format!(
        "project 1 role 0 on forge forge.example:3847 requires {required} distinct known forge people; currently {eligible} eligible"
    )), "{text}");
    assert_eq!(world.settings_rows(), before, "a refused proposal changes no settings");
}

fn approval_context(world: &mut World, run: &engine::Assignment, requester: &str) {
    let (inventory, _) = see_whole(&mut world.driver, run, 1, "settings");
    assert_eq!(inventory.lines().find(|line| line.starts_with("  person 1 person:")), Some(requester), "{inventory}");
    assert!(
        inventory.contains(
            "forge forge.example:3847 configured owned-forge account ada; project membership unverified by this row"
        ),
        "{inventory}"
    );
    approval_documentation(world, run);
    let words = std::str::from_utf8(&run.run.policy.instructions).expect("stock instructions");
    for rule in [
        "propose their own role: owner (0)",
        "Read the inventory first",
        "needs nothing added",
        "is not a person",
        "the project, not the person, asks for the approval",
        "current permission mapping",
        "maintainer (1), never owner (0)",
        "never promise that an admin",
        "read the inventory again and propose afresh",
    ] {
        assert!(words.contains(rule), "missing stock rule: {rule}");
    }
}

fn approval_documentation(world: &mut World, run: &engine::Assignment) {
    let (documentation, _) = see_whole(&mut world.driver, run, 20, "help landing-rule");
    assert!(documentation.contains("Stock maps admin to maintainer (1)"), "{documentation}");
}

#[test]
fn a_box_person_known_by_the_forges_link_approves_with_nothing_added() {
    use temper_engine_domain_forge as forge;
    const LINKED_REQUESTER: &str = "  person 1 person: role 0, requester, forge forge.example:3847 user 7";
    const UNKNOWN_REQUESTER: &str =
        "  person 1 person: role 0, requester, forge identity unknown: cannot count for approvals";
    for (name, link, second_owner, requester, eligible) in [
        ("owned Forgejo linked", Some(7), false, LINKED_REQUESTER, 1),
        ("two deployment owners after restart", Some(7), true, UNKNOWN_REQUESTER, 0),
        ("no link, then admin adoption", None, false, UNKNOWN_REQUESTER, 0),
    ] {
        eprintln!("box approval world: {name}");
        let mut world = box_approval_world(second_owner);
        if let Some(user) = link {
            world.driver.send(Event::Forge(Box::new(forge::Event::PersonOf {
                forge: 1,
                answer: forge::PersonAnswer::Is(user),
            })));
            world.driver.settle();
        }
        if second_owner {
            restart_box_with_second_owner(&mut world);
        }
        let run =
            world.chat_words(ASK_TEMPLATE, world.owner, 20, b"Keep CI and agent review. Ask who else should approve.");
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
        approval_context(&mut world, &run, requester);
        let items = approval_items(&mut world, &run, 40, 0, 1);
        if eligible == 1 {
            let proposal = propose(&mut world.driver, &run, 41, 1, &items);
            world.decide(&run, proposal, 22, people::ProposalDecision::Accept, people::Running::Stays);
            assert_eq!(world.head().number, 2, "a linked owner needs nothing added on the forge");
            let items = approval_items(&mut world, &run, 50, 0, 2);
            refuse_owner_approval(&mut world, &run, 51, 2, &items, (2, 1));
        } else {
            refuse_owner_approval(&mut world, &run, 41, 1, &items, (1, 0));
            world.ask(
                world.owner,
                23,
                people::Ask::Say {
                    project: PROJECT,
                    task: run.task,
                    words: b"The proposal was refused: role 0 has 0 eligible people.".as_slice().into(),
                },
            );
            if !second_owner {
                adopted_approval(&mut world, &run);
            }
        }
    }
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

fn adopted_approval(world: &mut World, run: &engine::Assignment) {
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
    let (inventory, _) = see_whole(&mut world.driver, &run, 50, "settings");
    assert!(inventory.contains("person 1 person: role 0, requester, forge identity unknown"), "{inventory}");
    assert!(inventory.contains("person 3: role 1, forge forge.example:3847 user 2"), "{inventory}");
    let items = approval_items(world, &run, 51, 0, 1);
    refuse_owner_approval(world, &run, 52, 1, &items, (1, 0));
    let items = approval_items(world, &run, 60, 1, 1);
    let proposal = propose(&mut world.driver, &run, 61, 1, &items);
    world.decide(&run, proposal, 27, people::ProposalDecision::Accept, people::Running::Stays);
    assert_eq!(world.head().number, 2, "fresh maintainer proposal accepted");
}

#[test]
fn an_inventory_names_its_own_projects_people() {
    let (mut world, _) = project_people_world(temper_engine_domain_forge::Mode::Owned, b"box-person");
    let run = world.chat(ASK_TEMPLATE, world.owner, 20);
    let (inventory, _) = see_whole(&mut world.driver, &run, 1, "settings");
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
    for (completion, input) in [(10, "settings charter 1"), (11, "revision 1"), (12, "revisions")] {
        let (text, _) = see_whole(&mut world.driver, &run, completion, input);
        assert!(!text.contains("project people:") && !text.contains("forge people:"), "{text}");
    }
    let (body, error) = host_tool(&mut world.driver, &run, 13, b"see", br#"{"address":"settings charter 999"}"#);
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
    // Four model alternatives and Ask temper's instructions share each saved assignment.
    bounds.journal.run_bytes = 32_768;
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
            list: None,
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
    let mut world = claude_offer_world();
    let run = world.chat(ASK_TEMPLATE, world.owner, 20);
    remove_scripted_account(&mut world);
    let (inventory, _) = see_whole(&mut world.driver, &run, 1, "settings");
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
    let mut world = box_approval_world(false);
    let run = world.chat(ASK_TEMPLATE, world.owner, 20);
    remove_scripted_account(&mut world);
    let (inventory, _) = see_whole(&mut world.driver, &run, 1, "settings");
    assert!(inventory.ends_with("models you can offer: none; no account can run a model yet\n"), "{inventory}");
}

#[test]
fn settings_and_revision_reads_carry_no_model_offer() {
    let mut world = claude_offer_world();
    let run = world.chat(ASK_TEMPLATE, world.owner, 20);
    remove_scripted_account(&mut world);
    for (completion, input) in [(1, "settings charter 11"), (100, "revision 1"), (110, "revisions")] {
        let (text, _) = see_whole(&mut world.driver, &run, completion, input);
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
fn a_members_chat_reads_the_same_people() {
    let (mut world, _) = project_people_world(temper_engine_domain_forge::Mode::Owned, b"box-person");
    let owner_run = world.chat(ASK_TEMPLATE, world.owner, 20);
    let (owner, _) = see_whole(&mut world.driver, &owner_run, 1, "settings");
    let (mut world, member) = project_people_world(temper_engine_domain_forge::Mode::Owned, b"box-person");
    let member_run = world.chat(ASK_TEMPLATE, member, 20);
    let (member, _) = see_whole(&mut world.driver, &member_run, 1, "settings");
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
        let run = world.chat(ASK_TEMPLATE, world.owner, 20);
        let (inventory, _) = see_whole(&mut world.driver, &run, 1, "settings");
        assert!(!inventory.contains("forge people:"), "{mode:?}: {inventory}");
    }
}

#[test]
fn an_asking_chat_discovers_the_forge_host_for_charter_documents() {
    use serde_json::json;
    let mut world = repository_projects();
    configuring_repository_text(&mut world, &["rules\n".into(), "words\n".into()]);
    adopt_repository(&mut world, 2, b"other", 28);
    let run = world.chat(ASK_TEMPLATE, world.owner, 20);
    let (inventory, _) = see_whole(&mut world.driver, &run, 1, "settings");
    let row = inventory
        .lines()
        .find_map(|line| line.strip_prefix("  forge "))
        .expect("settings inventory exposes the canonical forge host and repository");
    let (host, repository) = row.split_once(" repository ").expect("repository identity");
    assert_eq!(host, "forge.example:3847", "the canonical host keeps its port");
    assert_eq!(repository, "org/repo");
    assert!(!inventory.contains("org/other"), "another project's repository stays private");
    let mut value = see_charter(&mut world.driver, &run, 10, CHAT);
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
fn an_owners_asking_chat_proposes_and_hears_it_accepted() {
    use serde_json::json;
    let mut world = World::stock();
    let run = world.chat(ASK_TEMPLATE, world.owner, 20);
    assert_eq!(run.charter, ASK_CHARTER);
    assert_eq!(
        run.run.policy.instructions.as_ref(),
        include_bytes!("../../../../../crates/temper-engine-service/src/stock/ask.txt")
    );
    let offered = temper_engine_smith::charter(&run);
    for name in [b"see".as_slice(), b"document", b"remember", b"propose_settings"] {
        assert!(offered.grants.host_tools.iter().any(|tool| tool.name.as_ref() == name));
    }
    let (inventory, error) = host_tool(&mut world.driver, &run, 1, b"see", br#"{"address":"settings"}"#);
    assert!(!error);
    assert!(inventory.starts_with(b"newest revision 1\n"));
    let value = see_charter(&mut world.driver, &run, 2, CHAT);
    let mut items = charter_item(value, b"Push early and report the result.");
    let line = "Pushes early and reports the result";
    let words = json!({"headline":"Push early", "lede":"Agents push each change early and report its result."});
    items.as_array_mut().expect("proposal items").extend([
        json!({"setting":{"kind":"charter-words","number":CHAT},"set":line}),
        json!({"setting":{"kind":"project-words","project":PROJECT},"set":words}),
    ]);
    let proposal = propose(&mut world.driver, &run, 3, 1, &items);
    let review = world.review(&run, proposal);
    assert_eq!(review.reason.bytes.as_ref(), REASON.as_bytes());
    let core::documents::ProposalAction::Settings { review: Some(settings), .. } = review.action else {
        panic!("settings review")
    };
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
    for (completion, address, expected) in
        [(10, format!("settings charter-words {CHAT}"), json!(line)), (11, "settings project-words".into(), words)]
    {
        let (text, _) = see_whole(&mut world.driver, &run, completion, &address);
        let value = text.lines().nth(1).expect("words read back after acceptance");
        assert_eq!(value, expected.to_string(), "{address}: {text}");
    }
    assert_eq!(
        settings.items.iter().map(|item| item.setting).collect::<Vec<_>>(),
        [
            s::Setting::Charter(CHAT),
            s::Setting::Application { kind: 9, scope: 0, number: CHAT },
            s::Setting::Application { kind: 10, scope: PROJECT, number: 1 },
        ],
        "one proposal holds the charter, its words and the project's words"
    );
    let next = world.next(&run);
    assert_heard(&next, b"accepted: revision 2 made");
}

#[test]
fn a_proposal_selects_the_template_it_installs() {
    use serde_json::json;
    for reverse in [false, true] {
        let mut world = World::stock();
        let run = world.chat(ASK_TEMPLATE, world.owner, 20);
        let mut value = see_value(&mut world.driver, &run, 1, "settings template 5");
        value["number"] = json!(8);
        value["tracked"] = json!(1);
        let ways = json!({"drafts":true,"goals":true,"changes":true,"way":8});
        let mut items = vec![
            json!({"setting":{"kind":"template","number":8},"set":value}),
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
        let (_, error) = host_tool(&mut world.driver, &run, 12, b"see", br#"{"address":"settings template 8"}"#);
        assert!(error, "the proposed template is not published");
        let (text, _) = see_whole(&mut world.driver, &run, 13, "settings project-ways");
        assert!(text.contains("\"way\":0"), "proposing keeps the stock way: {text}");
        world.decide(&run, proposal, 21, people::ProposalDecision::Accept, people::Running::Stays);
        assert_eq!(world.head().number, 2, "one acceptance makes one revision");
        let installed = see_value(&mut world.driver, &run, 30, "settings template 8");
        assert_eq!(installed, value);
        let (text, _) = see_whole(&mut world.driver, &run, 40, "settings project-ways");
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
        .find(|value| value.number == ASK_TEMPLATE)
        .expect("asking template");
    let added = s::Item {
        setting: s::Setting::Template(8),
        edit: s::Edit::Set(s::Value::Template(Box::new(core::Template { number: 8, ..template }))),
    };
    for (name, item, way, accepted) in [
        ("zero", added.clone(), 0, true),
        ("current", added.clone(), ASK_TEMPLATE, true),
        ("installed", added.clone(), 8, true),
        ("absent", added, 9, false),
        (
            "removed",
            s::Item { setting: s::Setting::Template(ASK_TEMPLATE), edit: s::Edit::Removed },
            ASK_TEMPLATE,
            false,
        ),
        ("reset addition", s::Item { setting: s::Setting::Template(8), edit: s::Edit::Reset }, 8, false),
        (
            "reset default",
            s::Item { setting: s::Setting::Template(ASK_TEMPLATE), edit: s::Edit::Reset },
            ASK_TEMPLATE,
            true,
        ),
    ] {
        let mut world = World::stock();
        if name == "reset default" {
            world.change(
                1,
                21,
                Box::new([s::Item { setting: s::Setting::Template(ASK_TEMPLATE), edit: s::Edit::Removed }]),
            );
        }
        let before = world.settings_rows();
        let against = world.head().number;
        let ways =
            kinds::item(7, PROJECT, 1, &Value::Ways(ProjectWays { drafts: true, goals: true, changes: true, way }));
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
    let mut world = World::new(member_config());
    let member = world.member();
    let run = world.chat(ASK_TEMPLATE, member, 20);
    for (completion, address, answer) in [
        (1, "project-ways", format!("project-ways {PROJECT}, revision 1\n{{\"drafts\":true,\"goals\":false,\"changes\":false,\"way\":0}}\nthe stock\n")),
        (2, "project-questions", format!("project-questions {PROJECT}, revision 1\n\"none\"\nthe stock\n")),
        (3, "charter-words 4", "charter-words 4, revision 1\n\"reviews each change at its head, and holds it until it approves\"\ndefault, 67 bytes\n".to_owned()),
        (5, "project-classes", format!("project-classes {PROJECT}, revision 1\n{}\nthe stock\n", r#"{"items":[{"kind":"wording","words":"","class":"decide"},{"kind":"retry","words":"","class":"decide"},{"kind":"tests","words":"","class":"decide"},{"kind":"implementation","words":"","class":"ask"},{"kind":"dependency","words":"","class":"ask"},{"kind":"departure","words":"","class":"ask"}],"sets":[]}"#)),
        (4, "charter-words 7", "charter-words 7, revision 1\ncharter-words 7: none kept; Roles says the charter by its instructions' first sentence.\n".to_owned()),
    ] {
        let (text, _) = see_whole(&mut world.driver, &run, completion, &format!("settings {address}"));
        assert_eq!(text, answer, "{address}");
    }
}

fn owner_words_world() -> (World, engine::Assignment) {
    let mut config = member_config();
    config.kinds_stock = config.kinds_stock.into_vec().into_iter().filter(|row| row.kind != 9).collect();
    let mut world = World::new(config);
    let member = world.member();
    let run = world.chat(ASK_TEMPLATE, member, 20);
    (world, run)
}

fn without_template_goal(mut configuration: engine::Config) -> engine::Config {
    configuration.kinds_stock = configuration.kinds_stock.into_iter().filter(|stock| stock.kind != 11).collect();
    configuration
}

fn owner_template_goal() -> s::Item {
    temper_engine_domain_world::kinds::item(
        11,
        0,
        4,
        &temper_engine_domain_forge::settings::Value::Goal(temper_engine_domain_forge::settings::TemplateGoal {
            design: true,
            close_out: true,
        }),
    )
}

#[test]
fn a_projects_stored_ways_read_as_stored() {
    use serde_json::json;
    for (kind, value) in [
        ("template-goal", json!({"design":true,"close_out":true})),
        ("project-ways", json!({"drafts":true,"goals":true,"changes":true,"way":1})),
        ("project-classes", json!({"items":[{"kind":"implementation","words":"","class":"decide"}],"sets":[]})),
    ] {
        let mut world = World::new(without_template_goal(member_config()));
        let member = world.member();
        let run = world.chat(ASK_TEMPLATE, member, 20);
        let goal = kind == "template-goal";
        let address = if goal { "settings template-goal 4".to_owned() } else { format!("settings {kind}") };
        let (before, _) = see_whole(&mut world.driver, &run, 1, &address);
        if goal {
            assert_eq!(before, "template-goal 4: none kept; its coordinator plans and makes no close-out.\n");
            world.change(1, 21, Box::new([owner_template_goal()]));
        } else if kind == "project-ways" {
            let setting = json!({"kind":kind,"project":PROJECT});
            let items = json!([{"setting":setting,"set":value}]);
            let proposal = propose(&mut world.driver, &run, 2, 1, &items);
            world.decide(&run, proposal, 21, people::ProposalDecision::Accept, people::Running::Stays);
        } else {
            use temper_protocol_reaches::{Class, Classes, Item, Kind, Subject};
            let classes = Classes {
                items: Box::new([Item {
                    subject: Subject::Kind(Kind::Implementation),
                    words: Box::default(),
                    class: Class::Decide,
                }]),
                sets: Box::default(),
            };
            let bytes =
                temper_engine_smith::kinds::encode(&temper_engine_domain_forge::settings::Value::Classes(classes));
            world.change(
                1,
                21,
                Box::new([s::Item {
                    setting: s::Setting::Application { kind: 8, scope: PROJECT, number: 1 },
                    edit: s::Edit::Set(s::Value::Application(bytes)),
                }]),
            );
        }
        assert_eq!(world.head().number, 2);
        let (text, _) = see_whole(&mut world.driver, &run, 3, &address);
        let heading = format!("{kind} {}, revision 2\n", if goal { 4 } else { PROJECT });
        let body = text.strip_prefix(&heading).expect("newest setting heading");
        let (body, state) = body.split_once('\n').expect("value and application state");
        assert_eq!(serde_json::from_str::<serde_json::Value>(body).expect("project JSON"), value);
        assert!(state.starts_with("changed at revision 2, "), "{text}");
        assert!(!text.contains("the stock"), "{text}");
    }
    for (line, headline, lede) in [("Reviews each change", "Our way", "We review each change before landing.")] {
        use temper_engine_domain_forge::settings::{ProjectWords, Value};
        use temper_engine_domain_world::kinds;
        let (mut world, run) = owner_words_world();
        let (text, _) = see_whole(&mut world.driver, &run, 1, "settings project-words");
        assert!(text.contains("project-words: none kept; How work runs says the way's own words."), "{text}");
        let (text, _) = see_whole(&mut world.driver, &run, 4, "settings charter-words 4");
        assert!(
            text.contains("charter-words 4: none kept; Roles says the charter by its instructions' first sentence."),
            "{text}"
        );
        let items = Box::new([
            kinds::item(9, 0, 4, &Value::CharterWords(line.as_bytes().into())),
            kinds::item(
                10,
                PROJECT,
                1,
                &Value::ProjectWords(ProjectWords {
                    headline: headline.as_bytes().into(),
                    lede: lede.as_bytes().into(),
                }),
            ),
        ]);
        world.change(1, 21, items);
        for (completion, address, value) in [
            (2, "settings charter-words 4", json!(line)),
            (3, "settings project-words", json!({"headline":headline,"lede":lede})),
        ] {
            let (text, _) = see_whole(&mut world.driver, &run, completion, address);
            let body = text.lines().nth(1).expect("words value after heading");
            assert_eq!(serde_json::from_str::<serde_json::Value>(body).expect("words JSON"), value);
            assert!(text.contains("revision 2"), "{text}");
            assert!(!text.contains("none kept"), "{text}");
        }
    }
}

#[test]
fn a_read_that_finds_nothing_says_it_was_not_read() {
    use serde_json::json;
    let mut world = World::new(settings_config());
    let run = world.chat(SETTINGS_TEMPLATE, world.owner, 20);
    let asking = world.chat(ASK_TEMPLATE, world.owner, 22);
    world.change(1, 21, Box::new([owner_charter(b"Keep commits small.")]));
    let settings_refusal = "not read: no such setting at that revision. This says nothing about the settings: read the inventory; project-ways, project-questions, project-classes, project-words: project, no number; charter-words, template-goal: number, no project.\nend\n";
    let see_refusal = "see settings charter 999: not read: no such setting. This says nothing about the settings: see settings lists them.";
    for (run, completion, tool, input, expected) in [
        (&run, 1, b"settings".as_slice(), json!({"setting":{"kind":"project-ways","project":999}}), settings_refusal),
        (&run, 2, b"settings", json!({"setting":{"kind":"project-questions","project":999}}), settings_refusal),
        (&run, 3, b"settings", json!({"setting":{"kind":"charter","number":REMOVED}}), settings_refusal),
        (
            &run,
            4,
            b"settings",
            json!({"setting":{"kind":"project-ways","project":PROJECT},"revision":1}),
            settings_refusal,
        ),
        (
            &run,
            5,
            b"settings",
            json!({"setting":{"kind":"project-questions","project":PROJECT},"revision":1}),
            settings_refusal,
        ),
        (&asking, 1, b"see", json!({"address":"settings charter 999"}), see_refusal),
    ] {
        let before = world.driver.store.applied;
        let (body, error) = host_tool(&mut world.driver, run, completion, tool, input.to_string().as_bytes());
        let text = std::str::from_utf8(&body).expect("read refusal UTF-8");
        assert!(error, "{input}: {text}");
        assert_eq!(text, expected);
        assert_eq!(world.driver.store.applied, before, "a not-read answer commits nothing");
    }
}

#[test]
fn a_members_proposal_waits_for_an_owner() {
    let mut world = World::new(member_config());
    let member = world.member();
    let run = world.chat(ASK_TEMPLATE, member, 20);
    let value = see_charter(&mut world.driver, &run, 1, CHAT);
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
    let run = world.chat(ASK_TEMPLATE, world.owner, 20);
    let value = see_charter(&mut world.driver, &run, 1, CHAT);
    let items = charter_item(value, b"Push early.");
    let stale = propose(&mut world.driver, &run, 2, 1, &items);
    world.change(1, 21, Box::new([owner_charter(b"Keep commits small.")]));
    let next = world.next(&run);
    assert_heard(&next, b"withdrawn as stale: the newest is 2");
    let (inventory, error) = host_tool(&mut world.driver, &next, 1, b"see", br#"{"address":"settings"}"#);
    assert!(!error);
    assert!(inventory.starts_with(b"newest revision 2\n"));
    let value = see_charter(&mut world.driver, &next, 2, CHAT);
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
    let run = world.chat(ASK_TEMPLATE, world.owner, 20);
    let value = see_charter(&mut world.driver, &run, 1, CHAT);
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
    let value = see_charter(&mut world.driver, &next, 1, CHAT);
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
    let run = world.chat(ASK_TEMPLATE, world.owner, 22);
    let value = see_charter(&mut world.driver, &run, 1, CHAT);
    let mut items = charter_item(value, b"Push each complete change.");
    let mut rules = see_value(&mut world.driver, &run, 10, "settings rules");
    omit_removable(&mut rules["ceiling"]);
    let mut policy = see_value(&mut world.driver, &run, 30, "settings policy 1");
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
fn the_stock_gives_each_project_rule_one_and_reset_restores_it() {
    use serde_json::json;
    let mut world = World::stock();
    world.ask(world.owner, 19, people::Ask::MakeProject { name: "Other project".into(), policy: 1 });
    let people::Reply::Outcome(people::Outcome::ProjectMade { project: other }) = *world.reply(19) else {
        panic!("second project made: {:?}", world.reply(19))
    };
    assert_ne!(other, PROJECT);
    let run = world.chat(ASK_TEMPLATE, world.owner, 20);
    let stock = json!({
        "pattern":{"segments":["6272616e6368"],"last":{"open":""}},
        "enforce":true,"ci":true,"up_to_date":true,
        "gates":[{"number":1,"blocking":true,"freshness":"clean"}],"approvals":[]
    });
    let mut changed = stock.clone();
    changed["up_to_date"] = json!(false);
    changed["gates"] = json!([]);
    let value = temper_engine_domain_forge::settings::Value::Rule(temper_engine_domain_forge::settings::LandingRule {
        sets: Box::new([]),
        pattern: temper_engine_domain_forge::GatePattern {
            segments: Box::new([b"branch".as_slice().into()]),
            last: temper_engine_domain_forge::GateLast::Open(Box::default()),
        },
        enforce: true,
        ci: true,
        up_to_date: false,
        gates: Box::default(),
        approvals: Box::default(),
    });
    let bytes = temper_engine_smith::kinds::encode(&value);
    let setting = s::Setting::Application { kind: 2, scope: PROJECT, number: 1 };
    let mut completion = 1;
    for (revision, expected) in [(1, &stock), (2, &changed), (3, &stock), (4, &stock)] {
        for (project, expected) in [(PROJECT, expected), (other, &stock)] {
            let (text, _) = settings_whole(
                &mut world.driver,
                &run,
                completion,
                json!({"setting":{"kind":"landing-rule","project":project,"number":1}}),
            );
            let (_, text) = text.split_once('\n').expect("landing rule heading");
            let value = serde_json::Deserializer::from_str(text)
                .into_iter::<serde_json::Value>()
                .next()
                .expect("rule value")
                .expect("rule JSON");
            assert_eq!(&value, expected, "project {project} at revision {revision}");
            completion += 10;
        }
        let (body, error) = host_tool(
            &mut world.driver,
            &run,
            completion,
            b"settings",
            br#"{"setting":{"kind":"landing-rule","number":1}}"#,
        );
        assert!(error, "the deployment has no stock rule: {}", String::from_utf8_lossy(&body));
        assert!(String::from_utf8_lossy(&body).starts_with("not read: no such setting at that revision."));
        completion += 10;
        if revision == 1 {
            world.change(
                1,
                21,
                Box::new([s::Item { setting, edit: s::Edit::Set(s::Value::Application(bytes.clone())) }]),
            );
        } else if revision == 2 {
            world.change(2, 22, Box::new([s::Item { setting, edit: s::Edit::Reset }]));
        } else if revision == 3 {
            // jig rebuilds default metadata at startup; the forge's private
            // every-project rule must still answer after that rebuild.
            restart_stock_world(&mut world, &run);
        }
    }
}

#[test]
fn the_asking_chat_reads_each_kind() {
    let mut world = World::new(without_template_goal(config()));
    let run = world.chat(ASK_TEMPLATE, world.owner, 20);
    world.change(
        1,
        21,
        Box::new([
            owner_charter(b"The newest chat instructions."),
            temper_engine_domain_world::kinds::item(
                9,
                0,
                4,
                &temper_engine_domain_forge::settings::Value::CharterWords(b"Reviews each change".as_slice().into()),
            ),
        ]),
    );
    let before = world.driver.store.rows.clone();
    let mut completion = 10;
    for (input, heading, content) in [
        ("settings", "newest revision 2\n", "held revisions:\n  revision 1,"),
        ("settings charter 1", "charter 1, revision 2\n", "The newest chat instructions."),
        ("settings template 1", "template 1, revision 2\n", "\"bytes\":\"chat\""),
        ("settings policy 1", "policy 1, revision 2\n", "\"roles\":"),
        ("settings rules", "rules, revision 2\n", "\"ceiling\":"),
        ("settings landing-rule 1 project", "landing-rule 1 --project 1, revision 2\n", "\"enforce\":true"),
        ("settings forge-charter 1", "forge-charter 1, revision 2\n", "default, "),
        ("settings project-ways", "project-ways 1, revision 2\n", "the stock"),
        ("settings project-questions", "project-questions 1, revision 2\n", "the stock"),
        ("settings project-classes", "project-classes 1, revision 2\n", "\"class\":\"ask\""),
        ("settings template-goal 4", "template-goal 4: none kept;", "its coordinator plans and makes no close-out."),
        ("revision 2", "revision 2, against revision 1,", "The newest chat instructions."),
        ("revisions", "revision 2, against revision 1,", "revision 1, against revision 0,"),
        ("settings charter-words 4", "charter-words 4, revision 2\n", "Reviews each change"),
        (
            "settings project-words",
            "project-words 1, revision 2\n",
            "project-words: none kept; How work runs says the way's own words.",
        ),
    ] {
        let (text, pages) = see_whole(&mut world.driver, &run, completion, input);
        assert!(text.starts_with(heading), "{text}");
        assert!(text.contains(content), "{text}");
        if input == "settings" {
            let (body, error) =
                host_tool(&mut world.driver, &run, completion + pages, b"see", br#"{"address":"help see"}"#);
            assert!(!error);
            assert!(String::from_utf8_lossy(&body).contains("## settings template-goal N\n"));
            completion += 1;
        }
        completion += pages;
    }
    let (text, pages) = see_whole(&mut world.driver, &run, completion + 10, "settings charter 11");
    assert!(pages > 1, "Ask temper instructions are reassembled within see's part bound");
    let (_, value) = text.split_once('\n').expect("charter heading");
    let value: serde_json::Value = serde_json::from_str(value).expect("reassembled charter JSON");
    assert_eq!(value["instructions"], std::str::from_utf8(&run.run.policy.instructions).expect("stock instructions"));
    assert_eq!(world.driver.store.rows, before, "settings reads commit nothing");
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one stock-world table checks every address, continuation and read authority"
)]
fn an_asking_chat_reads_each_address() {
    use serde_json::json;
    let mut world = World::new(without_template_goal(settings_config()));
    world.change(
        1,
        21,
        Box::new([
            owner_charter(b"The newest chat instructions."),
            temper_engine_domain_world::kinds::item(
                9,
                0,
                4,
                &temper_engine_domain_forge::settings::Value::CharterWords(b"Reviews each change".as_slice().into()),
            ),
        ]),
    );
    let run = world.chat(ASK_TEMPLATE, world.owner, 20);
    assert_eq!(run.charter, ASK_CHARTER);
    assert!(std::str::from_utf8(&run.run.policy.instructions).unwrap().contains("settings [KIND N | rules"));
    let before = world.driver.store.rows.clone();
    let mut completion = 1;
    for (address, heading, content, expected_error) in [
        ("settings", "newest revision 2", "held revisions:", false),
        ("settings charter 1", "charter 1, revision 2", "The newest chat instructions.", false),
        ("settings template 5", "template 5, revision 2", "ask", false),
        ("settings model 1", "model 1, revision 2", "scripted", false),
        ("settings rules", "rules, revision 2", "\"ceiling\":", false),
        ("settings landing-rule 1 project", "landing-rule 1 --project 1, revision 2", "\"enforce\":true", false),
        ("settings forge-charter 1", "forge-charter 1, revision 2", "default, ", false),
        (
            "settings template-goal 4",
            "template-goal 4: none kept;",
            "its coordinator plans and makes no close-out.",
            false,
        ),
        ("settings project-ways", "project-ways 1, revision 2", "the stock", false),
        ("settings project-questions", "project-questions 1, revision 2", "the stock", false),
        ("settings project-classes", "project-classes 1, revision 2", "the stock", false),
        ("revisions", "revision 2, against revision 1,", "revision 1, against revision 0,", false),
        ("revision 2", "revision 2, against revision 1,", "The newest chat instructions.", false),
        ("help", "index: temper", "What temper is", false),
        ("help settings", "settings: Settings", "Settings", false),
        (
            "help no-such-page",
            "no page no-such-page; the sections holding its words:",
            "sections holding its words:",
            false,
        ),
        (
            "help search landing rule",
            "index: The map: - temper help change: a change's way to landing: its pull request, CI,",
            "landing-rule:",
            false,
        ),
        ("ways", "You approve", "You may change this project's settings.", false),
        ("bogus", "see: no address bogus; addresses:", temper_engine_smith::ADDRESSES, true),
        (
            "settings charter 99",
            "see settings charter 99: not read: no such setting.",
            "see settings lists them.",
            true,
        ),
        ("settings charter-words 4", "charter-words 4, revision 2", "Reviews each change", false),
        (
            "settings project-words",
            "project-words 1, revision 2",
            "project-words: none kept; How work runs says the way's own words.",
            false,
        ),
    ] {
        let (body, error) =
            host_tool(&mut world.driver, &run, completion, b"see", json!({"address":address}).to_string().as_bytes());
        let text = std::str::from_utf8(&body).unwrap();
        assert_eq!(error, expected_error, "{address}: {text}");
        assert!(text.starts_with(heading), "{address}: {text}");
        assert!(text.contains(content), "{address}: {text}");
        assert!(body.len() <= usize::try_from(world.driver.env.limits.call_answer_bytes - 3).unwrap());
        if let Some(bound) = world.driver.env.limits.host_reply_bytes {
            assert!(body.len() <= usize::try_from(bound).unwrap());
        }
        if !error {
            assert!(text.lines().last().unwrap().starts_with("see: "), "{address}: {text}");
            let end = text.find("more: ").or_else(|| text.find("see: ")).unwrap();
            assert!(end <= usize::try_from(temper_engine_smith::SEE_PART).unwrap());
        }
        assert_eq!(world.driver.store.rows, before, "{address} commits nothing");
        if address == "settings" {
            assert!(text.contains("charter-words 4: changed in revision 2"), "{text}");
            assert!(text.ends_with("settings project-classes · revisions · help settings\n"), "{text}");
            eprintln!("see settings first part: {} bytes", body.len());
        }
        completion += 1;
    }
    let mut address = "settings policy 1".to_string();
    let mut whole = String::new();
    let mut from = 0_usize;
    loop {
        let (body, error) =
            host_tool(&mut world.driver, &run, completion, b"see", json!({"address":address}).to_string().as_bytes());
        assert!(!error, "{}", String::from_utf8_lossy(&body));
        let text = std::str::from_utf8(&body).unwrap();
        let end = text.find("more: ").or_else(|| text.find("see: ")).unwrap();
        assert!(end <= usize::try_from(temper_engine_smith::SEE_PART).unwrap());
        let next = text.lines().find_map(|line| line.strip_prefix("more: "));
        let next_offset = next.map(|token| token.rsplit_once('@').unwrap().1.parse::<usize>().unwrap());
        let bytes = next_offset.map_or(end, |offset| offset - from);
        whole.push_str(&text[..bytes]);
        from += bytes;
        assert_eq!(world.driver.store.rows, before, "each part commits nothing");
        completion += 1;
        if let Some(token) = next {
            address = format!("more {token}");
        } else {
            break;
        }
    }
    assert!(whole.len() > 7000);
    let (_, value) = whole.split_once('\n').unwrap();
    let actual: serde_json::Value = serde_json::from_str(value.trim()).unwrap();
    let settings = world.chat(SETTINGS_TEMPLATE, world.owner, 22);
    let expected = settings_value(&mut world.driver, &settings, 1, json!({"setting":{"kind":"policy","number":1}}));
    assert_eq!(actual, expected);
    let mut ask = world.driver.chat_ask(PROJECT, b"Read without settings authority");
    let people::Ask::Create { batch, .. } = &mut ask else { panic!("chat") };
    batch[0].executor = people::TaskExecutor::Agent { charter: 11 };
    batch[0].authority.tools &= !core::ToolFamilies::JIG.settings;
    batch[0].budget = people::Budget::AsAllowed { cap: Some(128) };
    world.ask(world.owner, 23, ask);
    let run = world.driver.last_assignment();
    assert_eq!(run.charter, ASK_CHARTER);
    let (body, error) = host_tool(&mut world.driver, &run, 1, b"see", br#"{"address":"settings"}"#);
    assert!(error);
    assert!(body.starts_with(b"see settings: "), "{}", String::from_utf8_lossy(&body));
    assert!(String::from_utf8_lossy(&body).contains("tool family settings"));
}

#[test]
fn propose_settings_refuses_a_stale_revision_and_an_item_out_of_shape() {
    use serde_json::json;
    // A charter's line holds at most 120 bytes (protocol/configuration/settings.md, 2).
    const PAST_LINE_BYTES: usize = 121;
    let mut world = World::stock();
    let run = world.chat(ASK_TEMPLATE, world.owner, 20);
    let charter = see_charter(&mut world.driver, &run, 1, CHAT);
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
    for (completion, items, member) in [
        (11, json!([{"setting":{"kind":"charter","number":CHAT},"set":unknown}]), "unknown"),
        (12, json!([{"setting":{"kind":"charter","number":CHAT},"set":missing}]), "instructions"),
        (
            13,
            json!([{"setting":{"kind":"charter-words","number":CHAT},"set":"x".repeat(PAST_LINE_BYTES)}]),
            "charter-words: wire value bound",
        ),
    ] {
        let input = json!({"against":2,"reason":REASON,"items":items});
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
    let mut world = World::new(settings_config());
    let run = world.chat(SETTINGS_TEMPLATE, world.owner, 20);
    let (inventory, error) = host_tool(&mut world.driver, &run, 1, b"settings", br"{}");
    assert!(!error);
    assert!(inventory.starts_with(b"newest revision 1\n"));
    let policy =
        settings_value(&mut world.driver, &run, 100, json!({"setting":{"kind":"project-policy","project":PROJECT}}));
    assert!(
        policy["permissions"]
            .as_array()
            .expect("permission mapping")
            .iter()
            .any(|row| row == &json!({"connector":1,"permission":3,"role":1})),
        "{policy}"
    );
    for (completion, kind, value) in [
        (101, "project-ways", r#"{"drafts":true,"goals":false,"changes":false,"way":0}"#),
        (102, "project-questions", r#""none""#),
    ] {
        let (text, _) = settings_whole(
            &mut world.driver,
            &run,
            completion,
            json!({"setting":{"kind":kind,"project":PROJECT},"revision":1}),
        );
        assert_eq!(
            text,
            format!("{kind} {PROJECT}, revision 1\n{value}\nthe stock\n"),
            "the named newest revision also takes the stock"
        );
    }
    let input = json!({"setting":{"kind":"policy","number":1}});
    let (whole, pages) = settings_whole(&mut world.driver, &run, 2, input.clone());
    assert!(whole.len() > 7000, "stock policy exercises a long JSON value");
    assert_eq!(pages, 1, "the configured host allowance holds the whole stock policy");
    let held = settings_charter(&mut world.driver, &run, 110, CHAT);
    world.change(1, 21, Box::new([owner_charter(b"The newest chat instructions.")]));
    let older =
        settings_value(&mut world.driver, &run, 120, json!({"setting":{"kind":"charter","number":CHAT},"revision":1}));
    assert_eq!(older, held, "a live run holds its older charter revision");
    let mut small = limits();
    small.call_answer_bytes = 2048;
    small.authority.segments = 8;
    small.people.people = 4;
    small.people.holdings = 4;
    small.forge.client.answer_bytes = 65_536;
    small.journal.transcript_bytes = 65_536;
    let mut world = World::with_limits(settings_config(), &small);
    configuring_repository(&mut world);
    let run = world.chat(SETTINGS_TEMPLATE, world.owner, 20);
    let (first, error) = host_tool(&mut world.driver, &run, 9, b"settings", input.to_string().as_bytes());
    assert!(!error);
    assert!(first.len() > 1000, "a long value fills the page after its heading");
    let (continued, pages) = settings_whole(&mut world.driver, &run, 10, input);
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
    adopt_repository_as(world, project, name, key, temper_engine_domain_forge::Role::Owned);
}

fn adopt_repository_as(world: &mut World, project: u32, name: &[u8], key: u8, role: temper_engine_domain_forge::Role) {
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
            role,
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
fn a_settings_charter_proposes_large_settings_from_a_note() {
    use serde_json::json;
    let mut world = note_world();
    let run = world.chat(SETTINGS_TEMPLATE, world.owner, 20);
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
    let (record, _) = settings_whole(&mut world.driver, run, 200, json!({"revision":2}));
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
        let (text, _) = settings_whole(
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
            let mut value = settings_charter(&mut world.driver, run, 1 + u32::try_from(at).expect("item") * 3, 2);
            value["number"] = json!(number);
            let mut instructions = vec![item["set"]["instructions"].as_str().expect("shared instructions")];
            if let Some(parts) = item["set"].get("parts") {
                for part in parts.as_array().expect("reference parts") {
                    instructions.push(part["instructions"].as_str().expect("part instructions"));
                }
            }
            value["instructions"] = json!(instructions.join("\n\n"));
            // The fixture has no parts; transport preserves every part's words together.
            // Keep the fixture's model and authority.
            items.push(json!({"setting":item["setting"],"set":value}));
        }
    }
    for (at, setting) in
        [json!({"kind":"template","number":2}), json!({"kind":"rules"}), json!({"kind":"policy","number":1})]
            .into_iter()
            .enumerate()
    {
        let mut value = settings_value(
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
    // protocol/configuration/settings.md, 7: init's output 8,192 leaves about 10,662 payload bytes.
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
    bounds.tasks.charters = 13; // The stock, reference settings and the world's settings charter.
    bounds.authority.segments = 8;
    bounds.call_records = 64;
    bounds.forge.named_calls = 64;
    World::with_limits(settings_config(), &bounds)
}

#[test]
fn settings_note_sources_refuse_moved_missing_private_and_malformed_values() {
    use serde_json::json;
    for (case, expected) in
        [("moved", "revision moved"), ("missing", "missing"), ("private", "inaccessible"), ("json", "invalid JSON")]
    {
        let mut world = note_world();
        let run = world.chat(SETTINGS_TEMPLATE, world.owner, 20);
        let writer = if case == "private" { world.chat(SETTINGS_TEMPLATE, world.owner, 21) } else { run.clone() };
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
    let run = world.chat(SETTINGS_TEMPLATE, world.owner, 20);
    let value = settings_charter(&mut world.driver, &run, 1, CHAT);
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
    let run = world.chat(ASK_TEMPLATE, world.owner, 20);
    let value = see_charter(&mut world.driver, &run, 1, CHAT);
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
    let run = world.chat(ASK_TEMPLATE, world.owner, 20);
    let value = see_charter(&mut world.driver, &run, 1, CHAT);
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
fn no_answer_an_asking_run_hears_says_nothing_changed() {
    use serde_json::json;
    let mut world = World::stock();
    let run = world.chat(ASK_TEMPLATE, world.owner, 20);
    let (inventory, _) = see_whole(&mut world.driver, &run, 1, "settings");
    says_no_empty_change(inventory.as_bytes());
    let (charter, _) = see_whole(&mut world.driver, &run, 10, "settings charter 2");
    says_no_empty_change(charter.as_bytes());
    let input = json!({"action":"write","body":"# How this project works\n\nPush each complete change."});
    let (answer, error) = host_tool(&mut world.driver, &run, 20, b"document", input.to_string().as_bytes());
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
        let mut configuration = if from_note { settings_config() } else { config() };
        configuration.core.defaults.models[0].number = 4;
        configuration.core.defaults.models[0].shown = Some(b"Scripted model".as_slice().into());
        let mut bounds = limits();
        // A model edit retains one old version per charter, including the settings charter.
        bounds.settings.versions =
            u32::try_from(configuration.core.defaults.charters.len()).expect("one old version per charter");
        bounds.notes.body_bytes = 65_536;
        bounds.journal.transcript_bytes = 131_072;
        bounds.loads.bytes = 131_072;
        bounds.loads.reply_bytes = 131_072;
        bounds.loads.transcript_bytes = 131_072;
        bounds.tasks.charters = 11;
        bounds.authority.segments = 8;
        bounds.call_records = 64;
        bounds.forge.named_calls = 64;
        let mut world = World::with_limits(configuration, &bounds);
        let member = world.member();
        let run = world.chat(if from_note { SETTINGS_TEMPLATE } else { ASK_TEMPLATE }, member, 20);
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
        let (text, _) = model_read(&mut world, &run, 1, None);
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
        let (text, _) = model_read(&mut world, &run, 200, None);
        let (_, body) = text.split_once('\n').unwrap();
        assert_eq!(serde_json::from_str::<serde_json::Value>(body.trim()).unwrap(), after);
        let (history, _) = model_read(&mut world, &run, 300, Some(2));
        assert!(history.contains("Careful model") && history.contains("was: default"), "{history}");
        let mut next = after.clone();
        next["shown"] = json!("Model for careful work");
        let proposal = propose(&mut world.driver, &run, 400, 2, &json!([{"setting":selector,"set":next}]));
        world.decide(&run, proposal, 22, people::ProposalDecision::Accept, people::Running::Stays);
        assert_eq!(world.head().number, 3);
        let (history, _) = model_read(&mut world, &run, 500, Some(3));
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

fn model_read(world: &mut World, run: &engine::Assignment, completion: u32, revision: Option<u32>) -> (String, u32) {
    if run.charter == SETTINGS_CHARTER {
        let input = revision.map_or_else(
            || serde_json::json!({"setting":{"kind":"model","number":4}}),
            |number| serde_json::json!({"revision":number}),
        );
        settings_whole(&mut world.driver, run, completion, input)
    } else {
        let address = revision.map_or_else(|| "settings model 4".into(), |number| format!("revision {number}"));
        see_whole(&mut world.driver, run, completion, &address)
    }
}

#[test]
fn an_asking_chat_names_a_repository_as_the_inventory_does() {
    use serde_json::json;
    let mut world = repository_projects();
    configuring_repository_text(&mut world, &["rules\n".into(), "words\n".into()]);
    adopt_repository(&mut world, 2, b"other", 28);
    let run = world.chat(ASK_TEMPLATE, world.owner, 20);
    let mut value = see_charter(&mut world.driver, &run, 10, CHAT);
    value["documents"] = json!([{"repository":"org/repo","branch":"main","path":"docs/direction.md"}]);
    let items = json!([{"setting":{"kind":"charter","number":CHAT},"set":value}]);
    let proposal = propose(&mut world.driver, &run, 20, 1, &items);
    world.decide(&run, proposal, 21, people::ProposalDecision::Accept, people::Running::Stays);
    let value = see_charter(&mut world.driver, &run, 30, CHAT);
    assert_eq!(value["documents"][0]["forge"], "forge.example:3847");
    let mut value = value;
    value["documents"][0].as_object_mut().unwrap().remove("forge");
    value["documents"][0]["repository"] = json!("org/other");
    let before = world.driver.store.rows.clone();
    let input = json!({"against":2,"reason":REASON,"items":[{"setting":{"kind":"charter","number":CHAT},"set":value}]});
    let (body, error) = host_tool(&mut world.driver, &run, 40, b"propose_settings", input.to_string().as_bytes());
    assert!(error);
    assert_eq!(
        String::from_utf8(body.into()).unwrap(),
        "settings file item 1: charter.documents[0].repository: no repository org/other in this project; its repositories: org/repo"
    );
    assert_eq!(world.driver.store.rows, before);
}

#[test]
#[expect(clippy::too_many_lines, reason = "one run checks the ordered parts lifecycle and both restart guards")]
fn an_asking_chat_proposes_a_large_set_in_parts() {
    use serde_json::json;
    let mut bounds = limits();
    bounds.call_records = 8; // Eight proposing calls across two runs, including jig's refusal.
    let mut world = World::with_limits(settings_config(), &bounds);
    let run = world.chat(ASK_TEMPLATE, world.owner, 20);
    let changes: Vec<_> = [1, 2, 4]
        .into_iter()
        .map(|number| {
            let mut value = see_charter(&mut world.driver, &run, 100 * number, number);
            value["instructions"] = json!(format!("Changed charter {number}."));
            json!({"setting":{"kind":"charter","number":number},"set":value})
        })
        .collect();
    let before = world.driver.store.rows.clone();
    for (completion, item, more, expected, error) in [
        (
            1,
            1,
            true,
            "propose_settings: holding 1 item for revision 1, 1 in all; nothing is proposed until a call without more.",
            false,
        ),
        (2, 1, true, "propose_settings: charter 1 is held already, from an earlier part; nothing was added.", true),
        (6, 1, false, "propose_settings: charter 1 is held already, from an earlier part; nothing was added.", true),
        (
            3,
            2,
            true,
            "propose_settings: holding 1 item for revision 1, 2 in all; nothing is proposed until a call without more.",
            false,
        ),
    ] {
        let input =
            json!({"against":1,"reason":REASON,"items":[changes[usize::try_from(item - 1).unwrap()]],"more":more});
        let (body, failed) =
            host_tool(&mut world.driver, &run, completion, b"propose_settings", input.to_string().as_bytes());
        assert_eq!(String::from_utf8(body.into()).unwrap(), expected);
        assert_eq!(failed, error);
        assert_eq!(world.driver.store.rows, before, "parts and their refusals keep no records");
    }
    let invalid = json!([{"setting":{"kind":"charter","number":4},"set":{}}]);
    assert_eq!(
        parts_call(&mut world, &run, 7, 1, &invalid, false),
        ("settings file item 1: resume: missing member".into(), true)
    );
    assert_eq!(world.driver.store.rows, before, "a decode refusal leaves parts waiting");
    let own = json!([changes[2]]);
    let proposal = propose(&mut world.driver, &run, 4, 1, &own);
    let review = world.review(&run, proposal);
    let core::documents::ProposalAction::Settings { items, .. } = review.action else { panic!("settings review") };
    assert_eq!(
        items.iter().map(|item| item.setting).collect::<Vec<_>>(),
        [s::Setting::Charter(1), s::Setting::Charter(2), s::Setting::Charter(4)]
    );
    let proposal = propose(&mut world.driver, &run, 5, 1, &own);
    let core::documents::ProposalAction::Settings { items, .. } = world.review(&run, proposal).action else {
        panic!("settings review")
    };
    assert_eq!(items.len(), 1, "a proposing call consumes its parts");
    let first = json!([changes[0]]);
    let second = json!([changes[1]]);
    assert_eq!(
        parts_call(&mut world, &run, 20, 1, &first, true),
        (
            "propose_settings: holding 1 item for revision 1, 1 in all; nothing is proposed until a call without more."
                .into(),
            false
        )
    );
    world.change(1, 21, Box::new([owner_charter(b"An owner's revision.")]));
    assert_eq!(parts_call(&mut world, &run, 21, 2, &second, true), ("propose_settings: holding 1 item for revision 2, 1 in all; the 1 held for revision 1 were dropped. Nothing is proposed until a call without more.".into(), false));
    assert_eq!(parts_call(&mut world, &run, 22, 1, &own, false), ("propose_settings: the 1 item held are for revision 2, and this call names 1; nothing was proposed, and they were dropped. Give the whole set again against 1.".into(), true));
    let proposal = propose(&mut world.driver, &run, 23, 2, &own);
    assert_review_settings(&mut world, &run, proposal, &[s::Setting::Charter(4)]);
    assert!(!parts_call(&mut world, &run, 24, 2, &first, true).1);
    let next = world.next(&run);
    let proposal = propose(&mut world.driver, &next, 1, 2, &own);
    assert_review_settings(&mut world, &next, proposal, &[s::Setting::Charter(4)]);
    assert!(!parts_call(&mut world, &next, 2, 2, &first, true).1);
    restart_settings_world(&mut world, &next);
    assert_eq!(parts_call(&mut world, &next, 3, 2, &own, false), ("propose_settings: temper restarted during this run, so parts it gave may be lost; nothing was proposed. Give the whole set again, in parts if it is large.".into(), true));
    let proposal = propose(&mut world.driver, &next, 4, 2, &own);
    assert_review_settings(&mut world, &next, proposal, &[s::Setting::Charter(4)]);
    restart_settings_world(&mut world, &next);
    assert_eq!(
        parts_call(&mut world, &next, 5, 2, &first, true),
        (
            "propose_settings: holding 1 item for revision 2, 1 in all; nothing is proposed until a call without more."
                .into(),
            false
        )
    );
    let proposal = propose(&mut world.driver, &next, 6, 2, &second);
    assert_review_settings(&mut world, &next, proposal, &[s::Setting::Charter(1), s::Setting::Charter(2)]);
    let other = world.chat(SETTINGS_TEMPLATE, world.owner, 22);
    assert_eq!(
        parts_call(&mut world, &other, 1, 2, &first, true),
        (
            "propose_settings: more is not offered to this charter; give the whole set in one call, or from a note."
                .into(),
            true
        )
    );

    // Application items share the same total bound, even though jig counts its own separately.
    let most = s::capacity(&engine::core_limits(&limits())).unwrap() * 2 + 1;
    let application: Vec<_> =
        (1..=most).map(|number| json!({"setting":{"kind":"landing-rule","number":number},"edit":"reset"})).collect();
    for (index, chunk) in application.chunks(16).enumerate() {
        let (text, error) = parts_call(&mut world, &next, 10 + u32::try_from(index).unwrap(), 2, &json!(chunk), true);
        assert!(!error, "{text}");
    }
    assert_eq!(
        parts_call(&mut world, &next, 40, 2, &own, false),
        (
            format!(
                "propose_settings: a proposal holds at most {most} items, and this call would make {}; nothing was added.",
                most + 1
            ),
            true
        )
    );
    assert_eq!(
        parts_call(&mut world, &next, 41, 2, &json!([]), true),
        (
            format!(
                "propose_settings: holding 0 items for revision 2, {most} in all; nothing is proposed until a call without more."
            ),
            false
        )
    );
    let (_, error) = parts_call(&mut world, &next, 42, 2, &json!([]), false);
    assert!(error, "jig refuses this set and consumes the parts");
    let proposal = propose(&mut world.driver, &next, 43, 2, &own);
    assert_review_settings(&mut world, &next, proposal, &[s::Setting::Charter(4)]);
}

#[test]
fn an_asking_chat_is_refused_an_unsatisfiable_approval_at_the_call() {
    use temper_engine_domain_forge as forge;
    let mut world = box_approval_world(false);
    world.driver.send(Event::Forge(Box::new(forge::Event::PersonOf { forge: 1, answer: forge::PersonAnswer::Is(7) })));
    world.driver.settle();
    let run = world.chat(ASK_TEMPLATE, world.owner, 20);
    let items = approval_items(&mut world, &run, 40, 0, 2);
    let input = serde_json::json!({"against":1,"reason":REASON,"items":items});
    let before = world.driver.store.rows.clone();
    let (body, error) = host_tool(&mut world.driver, &run, 41, b"propose_settings", input.to_string().as_bytes());
    assert!(error, "{}", String::from_utf8_lossy(&body));
    let text = std::str::from_utf8(&body).unwrap();
    assert_eq!(
        text,
        "propose_settings: not proposed, as acceptance would refuse it: project 1 role 0 on forge forge.example:3847 requires 2 distinct known forge people; currently 1 eligible. Before accepting, add the person as a collaborator on the forge, give that role the permission in this project's policy if needed and adopt the repository again; then propose afresh against the newest revision, or remove the approval requirement."
    );
    assert_eq!(world.driver.store.rows, before, "no proposal or call record");
    let refused = text.to_owned();
    assert_eq!(
        parts_call(&mut world, &run, 42, 1, &items, true),
        (
            "propose_settings: holding 1 item for revision 1, 1 in all; nothing is proposed until a call without more."
                .into(),
            false
        )
    );
    let value = see_charter(&mut world.driver, &run, 100, CHAT);
    let own = charter_item(value, b"A call's own charter item.");
    assert_eq!(parts_call(&mut world, &run, 43, 1, &own, false), (refused, true), "held landing rules are judged too");
    assert_eq!(world.driver.store.rows, before);
    let items = approval_items(&mut world, &run, 200, 0, 1);
    propose(&mut world.driver, &run, 201, 1, &items);

    // Project 2 has no members of role 2; only project 1's eligibility reaches this chat.
    let mut world = repository_projects();
    configuring_repository_text(&mut world, &["rules\n".into(), "words\n".into()]);
    adopt_repository(&mut world, 2, b"other", 28);
    world.member();
    world.ask(
        world.owner,
        22,
        people::Ask::SetRoles {
            project: 2,
            holdings: Box::new([people::Holding { person: OWNER_PERSON, role: people::Role::Owner }]),
        },
    );
    assert_eq!(world.reply(22), &people::Reply::Outcome(people::Outcome::RolesSet { project: 2 }));
    let run = world.chat(ASK_TEMPLATE, world.owner, 20);
    let mut items = approval_items(&mut world, &run, 40, 2, 1);
    // A deployment rule also reaches project 2, whose people this chat cannot read.
    items[0]["setting"] = serde_json::json!({"kind":"landing-rule","number":1});
    items[0]["set"]["pattern"]["segments"] = serde_json::json!(["666f726765"]);
    let proposal = propose(&mut world.driver, &run, 41, 1, &items);
    world.ask(
        world.owner,
        21,
        people::Ask::DecideProposal {
            project: people::DEPLOYMENT,
            proposer: people::Proposer::Task(run.task),
            proposal,
            decision: people::ProposalDecision::Accept,
            running: Some(people::Running::Stays),
        },
    );
    let people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Judged { why, .. })) = world.reply(21) else {
        panic!("project 2 is checked only at acceptance: {:?}", world.reply(21))
    };
    let why = std::str::from_utf8(why).unwrap();
    assert!(
        why.contains(
            "project 2 role 2 on forge forge.example:3847 requires 1 distinct known forge people; currently 0 eligible"
        ),
        "{why}"
    );

    let (mut world, _) = project_people_world(forge::Mode::Fake, b"");
    let run = world.chat(ASK_TEMPLATE, world.owner, 20);
    let items = approval_items(&mut world, &run, 40, 0, 1);
    assert_eq!(parts_call(&mut world, &run, 41, 1, &items, false), ("propose_settings: not proposed, as acceptance would refuse it: project 1: the playground's forge has no accounts, so no one can approve there. Name a branch pattern or a project that leaves its repository out, or remove the approval requirement.".into(), true));
}

fn assert_review_settings(world: &mut World, run: &engine::Assignment, proposal: u64, expected: &[s::Setting]) {
    let core::documents::ProposalAction::Settings { items, .. } = world.review(run, proposal).action else {
        panic!("settings review")
    };
    assert_eq!(items.iter().map(|item| item.setting).collect::<Vec<_>>(), expected);
}

#[test]
fn an_owners_follow_proposal_is_accepted_kept_and_read_after_restart() {
    use serde_json::json;
    use temper_engine_domain_forge as forge;
    let mut world = World::stock();
    world.driver.forge = Some(temper_engine_domain_world::forge_stand::ForgeStand::with(&[
        (b"org", b"source", 12),
        (b"org", b"target", 13),
    ]));
    adopt_repository_as(&mut world, PROJECT, b"source", 28, forge::Role::Context);
    adopt_repository(&mut world, PROJECT, b"target", 29);
    let run = world.chat(ASK_TEMPLATE, world.owner, 20);
    let follow = json!({"source":{"forge":1,"repository":12},"target":{"forge":1,"repository":13},"owner":{"person":OWNER_PERSON},"check":{"workflow":".forgejo/workflows/vsr-tests.yml","artifact":"results","format":"junit"}});
    let items = json!([{"setting":{"kind":"follow","project":PROJECT,"number":1},"set":follow}]);
    let proposal = propose(&mut world.driver, &run, 1, 1, &items);
    assert_eq!(world.head().number, 1, "a proposal changes no setting");
    let review = world.review(&run, proposal);
    let core::documents::ProposalAction::Settings { review: Some(settings), .. } = review.action else {
        panic!("follow review")
    };
    assert_eq!(settings.items.len(), 1);
    assert_eq!(settings.items.get(0).unwrap().setting, s::Setting::Application { kind: 12, scope: PROJECT, number: 1 });
    world.decide(&run, proposal, 21, people::ProposalDecision::Accept, people::Running::Stays);
    assert_eq!(world.head().number, 2);
    let mut expected = follow;
    expected["source"]["branch"] = json!("main");
    expected["source"]["paths"] = json!([]);
    expected["target"]["branch"] = json!("main");
    expected["start"] = json!("owner");
    let (text, _) = see_whole(&mut world.driver, &run, 2, "settings follow 1");
    assert!(text.contains("vsr-tests.yml"), "accepted follow kept and readable: {text}");
    let read: serde_json::Value = serde_json::from_str(text.lines().nth(1).expect("follow JSON")).unwrap();
    assert_eq!(read, expected);
    let next = world.next(&run);
    assert_heard(&next, b"accepted: revision 2 made");
    restart_stock_world(&mut world, &next);
    assert_eq!(world.head().number, 2);
    let (text, _) = see_whole(&mut world.driver, &next, 10, "settings follow 1");
    let read: serde_json::Value = serde_json::from_str(text.lines().nth(1).expect("restored follow JSON")).unwrap();
    assert_eq!(read, expected);
}

#[test]
fn an_owners_dependency_proposal_is_accepted_kept_and_read_after_restart_and_target_removal() {
    use serde_json::json;
    let mut world = repository_projects();
    let run = world.chat(ASK_TEMPLATE, world.owner, 20);
    let dependency = json!({"project":2});
    let items = json!([{"setting":{"kind":"dependency","project":PROJECT,"number":1},"set":dependency}]);
    let proposal = propose(&mut world.driver, &run, 1, 1, &items);
    assert_eq!(world.head().number, 1);
    let review = world.review(&run, proposal);
    let core::documents::ProposalAction::Settings { review: Some(settings), .. } = review.action else {
        panic!("dependency review")
    };
    assert_eq!(settings.items.len(), 1);
    assert_eq!(settings.items.get(0).unwrap().setting, s::Setting::Application { kind: 13, scope: PROJECT, number: 1 });
    world.decide(&run, proposal, 21, people::ProposalDecision::Accept, people::Running::Stays);
    assert_eq!(world.head().number, 2);
    let (text, _) = see_whole(&mut world.driver, &run, 2, "settings dependency 1");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(text.lines().nth(1).expect("dependency JSON")).unwrap(),
        dependency
    );
    let next = world.next(&run);
    assert_heard(&next, b"accepted: revision 2 made");
    restart_stock_world(&mut world, &next);
    let (text, _) = see_whole(&mut world.driver, &next, 10, "settings dependency 1");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(text.lines().nth(1).expect("restored dependency JSON")).unwrap(),
        dependency
    );
    world.ask(world.owner, 22, people::Ask::RemoveProject { project: 2, grace: None });
    assert_eq!(world.reply(22), &people::Reply::Outcome(people::Outcome::ProjectRemoved { project: 2 }));
    // Deletion may still be draining; the dependency is retained throughout it and a restart.
    let (text, error) = host_tool(&mut world.driver, &next, 11, b"see", br#"{"address":"settings dependency 1"}"#);
    assert!(!error);
    let text = std::str::from_utf8(&text).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(text.lines().nth(1).expect("removed target dependency JSON"))
            .unwrap(),
        dependency
    );
    restart_stock_world(&mut world, &next);
    let (text, error) = host_tool(&mut world.driver, &next, 12, b"see", br#"{"address":"settings dependency 1"}"#);
    assert!(!error);
    let text = std::str::from_utf8(&text).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(text.lines().nth(1).expect("removed target after restart JSON"))
            .unwrap(),
        dependency
    );
}

#[test]
fn a_dependency_on_itself_or_a_missing_project_refuses_the_whole_proposal() {
    use serde_json::json;
    for (number, project, expected) in [
        (1, PROJECT, "dependency 1: project 1 cannot depend on itself"),
        (1, 99, "dependency 1: project 99 is not a project of this deployment"),
        (1, 0, "dependency 1: a value of another kind"),
        (5, 2, "dependency 5: a number this kind does not have"),
    ] {
        let mut world = repository_projects();
        let run = world.chat(ASK_TEMPLATE, world.owner, 20);
        let items = json!([
            {"setting":{"kind":"dependency","project":PROJECT,"number":2},"set":{"project":2}},
            {"setting":{"kind":"dependency","project":PROJECT,"number":number},"set":{"project":project}}
        ]);
        let proposal = propose(&mut world.driver, &run, 1, 1, &items);
        world.ask(
            world.owner,
            21,
            people::Ask::DecideProposal {
                project: people::DEPLOYMENT,
                proposer: people::Proposer::Task(run.task),
                proposal,
                decision: people::ProposalDecision::Accept,
                running: Some(people::Running::Stays),
            },
        );
        let people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Judged { why, .. })) = world.reply(21)
        else {
            panic!("refused dependency {expected}: {:?}", world.reply(21))
        };
        assert_eq!(why.as_ref(), expected.as_bytes());
        assert_eq!(world.head().number, 1);
        assert!(world.driver.store.rows.values().all(|record| !matches!(record, engine::Record::Forge { row } if matches!(row.as_ref(), temper_engine_domain_forge::Stored::Setting(row) if row.key.kind == 13 && row.key.slot == temper_engine_domain_forge::settings::SettingSlot::Change))), "no part of a refused proposal becomes current");
    }
}
