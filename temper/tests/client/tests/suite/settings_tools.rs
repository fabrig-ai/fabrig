//! Settings files through the joined roots (temper/docs/design/client/ctl/tools.md, 3, and temper/docs/design/client/ctl/testing.md, 2).
use jig_client::contract as c;
use jig_client_kit::harness::{ClientId, Settings, World};
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
const SCOPED_OWNERS: &str =
    "to the deployment's owners and project 1's owners; one person must own every affected scope";

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
    assert_eq!(recipient, SCOPED_OWNERS);
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

fn restore_configuration(limits: &engine::Limits) -> engine::Config {
    let mut config = configuration();
    let kinds = temper_engine_domain_world::kinds::configuration(limits);
    config.core.kinds = kinds.core.kinds;
    config.core.defaults.charters = kinds.core.defaults.charters;
    let mut stock = kinds.kinds_stock.into_vec();
    // This story keeps template 4's neither value independently of the binary's stock.
    stock.retain(|value| value.kind != 11 || value.number != 4);
    let value = temper_engine_domain_forge::settings::Value::Goal(temper_engine_domain_forge::settings::TemplateGoal {
        design: false,
        close_out: false,
    });
    stock.push(temper_engine_domain_forge::settings::Stock {
        kind: 11,
        scope: 0,
        number: 4,
        value: temper_engine_domain_forge::settings::SizedValue {
            size: temper_engine_smith::kinds::size(&value),
            value,
        },
    });
    config.kinds_stock = stock.into_boxed_slice();
    config
}

fn restore_world() -> ClientWorld {
    let mut world = fixtures::kinds_world(fixtures::SEED);
    let front = world.front();
    let mut limits = front.engine.env.limits;
    limits.people.initial_owners = 2;
    // Proposal, acceptance, restore, acceptance and forgotten restore each retain an answer.
    // The joint also sends its own Seen requests beside those five acts.
    limits.people.requests = 16;
    limits.documents.text_bytes = WHOLE_SETTING_BYTES;
    limits.settings.history = 1;
    let forge = front.engine.forge.take();
    front.engine = Driver::cold(Store::with_limits(&limits), restore_configuration(&limits), &limits);
    front.engine.forge = forge;
    front.engine.settle();
    world
}

struct SettingWalk {
    selector: &'static str,
    name: &'static str,
    show: &'static [&'static [u8]],
}

