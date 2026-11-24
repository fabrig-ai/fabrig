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

#[test]
fn serve_refuses_an_explicit_call_bound_below_the_agents_call_input() {
    let mut document = owned_document();
    let agent = owned_agent(&document);
    let files = owned_files(&document);
    // The owned agent declares tool_payload 63740; one below refuses, equality prepares.
    for (bound, expected) in [
        (
            63739,
            Some(concat!(
                "engine.limits.overrides.core.call_answer_bytes: 63739 bytes, short of the agent's call input (profile.declared.tool_payload), 63740 bytes; raise it to at least 63740 bytes, or omit it to follow the agent's call input\n",
                "engine.limits.overrides.core.call_answer_bytes: call input 63740 bytes; wire answer 63739 bytes; smith host_reply_bytes 63740 bytes\n",
            )),
        ),
        (63740, None),
    ] {
        document.jig_over("core.call_answer_bytes", bound);
        let prepared = support::admission_over(
            &serde_json::to_vec(&document).expect("owned JSON"),
            &document,
            &files,
            &agent,
            &temper_service::stock::Given::Stock,
        );
        match expected {
            Some(lines) => assert_eq!(prepared.expect_err("call input refused"), lines),
            None => assert!(prepared.is_ok(), "{}", prepared.err().unwrap_or_default()),
        }
    }
}

#[test]
fn serve_checks_the_owned_agents_admission_at_each_bound() {
    let document = owned_document();
    let mut figures = temper_playground::documents::Configuration::stock((*document).clone());
    figures.jig_over("tasks.message_bytes", 10660);
    figures.jig_over("tasks.result_bytes", 10660);
    figures.jig_over("tasks.inbox_bytes", 874_120);
    figures.limits.tool_payload = 10662;
    let providers = temper_service::stock::providers();
    let original = temper_agent_fixtures::with_figures(
        &document,
        &temper_service::stock::agents(),
        Path::new(OWNED_ROOT),
        &providers,
    );
    let files = owned_files(&document);
    for (member, value, expected) in [
        (
            "max_turns",
            7,
            Some("engine.charters[0].turns: needs 64 turns; the agent's profile.policy.max_turns allows 7\n"),
        ),
        (
            "max_turns",
            15,
            Some("engine.charters[0].turns: needs 64 turns; the agent's profile.policy.max_turns allows 15\n"),
        ),
        (
            "max_turns",
            799,
            Some("engine.charters[1].turns: needs 800 turns; the agent's profile.policy.max_turns allows 799\n"),
        ),
        ("max_turns", 800, None),
        (
            "check_timeout",
            300_001,
            Some(
                "engine.charters[1].reserve_time_ms: needs checks within 300000 ms; the agent's profile.policy.check_timeout is 300001 ms; raise charter 2's reserve_time (temper help settings) or lower the timeout\n",
            ),
        ),
        ("check_timeout", 300_000, None),
        ("check_timeout", 60_000, None),
        (
            "tool_payload",
            63739,
            Some(
                "workers[0].limits.overrides.hub.event_bytes: 63738 bytes, past agents setting declared.tool_payload less the 2-byte separator of 63737\n",
            ),
        ),
        ("tool_payload", 63740, None),
        (
            "tool_payload",
            600,
            Some(
                "engine.limits: needs 10660-byte messages (tasks.message_bytes); the agent offers 598 (tool_payload less 2)\ntemper serve: workers[0].agent.profile.declared.tool_payload: 600 bytes, short of temper's documentation answer (read_docs), 2048 bytes\n",
            ),
        ),
    ] {
        let mut agent = original.clone();
        let section = if member == "tool_payload" { "declared" } else { "policy" };
        agent["profile"][section][member] = value.into();
        support::configuration::turn_bounds::apply(
            &mut figures,
            support::configuration::turn_bounds::turn_body(&agent),
        );
        let result = prepare_owned_edit(&document, &figures, &files, &agent, &providers);
        match expected {
            Some(line) if member != "tool_payload" => {
                let mut service = result.expect("charter shortfalls warn");
                assert_eq!(
                    service.notes().pop(),
                    Some(temper_service::Note::Engine(temper_service::EngineNote::StartupWarning {
                        line: line.trim_end().as_bytes().into()
                    })),
                    "{member}={value}"
                );
                assert_eq!(service.notes().pop(), None);
            }
            Some(line) => assert_eq!(
                result.err().expect("admission refused"),
                format!("temper serve: {line}"),
                "{member}={value}"
            ),
            None => assert!(result.is_ok(), "{member}={value}: {}", result.err().unwrap_or_default()),
        }
    }
    let mut agent = original.clone();
    agent["profile"]["policy"]["max_turns"] = 5.into();
    agent["profile"]["policy"]["check_timeout"] = 300_000.into();
    agent["profile"]["declared"]["tool_payload"] = 600.into();
    support::configuration::turn_bounds::apply(&mut figures, support::configuration::turn_bounds::turn_body(&agent));
    assert_eq!(
        prepare_owned_edit(&document, &figures, &files, &agent, &providers).err().expect("old tuple refused"),
        "temper serve: engine.limits: needs 10660-byte messages (tasks.message_bytes); the agent offers 598 (tool_payload less 2)\ntemper serve: workers[0].agent.profile.declared.tool_payload: 600 bytes, short of temper's documentation answer (read_docs), 2048 bytes\n"
    );
}

