use crate::support;
use temper_box_shell::directories::jig_directories;

use serde_json::{Value, json};

fn launcher(directory: &std::path::Path, argument: &str, login: Option<&std::path::Path>) -> std::process::Command {
    let mut command = std::process::Command::new("/bin/sh");
    command
        .arg(skein_scratch::paths::tree().join("scripts/proof-live.sh"))
        .arg(argument)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", directory)
        .env("TMPDIR", directory)
        .env(temper_test_env::LIVE_STORE.name, directory.join("store"))
        .env(temper_test_env::LIVE_KEEP.name, directory.join("evidence"));
    if let Some(login) = login {
        command.env(temper_test_env::LIVE_CLAUDE.name, login);
    }
    command
}

#[test]
fn the_claude_launcher_refuses_invalid_held_directories() {
    use std::{
        fs,
        os::unix::fs::{PermissionsExt, symlink},
    };
    let scratch = support::Scratch::new("live-claude-refusals");
    let root = scratch.path();
    fs::write(root.join("file"), b"opaque").expect("file fixture");
    for (name, mode) in
        [("public", 0o755), ("empty", 0o700), ("Bad-name", 0o700), ("abcdefghijklmnopqrstuvwxyz1234567", 0o700)]
    {
        fs::create_dir(root.join(name)).expect("directory fixture");
        fs::set_permissions(root.join(name), fs::Permissions::from_mode(mode)).expect("mode");
    }
    symlink(root.join("empty"), root.join("link")).expect("link fixture");
    for (argument, login, message) in [
        ("forge-land-claude", None, "TEMPER_TEST_LIVE_CLAUDE is required for forge-land-claude"),
        ("forge-land-claude", Some("missing"), "TEMPER_TEST_LIVE_CLAUDE names no directory"),
        ("forge-land-claude", Some("file"), "TEMPER_TEST_LIVE_CLAUDE is not a directory or is a link"),
        ("forge-land-claude", Some("link"), "TEMPER_TEST_LIVE_CLAUDE is not a directory or is a link"),
        ("forge-land-claude", Some("link/"), "TEMPER_TEST_LIVE_CLAUDE is not a directory or is a link"),
        ("forge-land-claude", Some("public"), "TEMPER_TEST_LIVE_CLAUDE is not 0700"),
        ("forge-land-claude", Some("empty"), "TEMPER_TEST_LIVE_CLAUDE is empty"),
        (
            "forge-land-claude",
            Some("Bad-name"),
            "TEMPER_TEST_LIVE_CLAUDE has no valid source name (1 to 32 lowercase letters, digits or hyphens)",
        ),
        (
            "forge-land-claude",
            Some("abcdefghijklmnopqrstuvwxyz1234567"),
            "TEMPER_TEST_LIVE_CLAUDE has no valid source name (1 to 32 lowercase letters, digits or hyphens)",
        ),
        ("forge-land", Some("empty"), "TEMPER_TEST_LIVE_CLAUDE is for forge-land-claude"),
        ("", Some("empty"), "TEMPER_TEST_LIVE_CLAUDE is for forge-land-claude"),
    ] {
        let path = login.map(|name| root.join(name));
        let output = launcher(root, argument, path.as_deref()).output().expect("offline launcher checks");
        assert_eq!(output.status.code(), Some(2), "{argument}: {login:?}");
        assert_eq!(String::from_utf8_lossy(&output.stderr).trim(), message, "{argument}: {login:?}");
    }
    scratch.remove();
}

#[test]
fn a_claude_configuration_starts_on_inits_limits() {
    assert_live_startup("live-claude-startup", configure_claude);
}

fn assert_live_startup(case: &str, configure: fn(&[std::path::PathBuf; 3])) {
    use skein_io::kernel::Exit;
    use skein_lib::Duration;
    use skein_world::end_to_end::Command;
    use support::{boxed, forge_land};
    let (scratch, paths) = initialized_deployment(case);
    configure(&paths);
    // Stop at the sign-in line: no person signs in or starts the playground.
    // There is no login and the configured executable does not exist.
    let outcome = boxed::run_command(
        Command {
            program: skein_scratch::paths::binary("temper-box"),
            arguments: std::iter::once("serve".into())
                .chain(forge_land::deployment_flags(&paths).into_iter().map(Into::into))
                .collect(),
            directory: scratch.path().into(),
            environment: Vec::new(),
        },
        None,
        boxed::Plan::Stop,
        Duration::from_secs(10), // Startup and pipe drain only, within the focused suite's 30 s.
    );
    let stderr = String::from_utf8_lossy(&outcome.stderr);
    assert!(support::printed_link(&outcome.stdout).is_some(), "serve reaches its sign-in line: {stderr}");
    assert_eq!(outcome.exit, Some(Exit::Code(0)), "serve drains: {stderr}");
    assert_eq!(outcome.left, Some(vec![]), "no surviving children: {stderr}");
    if paths[1].join("programs").exists() {
        assert!(std::fs::read_dir(paths[1].join("programs")).expect("private programs").next().is_none());
    }
    scratch.remove();
}

fn initialized_deployment(case: &str) -> (support::Scratch, [std::path::PathBuf; 3]) {
    let scratch = support::Scratch::new(case);
    let paths = ["config", "data", "cache"].map(|name| scratch.path().join(name));
    support::forge_land::initialize(&paths);
    support::own_listener_files(&paths[0]);
    (scratch, paths)
}

fn configure_claude(paths: &[std::path::PathBuf; 3]) {
    use std::{fs, os::unix::fs::DirBuilderExt, path::Path};
    fs::DirBuilder::new().mode(0o700).create(paths[1].join("programs")).expect("fresh private programs");
    support::forge_land::configure_live_model(
        &paths[0],
        support::forge_land::LiveModel::ClaudeCode { login: &paths[1].join("programs/claude-login") },
        Path::new("/usr/bin/true"),
    );
}