const KINDS: &[SettingWalk] = &[
    SettingWalk { selector: r#"{"kind":"charter","number":1}"#, name: "charter 1", show: &[b"charter", b"1"] },
    SettingWalk { selector: r#"{"kind":"template","number":1}"#, name: "template 1", show: &[b"template", b"1"] },
    SettingWalk { selector: r#"{"kind":"policy","number":1}"#, name: "policy 1", show: &[b"policy", b"1"] },
    SettingWalk { selector: r#"{"kind":"rules"}"#, name: "rules", show: &[b"rules"] },
    SettingWalk {
        selector: r#"{"kind":"landing-rule","project":1,"number":1}"#,
        name: "landing-rule 1 --project 1",
        show: &[b"landing-rule", b"1", b"--project", b"1"],
    },
    SettingWalk {
        selector: r#"{"kind":"forge-charter","number":1}"#,
        name: "forge-charter 1",
        show: &[b"forge-charter", b"1"],
    },
    SettingWalk {
        selector: r#"{"kind":"template-goal","number":4}"#,
        name: "template-goal 4",
        show: &[b"template-goal", b"4"],
    },
    SettingWalk {
        selector: r#"{"kind":"project-policy","project":1}"#,
        name: "project-policy 1",
        show: &[b"policy", b"1"],
    },
];

fn show_value(world: &mut ClientWorld, agent: ClientId, kind: &SettingWalk) -> (c::Setting, c::SettingValue) {
    let mut words = vec![b"settings".as_slice(), b"show"];
    words.extend_from_slice(kind.show);
    words.push(b"--json");
    let shown = call(world, agent, &words);
    let value = shown.trim_end().split_once(",\"value\":").expect("setting value").1;
    let value = match value.split_once(",\"application\":") {
        Some((value, _)) => value,
        None => value.strip_suffix('}').expect("setting envelope"),
    };
    let file = format!(r#"{{"items":[{{"setting":{},"set":{value}}}]}}"#, kind.selector);
    let decoded =
        temper_client_tools::jig::settings_file::decode(file.as_bytes(), &temper_client_tools::limits::tools())
            .unwrap_or_else(|why| panic!("{}: {}", kind.name, String::from_utf8_lossy(&why)));
    let item = decoded.get(0).expect("one shown item");
    let c::SettingEdit::Set(value) = &item.edit else { panic!("whole shown value") };
    (item.setting, value.value.clone())
}

fn change_one_member(setting: c::Setting, value: &mut c::SettingValue) {
    use skein_lib::{Duration, Reader, Writer};
    use temper_client_protocol_wire::v1 as w;
    match value {
        c::SettingValue::Model(value) => value.picker = !value.picker,
        c::SettingValue::Charter(value) => {
            value.instructions = c::Text::new(INSTRUCTIONS.into(), WHOLE_SETTING_BYTES).expect("instructions");
        }
        // jig/docs/design/domain/engine/settings.md, 2: the template stays within the task grace bound.
        c::SettingValue::Template(value) => value.grace = Some(Duration::from_secs(4)),
        c::SettingValue::Policy(value) => value.period_spend = CHANGED_SPEND,
        c::SettingValue::Rules(value) => value.maximum_run_spend += 1,
        c::SettingValue::Application(bytes) => {
            if matches!(setting, c::Setting::Application { kind: 3, .. }) {
                let mut parts =
                    w::ForgeCharter::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("charter").into_parts();
                parts.charter = if parts.charter == 1 { 2 } else { 1 };
                let value = w::ForgeCharter::new(&w::CEILINGS, parts).expect("changed charter");
                let mut writer = Writer::new(usize::try_from(value.measure()).expect("charter bytes"));
                value.encode(&mut writer).expect("measured charter");
                *bytes = writer.finish();
            } else if matches!(setting, c::Setting::Application { kind: 11, .. }) {
                let value = w::TemplateGoal::new(&w::CEILINGS, w::TemplateGoalParts { design: true, close_out: true })
                    .expect("changed goal work");
                let mut writer = Writer::new(usize::try_from(value.measure()).expect("goal work bytes"));
                value.encode(&mut writer).expect("measured goal work");
                *bytes = writer.finish();
            } else {
                let mut parts =
                    w::LandingRule::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("rule").into_parts();
                parts.ci = !parts.ci;
                let value = w::LandingRule::new(&w::CEILINGS, parts).expect("changed rule");
                let mut writer = Writer::new(usize::try_from(value.measure()).expect("rule bytes"));
                value.encode(&mut writer).expect("measured rule");
                *bytes = writer.finish();
            }
        }
    }
}

fn submit_file(world: &mut ClientWorld, agent: ClientId, file: &str, owners: &str) -> String {
    let number = fixtures::give_with_file(
        world,
        agent,
        &[
            b"settings",
            b"propose",
            b"settings.json",
            b"Review every kind together",
            b"--against",
            b"1",
            b"--key",
            b"every-kind",
        ],
        file.as_bytes(),
    );
    let printed = fixtures::answered(world, agent, number);
    assert_eq!(printed.status, 0, "{printed:?}");
    let printed = String::from_utf8(printed.stdout.into_vec()).expect("proposal UTF-8");
    let (proposal, recipient) =
        printed.strip_prefix("proposed: ").expect("proposal").trim_end().split_once(", ").expect("recipient");
    assert_eq!(recipient, owners);
    proposal.to_owned()
}

fn ctl_accept(world: &mut ClientWorld, agent: ClientId, proposal: &str, key: &[u8]) {
    assert_eq!(
        call(
            world,
            agent,
            &[b"proposal", b"decide", proposal.as_bytes(), b"accept", b"--running", b"stays", b"--key", key]
        ),
        "done: decide\n"
    );
}

#[test]
fn each_kind_through_ctl_proposed_shown_accepted_and_restored() {
    let mut world = restore_world();
    let agent = fixtures::agent(&mut world);
    let project_before = call(&mut world, agent, &[b"project", b"show", b"1"]);
    assert_eq!(
        call(&mut world, agent, &[b"settings", b"show", b"template-goal", b"4"]),
        "template-goal 4, revision 1\n{\"design\":false,\"close_out\":false}\ndefault, 2 bytes\n"
    );
    let originals: Vec<_> = KINDS.iter().map(|kind| show_value(&mut world, agent, kind)).collect();
    let mut changed = originals.clone();
    let mut items = Vec::new();
    for (kind, (setting, value)) in KINDS.iter().zip(&mut changed) {
        change_one_member(*setting, value);
        items.push(format!(r#"{{"setting":{},"set":{}}}"#, kind.selector, value_json(*setting, value.clone())));
    }
    let file = format!(r#"{{"items":[{}]}}"#, items.join(","));
    let proposal = submit_file(&mut world, agent, &file, SCOPED_OWNERS);
    let review = call(&mut world, agent, &[b"proposal", b"show", proposal.as_bytes()]);
    for ((kind, (setting, original)), (_, changed)) in KINDS.iter().zip(&originals).zip(&changed) {
        let now = format!("{} · set · now {}", kind.name, value_json(*setting, original.clone()));
        let would_be = format!(" · would be {}", value_json(*setting, changed.clone()));
        assert!(
            review.lines().any(|shown| shown.trim_start().starts_with(&now) && shown.ends_with(&would_be)),
            "missing {now} ... {would_be}\n{review}"
        );
    }
    ctl_accept(&mut world, agent, &proposal, b"accept-every-kind");
    let inventory = call(&mut world, agent, &[b"settings", b"show"]);
    assert!(inventory.starts_with("newest revision 2\n"));
    for name in ["landing-rule", "forge-charter", "template-goal"] {
        assert!(inventory.contains(name), "{inventory}");
    }
    let revision = call(&mut world, agent, &[b"revision", b"show", b"2"]);
    for (kind, (setting, changed)) in KINDS.iter().zip(&changed) {
        let mut words = vec![b"settings".as_slice(), b"show"];
        words.extend_from_slice(kind.show);
        let shown = call(&mut world, agent, &words);
        assert!(shown.contains(&value_json(*setting, changed.clone())), "{shown}");
        if matches!(setting, c::Setting::Application { kind: 11, .. }) {
            assert_eq!(
                shown,
                "template-goal 4, revision 2\n{\"design\":true,\"close_out\":true}\nchanged at revision 2, 2 bytes\n"
            );
        }
        assert_eq!(show_value(&mut world, agent, kind), (*setting, changed.clone()), "{}", kind.name);
        let line = revision
            .lines()
            .position(|line| line.trim_start().starts_with(&format!("{}:", kind.name)))
            .unwrap_or_else(|| panic!("missing {}\n{revision}", kind.name));
        // temper/docs/design/client/ctl/tools.md, 2: a set value is printed whole before what it replaced.
        let mut lines = revision.lines().skip(line + 1);
        assert_eq!(lines.next().expect("set value"), value_json(*setting, changed.clone()), "{revision}");
        assert!(lines.next().expect("replaced line").trim_start().starts_with("was: "), "{revision}");
    }
    let restored = call(
        &mut world,
        agent,
        &[b"revision", b"restore", b"1", b"Return every kind", b"--key", b"restore-every-kind"],
    );
    let restored =
        restored.strip_prefix("proposed restoring revision 1 as proposal ").expect("restored outcome").trim_end();
    let proposal = restored.strip_suffix(" to role 0").expect("owner role");
    assert!(proposal.starts_with("p1/"));
    let review = call(&mut world, agent, &[b"proposal", b"show", proposal.as_bytes()]);
    for (kind, (setting, original)) in KINDS.iter().zip(&originals) {
        assert!(review.contains(kind.name), "{review}");
        // temper/docs/design/client/ctl/tools.md, 2 and jig/docs/design/domain/engine/settings.md, 10:
        // restored application defaults are reviewed as resets; their values are read after acceptance.
        let proposed = if matches!(setting, c::Setting::Application { .. }) {
            "would be default".to_owned()
        } else {
            format!("would be {}", value_json(*setting, original.clone()))
        };
        assert!(
            review.lines().any(|line| line.trim_start().starts_with(kind.name) && line.ends_with(&proposed)),
            "{review}"
        );
    }
    ctl_accept(&mut world, agent, proposal, b"accept-restored");
    assert!(call(&mut world, agent, &[b"settings", b"show"]).starts_with("newest revision 3\n"));
    for (kind, (setting, original)) in KINDS.iter().zip(&originals) {
        let (restored_setting, restored_value) = show_value(&mut world, agent, kind);
        assert_eq!(restored_setting, *setting);
        assert_eq!(&restored_value, original, "{}", kind.name);
    }
    assert_eq!(call(&mut world, agent, &[b"project", b"show", b"1"]), project_before);
    let number = fixtures::give(
        &mut world,
        agent,
        &[b"revision", b"restore", b"1", b"Forgotten", b"--key", b"restore-forgotten"],
    );
    let refused = fixtures::answered(&mut world, agent, number);
    assert_eq!(refused.status, 2, "{refused:?}");
    assert!(
        String::from_utf8_lossy(&refused.stderr).contains("revision 1 is forgotten; the oldest kept is revision 2"),
        "{refused:?}"
    );
}

fn review_file(world: &mut ClientWorld, agent: ClientId) -> String {
    let (setting, mut charter) = show_value(world, agent, &KINDS[0]);
    let c::SettingValue::Charter(value) = &mut charter else { panic!("charter") };
    value.instructions = c::Text::new(INSTRUCTIONS.into(), WHOLE_SETTING_BYTES).expect("instructions");
    value.waiting = skein_lib::Duration::from_secs(2);
    let charter = value_json(setting, charter);
    let (setting, mut rules) = show_value(world, agent, &KINDS[3]);
    change_one_member(setting, &mut rules);
    let rules = value_json(setting, rules);
    let kind =
        SettingWalk { selector: r#"{"kind":"charter","number":11}"#, name: "charter 11", show: &[b"charter", b"11"] };
    let (setting, mut instructions) = show_value(world, agent, &kind);
    change_one_member(setting, &mut instructions);
    let instructions = value_json(setting, instructions);
    format!(
        r#"{{"items":[{{"setting":{{"kind":"charter","number":1}},"set":{charter}}},{{"setting":{{"kind":"rules"}},"set":{rules}}},{{"setting":{{"kind":"charter","number":11}},"set":{instructions}}}]}}"#
    )
}

fn assert_review_effect(review: &str) {
    for words in [
        "New work follows this from the moment it is accepted. Nothing restarts.",
        "--running stays",
        "--running moves",
        "Conversations move at their next run",
        "charter is removed",
        "model is no longer allowed",
        "retain their revision",
        "Authority remains current",
    ] {
        assert!(review.contains(words), "missing {words}: {review}");
    }
}

#[test]
fn proposal_review_groups_changes_and_explains_running_work() {
    let mut world = fixtures::asking_world(fixtures::SEED, false);
    let agent = fixtures::asking_agent(&mut world);
    call(
        &mut world,
        agent,
        &[
            b"chat",
            b"start",
            b"--project",
            b"1",
            b"--template",
            b"5",
            b"Wait for my settings review",
            b"--key",
            b"review-chat",
        ],
    );
    assert!(!world.front().assignments.is_empty(), "scripted host keeps the chat live");
    let inventory = call(&mut world, agent, &[b"settings", b"show"]);
    assert!(inventory.contains("revision 1"), "{inventory}");
    assert!(inventory.contains("live tasks: 1"), "{inventory}");
    let file = review_file(&mut world, agent);
    let proposal = submit_file(&mut world, agent, &file, "to the deployment's owners");
    let review = call(&mut world, agent, &[b"proposal", b"show", proposal.as_bytes()]);
    assert_review_groups(&review);
    assert!(review.contains("against revision 1, newest 1"), "{review}");
    assert!(review.contains("Review every kind together"), "{review}");
    assert!(review.contains("revision 1: 1 live task"), "{review}");
    assert!(review.contains("\"waiting\":2000") && review.contains("Follow the project's new policy."), "{review}");
    assert!(review.contains(" · now ") && review.contains(" · would be "), "{review}");
    assert_review_effect(&review);
    let json = call(&mut world, agent, &[b"proposal", b"show", proposal.as_bytes(), b"--json"]);
    assert_review_json(&json);
    assert_review_effect(&json);
}

fn assert_review_groups(review: &str) {
    let (_, enforced) = review.split_once("temper enforces\n").expect("product enforcement group");
    let (enforced, followed) = enforced.split_once("your agents follow\n").expect("product instructions group");
    let followed = followed.split_once("Effect\n").expect("running-work effect").0;
    assert!(enforced.contains("charter 1 ·") && enforced.contains("rules"), "{review}");
    assert!(!enforced.contains("charter 11 ·"), "instructions alone are followed: {review}");
    assert!(followed.contains("charter 1 ·") && followed.contains("charter 11 ·"), "{review}");
    assert!(followed.contains("instructions") && !followed.contains("rules"), "{review}");
}

fn assert_review_json(json: &str) {
    for fact in [
        "\"against\":1",
        "\"newest\":1",
        "\"enforces\":[",
        "\"your_agents_follow\":[",
        "\"held_revisions\":[{\"revision\":1,\"live_tasks\":1}]",
        "\"restarts\":false",
        "\"authority\":\"current\"",
        "\"running_choices\":[\"stays\",\"moves\"]",
        "\"reason\":",
        "Review every kind together",
        "\"now\":",
        "\"would_be\":",
    ] {
        assert!(json.contains(fact), "missing {fact}: {json}");
    }
    let enforced = json.split_once("\"enforces\":[").expect("enforced array").1;
    let (enforced, followed) = enforced.split_once("\"your_agents_follow\":[").expect("followed array");
    let followed = followed.split_once("\"effect\":").expect("effect object").0;
    assert!(enforced.contains("\"kind\":\"rules\"") && enforced.contains("\"number\":1}"), "{json}");
    assert!(!enforced.contains("\"number\":11}"), "{json}");
    assert!(followed.contains("\"number\":1}") && followed.contains("\"number\":11}"), "{json}");
    assert!(!followed.contains("\"kind\":\"rules\""), "{json}");
}

fn revision_world() -> ClientWorld {
    let mut world = fixtures::kinds_world(fixtures::SEED);
    let front = world.front();
    let mut limits = front.engine.env.limits;
    limits.people.initial_owners = 2;
    limits.people.requests = 16;
    limits.settings.history = 4;
    limits.tasks.charters = 12; // Kind 9's stock lines run up to charter 12.
    let mut config = restore_configuration(&limits);
    let mut charter = config.core.defaults.charters[0].clone();
    charter.number = 6;
    let mut charters = config.core.defaults.charters.into_vec();
    charters.push(charter);
    config.core.defaults.charters = charters.into_boxed_slice();
    let forge = front.engine.forge.take();
    front.engine = Driver::cold(Store::with_limits(&limits), config, &limits);
    front.engine.forge = forge;
    front.engine.settle();
    world
}

fn accept_forge_file(world: &mut ClientWorld, agent: ClientId, against: &[u8], key: &[u8], file: &str) {
    let number = fixtures::give_with_file(
        world,
        agent,
        &[b"settings", b"propose", b"settings.json", b"Set the forge values", b"--against", against, b"--key", key],
        file.as_bytes(),
    );
    let printed = fixtures::answered(world, agent, number);
    assert_eq!(printed.status, 0, "{printed:?}");
    let text = String::from_utf8(printed.stdout.into_vec()).expect("proposal");
    let proposal = text.strip_prefix("proposed: ").expect("proposed outcome").split_once(", ").expect("recipient").0;
    ctl_accept(world, agent, proposal, &[key, b"-accept"].concat());
}

fn forge_revision_file(charter: u32, ci: bool) -> String {
    format!(
        r#"{{"items":[{{"setting":{{"kind":"landing-rule","number":1,"project":1}},"set":{{"pattern":{{"segments":["666f726765"],"last":{{"exact":"6d61696e"}}}},"enforce":true,"ci":{ci},"up_to_date":false,"gates":[],"approvals":[]}}}},{{"setting":{{"kind":"forge-charter","number":1}},"set":{charter}}}]}}"#
    )
}

#[test]
fn a_settings_file_rule_keeps_its_sets_when_applied_and_shown_as_json() {
    let mut world = revision_world();
    let agent = fixtures::agent(&mut world);
    let file = br#"{"items":[{"setting":{"kind":"landing-rule","project":1,"number":1},"set":{"pattern":{"segments":["6272616e6368"],"last":{"exact":"6d61696e"}},"enforce":true,"ci":true,"up_to_date":true,"gates":[],"approvals":[],"sets":[1]}}]}"#;
    let number = fixtures::give_with_file(
        &mut world,
        agent,
        &[b"settings", b"apply", b"sets.json", b"--against", b"1", b"--key", b"rule-sets"],
        file,
    );
    let applied = fixtures::answered(&mut world, agent, number);
    assert_eq!(applied.status, 0, "{applied:?}");
    for json in [false, true] {
        let mut words = vec![b"settings".as_slice(), b"show", b"landing-rule", b"1", b"--project", b"1"];
        if json {
            words.push(b"--json");
        }
        let shown = call(&mut world, agent, &words);
        assert!(shown.contains("\"sets\":[1]"), "{shown}");
    }
}

#[test]
fn revision_show_keeps_temper_values_after_both_settings_are_overwritten() {
    let mut world = revision_world();
    let agent = fixtures::agent(&mut world);
    let defaults = call(&mut world, agent, &[b"revision", b"show", b"1"]);
    accept_forge_file(&mut world, agent, b"1", b"first-forge", &forge_revision_file(6, false));
    let first = forge_revision_outputs(&mut world, agent);
    accept_forge_file(&mut world, agent, b"2", b"second-forge", &forge_revision_file(4, true));
    let after = forge_revision_outputs(&mut world, agent);
    assert_eq!(after, first, "historical values must survive later accepted edits");
    accept_forge_file(
        &mut world,
        agent,
        b"3",
        b"erase-forge",
        r#"{"items":[{"setting":{"kind":"landing-rule","number":1,"project":1},"edit":"removed"},{"setting":{"kind":"forge-charter","number":1},"edit":"reset"}]}"#,
    );
    assert_eq!(forge_revision_outputs(&mut world, agent), first);
    assert_eq!(call(&mut world, agent, &[b"revision", b"show", b"1"]), defaults);
    let erased = call(&mut world, agent, &[b"revision", b"show", b"4"]);
    assert!(erased.contains("removed") && erased.contains("reset"), "{erased}");
    for (kind, project) in [(b"landing-rule".as_slice(), true), (b"forge-charter".as_slice(), false)] {
        let mut words = vec![b"settings".as_slice(), b"show", kind, b"1", b"--revision", b"2"];
        if project {
            words.extend_from_slice(&[b"--project", b"1"]);
        }
        let shown = call(&mut world, agent, &words);
        assert!(shown.contains(if project { "\"ci\":false" } else { "\n6\n" }), "{shown}");
    }
}

fn forge_revision_outputs(world: &mut ClientWorld, agent: ClientId) -> [String; 2] {
    [false, true].map(|json| {
        let mut words = vec![b"revision".as_slice(), b"show", b"2"];
        if json {
            words.push(b"--json");
        }
        let shown = call(world, agent, &words);
        assert!(shown.contains("\"ci\":false"), "{shown}");
        assert!(shown.contains(if json { "\"set\":6" } else { "\n6\n" }), "{shown}");
        assert!(shown.contains("landing-rule") && shown.contains("forge-charter"), "{shown}");
        shown
    })
}

#[test]
fn restore_revision_shows_replaced_forge_values() {
    let mut world = revision_world();
    let agent = fixtures::agent(&mut world);
    accept_forge_file(&mut world, agent, b"1", b"before-restore", &forge_revision_file(6, false));
    let restored =
        call(&mut world, agent, &[b"revision", b"restore", b"1", b"Restore defaults", b"--key", b"restore-forge"]);
    let proposal = restored
        .strip_prefix("proposed restoring revision 1 as proposal ")
        .expect("restore proposed")
        .trim_end()
        .strip_suffix(" to role 0")
        .expect("owners");
    ctl_accept(&mut world, agent, proposal, b"accept-restore-forge");
    let first = restored_forge_outputs(&mut world, agent);
    accept_forge_file(&mut world, agent, b"3", b"after-restore", &forge_revision_file(4, true));
    assert_eq!(restored_forge_outputs(&mut world, agent), first);
}

fn restored_forge_outputs(world: &mut ClientWorld, agent: ClientId) -> [String; 2] {
    [false, true].map(|json| {
        let mut words = vec![b"revision".as_slice(), b"show", b"3"];
        if json {
            words.push(b"--json");
        }
        let shown = call(world, agent, &words);
        assert!(shown.contains("\"ci\":false"), "replaced landing rule: {shown}");
        assert!(shown.contains(if json { "\"set\":6" } else { "was: 6" }), "replaced charter: {shown}");
        assert!(shown.contains("landing-rule") && shown.contains("forge-charter"), "{shown}");
        shown
    })
}

const PROJECT_WAYS_FILE: &str = r#"{"items":[{"setting":{"kind":"project-ways","project":1},"set":{"drafts":true,"goals":true,"changes":false,"way":0}},{"setting":{"kind":"project-questions","project":1},"set":"up"}]}"#;

#[test]
#[expect(clippy::too_many_lines, reason = "the table checks ways, questions and words through the joined ctl path")]
fn settings_show_prints_a_projects_ways_and_questions() {
    let mut world = revision_world();
    let agent = fixtures::agent(&mut world);
    for (kind, value) in [
        (b"project-ways".as_slice(), r#"{"drafts":true,"goals":false,"changes":false,"way":0}"#),
        (b"project-questions", "\"none\""),
    ] {
        let shown = call(&mut world, agent, &[b"settings", b"show", kind, b"1"]);
        assert!(shown.contains(value) && shown.contains("the stock"), "{shown}");
    }
    let proposal = submit_file(&mut world, agent, PROJECT_WAYS_FILE, "to project 1's owners");
    let review = call(&mut world, agent, &[b"proposal", b"show", proposal.as_bytes()]);
    assert!(review.contains("project-ways 1") && review.contains("project-questions 1"), "{review}");
    assert!(!review.contains("application kind"), "{review}");
    ctl_accept(&mut world, agent, &proposal, b"accept-project-ways");
    let inventory = call(&mut world, agent, &[b"settings", b"show"]);
    assert!(inventory.contains("project-ways 1") && inventory.contains("project-questions 1"), "{inventory}");
    let project = configuration().core.seed_projects[0].name.as_bytes().to_vec();
    for (kind, value) in [
        (b"project-ways".as_slice(), r#"{"drafts":true,"goals":true,"changes":false,"way":0}"#),
        (b"project-questions", "\"up\""),
    ] {
        for selector in [b"1".as_slice(), project.as_ref()] {
            let shown = call(&mut world, agent, &[b"settings", b"show", kind, selector]);
            assert!(shown.contains(value) && !shown.contains("the stock"), "{shown}");
        }
        let json = call(&mut world, agent, &[b"settings", b"show", kind, b"1", b"--json"]);
        assert!(json.contains(&format!("\"value\":{value}")) && json.contains("\"project\":1"), "{json}");
        assert!(!json.contains("\"number\":1"), "{json}");
        let historical = call(&mut world, agent, &[b"settings", b"show", kind, b"1", b"--revision", b"2"]);
        assert!(historical.contains(value), "{historical}");
    }
    let revision = call(&mut world, agent, &[b"revision", b"show", b"2"]);
    assert!(revision.contains("project-ways 1") && revision.contains("project-questions 1"), "{revision}");
    assert!(!revision.contains("application kind"), "{revision}");
    let changed = PROJECT_WAYS_FILE.replace("up", "none").replace("\"changes\":false", "\"changes\":true");
    let number = fixtures::give_with_file(
        &mut world,
        agent,
        &[b"settings", b"apply", b"project.json", b"--against", b"2", b"--key", b"apply-project"],
        changed.as_bytes(),
    );
    let applied = fixtures::answered(&mut world, agent, number);
    assert_eq!(applied.status, 0, "{applied:?}");
    let shown = call(&mut world, agent, &[b"settings", b"show", b"project-questions", b"1"]);
    assert!(shown.contains("\"none\""), "{shown}");
    let historical =
        call(&mut world, agent, &[b"settings", b"show", b"project-questions", b"1", b"--revision", b"2", b"--json"]);
    assert!(historical.contains("\"value\":\"up\""), "{historical}");
    for (kind, selector, value, owners, name, initial) in [
        (
            b"charter-words".as_slice(),
            r#"{"kind":"charter-words","number":1}"#,
            r#""reviews every change""#,
            "to the deployment's owners",
            "charter-words 1",
            r#""talks with you and asks for the changes""#,
        ),
        (
            b"project-words",
            r#"{"kind":"project-words","project":1}"#,
            r#"{"headline":"Our way","lede":"We review every change."}"#,
            "to project 1's owners",
            "project-words 1",
            "null",
        ),
    ] {
        let mut world = revision_world();
        let agent = fixtures::agent(&mut world);
        for json in [false, true] {
            let mut argv = vec![b"settings".as_slice(), b"show", kind, b"1"];
            if json {
                argv.push(b"--json");
            }
            let shown = call(&mut world, agent, &argv);
            let expected = if json {
                format!("\"value\":{initial}")
            } else if initial == "null" {
                "\nnone\n".to_owned()
            } else {
                format!("\n{initial}\n")
            };
            assert!(shown.contains(&expected), "{shown}");
        }
        let file = format!(r#"{{"items":[{{"setting":{selector},"set":{value}}}]}}"#);
        let proposal = submit_file(&mut world, agent, &file, owners);
        for json in [false, true] {
            let mut argv = vec![b"proposal".as_slice(), b"show", proposal.as_bytes()];
            if json {
                argv.push(b"--json");
            }
            let shown = call(&mut world, agent, &argv);
            assert!(shown.contains(value), "{shown}");
            assert!(shown.contains(if json { core::str::from_utf8(kind).unwrap() } else { name }), "{shown}");
        }
        ctl_accept(&mut world, agent, &proposal, b"accept-words");
        let inventory = call(&mut world, agent, &[b"settings", b"show"]);
        assert!(inventory.contains(name), "{inventory}");
        for json in [false, true] {
            for historical in [false, true] {
                let mut argv = vec![b"settings".as_slice(), b"show", kind, b"1"];
                if historical {
                    argv.extend([b"--revision".as_slice(), b"2"]);
                }
                if json {
                    argv.push(b"--json");
                }
                let shown = call(&mut world, agent, &argv);
                assert!(shown.contains(value), "{shown}");
                assert!(!shown.contains("the stock"), "{shown}");
            }
            let mut argv = vec![b"revision".as_slice(), b"show", b"2"];
            if json {
                argv.push(b"--json");
            }
            let shown = call(&mut world, agent, &argv);
            assert!(shown.contains(value), "{shown}");
            assert!(shown.contains(if json { core::str::from_utf8(kind).unwrap() } else { name }), "{shown}");
        }
    }
}

#[test]
fn a_settings_file_changes_a_projects_questions_and_a_change_takes_them() {
    use temper_engine_domain_world::{forge_routes::Until, kinds};
    let mut world = kinds::forge_world();
    let goal = kinds::goal(&mut world);
    let items = temper_client_tools::jig::settings_file::decode(
        br#"{"items":[{"setting":{"kind":"project-questions","project":1},"set":"up"}]}"#,
        &temper_client_tools::limits::tools(),
    )
    .expect("questions settings file");
    let item = items.get(0).expect("questions item");
    let c::Setting::Application { kind, scope, number } = item.setting else { panic!("project setting") };
    let c::SettingEdit::Set(set) = &item.edit else { panic!("set questions") };
    let c::SettingValue::Application(bytes) = &set.value else { panic!("questions bytes") };
    let against = kinds::revision(&world.store.rows);
    let input = temper_service::clients::translation::to_root(jig_domain_core::Event::ChangeSettings {
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(94)),
        sign_in: world.signed_in.expect("owner"),
        key: [94; 16],
        digest: [94; 32],
        change: Box::new(settings::Change {
            against,
            running: settings::Running::Stays,
            items: Box::new([settings::Item {
                setting: settings::Setting::Application { kind, scope, number },
                edit: settings::Edit::Set(settings::Value::Application(bytes.clone())),
            }]),
        }),
    })
    .expect("client entrance");
    world.send(engine::Event::Party(input));
    world.until(Until::Ready);
    assert_eq!(kinds::revision(&world.store.rows), against + 1, "{:?}", world.web_replies);
    let (task, _) = kinds::goal_change(&mut world, &goal);
    assert_eq!(kinds::change_row(&world, task).questions.route, temper_engine_domain_forge_change::Route::Up);
}

#[test]
fn a_settings_file_asking_an_effort_its_model_does_not_offer_is_refused_by_name() {
    let mut world = world();
    let mut hello = temper_engine_domain_world::protocol::hello(skein_lib::Token::new(7), Box::new([]));
    if let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut hello
    {
        hello.bounds.message_bytes = world.front().engine.env.limits.tasks.message_bytes;
    }
    world.front().engine.send(hello);
    let agent = fixtures::agent(&mut world);
    let (setting, mut value) = show_value(&mut world, agent, &KINDS[1]);
    let c::SettingValue::Template(template) = &mut value else { panic!("template") };
    template.llm = Some(c::LlmChoice {
        later: None,
        account: None,
        main: c::ModelEffort {
            model: c::Text::new(b"scripted".as_slice().into(), 256).expect("model"),
            effort: Some(c::Text::new(b"xhigh".as_slice().into(), 256).expect("effort")),
        },
        subagents: skein_lib::List::with_capacity(0),
    });
    let file = format!(r#"{{"items":[{{"setting":{},"set":{}}}]}}"#, KINDS[1].selector, value_json(setting, value));
    let number = fixtures::give_with_file(
        &mut world,
        agent,
        &[b"settings", b"apply", b"settings.json", b"--against", b"1", b"--key", b"unsupported-effort"],
        file.as_bytes(),
    );
    let refused = fixtures::answered(&mut world, agent, number);
    assert_eq!(
        String::from_utf8_lossy(&refused.stderr),
        "settings file item 1: model scripted offers no effort xhigh; it offers none\n",
        "{refused:?}"
    );
    assert_eq!(refused.status, 1, "{refused:?}");
    assert_eq!(world.root(agent).agent().kept(), 0, "no kept change-settings");
    assert!(world.kept(agent).is_empty(), "no saved change-settings");
}