fn configuration_document(value: &serde_json::Value) -> skein_json::Document {
    jig_protocol_config::document(
        &serde_json::to_vec(value).expect("configuration JSON"),
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 65_536, depth: 32 },
    )
    .expect("configuration document")
}

#[test]
fn the_shipped_chat_is_ended_by_its_requester() {
    let bounded = support::owned::bounded(Path::new(OWNED_ROOT));
    let value: serde_json::Value = serde_json::from_slice(&bounded.document).expect("deployment JSON");
    let engine = temper_service::startup::engine(
        &configuration_document(&value["engine"]),
        0,
        &jig_protocol_config::Directories::working(),
    )
    .expect("engine configuration");
    assert_eq!(engine.templates[0].number, 1);
    assert_eq!(engine.templates[0].flags.ended_by, jig_domain_tasks::EndedBy::Requester);
    assert_eq!(engine.templates[1].number, 2);
    assert_eq!(engine.templates[1].flags.ended_by, jig_domain_tasks::EndedBy::Result);
}

#[test]
fn shipped_bounds_fit_the_configured_charters_and_worker() {
    use jig_protocol_config as config;
    use temper_service as service;

    let bounded = support::owned::bounded(Path::new(OWNED_ROOT));
    let value: serde_json::Value = serde_json::from_slice(&bounded.document).expect("deployment JSON");
    let engine =
        temper_service::startup::engine(&configuration_document(&value["engine"]), 0, &config::Directories::working())
            .expect("engine configuration");
    let agent = &bounded.agent;
    let core = config::core(&engine, [0; 16], 1);
    let needs = jig_domain_core::settings::fleet_needs::configured(&core, &engine.limits.core);
    assert_eq!(
        jig_domain_fleet::shortfall(&needs, &agent.bounds.clone(), jig_domain_fleet::HostKind::Worker,),
        None,
        "the shipped worker is admitted by the canonical fleet check"
    );
    // The minimum Smith payload is the engine's message requirement plus
    // Smith's rendered separator; its advertised message excludes that separator.
    let payload = u64::from(engine.limits.core.tasks.message_bytes)
        .checked_add(temper_worker_domain::SEPARATOR)
        .expect("rendered message");
    assert!(agent.host_limits.message_bytes >= payload);
    let named = service::named(&bounded.document, &config::Directories::working()).expect("owned configuration");
    let mut jig = config::worker(
        &configuration_document(&value["workers"][0]),
        0,
        &config::Directories::working(),
        &temper_service::startup::fixed_worker(),
    )
    .expect("worker configuration");
    jig.limits = Some(engine.limits);
    jig.agent = Some(temper_playground::documents::pinned_agent());
    let worker = &service::settings::Worker {
        jig,
        root: named.worker.expect("standalone worker").root,
        git: Some(b"/usr/bin/git".as_slice().into()),
    };
    let own = &temper_playground::documents::Configuration::stock(value.clone()).limits;
    assert!(
        worker.jig.limits.as_ref().expect("configured worker limits").hub.event_bytes
            >= u64::from(agent.bounds.message_bytes)
    );
    assert!(u64::from(own.checkout.message_bytes) >= agent.host_limits.call_bytes);
    let prepared = service::worker_settings::prepare(worker, agent, &service::limits::worker(own))
        .expect("complete worker bounds");
    // Each receiver one byte below its sender's bound fails its own rule.
    let mut short = prepared.root.clone();
    short.host.event_bytes = u64::from(agent.bounds.message_bytes).checked_sub(1).expect("nonempty message");
    assert_eq!(temper_worker_domain::unfit(&short).as_slice(), &[temper_worker_domain::Unfit::BoundsMessage]);
    let mut short = prepared.root.clone();
    short.checkout.message_bytes = u32::try_from(agent.host_limits.call_bytes)
        .expect("call fits checkout bound")
        .checked_sub(1)
        .expect("nonempty call");
    assert_eq!(temper_worker_domain::unfit(&short).as_slice(), &[temper_worker_domain::Unfit::CallBytes]);
}

