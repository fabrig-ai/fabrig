use crate::support;
use temper_box_shell::directories::jig_directories;

use serde_json::{Value, json};
use std::collections::BTreeSet;
use support::forge_land::{last_readme_line, live_roots, require_chat_progress, require_closed_roots};

fn roots_read(tasks: &[u64]) -> Value {
    json!({"tasks": {"rows": tasks.iter().map(|task| json!({"chip": {"task": task}})).collect::<Vec<_>>(), "more": false}})
}

#[test]
fn catalogue_reads_preserve_scripted_call_counts() {
    use skein_fake_llm_domain::api::{Caching, Query, ToolChoice};
    use support::forge_land::{Seen, assert_requests};

    let words = b"the restarted chat";
    let query = Query {
        credential: Some(0),
        caching: Caching::Unscoped,
        model: b"gpt-6-luna".as_slice().into(),
        system: words.as_slice().into(),
        tools: Box::new([]),
        choice: ToolChoice::None,
        messages: Box::new([]),
        max_tokens: 20,
    };
    for lists in [0, 1, 2] {
        let seen = Seen {
            requests: vec![query.clone(), query.clone()],
            answers: vec![Ok(()); 2],
            lists,
            schemes: vec![Some(skein_llm::Scheme::Bearer); 2 + lists],
        };
        assert_requests(&seen, 2, words, false);
    }
}

#[test]
fn a_fresh_sittings_close_leaves_no_live_root() {
    let before = live_roots(&roots_read(&[])).expect("whole fresh roots");
    assert_eq!(before, BTreeSet::new());
    assert_eq!(require_closed_roots(&roots_read(&[]), &before), Ok(()));
    assert_eq!(
        require_closed_roots(&roots_read(&[7]), &before),
        Err("live root tasks this sitting left: [7]; earlier live root tasks missing: []".to_owned())
    );
}

#[test]
fn a_kept_store_keeps_an_earlier_sittings_live_chat() {
    let before = live_roots(&roots_read(&[1])).expect("whole kept roots");
    assert_eq!(before, BTreeSet::from([1]));
    assert_eq!(require_closed_roots(&roots_read(&[1]), &before), Ok(()));
    assert_eq!(
        require_closed_roots(&roots_read(&[1, 4]), &before),
        Err("live root tasks this sitting left: [4]; earlier live root tasks missing: []".to_owned())
    );
}

#[test]
fn a_sitting_never_closes_an_earlier_root() {
    let before = BTreeSet::from([1]);
    assert_eq!(
        require_closed_roots(&roots_read(&[]), &before),
        Err("live root tasks this sitting left: []; earlier live root tasks missing: [1]".to_owned())
    );
}

#[test]
fn a_roots_read_must_be_whole() {
    for read in [
        json!({"tasks": {"rows": [], "more": true}}),
        json!({"tasks": {"more": false}}),
        json!({"tasks": {"rows": [{"chip": {}}], "more": false}}),
    ] {
        assert!(live_roots(&read).is_err(), "{read}");
    }
}

#[test]
fn a_readme_ignores_trailing_empty_lines() {
    for (readme, expected) in [
        ("starter\ntag\n\n", Some("tag")),
        ("tag", Some("tag")),
        ("tag\n", Some("tag")),
        ("tag\n\n\n", Some("tag")),
        ("tag\r\n", Some("tag")),
        ("tag\r\n\r\n", Some("tag")),
        ("", None),
        ("\n", None),
        ("\n\n", None),
        ("\r\n", None),
    ] {
        assert_eq!(last_readme_line(readme.as_bytes()), expected, "README {readme:?}");
    }
}

#[test]
fn a_readme_keeps_its_last_nonempty_line_exactly() {
    for (readme, expected) in [
        ("tag\nother\n\n", "other"),
        ("tag \n\n", "tag "),
        (" tag\n", " tag"),
        ("tag\n  \n", "  "),
        ("tag\n\t\n\n", "\t"),
    ] {
        assert_eq!(last_readme_line(readme.as_bytes()), Some(expected), "README {readme:?}");
    }
}

// The facts read by discovery, in task show --json's owner shape. The encoders live
// in temper-client-tools/src/jig/render/document/{chips,owner_chips,
// owner_authority,tasks,parts,llm}.rs.
fn empty_park() -> Value {
    json!({
        "chip": {"details": {"owner": {
            "phase": {"active": "idle"},
            "details": {"attempt": 1, "live_delegates": 0}
        }}},
        "waits_on": "news",
        "run_wait": null,
        "subscriptions": {"entries": []},
        "cut": false,
        "wake": {
            "words": "immediate", "notices": "immediate", "news": "immediate",
            "results": "last_or_failure", "questions": true, "answers": true, "timers": true
        }
    })
}

#[test]
fn an_empty_park_stops_the_story() {
    assert_eq!(require_chat_progress(&empty_park()), Err("chat parked without delegating a change".to_owned()));
}

fn continues_with(pointer: &str, replacement: Value) {
    let mut task = empty_park();
    *task.pointer_mut(pointer).expect("fixture field") = replacement;
    assert_eq!(require_chat_progress(&task), Ok(()), "{pointer}: {task}");
}