#[test]
fn a_claude_configuration_replaces_openai_and_keeps_the_demo() {
    // docs/help/box.md, step 1: Claude's fields over init's endpoint and model sizes.
    let (scratch, paths) = initialized_deployment("live-claude-configuration");
    let config = &paths[0];
    let agent_before = temper_agent_fixtures::read(config);
    let before: Value = serde_json::from_slice(
        &support::read_start(&config.join(temper_config_shell::FILE), temper_config_shell::Shape::Standalone).document,
    )
    .expect("JSON");
    configure_claude(&paths);
    let agent = temper_agent_fixtures::read(config);
    let value: Value = serde_json::from_slice(
        &support::read_start(&config.join(temper_config_shell::FILE), temper_config_shell::Shape::Standalone).document,
    )
    .expect("JSON");
    assert_eq!(
        agent["programs"],
        json!({"claude_code":{
            "path":"fetched", "directory":paths[1].join("programs")
        }})
    );
    assert_eq!(
        agent["endpoints"],
        json!([
            {"name":"claude", "number":2, "dialect":2, "provider":"anthropic", "program":"claude_code",
                "address":"api.anthropic.com:443", "transport":"tls", "server_name":"api.anthropic.com",
                "connect":10000, "handshake":10000, "headers":[],
                "limits": {
                    "window":1_048_576, "output":48_000, "reasoning_item":65536, "head":60000, "idle":300_000,"oversized_reasoning":"fail"}},
            agent_before["endpoints"].as_array().unwrap().iter().find(|endpoint| endpoint["name"] == "demo").unwrap()
        ])
    );
    assert_eq!(agent["profile"], agent_before["profile"]);
    assert_eq!(agent["environment"], agent_before["environment"]);
    assert_eq!(
        temper_agent_fixtures::providers::read(&paths[1]),
        json!({
            "providers":[{"name":"anthropic", "number":2, "signed_in_plan":false,
                "manage_usage":"https://claude.ai/settings/usage", "endpoint":"claude",
                "sources":{"programs":["claude_code"]}}],
            "endpoints":[{"name":"claude", "number":2, "dialect":2, "provider":"anthropic", "program":"claude_code",
                "address":{"host":"api.anthropic.com", "port":443}, "transport":"tls", "server_name":"api.anthropic.com",
                "headers":[], "connect_ms":10000, "handshake_ms":10000}]
        })
    );
    let application = before["application"].clone();
    assert_eq!(value["application"], application, "including init’s demo");
    assert_eq!(value["engine"]["limits"], before["engine"]["limits"]);
    assert_eq!(value["workers"], before["workers"]);
    let parsed = temper_worker_shell::agent::decoded_bytes(&serde_json::to_vec(&agent).expect("agent JSON"))
        .expect("ordinary agent schema")
        .resolve(&mut Vec::new())
        .expect("resolved agent");
    assert_eq!(parsed.program_endpoints.into_iter().collect::<Vec<_>>(), [2]);
    eprintln!("Claude agent largest turn: {} bytes", parsed.service.limits.channel.bodies.turn_body);
    scratch.remove();
}

#[test]
fn stock_charters_keep_seven_models_and_choose_the_boxs_scripted_default() {
    let (scratch, paths) = initialized_deployment("live-claude-charters");
    configure_claude(&paths);
    let deployment =
        support::read_start(&paths[0].join(temper_config_shell::FILE), temper_config_shell::Shape::Standalone).document;
    let document = jig_protocol_config::document(
        &deployment,
        &jig_protocol_config::DocumentLimits { bytes: 1 << 20, tokens: 1 << 20, depth: 32 },
    )
    .expect("document");
    let at = jig_protocol_config::read::Object::new(&document, 0)
        .expect("object")
        .member(&document, b"engine")
        .expect("engine");
    let given = temper_service::stock::Given::Stock.with_providers(
        temper_agent_fixtures::providers::values(&temper_agent_fixtures::providers::read(&paths[1])),
        Box::new([temper_service::stock::scripted(([127, 0, 0, 1], 8370).into())]),
    );
    let engine = temper_service::startup::engine_with(
        &document,
        at,
        &jig_directories(&support::configuration::directories(&paths[0])),
        &given,
    )
    .expect("engine schema");
    let defaults = given.defaults(&engine.llm, engine.limits.core.tasks.result_bytes).expect("stock");
    for charter in &defaults.charters {
        assert_eq!((charter.policy.model.name.as_ref(), charter.policy.model.endpoint), (b"demo".as_slice(), 3));
        let resolved = jig_domain_core::settings::charters::resolved(charter, &defaults.models, &engine.limits.core)
            .expect("stock charter");
        let mut alternatives: Vec<_> =
            resolved.policy.alternatives.iter().map(|model| (model.name.as_ref(), model.endpoint)).collect();
        let mut ordinary: Vec<_> = defaults
            .models
            .iter()
            .filter(|entry| entry.number != 8)
            .map(|entry| (entry.model.name.as_ref(), entry.model.endpoint))
            .collect();
        assert_eq!(ordinary.len(), 7);
        alternatives.sort();
        ordinary.sort();
        assert_eq!(alternatives, ordinary);
    }
    scratch.remove();
}

/// The Claude deployment's stock catalogue, endpoints, providers and charters 1 and 2
/// on the joined client world's engine (temper/docs/design/testing/live-tier.md and temper/docs/design/testing/checks.md, 3).
fn live_model_world(
    case: &str,
    configure: fn(&[std::path::PathBuf; 3]),
) -> (
    support::Scratch,
    jig_client_kit::harness::World<temper_client_world::app::Temper, temper_client_world::front::Joint>,
) {
    use jig_client_kit::harness::{Settings, World};
    use temper_client_world::{
        fixtures,
        front::{Joint, engine_limits},
    };
    use temper_engine_domain_world::{store::Store, walking};

    let (scratch, paths) = initialized_deployment(case);
    configure(&paths);
    let deployment =
        support::read_start(&paths[0].join(temper_config_shell::FILE), temper_config_shell::Shape::Standalone).document;
    let document = jig_protocol_config::document(
        &deployment,
        &jig_protocol_config::DocumentLimits { bytes: 1 << 20, tokens: 1 << 20, depth: 32 },
    )
    .expect("document");
    let at = jig_protocol_config::read::Object::new(&document, 0)
        .expect("object")
        .member(&document, b"engine")
        .expect("engine");
    let given = temper_service::stock::Given::Stock.with_providers(
        temper_agent_fixtures::providers::values(&temper_agent_fixtures::providers::read(&paths[1])),
        Box::new([temper_service::stock::scripted(([127, 0, 0, 1], 8370).into())]),
    );
    let engine = temper_service::startup::engine_with(
        &document,
        at,
        &jig_directories(&support::configuration::directories(&paths[0])),
        &given,
    )
    .expect("engine schema");
    let stock = temper_service::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    let mut limits = engine_limits();
    limits.people.initial_owners = 2;
    limits.accounts.accounts = 3;
    limits.accounts.holder_accounts = 3;
    limits.authority.accounts = 3;
    limits.fleet.grants = 4;
    // Init's OpenAI provider can read a list for the world's fixture accounts.
    limits.fleet.reads = limits.accounts.accounts;
    limits.tasks.charters = 2;
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
        config.core.defaults.charters = stock.charters.iter().take(2).cloned().collect();
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
    (scratch, world)
}

#[test]
fn a_fresh_claude_sitting_applies_its_settings_as_a_person_does() {
    use std::path::Path;
    use support::forge_land::LiveModel;
    assert_live_settings(
        "live-claude-settings",
        LiveModel::ClaudeCode { login: Path::new("/held/claude-login") },
        configure_claude,
    );
}