const OWNED_ROOT: &str = "/fixture/owned";

fn owned_document() -> temper_playground::documents::Configuration {
    let read = support::owned::example_read();
    temper_playground::documents::Configuration::from_jig(serde_json::from_slice(&read.document).expect("owned JSON"))
}

fn owned_agent(document: &serde_json::Value) -> temper_service::settings::Agent {
    let agent = temper_agent_fixtures::with_figures(
        document,
        &temper_service::stock::agents(),
        Path::new(OWNED_ROOT),
        &temper_service::stock::providers(),
    );
    support::prepare_agent(serde_json::to_vec(&agent).expect("agent JSON").into_boxed_slice()).agent
}

fn problems_text(why: &temper_service::startup::Problems) -> String {
    String::from_utf8_lossy(&temper_service::startup::lines(why)).into_owned()
}

fn prepare_owned_edit(
    document: &serde_json::Value,
    limits: &temper_playground::documents::Configuration,
    files: &temper_service::settings::Files,
    edited: &serde_json::Value,
    providers: &[jig_protocol_config::ProviderSetting],
) -> Result<temper_service::Service, String> {
    let mut document = document.clone();
    let setting = temper_agent_fixtures::compose_setting(&mut document, edited, providers);
    let given = temper_agent_fixtures::providers::given_with(setting.providers.clone(), None);
    let agents = temper_box_shell::testing::serve::agents_setting_over(Ok(Some(setting.bytes())), &mut std::io::sink())
        .expect("encoded agent copy");
    let bytes = serde_json::to_vec(&document).expect("owned JSON");
    let part = temper_service::startup::box_worker_with_limits(
        &bytes,
        &jig_protocol_config::Directories::working(),
        &given,
        &agents,
        4,
        &[],
        None,
        (&limits.jig, &limits.limits),
    )
    .map_err(|why| temper_service_shell::serve_error("temper", &problems_text(&why)))?
    .expect("owned worker");
    let agent = temper_worker_shell::agent::figures::configuration(
        part.figures.as_ref().expect("box figures"),
        None,
        Path::new(OWNED_ROOT),
        &part.environment,
    )
    .map_err(|why| temper_service_shell::serve_error("temper", &why))?;
    let agent = support::prepare_agent(agent).agent;
    prepare_box(&jig_protocol_config::Directories::working(), &bytes, Some(limits), files, &given, &agents, &agent, 4)
        .map_err(|why| temper_service_shell::serve_error("temper", &why))
}