#[test]
fn active_work_and_intentional_waits_continue() {
    continues_with("/chip/details/owner/details/attempt", Value::Null);
    continues_with("/chip/details/owner/details/live_delegates", json!(1));
    for stage in [
        json!("due"),
        json!("preparing"),
        json!({"claimed": {"attempt": 1}}),
        json!({"running": {"attempt": 1}}),
        json!({"backing_off": {"until": 10}}),
    ] {
        continues_with("/chip/details/owner/phase/active", stage);
    }
    let mut running = empty_park();
    running["chip"]["details"]["owner"]["phase"] = json!({"active": {"running": {"attempt": 1}}});
    for work in [
        json!("preparing"),
        json!("starting"),
        json!("active"),
        json!("writing"),
        json!({"tool": {"call": "01", "tool": "02"}}),
        json!("waiting"),
        json!("ending"),
    ] {
        running["chip"]["details"]["owner"]["details"]["observation"] =
            json!({"channel": 1, "work": work, "since": 1, "reported_at": 2, "started": 1});
        assert_eq!(require_chat_progress(&running), Ok(()), "{running}");
    }
    continues_with(
        "/chip/details/owner/phase",
        json!({"waiting": {"dependencies": [2], "more": false, "holds": false}}),
    );
    continues_with(
        "/chip/details/owner/phase",
        json!({"person": {"address": {"party": 1}, "taken_by": null, "since": null}}),
    );
    for waits in [
        json!("nothing"),
        json!("delegates"),
        json!({"dependencies": {"pending": [2], "count": 1}}),
        json!({"party": {"whom": {"person": 1}, "entries": 1, "task": 2, "kind": "question"}}),
    ] {
        continues_with("/waits_on", waits);
    }
    for wait in
        [json!("credential"), json!("host"), json!({"account": {"account": 1, "source": null, "why": "refreshing"}})]
    {
        continues_with("/run_wait", wait);
    }
    for kind in [
        json!({"timer": {"at": 10, "period": null}}),
        json!({"task": {"target": 2, "held": true, "result": true}}),
        json!({"topic": {"connector": 1, "topic": 2}}),
    ] {
        continues_with("/subscriptions/entries", json!([{"number": 1, "kind": kind}]));
    }
    continues_with("/cut", json!(true));
    for key in ["words", "notices", "news"] {
        for rule in [json!("never"), json!({"batch": {"count": 2, "age": 10}}), json!("unknown")] {
            continues_with(&format!("/wake/{key}"), rule);
        }
    }
    for rule in ["never", "each", "unknown"] {
        continues_with("/wake/results", json!(rule));
    }
    for key in ["questions", "answers", "timers"] {
        continues_with(&format!("/wake/{key}"), json!(false));
    }
    for key in ["words", "notices", "news", "results", "questions", "answers", "timers"] {
        let mut task = empty_park();
        task["wake"].as_object_mut().expect("wake object").remove(key);
        assert_eq!(require_chat_progress(&task), Ok(()), "missing wake {key}");
    }
    let mut task = empty_park();
    task["wake"]["future"] = json!(true);
    assert_eq!(require_chat_progress(&task), Ok(()), "unknown wake field");
    continues_with("/wake", Value::Null);
}

#[test]
fn a_terminal_chat_is_not_an_empty_park() {
    for phase in [
        json!({"held": {"reason": "drift", "revision": 1}}),
        json!({"closing": {"run": {"attempt": 1}}}),
        json!({"closing": "delegates"}),
        json!({"closing": "effects"}),
        json!({"closing": "releases"}),
        json!({"closing": "settled"}),
        json!("ended"),
    ] {
        continues_with("/chip/details/owner/phase", phase);
    }
}

#[test]
fn missing_chat_facts_report_the_shape_error() {
    for (parent, key) in [
        ("", "chip"),
        ("/chip", "details"),
        ("/chip/details", "owner"),
        ("/chip/details/owner", "phase"),
        ("/chip/details/owner", "details"),
        ("/chip/details/owner/details", "attempt"),
        ("/chip/details/owner/details", "live_delegates"),
        ("", "waits_on"),
        ("", "run_wait"),
        ("", "subscriptions"),
        ("/subscriptions", "entries"),
        ("", "cut"),
        ("", "wake"),
    ] {
        let mut task = empty_park();
        task.pointer_mut(parent).expect("fixture parent").as_object_mut().expect("object").remove(key);
        assert_eq!(require_chat_progress(&task), Err("chat progress facts missing".to_owned()), "{parent}/{key}");
    }
    for (pointer, malformed) in [
        ("/chip/details/owner/details/attempt", json!("1")),
        ("/chip/details/owner/details/attempt", json!(-1)),
        ("/chip/details/owner/details/live_delegates", Value::Null),
        ("/chip/details/owner/details/live_delegates", json!("0")),
        ("/subscriptions/entries", Value::Null),
        ("/cut", Value::Null),
    ] {
        let mut task = empty_park();
        *task.pointer_mut(pointer).expect("fixture field") = malformed;
        assert_eq!(require_chat_progress(&task), Err("chat progress facts missing".to_owned()), "{pointer}");
    }
}

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
    use skein_io::kernel::Exit;
    use skein_lib::Duration;
    use skein_world::end_to_end::Command;
    use support::{boxed, forge_land};
    let (scratch, paths) = initialized_deployment("live-claude-startup");
    configure_claude(&paths);
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
    assert!(std::fs::read_dir(paths[1].join("programs")).expect("private programs").next().is_none());
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
        Path::new("/machine/bin/claude"),
    );
}