#[test]
fn a_fresh_key_sitting_applies_claude_settings_on_inits_configuration() {
    use std::path::Path;
    use support::forge_land::LiveModel;
    assert_live_settings("live-key-settings", LiveModel::AnthropicKey { key: Path::new("/private/key") }, |_| {});
}

fn assert_live_settings(
    case: &str,
    model: support::forge_land::LiveModel<'_>,
    configure: fn(&[std::path::PathBuf; 3]),
) {
    use std::path::Path;
    use support::forge_land::{LiveModel, live_apply, live_settings};
    use temper_client_world::fixtures;
    let (scratch, mut world) = live_model_world(case, configure);
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
    ];
    let row = read(&mut world, &[b"settings", b"show", b"model", b"3", b"--json"]);
    let changes = live_settings(model, &inventory, [&charters[0], &charters[1]], &row).expect("live settings");
    let claude = live_settings(
        LiveModel::ClaudeCode { login: Path::new("/held/claude-login") },
        &inventory,
        [&charters[0], &charters[1]],
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
    for number in [b"1".as_slice(), b"2"] {
        let charter = read(&mut world, &[b"settings", b"show", b"charter", number, b"--json"]);
        let value = &charter["value"];
        assert_eq!(value["model"]["name"], "claude-haiku-5-5");
        assert_eq!(value["model"]["effort"], "low");
        assert_eq!(value["alternatives"], json!([]));
        assert_eq!(value["rest"], false);
    }
    scratch.remove();
}

fn stubbed_launcher(case: &str, body: &str, source: &str) -> (support::Scratch, std::process::Output) {
    stubbed_story_launcher(case, body, source, "forge-land-claude", false)
}

fn stubbed_story_launcher(
    case: &str,
    body: &str,
    source: &str,
    story: &str,
    default_store: bool,
) -> (support::Scratch, std::process::Output) {
    use std::{fs, os::unix::fs::PermissionsExt};
    let scratch = support::Scratch::new(case);
    let login = scratch.path().join(source);
    fs::create_dir(&login).expect("synthetic held directory");
    fs::set_permissions(&login, fs::Permissions::from_mode(0o700)).expect("private login fixture");
    fs::create_dir(login.join("opaque-entry")).expect("listing fixture only");
    let cargo = scratch.path().join("cargo");
    fs::write(&cargo, format!("#!/bin/sh\n{body}\n")).expect("offline build stub");
    fs::set_permissions(&cargo, fs::Permissions::from_mode(0o700)).expect("executable build stub");
    let mut command = launcher(scratch.path(), story, if story == "forge-land-claude" { Some(&login) } else { None });
    command.env("PATH", format!("{}:/usr/bin:/bin", scratch.path().display()));
    if story == "forge-land-claude-key" {
        let key = scratch.path().join("synthetic-key");
        support::write_private(&key, b"opaque fixture");
        command.env(temper_test_env::LIVE_ANTHROPIC_KEY.name, key);
    }
    if default_store {
        command.env_remove(temper_test_env::LIVE_STORE.name);
    }
    let output = command.output().expect("stubbed launcher");
    (scratch, output)
}

#[test]
fn the_launcher_returns_the_login_after_build_failure() {
    // SourceName's inclusive lower and upper lengths (protocol/credentials-llm-accounts.md, 1).
    for source in ["a", "abcdefghijklmnopqrstuvwxyz123456"] {
        let (scratch, output) = stubbed_launcher("live-claude-build-failure", "exit 17", source);
        assert_eq!(output.status.code(), Some(17));
        assert!(
            String::from_utf8_lossy(&output.stdout)
                .contains("CLAUDE LOGIN BACK: box stopped; same private directory, inode and owner; used in place")
        );
        assert!(!String::from_utf8_lossy(&output.stdout).contains("Approve the printed ChatGPT"));
        scratch.remove();
    }
}

#[test]
fn the_launcher_returns_the_login_after_a_signal() {
    let (scratch, output) = stubbed_launcher("live-claude-signal", "kill -TERM \"$PPID\"", "claude-login");
    assert_eq!(output.status.code(), Some(143));
    assert!(String::from_utf8_lossy(&output.stdout).contains("CLAUDE LOGIN BACK:"));
    scratch.remove();
}

#[test]
fn the_launcher_refuses_changed_login_metadata_on_exit() {
    let (scratch, output) =
        stubbed_launcher("live-claude-changed", "chmod 755 \"$TEMPER_TEST_LIVE_CLAUDE\"\nexit 17", "claude-login");
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains("CLAUDE LOGIN NOT BACK:"));
    scratch.remove();
}

#[test]
fn the_launcher_runs_only_the_claude_story_then_returns() {
    let body = "printf '%s\\n' \"$@\" >> \"$HOME/arguments\"\nexit 0";
    let (scratch, output) = stubbed_launcher("live-claude-pass", body, "claude-login");
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).contains("CLAUDE LOGIN BACK:"));
    let arguments = std::fs::read_to_string(scratch.path().join("arguments")).expect("stub arguments");
    assert!(arguments.contains("test(=a_person_asks_claude_code_to_land_a_change_on_the_owned_forge)"));
    scratch.remove();
}

/// The launcher exports the stores' directory and keeps its story's store beneath it.
#[test]
fn the_launcher_gives_each_landing_story_its_own_store() {
    let body = "printf '%s\\n' \"$TEMPER_TEST_LIVE_STORE\" \"$@\" >> \"$HOME/seen\"\nexit 0";
    for (story, test, default_store) in [
        ("forge-land", "a_person_asks_chatgpt_to_land_a_change_on_the_owned_forge", false),
        ("forge-land-claude", "a_person_asks_claude_code_to_land_a_change_on_the_owned_forge", false),
        ("forge-land-claude-key", "a_person_asks_claude_on_an_api_key_to_land_a_change_on_the_owned_forge", false),
        ("forge-land", "a_person_asks_chatgpt_to_land_a_change_on_the_owned_forge", true),
    ] {
        let (scratch, output) =
            stubbed_story_launcher("live-launcher-stores", body, "claude-login", story, default_store);
        assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
        let home = std::fs::canonicalize(scratch.path()).expect("resolved home");
        let stores = home.join(if default_store { "src/rust/tmp/live-store" } else { "store" });
        let stores = stores.to_str().expect("UTF-8 stores");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.lines().any(|line| line == format!("kept store {stores}/{story}")), "{stdout}");
        if story == "forge-land-claude-key" {
            assert!(!stdout.contains("Approve the printed ChatGPT"), "{stdout}");
        }
        let seen = std::fs::read_to_string(scratch.path().join("seen")).expect("stub arguments and stores");
        assert_eq!(
            seen.lines().collect::<Vec<_>>(),
            [
                stores,
                "nextest",
                "run",
                "-p",
                "temper-box",
                "--profile",
                "live-temper",
                "--no-run",
                stores,
                "nextest",
                "run",
                "-p",
                "temper-box",
                "--profile",
                "live-temper",
                "--ignore-default-filter",
                "--run-ignored",
                "only",
                "--no-capture",
                "-E",
                &format!("binary(=live) & test(={test})"),
            ],
            "{story}, default_store={default_store}"
        );
        scratch.remove();
    }
}