/// The box's typed startup after agent preflight, without adopting a store or root.
#[expect(
    clippy::too_many_arguments,
    reason = "checked startup uses the same supplied document, copies and files as the box"
)]
fn prepare_box(
    directories: &jig_protocol_config::Directories,
    bytes: &[u8],
    limits: Option<&temper_playground::documents::Configuration>,
    files: &temper_service::settings::Files,
    given: &temper_service::stock::Given,
    agents: &temper_client_protocol_wire::v1::AgentFigures,
    agent: &temper_service::settings::Agent,
    places: u32,
) -> Result<temper_service::Service, String> {
    use support::admission::{construct_over, seeds};
    use temper_service::startup::{self, LimitInputs};

    let seeds = seeds();
    let limits = limits.map(|limits| (&limits.jig, &limits.limits));
    let inputs = LimitInputs::new(bytes, directories, files, Some(&agent.declaration()), &seeds, given, true, limits);
    let resolved = startup::resolve_limits(&inputs, &[], &[]).map_err(|why| problems_text(&why))?;
    let part = startup::box_worker_with_limits(
        bytes,
        directories,
        given,
        agents,
        places,
        &[],
        Some(&agent.declaration()),
        (&resolved.jig, &resolved.temper),
    )
    .map_err(|why| problems_text(&why))?;
    let checked = startup::checked_start_with_resolved(
        bytes,
        directories,
        files,
        Some(agent),
        &seeds,
        skein_lib::List::with_capacity(0),
        None,
        given,
        None,
        part,
        &resolved,
    )
    .map_err(|why| problems_text(&why))?;
    construct_over(checked, files, Some(agent), &seeds, given)
}

fn owned_files(document: &temper_playground::documents::Configuration) -> temper_service::settings::Files {
    read_owned_files(document, |_, _, _| Err(skein_shell::Unread::Absent))
}

fn named_owned(document: &temper_playground::documents::Configuration) -> temper_service::Named {
    temper_service::startup::named_with_limits(
        &serde_json::to_vec(document).expect("owned JSON"),
        &jig_protocol_config::Directories::working(),
        &temper_service::stock::Given::Stock,
        &document.jig,
        &document.limits,
    )
    .expect("owned example named")
}

fn read_owned_files(
    document: &temper_playground::documents::Configuration,
    mut extra: impl FnMut(&Path, u32, bool) -> Result<skein_lib::Secret, skein_shell::Unread>,
) -> temper_service::settings::Files {
    let named = named_owned(document);
    assert!(named.files.iter().chain(&named.asked_files).all(|file| !file.path.starts_with(b"forge-1/")));
    let inventory = support::owned::example_files();
    temper_service_shell::read_files_with_asked_using(named.files, named.asked_files, 1000, |path, max, private| {
        let key = path.as_os_str().as_encoded_bytes();
        match inventory.entries.iter().find(|(name, _)| name.as_ref() == key) {
            Some((_, bytes)) => Ok(bytes.clone()),
            None => extra(path, max, private),
        }
    })
    .expect("owned named files")
}

#[test]
fn the_owned_example_starts_without_credential_files() {
    let document = owned_document();
    let files = read_owned_files(&document, |path, _, _| panic!("unexpected read of {}", path.display()));
    prepare_box(
        &jig_protocol_config::Directories::working(),
        &serde_json::to_vec(&document).expect("owned JSON"),
        None,
        &files,
        &temper_service::stock::Given::Stock,
        &temper_service::stock::agents(),
        &owned_agent(&document),
        4,
    )
    .expect("owned example prepares without forge files");
}