#[test]
fn a_claude_configuration_replaces_openai_and_keeps_the_demo() {
    use std::fs;
    // docs/help/box.md, step 1: Claude's fields over init's endpoint and model sizes.
    let (scratch, paths) = initialized_deployment("live-claude-configuration");
    let config = &paths[0];
    let agent_before: Value =
        serde_json::from_slice(&fs::read(config.join("agent.json")).expect("agent")).expect("JSON");
    let before: Value =
        serde_json::from_slice(&fs::read(config.join("temper.json")).expect("deployment")).expect("JSON");
    configure_claude(&paths);
    let agent: Value = serde_json::from_slice(&fs::read(config.join("agent.json")).expect("agent")).expect("JSON");
    let value: Value =
        serde_json::from_slice(&fs::read(config.join("temper.json")).expect("deployment")).expect("JSON");
    assert_eq!(
        agent["programs"],
        json!({"claude_code":{
            "path":"/machine/bin/claude", "directory":paths[1].join("programs")
        }})
    );
    assert_eq!(
        agent["endpoints"],
        json!([
            {"name":"claude", "number":2, "dialect":2, "provider":"anthropic", "program":"claude_code",
                "address":"api.anthropic.com:443", "transport":"tls", "server_name":"api.anthropic.com",
                "connect":10000, "handshake":10000, "headers":[],
                "limits": {
                    "window":1_048_576, "output":8192, "reasoning_item":65536, "head":60000, "idle":300_000}},
            agent_before["endpoints"].as_array().unwrap().iter().find(|endpoint| endpoint["name"] == "demo").unwrap()
        ])
    );
    assert_eq!(agent["profile"], agent_before["profile"]);
    assert_eq!(agent["environment"], agent_before["environment"]);
    assert_eq!(
        value["engine"]["llm"],
        json!({
            "providers":[{"name":"anthropic", "number":2, "signed_in_plan":false,
                "manage_usage":"https://claude.ai/settings/usage", "endpoint":"claude",
                "sources":{"programs":["claude_code"]}}, before["engine"]["llm"]["providers"].as_array().unwrap().iter().find(|entry| entry["name"] == "demo").unwrap()],
            "endpoints":[{"name":"claude", "number":2, "dialect":2, "provider":"anthropic", "program":"claude_code",
                "address":{"host":"api.anthropic.com", "port":443}, "transport":"tls", "server_name":"api.anthropic.com",
                "headers":[], "connect_ms":10000, "handshake_ms":10000}, before["engine"]["llm"]["endpoints"].as_array().unwrap().iter().find(|entry| entry["name"] == "demo").unwrap()]
        })
    );
    assert_eq!(value["application"], before["application"], "including init's demo");
    assert_eq!(value["engine"]["limits"], before["engine"]["limits"]);
    assert_eq!(value["workers"], before["workers"]);
    let parsed = smith_agent_shell::config::read(&config.join("agent.json")).expect("ordinary agent schema");
    assert_eq!(parsed.program_endpoints.into_iter().collect::<Vec<_>>(), [2]);
    eprintln!("Claude agent largest turn: {} bytes", parsed.service.limits.channel.bodies.turn_body);
    scratch.remove();
}

#[test]
fn stock_charters_choose_claude_before_the_demo_model() {
    let (scratch, paths) = initialized_deployment("live-claude-charters");
    configure_claude(&paths);
    let deployment = std::fs::read(paths[0].join("temper.json")).expect("deployment");
    let document = jig_protocol_config::document(
        &deployment,
        &jig_protocol_config::DocumentLimits { bytes: 1 << 20, tokens: 1 << 20, depth: 32 },
    )
    .expect("document");
    let at = jig_protocol_config::read::Object::new(&document, 0)
        .expect("object")
        .member(&document, b"engine")
        .expect("engine");
    let engine = temper_service::startup::engine(
        &document,
        at,
        &jig_directories(&support::configuration::directories(&paths[0])),
    )
    .expect("engine schema");
    let defaults = temper_service::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    for charter in &defaults.charters {
        assert_eq!(
            (charter.policy.model.name.as_ref(), charter.policy.model.endpoint),
            (b"claude-opus-5-5".as_slice(), 2)
        );
        let resolved = jig_domain_core::settings::charters::resolved(charter, &defaults.models, &engine.limits.core)
            .expect("stock charter");
        let alternatives: Vec<_> =
            resolved.policy.alternatives.iter().map(|model| (model.name.as_ref(), model.endpoint)).collect();
        assert_eq!(
            alternatives,
            [(b"claude-sonnet-5-5".as_slice(), 2), (b"claude-fable-5-1".as_slice(), 2), (b"demo".as_slice(), 3)]
        );
    }
    scratch.remove();
}

