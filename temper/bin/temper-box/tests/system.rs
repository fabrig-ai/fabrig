//! Live settings over the joined client and engine, on composed profiles.
//! temper/docs/design/testing/box/README.md, 2.7.
use serde_json::{Value, json};
use std::path::Path;
use temper_box_kit::forge_land::{LiveModel, LiveProfile, live_apply, live_settings};

type ClientWorld = jig_client_kit::harness::World<temper_client_world::app::Temper, temper_client_world::front::Joint>;

/// The live deployment's stock catalogue, endpoints, providers and charters 1, 2 and 4
/// on the joined client world's engine (temper/docs/design/testing/live-tier.md and temper/docs/design/testing/checks.md, 3).
fn live_model_world(model: LiveModel<'_>) -> ClientWorld {
    use jig_client_kit::harness::{Settings, World};
    use temper_client_world::{
        fixtures,
        front::{Joint, engine_limits},
    };
    use temper_engine_domain_world::{store::Store, walking};

    let root = Path::new("/fixture/live-settings");
    let profile = LiveProfile::compose(&root.join("data"), model, Path::new("/usr/bin/true"));
    let (engine, _) = profile.engine(&root.join("config"));
    let stock = temper_service::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    let mut limits = engine_limits();
    limits.people.initial_owners = 2;
    limits.accounts.accounts = 3;
    limits.accounts.holder_accounts = 3;
    limits.authority.accounts = 3;
    limits.fleet.grants = 4;
    // Init's OpenAI provider can read a list for the world's fixture accounts.
    limits.fleet.reads = limits.accounts.accounts;
    limits.tasks.charters = 3;
    limits.templates = 3;
    limits.documents.text_bytes = 8192;
    limits.journal.run_bytes = engine.limits.core.run_bytes;
    // Stock charter versions must fit the store's decoded settings pages.
    let page_bytes = u32::try_from(engine.limits.core.load_bytes).expect("stock settings page fits u32");
    limits.loads.bytes = page_bytes;
    limits.loads.reply_bytes = page_bytes;
    limits.loads.transcript_bytes = page_bytes;
    let configs = std::array::from_fn(|_| {
        let mut config = walking::config(fixtures::SEED);
        config.core.owners = Box::new([
            config.core.owners[0].clone(),
            jig_domain_people::InitialOwner { project: 0, identity: config.core.owners[0].identity.clone() },
        ]);
        config.core.defaults.models.clone_from(&stock.models);
        config.core.defaults.charters = stock.charters.iter().take(3).cloned().collect();
        let core = jig_protocol_config::core(&engine, config.core.deployment, fixtures::SEED);
        config.core.defaults.providers = core.defaults.providers;
        config.core.fixed_providers = core.fixed_providers;
        let llm = core.settings;
        config.core.settings.models = llm.models;
        config.core.settings.endpoints = llm.endpoints;
        config.core.settings.llm_providers = llm.llm_providers;
        for template in &mut config.core.defaults.templates {
            template.llm = None;
        }
        config
    });
    let front = Joint::with_engine(
        fixtures::SEED,
        &fixtures::domain_limits(),
        Store::with_limits(&limits),
        configs,
        &limits,
        1,
    );
    let mut world = World::new(
        Settings { seed: fixtures::SEED, words: fixtures::decision_words(), iterations: 4096, ..Settings::default() },
        front,
    );
    world.randomness(2, 0, 0);
    world
}

#[test]
fn a_fresh_claude_sitting_applies_its_settings_as_a_person_does() {
    assert_live_settings(LiveModel::ClaudeCode { login: Path::new("/held/claude-login") });
}

#[test]
fn a_fresh_key_sitting_applies_claude_settings_on_inits_configuration() {
    assert_live_settings(LiveModel::AnthropicKey { key: Path::new("/private/key") });
}

fn assert_live_settings(model: LiveModel<'_>) {
    use temper_client_world::fixtures;
    let mut world = live_model_world(model);
    let agent = fixtures::agent(&mut world);
    if let LiveModel::ClaudeCode { .. } = model {
        let call = fixtures::give(
            &mut world,
            agent,
            &[b"account", b"declare", b"2", b"claude_code", b"claude-login", b"--key", b"live-program"],
        );
        let declared = fixtures::answered(&mut world, agent, call);
        assert_eq!(declared.status, 0, "{declared:?}");
        assert!(declared.stderr.is_empty(), "{declared:?}");
    }
    let read = |world: &mut jig_client_kit::harness::World<_, _>, words: &[&[u8]]| {
        let call = fixtures::give(world, agent, words);
        let printed = fixtures::answered(world, agent, call);
        assert_eq!(printed.status, 0, "{printed:?}");
        assert!(printed.stderr.is_empty(), "{printed:?}");
        serde_json::from_slice::<Value>(&printed.stdout).expect("settings JSON")
    };
    let inventory = read(&mut world, &[b"settings", b"show", b"--json"]);
    let charters = [
        read(&mut world, &[b"settings", b"show", b"charter", b"1", b"--json"]),
        read(&mut world, &[b"settings", b"show", b"charter", b"2", b"--json"]),
        read(&mut world, &[b"settings", b"show", b"charter", b"4", b"--json"]),
    ];
    let row = read(&mut world, &[b"settings", b"show", b"model", b"3", b"--json"]);
    let changes = live_settings(model, &inventory, charters.each_ref(), &row).expect("live settings");
    let claude = live_settings(
        LiveModel::ClaudeCode { login: Path::new("/held/claude-login") },
        &inventory,
        charters.each_ref(),
        &row,
    )
    .expect("Claude Code settings");
    assert!(changes == claude, "the key applies exactly Claude Code's model and charter changes");
    let newest = inventory["newest"].as_u64().expect("newest");
    for (at, change) in changes.iter().enumerate() {
        let file = format!("{}.json", change.key);
        let argv = live_apply(change, at, newest, &file);
        let argv: Vec<_> = argv.iter().map(String::as_bytes).collect();
        let bytes = serde_json::to_vec(&change.file).expect("settings bytes");
        let call = fixtures::give_with_file(&mut world, agent, &argv, &bytes);
        let applied = fixtures::answered(&mut world, agent, call);
        assert_eq!(applied.status, 0, "{}: stderr={}", change.key, String::from_utf8_lossy(&applied.stderr));
        assert!(applied.stderr.is_empty(), "{applied:?}");
    }
    let model = read(&mut world, &[b"settings", b"show", b"model", b"64", b"--json"]);
    let value = &model["value"];
    assert_eq!(value["model"]["name"], "claude-haiku-5-5");
    assert_eq!(value["source"], "person");
    assert_eq!(value["estimated"], true);
    assert_eq!(value["model"]["effort"], "low");
    assert!(value["model"]["efforts"].as_array().expect("offered efforts").contains(&json!("low")));
    // temper/docs/design/testing/live-tier.md, 2: the three per-activation bounds and stock reserve.
    for (number, turns) in [(b"1".as_slice(), 5), (b"2", 10), (b"4", 4)] {
        let charter = read(&mut world, &[b"settings", b"show", b"charter", number, b"--json"]);
        let value = &charter["value"];
        assert_eq!(value["turns"], turns);
        assert_eq!(value["reserve_turns"], 1);
        if number == b"4" {
            assert_eq!(value["model"], charters[2]["value"]["model"]);
        } else {
            assert_eq!(value["model"]["name"], "claude-haiku-5-5");
            assert_eq!(value["model"]["effort"], "low");
            assert_eq!(value["alternatives"], json!([]));
            assert_eq!(value["rest"], false);
        }
    }
}