fn eight_slots() -> temper_playground::documents::Configuration {
    let mut document = owned_document();
    document.jig_over("hub.slots", 8);
    document.jig_over("fleet.slots", 8);
    document.temper_over("checkout.workspaces", 10);
    document.temper_over("git_component.operations", 10);
    document
}

#[test]
fn serve_prepares_eight_slots_and_ten_workspaces_on_the_owned_example() {
    let document = eight_slots();
    support::admission_over(
        &serde_json::to_vec(&document).expect("owned JSON"),
        &document,
        &owned_files(&document),
        &owned_agent(&document),
        &temper_service::stock::Given::Stock,
    )
    .expect("eight-place startup");
    assert!(document["engine"].get("limits").is_none());
}

#[test]
fn serve_sizes_its_worker_from_eight_setting_places_not_document_slots() {
    let mut document = eight_slots();
    let files = owned_files(&document);
    let agent = owned_agent(&document);
    let given = temper_service::stock::Given::Stock;
    let agents = temper_service::stock::agents();
    support::admission_over(&serde_json::to_vec(&document).expect("owned JSON"), &document, &files, &agent, &given)
        .expect("complete figures");

    let mut log = Vec::new();
    let local = temper_box_shell::testing::serve::local_setting_over(
        Ok(Some(support::limit_settings::local_copy(8))),
        &mut log,
    )
    .expect("worker setting");
    assert_eq!(local.places, 8);
    assert_eq!(document["workers"][0]["slots"], 1, "the file carries no places");
    assert_eq!(log, [] as [u8; 0]);
    document.jig_over("hub.slots", 7);
    document.temper_over("checkout.workspaces", 7);
    document.temper_over("git_component.operations", 7);
    let bytes = serde_json::to_vec(&document).expect("owned JSON");
    let part = temper_service::startup::box_worker_with_limits(
        &bytes,
        &jig_protocol_config::Directories::working(),
        &given,
        &agents,
        local.places,
        &[],
        None,
        (&document.jig, &document.limits),
    )
    .expect("engine figures")
    .expect("worker");
    assert_eq!(part.worker.jig.slots, 8);
    assert_eq!(part.worker.jig.limits.expect("applied figures").hub.slots, 8);
    assert_eq!(part.limits.checkout.workspaces, 8);
    assert_eq!(part.limits.git_component.operations, 8);
    prepare_box(
        &jig_protocol_config::Directories::working(),
        &bytes,
        Some(&document),
        &files,
        &given,
        &agents,
        &agent,
        local.places,
    )
    .expect("derived capacities prepare");
}

#[test]
fn serve_prepares_once_on_a_bare_init() {
    let generated = temper_init::bare(
        &temper_init::Bare { binary: b"/fixture/temper-box", git: Some(b"/fixture/git"), claude: None },
        &temper_init::Directories { config: b"/fixture/init/config", data: b"/fixture/init/data" },
        &[7; 32],
    )
    .expect("bare init");
    assert!(generated.iter().all(|file| !file.name.starts_with(b"demo")), "bare init creates no demo");
    let mut entries = skein_lib::List::with_capacity(u32::try_from(generated.len()).expect("bare init file count"));
    for file in generated.into_vec() {
        let temper_init::Content::Bytes { bytes, .. } = file.content else { panic!("bare init only writes bytes") };
        entries.push((file.name.into(), skein_lib::Secret::new(bytes))).expect("bare init files");
    }
    let bytes = temper_config_shell::init_document(false);
    let document = serde_json::from_slice(&bytes).expect("init start document");
    prepare_box(
        &jig_protocol_config::Directories::working(),
        &bytes,
        None,
        &temper_service::settings::Files { entries },
        &temper_service::stock::Given::Stock,
        &temper_service::stock::agents(),
        &owned_agent(&document),
        4,
    )
    .expect("bare init prepares from its generated files");
}

