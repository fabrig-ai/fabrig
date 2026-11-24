//! Harness parsers, configuration naming and embedded bundle values.
use crate::support;
use std::{fmt::Write, path::Path};
use support::bundle_fixture::{BUNDLE_BYTES, BUNDLE_COUNT, BUNDLE_FILES, BUNDLE_LIMITS, PAGE, PATH_STEM_BOUND};

#[test]
fn a_rung_reads_created_chats_waited_changes_and_both_child_sets_in_ctls_forms() {
    use serde_json::json;
    use support::forge_land::rung::{child_rows, created_chat, plan_question, task_digest, waited};
    // The chat-start document supplies the created range; the top-level task
    // names an operation target and is null when creating a chat.
    let created = json!({"operation":"chat","task":null,"to":null,
        "outcome":{"done":{"document":{"created":{"first":7,"last":7,"given":{"task":7,"amount":207_552}}}}}});
    assert_eq!(created_chat(&created), Ok(7));
    assert!(created_chat(&json!({"task":7})).is_err(), "an operation target is not creation evidence");
    let panel = json!({"task":3,"pull":{"number":2},"head":"abc","gates":[]});
    let change = json!({"change":panel,"met":true});
    assert_eq!(waited(&["change", "show", "3"], change), Ok(panel));
    assert!(waited(&["change", "show", "3"], json!({"met":true})).is_err());
    assert!(waited(&["change", "show", "3"], json!({"met":false})).is_err());
    let task = json!({"task":{"chip":{"task":2},"digest":"abc"},"met":true});
    assert_eq!(waited(&["task", "show", "2"], task.clone()), Ok(task));
    assert_eq!(task_digest(&json!({"chip":{"task":2},"digest":"plain"})), Ok("plain"));
    assert_eq!(task_digest(&json!({"task":{"digest":"compound"},"tree":{}})), Ok("compound"));
    assert!(task_digest(&json!({"chip":{"task":2}})).is_err());
    let accounts = json!({"accounts":{"party":[]},"met":true});
    assert_eq!(waited(&["account", "list"], accounts.clone()), Ok(accounts));
    let live = json!({"tasks":{"rows":[{"chip":{"task":3}}],"more":false,"digest":"abc"},"read_on":null});
    let ended = json!({"live":[],"live_after":null,
        "ended":{"rows":{"rows":[{"chip":{"task":4}}],"after":null,"cut":false}}});
    let rows = child_rows(&live, &ended).expect("both sets");
    assert_eq!(rows, vec![json!({"chip":{"task":3}}), json!({"chip":{"task":4}})]);
    assert_eq!(child_rows(&live, &json!({"ended":"none"})).expect("no ended children").len(), 1);
    let mut cut = ended.clone();
    cut["ended"]["rows"]["cut"] = true.into();
    assert!(child_rows(&live, &cut).is_err(), "a partial set cannot prove no change");
    let mut more = live;
    more["tasks"]["more"] = true.into();
    assert!(child_rows(&more, &ended).is_err(), "an unread live child cannot disappear");
    let question = json!({"event":{"owner":{"source":{"line":{"words":{
        "kind":"question","by":{"task":2},"message":7,"words":"May I go ahead with this plan?"
    }}}}}});
    assert_eq!(plan_question(std::slice::from_ref(&question), 2, 7), Ok("May I go ahead with this plan?"));
    assert!(plan_question(std::slice::from_ref(&question), 3, 7).is_err(), "actual author, not just any question");
    assert!(plan_question(std::slice::from_ref(&question), 2, 8).is_err(), "the exact question in the person's inbox");
}

#[test]
fn a_rung_recognizes_the_exact_delivered_approval_in_explicit_message_activity() {
    use serde_json::json;
    use support::forge_land::rung::approval_at;
    let mut delivered = json!({"position":9,"at":42,"event":{"owner":{"source":{"line":{"words":{
        "message":3,"by":{"task":1},"kind":{"answer":{"question":1}},
        "words":"Go ahead. The CSV's first line must be the header fruit,count."
    }}}}}});
    assert_eq!(approval_at(std::slice::from_ref(&delivered), 1), Some(42));
    assert_eq!(approval_at(std::slice::from_ref(&delivered), 2), None, "the chat must send it");
    delivered["event"]["owner"]["source"]["line"]["words"]["words"] = "Go ahead.".into();
    assert_eq!(approval_at(&[delivered], 1), None, "the header condition must reach the work too");
}