/// `TEMPER_TEST_WRAPPER` names one command that runs each of the launcher's builds, cargo's
/// own arguments following it.
#[test]
fn the_launcher_runs_its_builds_through_the_wrapper_it_is_given() {
    use std::{fs, os::unix::fs::PermissionsExt};
    let scratch = support::Scratch::new("live-launcher-wrapper");
    let root = scratch.path();
    for (name, body) in [("cargo", "exit 99"), ("wrap", "printf '%s\\n' \"$@\" >> \"$HOME/seen\"\nexit 0")] {
        let stub = root.join(name);
        support::write_private(&stub, format!("#!/bin/sh\n{body}\n").as_bytes());
        fs::set_permissions(&stub, fs::Permissions::from_mode(0o700)).expect("executable build stub");
    }
    let output = launcher(root, "forge-land", None)
        .env("PATH", format!("{}:/usr/bin:/bin", root.display()))
        .env(temper_test_env::WRAPPER.name, "wrap")
        .output()
        .expect("wrapped launcher");
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    let seen = fs::read_to_string(root.join("seen")).expect("wrapper arguments");
    assert_eq!(
        seen.lines().collect::<Vec<_>>(),
        [
            "cargo",
            "nextest",
            "run",
            "-p",
            "temper-box",
            "--profile",
            "live-temper",
            "--no-run",
            "cargo",
            "nextest",
            "run",
            "-p",
            "temper-box",
            "--profile",
            "live-temper",
            "--ignore-default-filter",
            "--run-ignored",
            "only",
            "--no-capture",
            "-E",
            "binary(=live) & test(=a_person_asks_chatgpt_to_land_a_change_on_the_owned_forge)",
        ]
    );
    scratch.remove();
}

/// testing/process-tier.md: the offline driver proofs run in the process tier.
#[test]
fn the_offline_rung_runs_its_proofs_in_the_process_profile() {
    use std::{fs, os::unix::fs::PermissionsExt};
    let scratch = support::Scratch::new("offline-rung-wrapper");
    let root = scratch.path();
    let stub = root.join("wrap");
    support::write_private(
        &stub,
        b"#!/bin/sh\n[ \"$NEXTEST_DOUBLE_SPAWN\" = 0 ] || exit 98\nprintf '%s\\n' \"$@\" > \"$HOME/seen\"\nexit 0\n",
    );
    fs::set_permissions(&stub, fs::Permissions::from_mode(0o700)).expect("executable wrapper stub");
    let output = launcher(root, "rung-b0-offline", None)
        .env("PATH", format!("{}:/usr/bin:/bin", root.display()))
        .env(temper_test_env::WRAPPER.name, "wrap")
        .output()
        .expect("wrapped offline launcher");
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    let seen = fs::read_to_string(root.join("seen")).expect("wrapper arguments");
    assert_eq!(
        seen.lines().collect::<Vec<_>>(),
        [
            "cargo",
            "nextest",
            "run",
            "-p",
            "temper-box",
            "--test",
            "proof_live",
            "--profile",
            "process-temper",
            "--no-fail-fast"
        ]
    );
    scratch.remove();
}

struct HeldBox {
    scratch: support::Scratch,
    paths: [std::path::PathBuf; 3],
    record: std::path::PathBuf,
    stub: std::path::PathBuf,
    starts: usize,
    returned_before_stop: Vec<bool>,
}

impl HeldBox {
    fn new(case: &str) -> Self {
        use std::{fs, os::unix::fs::PermissionsExt};
        let scratch = support::Scratch::new(case);
        let paths = ["config", "data", "cache"].map(|name| scratch.path().join(name));
        let record = scratch.path().join("record");
        support::write_private(&record, b"synthetic fixture");
        let stub = scratch.path().join("ctl");
        support::write_private(
            &stub,
            br#"#!/bin/sh
set -eu
root=$(dirname "$0")
test -e "$root/running"
test "$2" = "$(cat "$root/running")/client"
case "$4" in
import) mv "$5" "$root/inside" ;;
list) printf '%s\n' '{"accounts":{"party":[{"provider":1,"plan":true,"number":7}]}}' ;;
export) test "$5" = 7; mv "$root/inside" "$6" ;;
*) exit 1 ;;
esac
"#,
        );
        fs::set_permissions(&stub, fs::Permissions::from_mode(0o700)).expect("executable fake ctl");
        Self { scratch, paths, record, stub, starts: 0, returned_before_stop: vec![] }
    }

    fn import(plan: &support::forge_land::LivePlan<'_>, stub: &std::path::Path, record: &std::path::Path) {
        let (commands, _) = support::forge_land::live_commands(plan.first, plan.model);
        let import = commands
            .iter()
            .find(|args| args.starts_with(&["account".into(), "import".into()]))
            .expect("held record is imported for each box");
        let words = import.iter().map(String::as_str).collect::<Vec<_>>();
        assert_eq!(words.last(), Some(&if plan.first { "connect-plan" } else { "restart-plan" }));
        let imported = support::ctl::alone(plan.client, plan.held_import(stub, &words));
        assert_eq!(imported.exit, Some(skein_io::kernel::Exit::Code(0)));
        assert!(!record.exists(), "import moves the source into the box");
    }

    fn run(&mut self, first: bool, failed: bool) -> Result<(), &'static str> {
        use support::{
            forge_land::{LiveModel, LivePlan},
            held_login::{End, Ending},
        };
        let root = self.scratch.path();
        let client = root.join(if first { "first" } else { "second" });
        std::fs::create_dir(&client).expect("fake box client");
        support::write_private(&root.join("running"), client.as_os_str().as_encoded_bytes());
        self.starts += 1;
        let plan = LivePlan {
            paths: &self.paths,
            client: &client,
            first,
            kept: false,
            model: LiveModel::ChatGpt { account: Some(&self.record) },
            address: None,
            tasks: None,
            baseline: None,
            roots: None,
            words: "",
            line: "",
        };
        plan.install_return_hook(&self.stub).expect("install return hook");
        Self::import(&plan, &self.stub, &self.record);
        let now = skein_lib::Time::ZERO;
        let mut ending = Ending::new(true, first, failed, now);
        let mut exit = None;
        loop {
            match ending.next(now, true, exit) {
                End::Return => {
                    exit = Some(support::ctl::alone(&client, plan.return_call()).exit);
                }
                End::Stop(failure) => {
                    use std::os::unix::fs::PermissionsExt;
                    self.returned_before_stop.push(std::fs::symlink_metadata(&self.record).is_ok_and(|file| {
                        file.is_file() && file.len() > 0 && file.permissions().mode() & 0o777 == 0o600
                    }));
                    std::fs::remove_file(root.join("running")).expect("stop fake box");
                    return failure.or(failed.then_some("run failed")).map_or(Ok(()), Err);
                }
                End::Wait => panic!("fake ctl already ended"),
            }
        }
    }
}