/// The Claude deployment's stock catalogue, endpoints, providers and charters 1 and 2
/// on the joined client world's engine (temper/docs/design/testing/live-tier.md and temper/docs/design/testing/checks.md, 3).
fn live_claude_world() -> (
    support::Scratch,
    jig_client_kit::harness::World<temper_client_world::app::Temper, temper_client_world::front::Joint>,
) {
    use jig_client_kit::harness::{Settings, World};
    use temper_client_world::{
        fixtures,
        front::{Joint, engine_limits},
    };
    use temper_engine_domain_world::{store::Store, walking};

    let (scratch, paths) = initialized_deployment("live-claude-settings");
    configure_claude(&paths);
    let deployment = std::fs::read(paths[0].join("temper.json")).expect("deployment");
    let document = jig_protocol_config::document(
        &deployment,
        &jig_protocol_config::DocumentLimits { bytes: 1 << 20, tokens: 1 << 20, depth: 32 },
    )
    .expect("document");
    let at = jig_protocol_config::read::Object::new(&document, 0)
        .expect("object")
        .member(&document, b"engine")
        .expect("engine");
    let engine = temper_service::startup::engine(
        &document,
        at,
        &jig_directories(&support::configuration::directories(&paths[0])),
    )
    .expect("engine schema");
    let stock = temper_service::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    let mut limits = engine_limits();
    limits.people.initial_owners = 2;
    limits.accounts.accounts = 3;
    limits.accounts.holder_accounts = 3;
    limits.authority.accounts = 3;
    limits.fleet.grants = 4;
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
        let llm = jig_protocol_config::core(&engine, config.core.deployment, fixtures::SEED).settings;
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
    use support::forge_land::{LiveModel, live_apply, live_settings};
    use temper_client_world::fixtures;
    let (scratch, mut world) = live_claude_world();
    let agent = fixtures::agent(&mut world);
    let call = fixtures::give(
        &mut world,
        agent,
        &[b"account", b"declare", b"2", b"claude_code", b"claude-login", b"--key", b"live-program"],
    );
    let declared = fixtures::answered(&mut world, agent, call);
    assert_eq!(declared.status, 0, "{declared:?}");
    assert!(declared.stderr.is_empty(), "{declared:?}");
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
    let changes = live_settings(
        LiveModel::ClaudeCode { login: Path::new("/held/claude-login") },
        &inventory,
        [&charters[0], &charters[1]],
        &row,
    )
    .expect("live settings");
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

#[test]
fn a_claude_plan_declares_without_import_or_sign_in() {
    use std::path::Path;
    use support::forge_land::{LiveModel, live_commands};
    let (commands, account_at) = live_commands(true, LiveModel::ClaudeCode { login: Path::new("/held/claude-login") });
    assert_eq!(
        &commands[account_at - 1],
        &["account", "declare", "2", "claude_code", "claude-login", "--key", "live-program"]
    );
    assert!(commands.iter().all(|args| args[0] != "program-login"
        && (args[0] != "account" || !matches!(args[1].as_str(), "connect" | "import"))));
    let (restart, _) = live_commands(false, LiveModel::ClaudeCode { login: Path::new("/held/claude-login") });
    assert!(
        restart.iter().all(
            |args| args[0] != "account" || !matches!(args[1].as_str(), "declare" | "connect" | "import" | "verify")
        )
    );
}

#[test]
fn a_fresh_sitting_selects_its_model_in_every_charter() {
    use serde_json::json;
    use std::path::Path;
    use support::forge_land::{LiveChange, LiveModel, live_settings};

    let mut lines: Vec<_> = (1..=8).map(|number| json!({"setting":{"kind":"model","number":number}})).collect();
    lines.extend([json!({"setting":{"kind":"charter","number":1}}), json!({"setting":{"kind":"charter","number":2}})]);
    let inventory = json!({"newest":0,"settings":lines});
    let charters = [1, 2].map(|number| {
        json!({
            "setting":{"kind":"charter","number":number},
            "value":{"instructions":format!("charter {number}"),"model":{"name":"old"},
                "alternatives":[{"name":"fallback"}],"rest":true,"time":1}
        })
    });
    let stock_model = json!({
        "provider":2,"dialect":2,"endpoint":2,"program":"claude_code","name":"claude-fable-5-1",
        "window":200_000,"output":8192,"input_price":3,"cached_price":1,"cache_write_price":4,
        "output_price":15,"price_unit":1_000_000,"efforts":["low","medium","high","xhigh","max"],"effort":"high"
    });
    let mut gpt_model = stock_model.clone();
    gpt_model["name"] = "gpt-6-luna".into();
    let mut haiku_model = stock_model.clone();
    haiku_model["name"] = "claude-haiku-5-5".into();
    haiku_model["effort"] = "low".into();
    let charter_items = |model: &serde_json::Value| {
        [1, 2].map(|number| {
            json!({
                "setting":{"kind":"charter","number":number},
                "set":{"instructions":format!("charter {number}"),"model":model,
                    "alternatives":[],"rest":false,"time":240_000_000_000_u64}
            })
        })
    };
    let gpt_items = charter_items(&gpt_model);
    let haiku_items = charter_items(&haiku_model);
    let claude = LiveModel::ClaudeCode { login: Path::new("/held/claude-login") };
    let cases = [
        (
            LiveModel::ChatGpt { account: None },
            "gpt-6-luna",
            "gpt-6-luna",
            Ok(vec![LiveChange { key: "live-settings", file: json!({"items":gpt_items}) }]),
        ),
        (
            claude,
            "claude-fable-5-1",
            "claude-haiku-5-5",
            Ok(vec![
                LiveChange {
                    key: "live-model",
                    file: json!({"items":[
                        {"setting":{"kind":"model","number":64},"set":{
                            "number":64,"model":haiku_model,"shown":null,"description":null,
                            "picker":true,"withdrawn":false,"source":"person","estimated":true
                        }}
                    ]}),
                },
                LiveChange { key: "live-settings", file: json!({"items":haiku_items}) },
            ]),
        ),
        (
            claude,
            "another-model",
            "claude-haiku-5-5",
            Err("the selected live stock model: claude-fable-5-1".to_owned()),
        ),
    ];
    for (model, row_name, selected, expected) in cases {
        let mut row_model = stock_model.clone();
        row_model["name"] = row_name.into();
        let number = if row_name == "gpt-6-luna" { 7 } else { 3 };
        let row = json!({"setting":{"kind":"model","number":number},"value":{
            "number":number,"model":row_model,"shown":"Stock model","description":"Stock description",
            "picker":false,"withdrawn":true,"source":"stock","estimated":false
        }});
        assert_eq!(model.name(), selected, "chat's --model for {row_name}");
        assert_eq!(live_settings(model, &inventory, [&charters[0], &charters[1]], &row), expected, "{row_name}");
    }
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
    if default_store {
        command.env_remove(temper_test_env::LIVE_STORE.name);
    }
    let output = command.output().expect("stubbed launcher");
    (scratch, output)
}

#[test]
fn the_launcher_returns_the_login_after_build_failure() {
    // SourceName's inclusive lower and upper lengths (credentials.md, 3).
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

fn program_accounts() -> Value {
    json!({"accounts":{
        "party":[{"number":7, "provider":2, "sources":[{"kind":"program", "name":"claude-login", "usable":true}]}],
        "program_hosts":[{"name":"worker", "connected":true, "programs":[{"name":"claude-login", "state":"usable"}]}]
    }})
}

#[test]
fn a_claude_source_is_verified_by_its_declared_account() {
    use support::forge_land::program_verification;
    assert_eq!(
        program_verification(std::path::Path::new("/held/claude-login"), &program_accounts()),
        Ok(vec![
            vec!["account", "verify", "7", "--source", "program", "--json"].into_iter().map(str::to_owned).collect(),
            vec!["account", "list", "--until", "usable", "--timeout", "600s", "--json"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            vec!["account", "list", "--json"].into_iter().map(str::to_owned).collect(),
        ])
    );
}

#[test]
fn a_claude_account_needs_its_login_usable_on_worker() {
    use support::forge_land::LiveModel;
    let model = LiveModel::ClaudeCode { login: std::path::Path::new("/held/claude-login") };
    assert_eq!(model.ready(&program_accounts()), Ok(()));
}

#[test]
fn a_claude_plan_refuses_missing_or_unusable_accounts() {
    use support::forge_land::LiveModel;
    let model = LiveModel::ClaudeCode { login: std::path::Path::new("/held/claude-login") };
    for (pointer, value, message) in [
        ("/accounts/party/0/provider", json!(1), "held Claude Code account missing"),
        ("/accounts/party/0/sources/0/name", json!("other-login"), "held Claude Code account missing"),
        ("/accounts/party/0/sources/0/usable", json!(false), "held Claude Code source is not usable"),
        ("/accounts/program_hosts/0/connected", json!(false), "held Claude Code login is not usable on the worker"),
        (
            "/accounts/program_hosts/0/programs/0/state",
            json!("needs_sign_in"),
            "held Claude Code login is not usable on the worker",
        ),
    ] {
        let mut accounts = program_accounts();
        *accounts.pointer_mut(pointer).expect("fixture field") = value;
        assert_eq!(model.ready(&accounts), Err(message.to_owned()), "{pointer}");
    }
}

#[test]
fn a_claude_plan_selects_provider_two_beside_demo_accounts() {
    use support::forge_land::{LiveModel, program_verification};
    let login = std::path::Path::new("/held/claude-login");
    let mut accounts = program_accounts();
    // The playground's key account, if listed, does not select the held program account.
    accounts["accounts"]["party"]
        .as_array_mut()
        .expect("party accounts")
        .insert(0, json!({"number":8, "provider":3, "sources":[{"kind":"key", "usable":true}]}));
    assert_eq!(LiveModel::ClaudeCode { login }.ready(&accounts), Ok(()));
    assert_eq!(
        program_verification(login, &accounts).expect("held account verification")[0],
        ["account", "verify", "7", "--source", "program", "--json"]
    );
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
        plan.install_return_hook(&self.stub);
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

/// testing/live-tier.md: validation between boxes cannot strand an imported login.
#[test]
fn a_held_login_returns_when_validation_panics_between_boxes() {
    let mut boxes = HeldBox::new("held-between-boxes");
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        boxes.run(true, false).expect("first box succeeds");
        let _ = support::forge_land::landed_identity("", "");
        boxes.run(false, false).expect("second box succeeds");
    }));
    let failure = result.expect_err("validation remains a failure");
    let message =
        failure.downcast_ref::<String>().map(String::as_str).or_else(|| failure.downcast_ref::<&str>().copied());
    assert_eq!(message, Some("landed result line"));
    assert_eq!(boxes.starts, 1, "validation panics before the second box starts");
    assert_eq!(boxes.returned_before_stop, [true], "record is private, nonempty and back before the first stop");
    assert!(boxes.record.is_file(), "the stopped box no longer owns the record");
    boxes.scratch.remove();
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
    let installed = std::panic::catch_unwind(|| plan.install_return_hook(&boxes.stub));
    assert!(installed.is_err(), "terminal cleanup must exclude replacement of its return hook");
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
    support::write_private(&cargo, format!("#!/bin/sh\nset -eu\n{body}\n").as_bytes());
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
        let (scratch, output) = held_launcher("held-build-failure", story, "exit 17");
        assert_eq!(output.status.code(), Some(17), "{}", String::from_utf8_lossy(&output.stderr));
        assert!(String::from_utf8_lossy(&output.stdout).contains("LOGIN BACK: regular file mode=600"));
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
test -d "/proc/$(cat "$TEMPER_TEST_LIVE_STORE/runner")"
mv "$TEMPER_TEST_LIVE_STORE/synthetic-import" "$TEMPER_TEST_LIVE_ACCOUNT"
printf '%s\n' 'HOOK RETURNED'
HOOK
chmod 700 "$TEMPER_TEST_LIVE_STORE/give-back"
printf '%s\n' "$$" > "$TEMPER_TEST_LIVE_STORE/runner"
for directory in "$TMPDIR/owned" "$TEMPER_TEST_LIVE_STORE/owned"; do
    mkdir -p "$directory"
    (cd "$directory"; exec sleep 30) >/dev/null 2>&1 &
    printf '%s\n' "$!" >> "$HOME/owned-children"
done
kill -TERM "$PPID"
# Stay alive until the launcher has run the hook, then accept its TERM.
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
        let children = std::fs::read_to_string(scratch.path().join("owned-children")).expect("owned stub children");
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
        scratch.remove();
    }
}