#[test]
fn a_rungs_setup_keeps_the_final_json_answer_after_progress_and_refuses_a_broken_stream() {
    use support::forge_land::rung::ctl_json;
    assert_eq!(ctl_json(b"{\"setting_up\":3}\n{\"project\":2}\n"), Ok(serde_json::json!({"project":2})));
    assert!(ctl_json(b"{\"setting_up\":3}\nnot JSON\n{\"project\":2}").is_err());
    assert!(ctl_json(b"").is_err());
}

#[test]
fn the_owned_example_is_named_without_a_problem() {
    // Only naming a fixture document; no deployment or files are opened.
    let named = temper_service::named(support::owned::EXAMPLE, &jig_protocol_config::Directories::working())
        .expect("owned example named");
    assert_eq!(named.worker.as_ref().expect("standalone worker").agent.as_ref(), b"agent.json");
    assert_eq!(named.worker.as_ref().expect("standalone worker").root.as_ref(), b"workspaces");
    assert_eq!(named.deployment.as_ref(), b"state");
    for path in [b"forge-1/api_token".as_slice(), b"forge-1/user", b"forge-1/webhook_secret", b"forge-1/git_token"] {
        assert_eq!(
            named.files.iter().find(|file| file.path.as_ref() == path).map(|file| file.private),
            None,
            "records-only owned input names no file: {path:?}"
        );
    }
}

#[test]
fn embedded_files_refuse_each_spoiled_bundle_and_limit() {
    use temper_service::limits::Clients;
    let long = format!("0123456789abcdef/{}.js", "a".repeat(PATH_STEM_BOUND + 1)); // 1025 bytes below the digest.
    let rows = [
        ("no page", BUNDLE_FILES[1..].to_vec(), BUNDLE_LIMITS, "index.html: absent".to_owned()),
        ("no digest", BUNDLE_FILES[..1].to_vec(), BUNDLE_LIMITS, "no digest directory".to_owned()),
        (
            "two digests",
            vec![("index.html", PAGE), ("0123456789abcdef/a.js", b""), ("fedcba9876543210/a.js", b"")],
            BUNDLE_LIMITS,
            "fedcba9876543210: more than one digest directory".to_owned(),
        ),
        (
            "bad digest",
            vec![("index.html", PAGE), ("0123456789abcdeF/a.js", b"")],
            BUNDLE_LIMITS,
            "0123456789abcdeF: digest must be 16 lowercase hex digits".to_owned(),
        ),
        (
            "short digest",
            vec![("index.html", PAGE), ("abc/a.js", b"")],
            BUNDLE_LIMITS,
            "abc: digest must be 16 lowercase hex digits".to_owned(),
        ),
        (
            "parent path",
            vec![("index.html", PAGE), ("0123456789abcdef/../a.js", b"")],
            BUNDLE_LIMITS,
            "0123456789abcdef/../a.js: path must be plain and at most 1024 bytes".to_owned(),
        ),
        (
            "dot path",
            vec![("index.html", PAGE), ("0123456789abcdef/./a.js", b"")],
            BUNDLE_LIMITS,
            "0123456789abcdef/./a.js: path must be plain and at most 1024 bytes".to_owned(),
        ),
        (
            "empty component",
            vec![("index.html", PAGE), ("0123456789abcdef//a.js", b"")],
            BUNDLE_LIMITS,
            "0123456789abcdef//a.js: path must be plain and at most 1024 bytes".to_owned(),
        ),
        (
            "long path",
            vec![("index.html", PAGE), (long.as_str(), b"")],
            BUNDLE_LIMITS,
            format!("{long}: path must be plain and at most 1024 bytes"),
        ),
        (
            "closed list",
            vec![("index.html", PAGE), ("0123456789abcdef/data.bin", b"")],
            BUNDLE_LIMITS,
            "0123456789abcdef/data.bin: file type is not on the closed list".to_owned(),
        ),
        (
            "files",
            BUNDLE_FILES.to_vec(),
            Clients { bundle_files: BUNDLE_COUNT - 1, ..BUNDLE_LIMITS },
            "0123456789abcdef/files/robots.txt: past clients.bundle_files".to_owned(),
        ),
        (
            "bytes",
            BUNDLE_FILES.to_vec(),
            Clients { bundle_bytes: BUNDLE_BYTES - 1, ..BUNDLE_LIMITS },
            "0123456789abcdef/files/robots.txt: past clients.bundle_bytes".to_owned(),
        ),
    ];
    for (name, files, limits, why) in rows {
        assert_eq!(
            temper_box_shell::bundle::embedded(&files, limits).expect_err("refused"),
            format!("application.clients.bundle: the binary's bundle: {why}"),
            "{name}"
        );
    }
}