#[test]
fn the_live_github_configuration_passes_startup_admission() {
    use temper_playground::documents::Configuration;
    use temper_worker_service::{figures, startup};

    let root = Path::new("/fixture/live-github");
    let templates = skein_scratch::paths::package().join("tests/fixtures/github");
    let api = "127.0.0.1:31001".parse().expect("synthetic API address");
    let git = "127.0.0.1:31002".parse().expect("synthetic git address");
    let workers = "127.0.0.1:31003".parse().expect("synthetic worker address");
    let mut cluster = support::cluster::Cluster::description_from(root, &templates);
    for (name, value) in [
        ("proc.forge.ready.address", "127.0.0.1:31002"),
        ("proc.llm.ready.address", "127.0.0.1:31004"),
        ("proc.engine.ready.workers", "127.0.0.1:31003"),
        ("story.clients", "127.0.0.1:31005"),
    ] {
        cluster.bind(name, value);
    }
    let rendered = cluster.render_files("engine");
    let mut engine = Configuration::from_jig(
        serde_json::from_slice(&rendered.start.expect("engine start").document).expect("engine JSON"),
    );
    support::live_github::configure_engine(&mut engine, "fabrig-ai", api, git, workers);
    let providers = rendered
        .files
        .iter()
        .find(|(path, _)| path.ends_with("providers.setting"))
        .map(|(_, bytes)| temper_agent_fixtures::providers::parse(bytes))
        .expect("GitHub fixture providers");
    let bytes = serde_json::to_vec(&engine).expect("GitHub engine JSON");
    // The fixture's provider copy is not part of the product document.
    engine["world_providers"] = temper_agent_fixtures::providers::flat(&providers);
    let given = temper_agent_fixtures::providers::given_with(providers, None);
    let directories = jig_protocol_config::Directories::working();
    let named = temper_service::startup::named_with_limits(&bytes, &directories, &given, &engine.jig, &engine.limits)
        .expect("GitHub engine named");
    let files = rendered_files(named.files, named.asked_files, &rendered.files);
    let service =
        support::admission::startup_over(&bytes, &engine, &files, None, &given).expect("live GitHub engine admission");
    assert!(service.operations() > 0);

    let rendered = cluster.render_files("w1");
    let data = cluster.dir("w1").join("data");
    let directories = temper_service_shell::jig_directories(&support::configuration::directories(&data));
    let bytes = rendered.start.expect("worker start").document;
    let mut part = startup::worker_part(&bytes, &directories, None, &[]).expect("worker part");
    part.named.program = b"/fixture/temper-box".as_slice().into();
    part.named.arguments = Box::new([part.named.agent.clone()]);
    let files = rendered_files(core::mem::take(&mut part.files), Box::new([]), &rendered.files);
    figures::apply(&mut part, support::cluster::figures_with_limits(&engine)).expect("applied GitHub engine figures");
    let bytes = temper_worker_shell::agent::figures::configuration(
        part.figures.as_ref().expect("worker figures"),
        None,
        &data,
        &part.environment,
    )
    .expect("worker agent bytes");
    let agent = support::prepare_agent(bytes).agent;
    let service = startup::worker::worker_alone(part, &files, &agent, support::admission::worker_runtime(), 7)
        .expect("live GitHub w1 admission");
    assert!(service.operations() > 0);
}

fn rendered_files(
    required: Box<[jig_protocol_config::File]>,
    asked: Box<[jig_protocol_config::File]>,
    rendered: &[(std::path::PathBuf, Vec<u8>)],
) -> temper_service::settings::Files {
    temper_service_shell::read_files_with_asked_using(required, asked, 1000, |path, _, _| {
        rendered
            .iter()
            .find(|(file, _)| file == path)
            .map(|(_, bytes)| skein_lib::Secret::new(bytes.as_slice().into()))
            .ok_or(skein_shell::Unread::Absent)
    })
    .expect("composed named files")
}