/// Both successful boxes return before stopping; failures in either do the same.
#[test]
fn a_held_login_returns_on_restart_and_failures_in_either_box() {
    for failure in [None, Some(true), Some(false)] {
        let mut boxes = HeldBox::new("held-restart-endings");
        let first = boxes.run(true, failure == Some(true));
        assert_eq!(first.is_err(), failure == Some(true));
        assert_eq!(boxes.returned_before_stop, [true], "first box returns before stopping");
        if first.is_ok() {
            assert_eq!(boxes.run(false, failure == Some(false)).is_err(), failure == Some(false));
        }
        assert!(boxes.returned_before_stop.iter().all(|returned| *returned), "failure={failure:?}");
        assert!(boxes.record.is_file());
        boxes.scratch.remove();
    }
}

/// A terminal give-back wins over hook installation and every later import.
#[test]
fn terminal_login_cleanup_excludes_a_restart_import() {
    use support::forge_land::{LiveModel, LivePlan};
    let mut boxes = HeldBox::new("held-terminal-return");
    boxes.run(true, false).expect("first box returns");
    let root = boxes.scratch.path();
    let terminal = support::ctl::alone(root, support::ctl::Call::command(&root.join("give-back"), &[]));
    assert_eq!(terminal.exit, Some(skein_io::kernel::Exit::Code(0)));
    let client = root.join("second");
    let plan = LivePlan {
        paths: &boxes.paths,
        client: &client,
        first: false,
        kept: false,
        model: LiveModel::ChatGpt { account: Some(&boxes.record) },
        address: None,
        tasks: None,
        baseline: None,
        roots: None,
        words: "",
        line: "",
    };
    let installed = plan.install_return_hook(&boxes.stub);
    assert_eq!(installed, Err("terminal return excludes another import".into()));
    let words = ["account", "import", boxes.record.to_str().expect("record path"), "--key", "restart-plan"];
    let imported = support::ctl::alone(root, plan.held_import(&boxes.stub, &words));
    assert_eq!(imported.exit, Some(skein_io::kernel::Exit::Code(1)));
    assert!(boxes.record.is_file());
    assert_eq!(boxes.starts, 1);
    boxes.scratch.remove();
}

fn held_launcher(case: &str, story: &str, body: &str) -> (support::Scratch, std::process::Output) {
    use std::{fs, os::unix::fs::PermissionsExt};
    let scratch = support::Scratch::new(case);
    let root = scratch.path();
    let account = root.join("synthetic-record");
    support::write_private(&account, b"synthetic fixture");
    let cargo = root.join("cargo");
    support::write_private(
        &cargo,
        format!(
            r#"#!/bin/sh
set -eu
printf '%s :: %s\n' "$TEMPER_TEST_LIVE_STORE" "$*" >> "$HOME/cargo-calls"
{body}
"#
        )
        .as_bytes(),
    );
    fs::set_permissions(&cargo, fs::Permissions::from_mode(0o700)).expect("executable build stub");
    let output = launcher(root, story, None)
        .env(temper_test_env::LIVE_ACCOUNT.name, &account)
        .env("PATH", format!("{}:/usr/bin:/bin", root.display()))
        .output()
        .expect("held-login launcher");
    (scratch, output)
}

#[test]
fn the_launcher_returns_the_ordinary_held_login_after_build_failure() {
    for story in ["forge-land", "rung-b0"] {
        let body = r#"mkdir -p "$HOME/STORY_STORE"
printf '#!/bin/sh\ntouch "$HOME/hook-ran"\n' > "$HOME/STORY_STORE/give-back"
chmod 700 "$HOME/STORY_STORE/give-back"
exit 17"#;
        let body = body.replace("STORY_STORE", if story == "forge-land" { "store/forge-land" } else { "store" });
        let (scratch, output) = held_launcher("held-build-failure", story, &body);
        assert_eq!(output.status.code(), Some(17), "{}", String::from_utf8_lossy(&output.stderr));
        assert!(String::from_utf8_lossy(&output.stdout).contains("LOGIN BACK: regular file mode=600"));
        let calls = std::fs::read_to_string(scratch.path().join("cargo-calls")).expect("cargo calls");
        assert_eq!(calls.lines().count(), 1, "a record at rest needs no return test");
        assert!(scratch.path().join("synthetic-record").is_file());
        assert!(!scratch.path().join("hook-ran").exists(), "a record at rest needs no hook");
        scratch.remove();
    }
}

#[test]
fn the_launcher_runs_the_ordinary_return_hook_before_stopping_after_a_signal() {
    for story in ["forge-land", "rung-b0"] {
        // The first cargo call represents the build; the second imports a
        // synthetic record and installs the same hook contract as the story.
        let body = r#"case " $* " in *" --no-run "*) exit 0 ;; esac
mkdir -p "$TEMPER_TEST_LIVE_STORE"
mv "$TEMPER_TEST_LIVE_ACCOUNT" "$TEMPER_TEST_LIVE_STORE/synthetic-import"
cat > "$TEMPER_TEST_LIVE_STORE/give-back" <<'HOOK'
#!/bin/sh
set -eu
test ! -d "/proc/$(cat "$TEMPER_TEST_LIVE_STORE/runner")"
kill -0 "$(cat "$TEMPER_TEST_LIVE_STORE/box")"
mv "$TEMPER_TEST_LIVE_STORE/synthetic-import" "$TEMPER_TEST_LIVE_ACCOUNT"
printf '%s\n' 'HOOK RETURNED'
HOOK
chmod 700 "$TEMPER_TEST_LIVE_STORE/give-back"
printf '%s\n' "$$" > "$TEMPER_TEST_LIVE_STORE/runner"
for directory in "$TMPDIR/owned" "$TEMPER_TEST_LIVE_STORE/owned"; do
    mkdir -p "$directory"
    (cd "$directory"; exec sleep 30) >/dev/null 2>&1 &
    printf '%s\n' "$!" >> "$HOME/owned-children"
    printf '%s\n' "$!" > "$TEMPER_TEST_LIVE_STORE/box"
done
kill -TERM "$PPID"
# The launcher must end and reap this PID before running the hook.
exec sleep 30"#;
        let body = body.replace(
            "$TEMPER_TEST_LIVE_STORE",
            if story == "forge-land" { "$TEMPER_TEST_LIVE_STORE/forge-land" } else { "$TEMPER_TEST_LIVE_STORE" },
        );
        let (scratch, output) = held_launcher("held-signal", story, &body);
        assert_eq!(output.status.code(), Some(143), "{}", String::from_utf8_lossy(&output.stderr));
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("HOOK RETURNED"), "{stdout}");
        assert!(stdout.contains("LOGIN BACK: regular file mode=600"), "{stdout}");
        let calls = std::fs::read_to_string(scratch.path().join("cargo-calls")).expect("cargo calls");
        assert_eq!(calls.lines().count(), 2, "the installed hook needs no new box");
        assert_launcher_children_stopped(scratch.path());
        scratch.remove();
    }
}