#[test]
fn embedded_files_at_each_bound_are_admitted() {
    let bound = format!("0123456789abcdef/{}.js", "a".repeat(PATH_STEM_BOUND));
    for (name, files, limits) in [
        (
            "files and bytes",
            BUNDLE_FILES.to_vec(),
            temper_service::limits::Clients { bundle_files: BUNDLE_COUNT, bundle_bytes: BUNDLE_BYTES },
        ),
        ("path", vec![("index.html", PAGE), (bound.as_str(), b"")], BUNDLE_LIMITS),
    ] {
        temper_box_shell::bundle::embedded(&files, limits).unwrap_or_else(|why| panic!("{name}: {why}"));
    }
}

#[test]
fn a_story_purse_is_priced_from_the_init_template() {
    assert!(support::first_repository::run_spend(5).is_power_of_two());
}

#[test]
fn the_store_page_holds_the_templates_largest_turn_frame() {
    let bounded = support::owned::bounded_template(Path::new("/fixture"));
    let store = temper_service::startup::store_limits(
        &bounded.document,
        &jig_protocol_config::Directories::working(),
        Some(&bounded.agent.declaration()),
    )
    .expect("template store limits");
    let turn = u64::from(bounded.agent.limits.bodies.turn_body);
    eprintln!("template T={turn}, kv.page.bytes={}, T+64={}", store.store.store.page.bytes, turn + 64);
    assert!(u64::from(store.store.store.page.bytes) >= turn + 64);
}

#[test]
fn the_reference_profile_passes_startup_admission() {
    let profile = support::owned::bounded_reference(Path::new("/fixture"));
    let admission = profile.admission();
    if let Err(why) = admission {
        panic!("{}", String::from_utf8_lossy(&temper_service::startup::lines(&why)));
    }
}

#[test]
fn checked_in_deployment_profiles_pass_startup_admission() {
    let profiles = [
        "docs/examples/owned.json".to_owned(),
        "init --box".to_owned(),
        format!("bin/temper-box/tests/fixtures/cluster/engine/{}", temper_config_shell::FILE),
        format!("bin/temper-box/tests/fixtures/cluster/w1/{}", temper_config_shell::FILE),
        format!("bin/temper-box/tests/fixtures/cluster/w2/{}", temper_config_shell::FILE),
    ];
    let mut failures = String::new();
    for relative in &profiles {
        let profile = match relative.as_str() {
            "docs/examples/owned.json" => support::owned::bounded(Path::new("/fixture")),
            "init --box" => support::owned::bounded_template(Path::new("/fixture")),
            _ => support::owned::deployment_profile(Path::new("/fixture"), relative),
        };
        let turn = profile.agent.limits.bodies.turn_body;
        let record = size_of::<temper_engine_domain::Record>();
        match profile.admission() {
            Ok(()) => eprintln!("{relative}: startup admission passed; T={turn}, Record={record}"),
            Err(why) => {
                writeln!(
                    failures,
                    "{relative}: T={turn}, Record={record}\n{}",
                    String::from_utf8_lossy(&temper_service::startup::lines(&why))
                )
                .expect("diagnostic");
            }
        }
    }
    assert!(failures.is_empty(), "{failures}");
}

fn turn_bound_members(document: &serde_json::Value, source: &str, turn: u32) {
    assert!(turn > 0, "the generated agent has a bounded turn");
    for name in ["engine", "application"] {
        if let Some(part) = document.get(name) {
            assert!(part.get("limits").is_none(), "{source}: {name}.limits moved to settings");
        }
    }
    if let Some(workers) = document["workers"].as_array() {
        for worker in workers {
            assert!(worker.get("limits").is_none(), "{source}: workers take engine figures");
        }
    }
}