#[test]
fn the_launcher_refuses_changed_ordinary_held_login_metadata() {
    for story in ["forge-land", "rung-b0"] {
        let (scratch, output) =
            held_launcher("held-metadata", story, r#"chmod 644 "$TEMPER_TEST_LIVE_ACCOUNT"; exit 17"#);
        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&output.stderr).contains("LOGIN NOT BACK:"));
        scratch.remove();
    }
}

#[test]
fn the_launcher_records_a_failed_ordinary_return_hook() {
    let body = r#"mkdir -p "$TEMPER_TEST_LIVE_STORE/forge-land"
printf '#!/bin/sh\nexit 9\n' > "$TEMPER_TEST_LIVE_STORE/forge-land/give-back"
chmod 700 "$TEMPER_TEST_LIVE_STORE/forge-land/give-back"
exit 17"#;
    let (scratch, output) = held_launcher("held-return-failure", "forge-land", body);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("LOGIN NOT BACK: export/stat failed"));
    assert!(scratch.path().join("store/forge-land/give-back").is_file(), "store retained for recovery");
    scratch.remove();
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
    plan.install_return_hook(&stub);
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
fn an_imported_records_path_is_printed_as_file() {
    use support::forge_land::printed_command;
    let words = |words: &[&str]| words.iter().map(|word| (*word).to_owned()).collect::<Vec<_>>();
    assert_eq!(
        printed_command(&words(&["account", "import", "/held/record", "--key", "connect-plan"])),
        words(&["account", "import", "FILE", "--key", "connect-plan"])
    );
    for kept in [
        &["account", "list", "--json"][..],
        &["account", "connect", "1", "--key", "connect-plan"],
        &["account", "import"],
    ] {
        assert_eq!(printed_command(&words(kept)), words(kept));
    }
}