fn assert_launcher_children_stopped(root: &std::path::Path) {
    let children = std::fs::read_to_string(root.join("owned-children")).expect("owned stub children");
    let mut survivors = Vec::new();
    for pid in children.lines() {
        let running = std::process::Command::new("kill")
            .args(["-0", pid])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .expect("owned stub status")
            .success();
        if running {
            let _cleanup = std::process::Command::new("kill").args(["-KILL", pid]).status();
            survivors.push(pid);
        }
    }
    assert!(survivors.is_empty(), "the sitting stops its scratch and store descendants: {survivors:?}");
}

#[test]
fn the_launcher_refuses_changed_ordinary_held_login_metadata() {
    use std::os::unix::fs::PermissionsExt;
    for story in ["forge-land", "rung-b0"] {
        let body = r#"mkdir -p "$HOME/STORY_STORE"
printf '#!/bin/sh\ntouch "$HOME/hook-ran"\n' > "$HOME/STORY_STORE/give-back"
chmod 700 "$HOME/STORY_STORE/give-back"
chmod 644 "$TEMPER_TEST_LIVE_ACCOUNT"
exit 17"#;
        let body = body.replace("STORY_STORE", if story == "forge-land" { "store/forge-land" } else { "store" });
        let (scratch, output) = held_launcher("held-metadata", story, &body);
        assert_eq!(output.status.code(), Some(1));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("LOGIN NOT BACK:"), "{stderr}");
        assert!(!stderr.contains("synthetic-record"), "only the store is named");
        let metadata = std::fs::symlink_metadata(scratch.path().join("synthetic-record")).expect("unchanged record");
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o644);
        assert_eq!(metadata.len(), u64::try_from(b"synthetic fixture".len()).expect("fixture length"));
        assert!(!scratch.path().join("hook-ran").exists());
        let calls = std::fs::read_to_string(scratch.path().join("cargo-calls")).expect("cargo calls");
        assert_eq!(calls.lines().count(), 1, "unsafe metadata runs no return test");
        scratch.remove();
    }
}

#[test]
fn the_launcher_records_a_failed_ordinary_return_hook() {
    for story in ["forge-land", "rung-b0"] {
        for returns in [true, false] {
            let body = r#"case " $* " in *" --no-run "*) exit 0 ;; esac
case " $* " in
*the_held_record_returns_on_a_box_of_its_own*)
    test "$TEMPER_TEST_LIVE_STORE" = "$HOME/STORY_STORE"
    test -f "$TEMPER_TEST_LIVE_STORE/hook-ran"
    test "$TMPDIR" != "$HOME"
    mkdir -p "$TMPDIR/return-box"
    (cd "$TMPDIR/return-box"; exec sleep 30) >/dev/null 2>&1 &
    printf '%s\n' "$!" >> "$HOME/owned-children"
    RETURN
    ;;
esac
mkdir -p "$HOME/STORY_STORE"
mv "$TEMPER_TEST_LIVE_ACCOUNT" "$HOME/STORY_STORE/synthetic-import"
cat > "$HOME/STORY_STORE/give-back" <<'HOOK'
#!/bin/sh
touch "$HOME/STORY_STORE/hook-ran"
exit 9
HOOK
chmod 700 "$HOME/STORY_STORE/give-back"
for directory in "$TMPDIR/owned" "$HOME/STORY_STORE/owned"; do
    mkdir -p "$directory"
    (cd "$directory"; exec sleep 30) >/dev/null 2>&1 &
    printf '%s\n' "$!" >> "$HOME/owned-children"
done
exit 17"#;
            let story_store = if story == "forge-land" { "store/forge-land" } else { "store" };
            let body = body.replace("STORY_STORE", story_store).replace(
                "    RETURN",
                if returns {
                    "    mv \"$TEMPER_TEST_LIVE_STORE/synthetic-import\" \"$TEMPER_TEST_LIVE_ACCOUNT\"; exit 0"
                } else {
                    "    exit 19"
                },
            );
            let (scratch, output) = held_launcher("held-return-failure", story, &body);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert_eq!(output.status.code(), Some(if returns { 17 } else { 1 }), "{stdout}\n{stderr}");
            let store = scratch.path().join(story_store);
            let calls = std::fs::read_to_string(scratch.path().join("cargo-calls")).expect("cargo calls");
            assert_eq!(calls.lines().count(), 3, "build, story, then the return: {calls}");
            let returned = calls.lines().nth(2).expect("return invocation");
            assert!(returned.starts_with(&format!("{} :: ", store.display())), "{returned}");
            assert!(returned.contains("--ignore-default-filter --run-ignored only"), "{returned}");
            assert!(
                returned.ends_with("binary(=live) & test(=the_held_record_returns_on_a_box_of_its_own)"),
                "{returned}"
            );
            if returns {
                assert!(stdout.contains("LOGIN BACK: regular file mode=600"), "{stdout}");
            } else {
                assert!(stderr.contains(&support::held_login::not_back("final stat failed", &store)), "{stderr}");
                assert!(
                    !stdout.contains("synthetic-record") && !stderr.contains("synthetic-record"),
                    "only the store is named"
                );
                assert!(!scratch.path().join("synthetic-record").exists(), "no record created");
                assert!(store.join("synthetic-import").is_file(), "import retained");
            }
            assert!(store.join("give-back").is_file(), "store retained for recovery");
            assert_launcher_children_stopped(scratch.path());
            scratch.remove();
        }
    }
}

