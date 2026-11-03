//! Settings files through the joined roots (temper/docs/design/client/tools.md, 5.3 and 11.2).
use jig_client::contract as c;
use jig_client_world::harness::{ClientId, Settings, World};
use jig_domain_core::settings;
use jig_domain_people as people;
use temper_client_world::{app::Temper, fixtures, front::Joint};
use temper_engine_domain as engine;
use temper_engine_domain_world::{direct::Driver, store::Store, walking};

type ClientWorld = World<Temper, Joint>;
const PROJECT: u32 = 1;
const POLICY: u32 = 1;
const CHARTER: u32 = 1;
const CHANGED_SPEND: u64 = 900;
// Whole policy and charter values fit a review's per-value allowance.
const WHOLE_SETTING_BYTES: u32 = 4096;
const INSTRUCTIONS: &[u8] = b"Follow the project's new policy.";

fn configuration() -> engine::Config {
    let mut config = walking::config(fixtures::SEED);
    config.core.owners = Box::new([
        config.core.owners[0].clone(),
        people::InitialOwner { project: 0, identity: config.core.owners[0].identity.clone() },
    ]);
    // This story's one configured role owns each seeded forge permission.
    for permission in &mut config.core.seed_projects[0].policy.permissions {
        permission.role = people::Role::Owner.number();
    }
    config.core.defaults.policies.insert(POLICY, config.core.seed_projects[0].policy.clone()).expect("policy 1");
    config
}

fn world() -> ClientWorld {
    let mut front = Joint::new(fixtures::SEED, fixtures::domain_limits());
    let mut limits = front.engine.env.limits;
    limits.people.initial_owners = 2;
    limits.documents.text_bytes = WHOLE_SETTING_BYTES;
    let forge = front.engine.forge.take();
    front.engine = Driver::configured(Store::with_limits(&limits), [configuration(), configuration()], &limits);
    front.engine.forge = forge;
    front.engine.settle();
    let mut world = World::new(
        Settings { seed: fixtures::SEED, words: fixtures::decision_words(), iterations: 4096, ..Settings::default() },
        front,
    );
    world.randomness(2, 0, 0);
    world
}

fn call(world: &mut ClientWorld, agent: ClientId, words: &[&[u8]]) -> String {
    let number = fixtures::give(world, agent, words);
    let printed = fixtures::answered(world, agent, number);
    assert_eq!(printed.status, 0, "{printed:?}");
    String::from_utf8(printed.stdout.into_vec()).expect("ctl UTF-8")
}

fn client_value(value: &settings::Value) -> c::SettingValue {
    let limits = temper_client_tools::limits::tools();
    let wire = jig_protocol_engine_clients::translate::settings::value(value, &[], &limits.settings)
        .expect("fixture's whole value on wire");
    let client = temper_client_tools::limits::client().client;
    let receiving = jig_client_protocol_link::Limits::new(client.domain, client.exchanges).expect("tools link bounds");
    jig_client_protocol_link::translate::settings::restore_setting_value(&wire, &receiving)
        .expect("fixture's whole client value")
}

fn value_json(setting: c::Setting, value: c::SettingValue) -> String {
    let text = temper_client_tools::jig::render::settings_text(
        &c::ReadValue::Setting(Box::new(c::SettingDocument {
            setting,
            revision: 1,
            value: Some(value),
            application: None,
            proposed: None,
        })),
        None,
        &temper_client_tools::limits::tools(),
    );
    let text = String::from_utf8(text.into_vec()).expect("setting UTF-8");
    text.split_once('\n').expect("setting header and whole JSON").1.trim_end().to_owned()
}