/// testing/live-tier.md: the chat's delegate is a change or the story stops, naming it.
#[test]
fn the_chats_delegate_is_a_change_or_the_story_names_it() {
    use support::forge_land::change_delegate;
    for (executor, expected) in [
        (json!({"procedure": {"connector": 1, "kind": 2}}), Ok(3)),
        (json!({"agent": {"charter": 1}}), Err("the chat's delegate task 3 is an agent's on charter 1, not a change: the shipped chat chooses change (testing/live-tier.md)".to_owned())),
        (Value::Null, Err("delegate executor missing (chip.executor)".to_owned())),
        (json!({"job": {"kind": 1}}), Err("the chat's delegate task 3 is a job's, not a change: the shipped chat chooses change (testing/live-tier.md)".to_owned())),
        (json!({"party": {"party": 1}}), Err("the chat's delegate task 3 is a party's, not a change: the shipped chat chooses change (testing/live-tier.md)".to_owned())),
        (json!({"role": {"project": 1, "role": 2}}), Err("the chat's delegate task 3 is a role's, not a change: the shipped chat chooses change (testing/live-tier.md)".to_owned())),
    ] {
        let row = json!({"chip": {"task": 3, "executor": executor}});
        assert_eq!(change_delegate(&row), expected, "{row}");
    }
}

fn delegate_read(rows: &[u64], cut: bool, after: Option<&str>) -> Value {
    json!({
        "live": [], "live_after": null,
        "ended": {"rows": {
            "rows": rows.iter().map(|task| json!({"chip": {"task": task}})).collect::<Vec<_>>(),
            "cut": cut,
            "after": after.map(|value| json!({"value": value}))
        }}
    })
}

/// testing/live-tier.md: the chat's and the change's ended delegates are read whole,
/// a page at a time, by the cursor each page prints.
#[test]
fn ended_delegates_are_read_whole_by_their_cursor() {
    use support::forge_land::EndedDelegates;
    let mut ended = EndedDelegates::new(2, [2, 3]);
    assert_eq!(ended.first(), ["task", "list", "--project", "2", "--parent", "2", "--ended", "--json"]);
    for (read, expected) in [
        (
            delegate_read(&[3], false, Some("3")),
            vec!["task", "list", "--project", "2", "--parent", "2", "--ended", "--json", "--after", "3"],
        ),
        (
            delegate_read(&[5], true, Some("5")),
            vec!["task", "list", "--project", "2", "--parent", "2", "--ended", "--json", "--after", "5"],
        ),
        (delegate_read(&[], false, None), vec!["task", "list", "--project", "2", "--parent", "3", "--ended", "--json"]),
    ] {
        assert_eq!(ended.read(&read), Ok(Some(expected.into_iter().map(str::to_owned).collect())), "{read}");
    }
    assert_eq!(ended.read(&delegate_read(&[7, 6], false, None)), Ok(None));
    assert_eq!(ended.whole(), [vec![3, 5], vec![6, 7]]);
}

/// testing/live-tier.md: what the story refuses of a delegate page, naming it.
#[test]
fn an_ended_delegate_read_refuses_live_rows_a_cut_without_a_cursor_and_a_33rd_page() {
    use support::forge_land::EndedDelegates;
    let mut live_row = delegate_read(&[], false, None);
    live_row["live"] = json!([{"chip": {"task": 4}}]);
    let mut live_after = delegate_read(&[], false, None);
    live_after["live_after"] = json!({"value": "4"});
    let mut missing_task = delegate_read(&[4], false, None);
    missing_task["ended"]["rows"]["rows"] = json!([{"chip": {}}]);
    let mut missing_rows = delegate_read(&[], false, None);
    missing_rows["ended"]["rows"]["rows"] = Value::Null;
    for (read, expected) in [
        (live_row, "no live delegates"),
        (live_after, "no live delegates"),
        (delegate_read(&[4], true, None), "ended delegate page cut without a cursor"),
        (missing_task, "delegate task missing (chip.task)"),
        (missing_rows, "ended delegate rows missing"),
    ] {
        let mut ended = EndedDelegates::new(2, [2, 3]);
        assert_eq!(ended.read(&read), Err(expected.to_owned()), "{read}");
    }
    // The bound holds within one parent and across both parents.
    for first in [delegate_read(&[3], false, Some("3")), delegate_read(&[], false, None)] {
        let mut ended = EndedDelegates::new(2, [2, 3]);
        assert!(ended.read(&first).expect("first delegate page").is_some());
        for _ in 1..EndedDelegates::PAGES {
            assert!(ended.read(&delegate_read(&[4], false, Some("4"))).expect("within page bound").is_some());
        }
        assert_eq!(
            ended.read(&delegate_read(&[], false, None)),
            Err("ended delegates exceed 32-page bound".to_owned())
        );
    }
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
        support::forge_land::assert_forgejo_setup_after_sign_in(
            "first",
            &calls,
            "forgejo-1: started\nrunner-1: started",
            None,
            scratch.path()
        ),
        2
    );
    let mut other_host = row;
    other_host["forges"][0]["host"] = json!("other.example:3000");
    calls[4].stdout = other_host.to_string().into_bytes();
    assert!(
        std::panic::catch_unwind(|| {
            support::forge_land::assert_forgejo_setup_after_sign_in(
                "first",
                &calls,
                "forgejo-1: started\nrunner-1: started",
                None,
                scratch.path(),
            )
        })
        .is_err(),
        "the text row must name the JSON row's resource host"
    );
    scratch.remove();
}

#[test]
fn setup_resolves_the_adopted_project_on_fresh_and_seeded_deployments() {
    use support::first_repository::project_number;
    for (projects, name, expected) in [
        (json!([{"project":1,"name":{"bytes":"hello"}}]), "hello", 1),
        (json!([{"project":1,"name":{"bytes":"widget"}}]), "widget", 1),
        (json!([{"project":1,"name":{"bytes":"seed"}},{"project":8,"name":{"bytes":"hello"}}]), "hello", 8),
    ] {
        assert_eq!(project_number(&json!({"projects":projects}), name), Ok(expected));
    }
}