#[test]
fn a_held_chatgpt_record_is_admitted_only_as_a_private_regular_file_never_named() {
    use std::{fs, os::unix::fs::PermissionsExt};
    use support::forge_land::held_account;
    let scratch = support::Scratch::new("held-account");
    let file = scratch.path().join("record");
    assert_eq!(held_account(&file), Err("TEMPER_TEST_LIVE_ACCOUNT names no file"));
    fs::write(&file, b"record").expect("record");
    for (mode, admitted) in [
        (0o600, Ok(())),
        (0o640, Err("TEMPER_TEST_LIVE_ACCOUNT is not 0600")),
        (0o644, Err("TEMPER_TEST_LIVE_ACCOUNT is not 0600")),
        (0o400, Err("TEMPER_TEST_LIVE_ACCOUNT is not 0600")),
        (0o700, Err("TEMPER_TEST_LIVE_ACCOUNT is not 0600")),
    ] {
        fs::set_permissions(&file, fs::Permissions::from_mode(mode)).expect("mode");
        assert_eq!(held_account(&file), admitted, "mode {mode:o}");
    }
    let link = scratch.path().join("link");
    std::os::unix::fs::symlink(&file, &link).expect("link");
    assert_eq!(held_account(&link), Err("TEMPER_TEST_LIVE_ACCOUNT is not a regular file"));
    assert_eq!(held_account(scratch.path()), Err("TEMPER_TEST_LIVE_ACCOUNT is not a regular file"));
    scratch.remove();
}

/// testing/live-tier.md: one `TEMPER_TEST_LIVE_STORE` holds a store per landing story.
#[test]
fn each_landing_story_takes_its_own_store_under_one_live_store() {
    use std::{fs, os::unix::fs::PermissionsExt, path::Path};
    use support::forge_land::{LiveModel, story_store};
    let scratch = support::Scratch::new("live-story-stores");
    let stores = scratch.path().join("parent/stores");
    let chatgpt = LiveModel::ChatGpt { account: None };
    let claude = LiveModel::ClaudeCode { login: Path::new("/held/claude-login") };
    let chatgpt_store = stores.join("forge-land");
    assert_eq!(story_store(&stores, chatgpt), Ok((chatgpt_store.clone(), false)));
    for directory in [stores.parent().expect("stores parent"), &stores, &chatgpt_store] {
        let metadata = fs::metadata(directory).expect("private directory");
        assert!(metadata.is_dir());
        assert_eq!(metadata.permissions().mode() & 0o777, 0o700);
    }
    fs::write(chatgpt_store.join("begun"), b"deployment begun").expect("begun deployment");
    assert_eq!(story_store(&stores, claude), Ok((stores.join("forge-land-claude"), false)));
    let key = LiveModel::AnthropicKey { key: Path::new("/private/key") };
    let key_store = stores.join("forge-land-claude-key");
    assert_eq!(story_store(&stores, key), Ok((key_store.clone(), false)));
    assert_eq!(story_store(&stores, key), Ok((key_store.clone(), false)), "an empty store is fresh");
    fs::write(key_store.join("begun"), b"deployment begun").expect("begun key deployment");
    assert_eq!(story_store(&stores, key), Err(format!("kept store {} lacks config", key_store.display())));
    let missing = format!("kept store {} lacks config", chatgpt_store.display());
    assert_eq!(story_store(&stores, chatgpt), Err(missing.clone()));
    assert!(!chatgpt_store.join("config").exists(), "a kept store is never repaired");
    assert_eq!(story_store(&stores, LiveModel::ChatGpt { account: Some(Path::new("/held/record")) }), Err(missing));
    scratch.remove();
}

/// testing/live-tier.md: the held record's import and give-back name the sitting's
/// client directory, never the run's root.
#[test]
fn the_held_records_import_and_give_back_name_the_sittings_client_directory() {
    use std::{fs, os::unix::fs::PermissionsExt};
    use support::forge_land::{LiveModel, LivePlan};
    let scratch = support::Scratch::new("held-login-client");
    let store = scratch.path().join("stores/forge-land");
    let paths = ["config", "data", "cache"].map(|name| store.join(name));
    let run = store.join("first");
    let client = run.join("client");
    let held = scratch.path().join("held");
    for directory in [store.parent().expect("stores"), &store, &paths[0], &paths[1], &paths[2], &run, &client, &held] {
        fs::create_dir(directory).expect("fixture directory");
        fs::set_permissions(directory, fs::Permissions::from_mode(0o700)).expect("private fixture directory");
    }
    let record = held.join("record");
    support::write_private(&record, b"synthetic fixture");
    let stub = scratch.path().join("ctl-stub's path");
    let seen = scratch.path().join("seen");
    support::write_private(
        &stub,
        br#"#!/bin/sh
set -eu
printf '%s\n' "$@" >> "$(dirname "$0")/seen"
case "$4" in
import) ;;
list) printf '%s\n' '{"accounts":{"party":[{"provider":1,"plan":true,"number":7}]}}' ;;
export) test "$5" = 7; umask 077; printf '%s' synthetic > "$6" ;;
*) exit 1 ;;
esac
"#,
    );
    fs::set_permissions(&stub, fs::Permissions::from_mode(0o700)).expect("executable synthetic ctl");
    let plan = LivePlan {
        paths: &paths,
        client: &run,
        first: true,
        kept: false,
        model: LiveModel::ChatGpt { account: Some(&record) },
        address: None,
        tasks: None,
        baseline: None,
        roots: None,
        words: "",
        line: "",
    };
    assert_eq!(plan.client_directory(), client);
    let arguments = ["account", "import", record.to_str().expect("record path"), "--key", "connect-plan"];
    let imported = support::ctl::alone(&run, plan.held_import(&stub, &arguments));
    assert_eq!(imported.exit, Some(skein_io::kernel::Exit::Code(0)));
    let imported = fs::read_to_string(&seen).expect("import arguments");
    assert_eq!(imported.lines().nth(1), plan.client_directory().to_str(), "import's DIRECTORY");
    assert_eq!(imported, format!("ctl\n{}\n{}\n", client.display(), arguments.join("\n")));
    fs::remove_file(&seen).expect("clear import arguments");
    plan.install_return_hook(&stub).expect("install return hook");
    fs::remove_file(&record).expect("synthetic import removes the record");
    let returned = std::process::Command::new(store.join("give-back"))
        .stdin(std::process::Stdio::null())
        .output()
        .expect("return hook");
    assert!(returned.status.success(), "{}", String::from_utf8_lossy(&returned.stderr));
    assert_eq!(
        fs::read_to_string(&seen).expect("give-back arguments"),
        format!(
            "ctl\n{client}\naccount\nlist\n--json\nctl\n{client}\naccount\nexport\n7\n{record}\n",
            client = client.display(),
            record = record.display(),
        )
    );
    let returned = fs::symlink_metadata(&record).expect("returned synthetic record");
    assert!(returned.is_file() && returned.len() > 0);
    assert_eq!(returned.permissions().mode() & 0o777, 0o600);
    scratch.remove();
}