#[test]
fn machine_startup_leaves_unconnected_forge_handover_files_unread() {
    use skein_io::{file::Unsafe, kernel::Error};
    use skein_lib::Secret;
    use skein_shell::Unread;

    let document = owned_document();
    let agent = owned_agent(&document);
    let inventory = support::owned::example_files();
    let siblings = ["api_token", "user", "git_token", "webhook_secret"];
    let forge = Path::new("forge-1");
    let token = forge.join("admin_token");
    for (row, handover) in [
        ("absent", Err(Unread::Absent)),
        ("exact", Ok(vec![b'x'; 4096])),
        ("over", Ok(vec![b'x'; 4097])),
        ("newline", Ok(b"test-admin\n".to_vec())),
        ("public", Err(Unread::Unsafe(Unsafe::Mode(0o644)))),
        ("link", Err(Unread::Unsafe(Unsafe::Link))),
        ("parent-link", Err(Unread::Unsafe(Unsafe::Link))),
        ("directory", Err(Unread::NotAFile)),
        ("unreadable (mode 000)", Err(Unread::Kernel(Error::Permission))),
    ] {
        let named = named_owned(&document);
        let mut reads = Vec::new();
        let result =
            temper_service_shell::read_files_with_asked_using(named.files, named.asked_files, 1000, |path, max, _| {
                reads.push(path.to_path_buf());
                if path == token {
                    return handover.as_ref().map_err(|why| *why).and_then(|bytes| {
                        if bytes.len() > usize::try_from(max).expect("file bound") {
                            Err(Unread::TooLarge { max })
                        } else {
                            Ok(Secret::new(bytes.as_slice().into()))
                        }
                    });
                }
                if siblings.iter().any(|name| path == forge.join(name)) {
                    return Err(Unread::Unsafe(Unsafe::Mode(0o644)));
                }
                inventory
                    .entries
                    .iter()
                    .find(|(name, _)| name.as_ref() == path.as_os_str().as_encoded_bytes())
                    .map(|(_, bytes)| bytes.clone())
                    .ok_or(Unread::Absent)
            });
        assert_eq!(reads, [Path::new("worker.secret")], "{row}: only the worker file is read");
        let files = result.unwrap_or_else(|why| panic!("{row}: {why}"));
        prepare_box(
            &jig_protocol_config::Directories::working(),
            &serde_json::to_vec(&document).expect("owned JSON"),
            None,
            &files,
            &temper_service::stock::Given::Stock,
            &temper_service::stock::agents(),
            &agent,
            4,
        )
        .unwrap_or_else(|why| panic!("unconnected machine leaves handover unread: {row}: {why}"));
    }
}

fn create_sign_in_credentials(
    root: &Path,
    directories: &temper_box_shell::directories::Directories,
    bytes: &[u8],
    named: &temper_engine_service::startup::Named,
) -> std::collections::BTreeMap<std::path::PathBuf, skein_lib::Secret> {
    use skein_lib::Secret;
    use std::collections::BTreeMap;
    use temper_engine_shell::forge_sign_in;

    // A committed, non-production key is linked into the test, never read at startup.
    let key = include_bytes!("../../../../../jig/crates/jig-protocol-config/src/tests/keys/rsa-3072-pkcs8.pem");
    let mut writes = Vec::new();
    let mut inventory =
        BTreeMap::from([(root.join("google.secret"), Secret::new(support::forge_sign_in::GOOGLE_SECRET.into()))]);
    let mut print = Vec::new();
    let status = forge_sign_in::run_with(
        directories,
        bytes,
        named,
        1,
        &[],
        |path, signing_key| {
            writes.push((path.to_path_buf(), signing_key));
            let bytes = if signing_key { key.as_slice() } else { b"fixture-forgejo-client-secret" };
            inventory.entry(path.into()).or_insert_with(|| Secret::new(bytes.into()));
            Ok(())
        },
        |instructions| {
            print.extend_from_slice(instructions);
            Ok(())
        },
    )
    .expect("forge sign-in 1");
    assert_eq!(status, 0);
    assert_eq!(writes, [(root.join("provider.key"), true), (root.join("forgejo.secret"), false)]);
    let print = std::str::from_utf8(&print).expect("sign-in instructions");
    assert!(print.contains(" admin auth add-oauth --name temper --provider openidConnect --key "));
    assert!(print.contains("http://127.0.0.1:7100/.well-known/openid-configuration"));
    inventory
}