#[test]
fn setup_refuses_missing_or_ambiguous_adopted_projects() {
    use support::first_repository::project_number;
    for me in [
        json!({}),
        json!({"projects":[]}),
        json!({"projects":[{"project":1,"name":{"bytes":"seed"}}]}),
        json!({"projects":[{"project":0,"name":{"bytes":"hello"}}]}),
        json!({"projects":[{"project":1,"name":{"bytes":"hello"}},{"project":2,"name":{"bytes":"hello"}}]}),
    ] {
        assert!(project_number(&me, "hello").is_err(), "{me}");
    }
}

fn referee_me() -> jig_client::contract::Me {
    use jig_client::contract as c;
    use skein_lib::List;
    let words = c::DocumentText::complete(c::Text::new(b"Person".as_slice().into(), 6).expect("name"));
    c::Me {
        party: c::MeParty { number: 1, kind: c::PartyKind::Person, login: words.clone(), name: words },
        projects: List::with_capacity(0),
        providers: List::with_capacity(0),
        providers_cut: false,
        key_bytes: 256,
        redirect_bytes: 256,
        words_bytes: 8192,
        needs: c::Needs { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 },
        needs_total: 0,
        updates: c::Count { value: 0, limited: false },
        results_seen: 0,
        cut: false,
        retention: skein_lib::Duration::from_secs(60),
        deployment_role: None,
    }
}

fn referee_json(argv: &[&str], page: &jig_client::contract::ActivityPage) -> Value {
    use jig_client::{Event, Up};
    use skein_lib::{Env, List, Queue, Time, Wall};
    use temper_client_tools::{jig::call::Parsed, root};
    let mut tools = temper_client_tools::limits::tools();
    tools.rows = 1; // One row fills a page so the referee follows its cursor.
    let limits = root::Limits::new(temper_client_tools::limits::client(), tools);
    let mut root = root::Root::unlinked(&limits, 1).expect("client limits");
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let words: Vec<_> = argv.iter().map(|word| word.as_bytes()).collect();
    let Parsed::Call { call, .. } = temper_client_tools::table::parse(&words).expect("read argv") else {
        panic!("read call")
    };
    let mut pending = std::collections::VecDeque::from([
        root::Event::Client { event: Event::StartUnlinked { kept: List::with_capacity(0) } },
        root::Event::Client { event: Event::SignIn { with: jig_client::With::Transport } },
        root::Event::Client { event: Event::Up { up: Up::Served { served: true, versions: None } } },
        root::Event::Call { number: 1, call, json: true, value: None },
    ]);
    for _ in 0..32 {
        let event = pending.pop_front().expect("read still pending");
        let mut out = Queue::uncapped();
        root::step(&mut root, &env, event, &mut out);
        while let Some(request) = out.pop() {
            if let root::Request::Printed { printed, .. } = request {
                assert_eq!(printed.status, 0);
                return serde_json::from_slice(&printed.stdout).expect("rendered JSON");
            }
            let Some(up) = referee_response(request, page) else { continue };
            pending.push_back(root::Event::Client { event: Event::Up { up } });
        }
    }
    panic!("read did not settle")
}

fn referee_response(
    request: temper_client_tools::root::Request,
    page: &jig_client::contract::ActivityPage,
) -> Option<jig_client::Up> {
    use jig_client::{Down, UnlinkedRequest, Up, contract as c};
    let temper_client_tools::root::Request::Unlinked { request: UnlinkedRequest::Down { down } } = request else {
        return None;
    };
    Some(match down {
        Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Me) } => {
            Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(referee_me()))) }
        }
        Down::Open { watch, .. } => Up::WatchEnded { watch, end: c::WatchEnd::SubjectEnded },
        Down::Read { read, .. } => Up::ReadDone {
            read,
            result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(Box::new(page.clone())))),
        },
        Down::Save { .. } | Down::Erase { .. } | Down::Send { .. } | Down::Continue { .. } | Down::Close { .. } => {
            return None;
        }
    })
}

fn referee_turn() -> jig_client::contract::TurnRead {
    use jig_client::contract as c;
    use skein_lib::List;
    let text = |bytes: &[u8]| c::Text::new(bytes.into(), 128).expect("bounded text");
    let mut blocks = List::with_capacity(4);
    for (number, tool, failed, outcome) in [
        (1, b"change".as_slice(), false, Some(b"created".as_slice())),
        (2, b"delegate".as_slice(), true, Some(b"refused".as_slice())),
        (3, b"delegate".as_slice(), false, None),
    ] {
        blocks
            .push(c::Block::Call(c::Call {
                number,
                tool: text(tool),
                input: text(b"request"),
                outcome: outcome.map(text),
                failed,
                objects: List::with_capacity(0),
                parts: List::with_capacity(0),
            }))
            .expect("three calls");
    }
    blocks.push(c::Block::Text(text(b"The change landed."))).expect("reply");
    let mut messages = List::with_capacity(1);
    messages.push(c::Message { speaker: c::Speaker::Assistant, blocks }).expect("assistant");
    c::TurnRead::Decoded(c::Turn {
        messages,
        used: c::Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
        spent: 0,
        read_through: None,
        llm: None,
        cut: false,
    })
}

fn referee_activity() -> jig_client::contract::ActivityPage {
    use jig_client::contract as c;
    use skein_lib::{List, Wall};
    let turn = referee_turn();
    let name = c::TurnName { attempt: 1, number: 1 };
    let mut rows = List::with_capacity(1);
    rows.push(c::DecodedActivityLine {
        source: Box::new(c::ActivityLine {
            task: 41,
            position: 1,
            at: Wall::EPOCH,
            line: c::Activity::Turn { reply: 1, attempt: 1, turn: 1, read: None },
        }),
        decoded: Some(c::ArchivedTurn { task: 41, name, cumulative: 0, read: None, at: Wall::EPOCH, turn }),
    })
    .expect("one archived turn");
    c::ActivityPage { cut: false, rows, before: None, after: None }
}