#[test]
fn turn_bounds() {
    use configuration::turn_bounds::turn_body;
    use support::{cluster, configuration, live_github, turn_landing_fixture};

    let tree = skein_scratch::paths::tree();
    let root = Path::new("/fixture");
    for provider_key in [false, true] {
        let agent = temper_agent_fixtures::stock(provider_key);
        let document = serde_json::from_slice(&temper_config_shell::init_document(provider_key)).expect("init start");
        turn_bound_members(&document, "init", turn_body(&agent));
    }
    let owned = serde_json::from_slice(support::owned::EXAMPLE).expect("owned JSON");
    let agent = temper_agent_fixtures::configuration(&owned);
    turn_bound_members(&owned, "docs/examples/owned.json", turn_body(&agent));

    let cluster = cluster::Cluster::documents(
        root,
        &[("forge", "address", "127.0.0.1:3"), ("llm", "address", "127.0.0.1:1"), ("engine", "workers", "127.0.0.1:2")],
    );
    let rendered = cluster.render_files("engine");
    let engine = serde_json::from_slice(&rendered.start.expect("cluster engine").document).expect("cluster JSON");
    let turn = turn_body(&cluster::agent_configuration(&engine, root));
    let path = tree.join("bin/temper-box/tests/fixtures/cluster/engine").join(temper_config_shell::FILE);
    let document = configuration::engine_template(&path, true, &[]);
    turn_bound_members(&document, &path.display().to_string(), turn);
    eprintln!("cluster fixture: T={turn}");

    let turn = temper_playground::documents::agent().limits.bodies.turn_body;
    let document = support::playground::story_a_document();
    turn_bound_members(&document, "crates/temper-playground/stories/a/deployment.json", turn);
    eprintln!("story A: T={turn}");

    let inert = "127.0.0.1:9";
    let cluster_turn = support::cluster_turn::engine_document(&[
        ("story.clients", inert),
        ("story.workers", inert),
        ("story.webhooks", inert),
        ("proc.llm.ready.address", inert),
        ("proc.forge.ready.address", inert),
        ("proc.engine.ready.workers", inert),
    ]);
    let qa = tree.join("docs/development/qa/setups");
    for (source, document) in [
        ("cluster-turn/engine/temper.toml", cluster_turn),
        (
            "turn-landing/config/engine.toml",
            turn_landing_fixture::engine_template(&qa.join("turn-landing"), root, inert),
        ),
        (
            "stock-landing/config/engine.toml",
            turn_landing_fixture::engine_template(&qa.join("stock-landing"), root, inert),
        ),
    ] {
        let mut engine = document.clone();
        // Only the LLM profile affects the turn; the worker listener is inert.
        engine["engine"]["hosts"]["listener"] = serde_json::json!([{"socket":inert}]);
        let turn = turn_body(&cluster::agent_configuration(&engine, root));
        turn_bound_members(&document, source, turn);
    }
    let engine = live_github::engine_document(
        root,
        &[
            ("proc.llm.ready.address", "127.0.0.1:1"),
            ("proc.forge.ready.address", "127.0.0.1:2"),
            ("story.clients", "127.0.0.1:3"),
        ],
    );
    let turn = turn_body(&cluster::agent_configuration(&engine, root));
    let document = live_github::engine_document(root, &[("proc.forge.ready.address", inert), ("story.clients", inert)]);
    turn_bound_members(&document, "github/engine/temper.toml", turn);
}

#[test]
fn the_base_recipe_passes_startup_admission() {
    let profile = support::recipe(Path::new("/fixture"), &support::Forge::Unreached);
    support::admission_over(
        &profile.read_start().document,
        &profile.configuration,
        &profile.files,
        &profile.agent.agent,
        &profile.given,
    )
    .expect("base recipe admission");
}

#[test]
fn rung_b0s_live_configuration_is_admitted_without_an_account_or_backend() {
    let root = Path::new("/fixture/rung-b0-admission");
    let paths = ["config", "data", "cache"].map(|name| root.join(name));
    let profile = support::forge_land::rung::deployment_profile(&paths);
    let result = temper_box_shell::testing::serve::operations_with_copies(
        "temper",
        &profile.directories,
        &profile.document,
        &temper_service::stock::Given::Stock,
        Some((&profile.configuration.jig, &profile.configuration.limits)),
        Ok(None), // Init writes no local.setting on a fresh deployment.
        Ok(Some(profile.agents)),
    );
    assert!(result.is_ok(), "B0's real configuration admits before any live account: {:?}", result.err());
}