#[test]
fn a_settings_file_sets_a_project_policy_through_ctl_proposed_shown_and_accepted() {
    let mut world = world();
    let agent = fixtures::agent(&mut world);
    let shown = call(&mut world, agent, &[b"settings", b"show", b"policy", b"1", b"--json"]);
    let config = configuration();
    let c::SettingValue::Policy(mut policy) =
        client_value(&settings::Value::Policy(Box::new(config.core.seed_projects[0].policy.clone())))
    else {
        panic!("fixture policy")
    };
    let selector = c::Setting::Policy(c::SettingNumber { number: POLICY });
    let now = value_json(selector, c::SettingValue::Policy(policy.clone()));
    assert!(shown.contains(&format!("\"value\":{now}")), "{shown}");
    policy.period_spend = CHANGED_SPEND;
    let would_be = value_json(selector, c::SettingValue::Policy(policy));
    let mut charter = config.core.defaults.charters[0].clone();
    charter.policy.instructions = INSTRUCTIONS.into();
    let charter = value_json(
        c::Setting::Charter(c::SettingNumber { number: CHARTER }),
        client_value(&settings::Value::Charter(Box::new(charter))),
    );
    let policy_item = format!(r#"{{"setting":{{"kind":"project-policy","project":{PROJECT}}},"set":{would_be}}}"#);
    let charter_item = format!(r#"{{"setting":{{"kind":"charter","number":{CHARTER}}},"set":{charter}}}"#);
    let file = format!(r#"{{"items":[{charter_item},{policy_item}]}}"#);
    let before = call(&mut world, agent, &[b"settings", b"show"]);
    assert!(before.starts_with("newest revision 1\n"), "{before}");
    let number = fixtures::give_with_file(
        &mut world,
        agent,
        &[
            b"settings",
            b"propose",
            b"settings.json",
            b"Set up the project and its charter",
            b"--against",
            b"1",
            b"--key",
            b"project-policy",
        ],
        file.as_bytes(),
    );
    let proposed = fixtures::answered(&mut world, agent, number);
    assert_eq!(proposed.status, 0, "{proposed:?}");
    let proposed = String::from_utf8(proposed.stdout.into_vec()).expect("proposal UTF-8");
    let (proposal, recipient) = proposed
        .strip_prefix("proposed: ")
        .expect("proposed outcome")
        .trim_end()
        .split_once(", ")
        .expect("proposal and recipient");
    assert_eq!(recipient, "to role 0 in project 0");
    assert!(proposal.starts_with("p1/"), "{proposed}");
    let review = call(&mut world, agent, &[b"proposal", b"show", proposal.as_bytes()]);
    assert!(
        review
            .lines()
            .any(|line| line.trim_start() == format!("project-policy 1 · set · now {now} · would be {would_be}")),
        "{review}"
    );
    let decided = call(
        &mut world,
        agent,
        &[b"proposal", b"decide", proposal.as_bytes(), b"accept", b"--running", b"stays", b"--key", b"accept-policy"],
    );
    assert_eq!(decided, "done: decide\n");
    let project = call(&mut world, agent, &[b"project", b"show", b"1"]);
    assert!(project.contains("spend per period: 900"), "{project}");
    assert!(project.contains("budget 900,"), "{project}");
    let after = call(&mut world, agent, &[b"settings", b"show"]);
    assert!(after.starts_with("newest revision 2\n"), "{after}");
    let rows = world.front().engine.store.rows.clone();
    let commits = world.front().engine.transactions.len();
    let loads = world.front().engine.loads.len();
    let second_policy = format!(r#"{{"setting":{{"kind":"project-policy","project":2}},"set":{would_be}}}"#);
    let invalid = format!(r#"{{"items":[{charter_item},{policy_item},{second_policy}]}}"#);
    let number = fixtures::give_with_file(
        &mut world,
        agent,
        &[
            b"settings",
            b"propose",
            b"settings.json",
            b"A second policy is out of shape",
            b"--against",
            b"2",
            b"--key",
            b"two-policies",
        ],
        invalid.as_bytes(),
    );
    let refused = fixtures::answered(&mut world, agent, number);
    assert_eq!(refused.status, 1, "{refused:?}");
    assert!(
        String::from_utf8_lossy(&refused.stderr)
            .contains("settings file item 3: setting: one project's policy per file"),
        "{refused:?}"
    );
    assert_eq!(world.front().engine.store.rows, rows, "invalid file sends no mutation");
    assert_eq!(world.front().engine.transactions.len(), commits);
    assert_eq!(world.front().engine.loads.len(), loads);
}