#[test]
fn setup_accepts_a_project_free_first_repository_row() {
    use skein_io::kernel::Exit;
    let scratch = support::Scratch::new("setup-project-free-row");
    let row = json!({"forges":[{
        "number":1,"host":"git.example:3000","mode":"owned","state":"connected",
        "first":{"state":"made","org":"work","repository":"hello","id":1}
    }]});
    let mut calls = [
        String::new(),
        json!({"forges":[]}).to_string(),
        "setting up\nproject: 1 · hello\nAdopted work/hello on forge 1 as owned.\nIt is the project's home.\n"
            .to_owned(),
        String::new(),
        row.to_string(),
        "forge 1 · host: git.example:3000 · Forgejo · owned · Connected as temper (2)".to_owned(),
    ]
    .map(|stdout| support::Ended { exit: Some(Exit::Code(0)), stdout: stdout.into_bytes(), stderr: vec![] });
    assert_eq!(
        support::forge_land::check_forgejo_setup_after_sign_in(
            "first",
            &calls,
            "forgejo-1: started\nrunner-1: started",
            None,
            scratch.path()
        ),
        Ok(2)
    );
    let mut other_host = row;
    other_host["forges"][0]["host"] = json!("other.example:3000");
    calls[4].stdout = other_host.to_string().into_bytes();
    let finding = support::forge_land::check_forgejo_setup_after_sign_in(
        "first",
        &calls,
        "forgejo-1: started\nrunner-1: started",
        None,
        scratch.path(),
    )
    .expect_err("the text row must name the JSON row's resource host");
    assert_eq!(
        finding,
        "first: forges row: forge 1 · host: git.example:3000 · Forgejo · owned · Connected as temper (2); box: forgejo-1: started\nrunner-1: started"
    );
    scratch.remove();
}

#[test]
fn an_anthropic_key_is_admitted_by_metadata_without_naming_its_path() {
    use std::{fs, os::unix::fs::PermissionsExt};
    use support::forge_land::anthropic_key;
    let scratch = support::Scratch::new("live-key-admission");
    let file = scratch.path().join("synthetic-key");
    assert_eq!(anthropic_key(None), Ok(None), "unset skips before any process");
    assert_eq!(anthropic_key(Some(file.clone().into())), Err("TEMPER_TEST_LIVE_ANTHROPIC_KEY names no file"));
    support::write_private(&file, b"opaque fixture");
    assert_eq!(anthropic_key(Some(file.clone().into())), Ok(Some(file.clone())));
    fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).expect("public fixture");
    assert_eq!(anthropic_key(Some(file.clone().into())), Err("TEMPER_TEST_LIVE_ANTHROPIC_KEY is not 0600"));
    fs::set_permissions(&file, fs::Permissions::from_mode(0o600)).expect("private fixture");
    let link = scratch.path().join("link");
    std::os::unix::fs::symlink(&file, &link).expect("link to good fixture");
    for path in [&link, scratch.path()] {
        assert_eq!(
            anthropic_key(Some(path.into())),
            Err("TEMPER_TEST_LIVE_ANTHROPIC_KEY is not a regular file or is a link")
        );
    }
    fs::write(&file, b"").expect("empty fixture");
    assert_eq!(anthropic_key(Some(file.into())), Err("TEMPER_TEST_LIVE_ANTHROPIC_KEY is empty"));
    scratch.remove();
}

#[test]
fn a_key_configuration_starts_as_init_writes_it() {
    // Provider 2's owner edit belongs to the limits bootstrap, not the init files.
    assert_live_startup("live-key-startup", |_| {});
}

#[test]
fn the_key_launcher_refuses_invalid_files_and_other_held_inputs() {
    use std::{fs, os::unix::fs::PermissionsExt};
    let scratch = support::Scratch::new("live-key-launcher-refusals");
    let root = scratch.path();
    let good = root.join("good");
    support::write_private(&good, b"opaque fixture");
    support::write_private(&root.join("public"), b"opaque fixture");
    fs::set_permissions(root.join("public"), fs::Permissions::from_mode(0o644)).expect("public fixture");
    support::write_private(&root.join("empty"), b"");
    std::os::unix::fs::symlink(&good, root.join("link")).expect("link to good fixture");
    for (file, extra, message) in [
        (None, None, "TEMPER_TEST_LIVE_ANTHROPIC_KEY is required for forge-land-claude-key"),
        (Some("missing"), None, "TEMPER_TEST_LIVE_ANTHROPIC_KEY names no file"),
        (Some("public"), None, "TEMPER_TEST_LIVE_ANTHROPIC_KEY is not 0600"),
        (Some("empty"), None, "TEMPER_TEST_LIVE_ANTHROPIC_KEY is empty"),
        (Some("link"), None, "TEMPER_TEST_LIVE_ANTHROPIC_KEY is not a regular file or is a link"),
        (Some("."), None, "TEMPER_TEST_LIVE_ANTHROPIC_KEY is not a regular file or is a link"),
        (Some("good"), Some(temper_test_env::LIVE_ACCOUNT.name), "TEMPER_TEST_LIVE_ACCOUNT is for forge-land"),
        (Some("good"), Some(temper_test_env::LIVE_CLAUDE.name), "TEMPER_TEST_LIVE_CLAUDE is for forge-land-claude"),
    ] {
        let mut command = launcher(root, "forge-land-claude-key", None);
        if let Some(file) = file {
            command.env(temper_test_env::LIVE_ANTHROPIC_KEY.name, root.join(file));
        }
        if let Some(extra) = extra {
            command.env(extra, &good);
        }
        let output = command.output().expect("offline key launcher refusal");
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(String::from_utf8_lossy(&output.stderr).trim(), message);
    }
    scratch.remove();
}

#[test]
fn a_key_store_is_kept_without_a_held_record() {
    use skein_forge::forgejo::pin::{RUNNER_VERSION, VERSION};
    use std::{fs, path::Path};
    use support::forge_land::{LiveModel, story_store};
    let scratch = support::Scratch::new("live-kept-key-store");
    let stores = scratch.path().join("stores");
    let model = LiveModel::AnthropicKey { key: Path::new("/private/synthetic-key") };
    let (store, kept) = story_store(&stores, model).expect("fresh key store");
    assert!(!kept);
    for directory in ["data/deployment", "data/forge-1/forgejo", "config", "data/box/started"] {
        fs::create_dir_all(store.join(directory)).expect("synthetic deployment directory");
    }
    for file in [
        format!("config/{}", temper_config_shell::FILE),
        "data/box/started/1".to_owned(),
        "data/forge-1/forgejo/app.ini".to_owned(),
        "data/forge-1/forgejo/forgejo.db".to_owned(),
        format!("cache/forgejo/{VERSION}/forgejo"),
        format!("cache/forgejo-runner/{RUNNER_VERSION}/forgejo-runner"),
    ] {
        let file = store.join(file);
        fs::create_dir_all(file.parent().expect("fixture parent")).expect("fixture directories");
        support::write_private(&file, b"synthetic deployment fixture");
    }
    assert_eq!(story_store(&stores, model), Ok((store, true)), "a key has no held record to import again");
    scratch.remove();
}