#[test]
fn a_landing_referee_reads_committed_tools_beyond_the_conversation() {
    use support::forge_land::conversation::Evidence;
    let mut page = referee_activity();
    page.before = Some(jig_client::contract::Cursor::new(Box::from([0x11]), 1).expect("cursor"));
    let chat = referee_json(&["chat", "show", "41", "--json"], &page);
    assert!(
        chat.pointer("/activity/rows/0/event/owner/decoded/turn/decoded/messages/0/blocks/0/call/outcome")
            .expect("call")
            .is_null()
    );
    let mut evidence = Evidence::new(41, 0);
    assert_eq!(evidence.read(&chat).expect("conversation page"), Some(read_before("chat", "show")));
    assert!(evidence.tools.is_empty(), "conversation payload omission proves no successful call");
    page.before = None;
    let chat = referee_json(&["chat", "show", "41", "--before", "11", "--json"], &page);
    let detail = evidence.read(&chat).expect("conversation complete").expect("missing detail fetch");
    assert_eq!(detail, ["task", "activity", "41", "--json"]);
    page.before = Some(jig_client::contract::Cursor::new(Box::from([0x11]), 1).expect("cursor"));
    let argv: Vec<_> = detail.iter().map(String::as_str).collect();
    let activity = referee_json(&argv, &page);
    assert_eq!(evidence.read(&activity).expect("detail page"), Some(read_before("task", "activity")));
    assert_eq!(evidence.tools, BTreeSet::from(["change".to_owned()]), "failed and unfinished delegates do not count");
    assert_eq!(evidence.failed, BTreeSet::from(["delegate".to_owned()]), "failed committed calls count");
    assert_eq!(support::forge_land::require_chat_change(&evidence), Ok(()));
    for (tools, failed, expected) in [
        (
            vec!["goal"],
            vec!["delegate"],
            "the chat's committed change call succeeded: its committed calls succeeded [goal] and failed [delegate]",
        ),
        (
            vec!["read_branch", "goal"],
            vec!["change"],
            "the chat's committed change call succeeded: its committed calls succeeded [goal, read_branch] and failed [change]",
        ),
        (
            vec![],
            vec!["read_branch", "change"],
            "the chat's committed change call succeeded: its committed calls succeeded [] and failed [change, read_branch]",
        ),
        (
            vec!["goal"],
            vec![],
            "the chat's committed change call succeeded: its committed calls succeeded [goal] and failed []",
        ),
        (vec![], vec![], "the chat's committed change call succeeded: its committed calls succeeded [] and failed []"),
    ] {
        evidence.tools = tools.into_iter().map(str::to_owned).collect();
        evidence.failed = failed.into_iter().map(str::to_owned).collect();
        assert_eq!(support::forge_land::require_chat_change(&evidence), Err(expected.to_owned()));
    }

    page.before = None;
    let row = page.rows.get_mut(0).expect("activity row");
    row.source.line =
        jig_client::contract::Activity::RunEnded { attempt: 1, end: jig_client::contract::RunEnd::Parked, spent: 0 };
    row.decoded = None;
    let activity = referee_json(&["task", "activity", "41", "--before", "11", "--json"], &page);
    assert_eq!(evidence.read(&activity), Ok(None));
    assert_eq!(evidence.replies.len(), 1, "reply survives the detail reads");
}

fn read_before(group: &str, verb: &str) -> Vec<String> {
    [group, verb, "41", "--json", "--before", "11"].map(str::to_owned).to_vec()
}

#[test]
fn a_landing_referee_keeps_a_successful_raw_delegate_in_its_evidence() {
    let mut evidence = support::forge_land::conversation::Evidence::new(41, 0);
    let mut page = referee_activity();
    let chat = referee_json(&["chat", "show", "41", "--json"], &page);
    assert!(evidence.read(&chat).expect("conversation read").is_some());
    let turn = &mut page.rows.get_mut(0).expect("turn").decoded.as_mut().expect("archive").turn;
    let jig_client::contract::TurnRead::Decoded(turn) = turn else { panic!("decoded") };
    let block = turn.messages.get_mut(0).expect("assistant").blocks.get_mut(1).expect("delegate");
    let jig_client::contract::Block::Call(call) = block else { panic!("call") };
    call.failed = false;
    let activity = referee_json(&["task", "activity", "41", "--json"], &page);
    assert_eq!(evidence.read(&activity), Ok(None));
    assert_eq!(evidence.tools, BTreeSet::from(["change".to_owned(), "delegate".to_owned()]));
}

#[test]
fn a_landing_referee_requires_whole_bounded_activity() {
    use jig_client::contract as c;
    use support::forge_land::conversation::Evidence;
    let chat = referee_json(&["chat", "show", "41", "--json"], &referee_activity());
    for defect in ["page", "turn", "undecodable"] {
        let mut evidence = Evidence::new(41, 0);
        assert!(evidence.read(&chat).expect("conversation").is_some());
        let mut page = referee_activity();
        if defect == "page" {
            page.cut = true;
        } else {
            let turn = &mut page.rows.get_mut(0).expect("row").decoded.as_mut().expect("archive").turn;
            if defect == "undecodable" {
                *turn = c::TurnRead::Undecodable { why: c::TurnProblem::Malformed };
            } else {
                let c::TurnRead::Decoded(turn) = turn else { panic!("decoded") };
                turn.cut = true;
            }
        }
        let detail = referee_json(&["task", "activity", "41", "--json"], &page);
        assert!(evidence.read(&detail).is_err(), "reject incomplete {defect}");
    }
    let mut evidence = Evidence::new(41, 0);
    assert!(evidence.read(&chat).expect("conversation").is_some());
    let mut page = referee_activity();
    page.before = Some(c::Cursor::new(Box::from([0x11]), 1).expect("cursor"));
    let detail = referee_json(&["task", "activity", "41", "--json"], &page);
    for _ in 0..32 {
        assert!(evidence.read(&detail).expect("within page bound").is_some());
    }
    assert_eq!(evidence.read(&detail), Err("bounded conversation evidence pages".to_owned()));
}