#[test]
fn a_bundle_with_a_forge_sign_in_entry_starts_before_any_forge_is_kept() {
    use temper_engine_shell::serve;
    use temper_service::startup;

    let root = Path::new("/fixture/bundle-forge-sign-in");
    let directories = support::configuration::directories(root);
    let jig = temper_service_shell::jig_directories(&directories);
    let profile = support::recipe(root, &support::Forge::Unreached);
    let bundle_path = root.join("bundle");
    let mut file = profile.read_start().file.clone();
    file.internal.bundle = Some(bundle_path.clone());
    support::forge_sign_in::add_to_file(&mut file, "127.0.0.1:3000", "127.0.0.1:7100", "127.0.0.1:7200");
    let shape = temper_config_shell::Shape::Standalone;
    let bytes = support::configuration::parse(&temper_config_shell::render(&file, shape), shape).document;
    let figures = &profile.configuration;
    let given = &profile.given;
    let named =
        startup::named_with_limits(&bytes, &jig, given, &figures.jig, &figures.limits).expect("bundle start named");
    assert!(named.forges.is_empty());
    let mut named = named.into_engine();
    let sign_in = named.sign_in.as_ref().expect("configured Google sign-in");
    assert_eq!(sign_in.forges.len(), 1);
    assert_eq!(sign_in.forges[0].forge, 1);

    let inventory = create_sign_in_credentials(root, &directories, &bytes, &named);

    let mut headed = BUNDLE_FILES.to_vec();
    headed[0].1 = b"<!doctype html><head></head><body>temper</body>";
    let serve::Inputs { files, google, bundle } = serve::before_root_using(
        &mut named.files,
        &mut named.asked_files,
        named.sign_in.as_ref(),
        named.bundle.as_deref(),
        named.bundle_limits,
        false,
        |required, asked| {
            temper_service_shell::read_files_with_asked_using(required, asked, 1000, |path, _, _| {
                if let Some(bytes) = inventory.get(path) {
                    return Ok(bytes.clone());
                }
                let relative = path.strip_prefix(root).unwrap_or(path);
                profile
                    .files
                    .entries
                    .iter()
                    .find(|(name, _)| {
                        name.as_ref() == path.as_os_str().as_encoded_bytes()
                            || name.as_ref() == relative.as_os_str().as_encoded_bytes()
                    })
                    .map(|(_, bytes)| bytes.clone())
                    .ok_or(skein_shell::Unread::Absent)
            })
        },
        |path, limits| {
            assert_eq!(path, bundle_path.as_os_str().as_encoded_bytes());
            temper_box_shell::bundle::embedded(&headed, limits)
        },
    )
    .expect("headed configured bundle and sign-in files");
    let bundle = bundle.expect("configured bundle");
    let page = std::str::from_utf8(&bundle.page).expect("HTML page");
    assert!(page.contains("<meta name=\"temper-sign-in\" content=\"google\"></head>"));
    assert_eq!(bundle.forge_meta_bytes, 0, "no kept forge contributes metadata");
    let mut service = prepare_box(
        &jig,
        &bytes,
        Some(figures),
        &files,
        given,
        &profile.agent_setting().figures,
        &profile.agent.agent,
        4,
    )
    .expect("bundle with a forge sign-in entry admits before a forge is kept");
    service.configure_google_endpoints(google);
    service.adopt_bundle(bundle);
}
