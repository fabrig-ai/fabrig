//! Harness contracts run once in focused (skein/docs/foundation/testing-strategy.md, 8).
use crate::support;
use temper_box_kit::cluster_turn as cluster_turn_setup;

#[test]
fn the_sign_in_fixture_reads_the_initialized_listeners_without_an_llm_member() {
    let scratch = support::Scratch::startup("sign-in-listeners");
    support::write_file(
        &scratch.path().join(temper_config_shell::FILE),
        &support::configuration::file(),
        temper_config_shell::Shape::Standalone,
    );
    let _issuer = support::sign_in::patch_configuration(scratch.path());
    let providers = temper_agent_fixtures::providers::read(scratch.path());
    let issuer = providers["providers"][0]["sources"]["sign_in"]["issuer"].as_str().expect("patched issuer");
    assert!(issuer.starts_with("http://127.0.0.1:"), "independent local issuer: {issuer}");
    scratch.remove();
}

#[test]
fn moving_listeners_preserves_the_retained_files_explicit_keys() {
    use temper_config_shell::{File, Internal, Shape};
    let scratch = support::Scratch::startup("retained-file-edit");
    let path = scratch.path().join(temper_config_shell::FILE);
    let file = File {
        listen: Some("127.0.0.1:7100".parse().expect("clients")),
        webhooks: Some("127.0.0.1:7101".parse().expect("webhooks")),
        public: Some("https://temper.example.invalid".into()),
        owner: Some("owner@example.invalid".into()),
        internal: Internal { bundle: Some(scratch.path().join("bundle")) },
        environment: vec![("FIXTURE".into(), "retained".into())],
        ..Default::default()
    };
    support::write_file(&path, &file, Shape::Standalone);
    support::own_listener_files(scratch.path());
    let moved = support::read_file(&path, Shape::Standalone);
    assert_ne!(moved.listen, file.listen);
    assert_ne!(moved.webhooks, file.webhooks);
    assert_ne!(moved.listen, moved.webhooks);
    let expected = File { listen: moved.listen, webhooks: moved.webhooks, ..file };
    assert_eq!(moved, expected, "all non-listener keys are retained");
    scratch.remove();
}

#[test]
fn rebinding_the_fake_model_keeps_the_split_deployments_issuer() {
    let scratch = support::Scratch::startup("split-provider-rebind");
    let config = scratch.path().join("config");
    let data = scratch.path().join("data");
    std::fs::create_dir(&config).expect("config directory");
    std::fs::create_dir(&data).expect("data directory");
    support::write_file(
        &config.join(temper_config_shell::FILE),
        &temper_config_shell::File::default(),
        temper_config_shell::Shape::Standalone,
    );
    let mut providers = temper_agent_fixtures::providers::stock();
    providers["providers"][0]["sources"]["sign_in"]["issuer"] = "http://127.0.0.1:19002".into();
    temper_agent_fixtures::providers::write(&data, &providers);
    let peer = support::fake_llm(Box::new([]));
    peer.configure(&config);
    let rebound = temper_agent_fixtures::providers::read(&data);
    assert_eq!(
        rebound["providers"][0]["sources"]["sign_in"]["issuer"],
        providers["providers"][0]["sources"]["sign_in"]["issuer"]
    );
    assert_eq!(rebound["endpoints"][0]["address"]["socket"], peer.address().to_string());
    scratch.remove();
}

// It reaches `forgejo_elsewhere`, whose capture witness uses Linux's abstract Unix sockets.
#[cfg(target_os = "linux")]
#[test]
fn the_elsewhere_deployment_keeps_the_whole_fixture_provider_in_its_data_directory() {
    let scratch = support::Scratch::startup("elsewhere-split-provider");
    let config = scratch.path().join("config");
    let data = scratch.path().join("data");
    std::fs::create_dir(&config).expect("config directory");
    std::fs::create_dir(&data).expect("data directory");
    support::configuration(&config, &support::Forge::Unreached);
    support::owned::producer_landing_settings(&config, 2);
    let before = temper_agent_fixtures::providers::read(&config);
    let peer = support::fake_llm(Box::new([]));
    support::forgejo_elsewhere::configure_llm(scratch.path(), &peer);

    // Read the startup copy through the product's reader, not the world's retained inputs.
    let running = temper_service_shell::providers::read(
        &data.join("providers.setting"),
        temper_service::stock::providers(),
        &mut Vec::new(),
    );
    let running = temper_agent_fixtures::providers::flat(&running);
    assert_eq!(running["endpoints"][0]["address"]["socket"], peer.address().to_string());
    assert_eq!(running["endpoints"][0]["transport"], "tls");
    assert_eq!(running["endpoints"][0]["server_name"], "skein.test");
    assert_eq!(running["endpoints"][0]["trust"], serde_json::json!(config.join("root.der")));
    let mut expected = before;
    expected["endpoints"][0]["address"] = serde_json::json!({"socket":peer.address().to_string()});
    assert_eq!(running, expected, "rebinding preserves the complete provider, not stock with a new socket");
    let settings: serde_json::Value =
        serde_json::from_slice(&std::fs::read(config.join("native-landing-settings.json")).expect("landing settings"))
            .expect("settings JSON");
    let edits = support::provider_setup::items(&running);
    assert!(settings["items"].as_array().expect("items").ends_with(&edits), "ctl commits the running providers");
    scratch.remove();
}

#[test]
fn cluster_turn_setup_declares_the_paced_gate_and_prepare_job() {
    use serde_json::json;
    use temper_service::startup;
    let scratch = support::Scratch::startup("cluster-turn-documents");
    let mut cluster = support::cluster::Cluster::description_from(scratch.path(), &cluster_turn_setup::fixture());
    let figures = cluster_turn_setup::prepare(&mut cluster, scratch.path(), &support::free_addresses(&[]));
    let forge = cluster_turn_setup::read(&cluster.dir("forge").join("forge.json"));
    assert_eq!(forge["workflows"], json!([{"file":"gate.yml","jobs":["gate"],"cue":{"declared":".fake-forge/gate"}}]));
    assert_eq!(forge["manual_merges"], json!({"allow":true,"autodetect":true}));
    assert_eq!(forge["pace"]["ci_ms"], cluster_turn_setup::CI_MS);
    temper_fake::forge::Document::decode(&serde_json::to_vec(&forge).expect("forge JSON")).expect("forge document");
    let llm = cluster_turn_setup::read(&cluster.dir("llm").join("llm.json"));
    temper_fake::llm::Document::decode(&serde_json::to_vec(&llm).expect("LLM JSON"))
        .expect("scripted producer and chat");
    let producer = &llm["cued"][0]["turns"];
    assert_eq!(producer[2]["lines"][0]["call"], "save");
    assert_eq!(
        producer[3]["lines"][0]["arguments"]["content"].as_str().expect("both lines").as_bytes(),
        cluster_turn_setup::CONTENT
    );
    assert_eq!(producer[4]["lines"][0]["call"], "deliver");
    let engine = support::start_document(
        &cluster.dir("engine").join("config").join(temper_config_shell::FILE),
        temper_config_shell::Shape::EngineAlone,
    );
    let landing = cluster_turn_setup::read(&cluster.dir("engine").join("config/landing-settings.json"));
    let project = &landing["items"][0]["set"];
    assert_eq!(project["by"], "turn");
    assert_eq!(project["batch"], 1);
    assert_eq!(landing["items"][1]["set"]["repositories"][0]["gates"][0]["workflow"], "gate.yml");
    let limits = jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 65_536, depth: 32 };
    let bytes = serde_json::to_vec(&engine["engine"]).expect("engine JSON");
    let document = jig_protocol_config::document(&bytes, &limits).expect("engine document");
    let engine = startup::engine_over(
        &document,
        0,
        &jig_protocol_config::Directories::working(),
        &temper_service::stock::Given::Stock,
        &mut figures.jig.clone(),
    )
    .expect("initial engine");
    assert!(
        temper_service::limits::prepare_jobs(&engine.limits).contains(&temper_service::limits::prepare_bound()),
        "the worker's merge train must be declared"
    );
    let document = support::start_document(
        &cluster.dir("engine").join("config").join(temper_config_shell::FILE),
        temper_config_shell::Shape::EngineAlone,
    );
    let bytes = serde_json::to_vec(&support::cluster::agent_configuration(&document, &cluster.dir("w1").join("data")))
        .expect("agent JSON");
    let agent = temper_worker_shell::agent::prepare_bytes(bytes.into()).expect("agent profile").agent;
    for process in ["engine", "w1"] {
        let root = cluster.dir(process);
        let directories = temper_real_loop_world::cluster::directories_in([
            &root.join("config"),
            &root.join("data"),
            &root.join("cache"),
        ]);
        let directories = temper_box_shell::directories::jig_directories(&directories);
        let bytes = support::read_start(
            &root.join("config").join(temper_config_shell::FILE),
            if process == "engine" {
                temper_config_shell::Shape::EngineAlone
            } else {
                temper_config_shell::Shape::WorkerAlone
            },
        )
        .document;
        if process == "engine" {
            startup::store_limits_with_limits(
                &bytes,
                &directories,
                Some(&agent.declaration()),
                &figures.jig,
                &figures.limits,
            )
            .unwrap_or_else(|why| panic!("{process}: {}", String::from_utf8_lossy(&startup::lines(&why))));
        }
        match startup::shape(&bytes, &directories).expect("separate shape") {
            startup::Shape::EngineAlone => {
                startup::named(&bytes, &directories).expect("checked engine alone");
            }
            startup::Shape::WorkerAlone => {
                startup::worker_part(&bytes, &directories, None, &[]).expect("checked worker alone");
            }
            startup::Shape::Standalone => panic!("separate deployment processes"),
        }
    }
    let core = jig_protocol_config::core(&engine, [0; 16], 1);
    let needs = jig_domain_core::settings::fleet_needs::configured(&core, &engine.limits.core);
    assert_eq!(
        jig_domain_fleet::shortfall(&needs, &agent.bounds, jig_domain_fleet::HostKind::Worker),
        None,
        "the sole worker admits the stock and the prepare job"
    );
    scratch.remove();
}

#[test]
fn cluster_turn_settings_leave_spend_for_preparation_and_keep_current_fields() {
    use serde_json::json;
    let scratch = support::Scratch::new("cluster-turn-settings");
    let calls: Vec<_> = [
        json!({}),
        json!({"newest":7}),
        json!({"setting":{"kind":"policy","number":1},"value":{
            "period_spend":48000,"roles":[{"number":0,"period_spend":48000}],"kept":"policy"}}),
        json!({"setting":{"kind":"charter","number":1},"value":{"documents":[1],"kept":"chat",
            "model":{"provider":2},"alternatives":[{"provider":1,"name":"fixture"}]}}),
        json!({"setting":{"kind":"charter","number":2},"value":{"documents":[2],"kept":"producer",
            "model":{"provider":2},"alternatives":[{"provider":1,"name":"fixture"}]}}),
    ]
    .iter()
    .map(|value| support::Ended {
        exit: Some(skein_io::kernel::Exit::Code(0)),
        stdout: serde_json::to_vec(value).expect("setting JSON"),
        stderr: Vec::new(),
    })
    .collect();
    let _apply = cluster_turn_setup::settings(scratch.path(), &calls);
    let settings = cluster_turn_setup::read(&scratch.path().join("turn-settings.json"));
    let policy = &settings["items"][0]["set"];
    assert_eq!(policy["roles"][0]["period_spend"], 41000, "7000 of the period remains for preparation");
    assert_eq!(policy["kept"], "policy");
    assert_eq!(settings["items"][1]["set"]["kept"], "chat");
    assert_eq!(settings["items"][2]["set"]["kept"], "producer");
    assert_eq!(
        settings["items"][1]["set"]["model"]["provider"], 1,
        "the chat uses the fake provider, not stock's first choice"
    );
    assert_eq!(settings["items"][2]["set"]["model"]["provider"], 1, "the producer uses the fake provider");
    assert_eq!(settings["items"][3]["set"]["gates"], json!([]), "turn's test gate is the only gate");
    scratch.remove();
}

#[test]
fn the_scripted_batch_fits_its_budget_and_call() {
    support::batch_landing::tests::the_scripted_batch_fits_its_budget_and_call();
}

#[test]
fn native_forgejo_settings_preserve_the_coding_producer() {
    support::forgejo::tests::native_settings_preserve_the_coding_producer();
}

#[test]
fn owned_settings_preserve_the_ordinary_producer_without_review() {
    let scratch = support::Scratch::new("owned_producer");
    let owned = support::owned::Owned { review: false, ..support::owned::Owned::example() };
    support::owned::lay(scratch.path(), &owned);
    let settings: serde_json::Value = serde_json::from_slice(
        &std::fs::read(scratch.path().join("native-landing-settings.json")).expect("landing settings"),
    )
    .expect("settings JSON");
    assert_eq!(settings["items"][1]["set"], 1, "owned fixture uses the ordinary charter");
    assert_eq!(settings["items"][0]["set"]["gates"], serde_json::json!([]));
    scratch.remove();
}

#[test]
fn native_landing_rules_replace_the_projects_stock_review() {
    let scratch = support::Scratch::new("project_landing_rule");
    for producer in [1, 2] {
        support::owned::producer_landing_settings(scratch.path(), producer);
        let settings: serde_json::Value = serde_json::from_slice(
            &std::fs::read(scratch.path().join("native-landing-settings.json")).expect("landing settings"),
        )
        .expect("settings JSON");
        assert_eq!(
            settings["items"][0],
            serde_json::json!({"setting":{"kind":"landing-rule","project":1,"number":1},"set":{
                "pattern":{"segments":["6272616e6368"],"last":{"open":""}},
                "enforce":true,"ci":true,"up_to_date":true,"gates":[],"approvals":[]}}),
            "no-review fixtures replace project rule 1 and keep CI and freshness"
        );
        assert_eq!(settings["items"][1]["set"], producer);
    }
    scratch.remove();
}

#[test]
fn turn_restart_configuration_passes_startup_admission() {
    landing_configuration_admission("turn-restart-admission", support::turn_restart::configure);
}

#[test]
fn batch_landing_configuration_passes_startup_admission() {
    landing_configuration_admission("batch-landing-admission", support::batch_landing::configure);
}

fn landing_configuration_admission(
    case: &str,
    configure: fn(&std::path::Path, u64) -> temper_playground::documents::Configuration,
) {
    use skein_lib::{List, Secret};
    use temper_service::settings::{Files, Seeds};

    let scratch = support::Scratch::startup(case);
    let root = scratch.path();
    let api = "127.0.0.1:3000".parse().expect("fixture forge");
    let forge = support::Forge::Throwaway {
        forgejo: std::path::Path::new("/fixture/forgejo"),
        api,
        git: format!("http://{api}/"),
    };
    support::configuration(root, &forge);
    let figures = configure(root, 1);
    let document =
        support::read_start(&root.join(temper_config_shell::FILE), temper_config_shell::Shape::Standalone).document;
    let directories = jig_protocol_config::Directories::working();
    let named = temper_service::named(&document, &directories).expect("named story files");
    let mut entries = List::with_capacity(u32::try_from(named.files.len()).expect("file count"));
    for file in named.files {
        let path = std::str::from_utf8(&file.path).expect("fixture path");
        let bytes = std::fs::read(root.join(path)).expect("fixture file");
        entries.push((file.path, Secret::new(bytes.into_boxed_slice()))).expect("named file count");
    }
    let agent = temper_agent_fixtures::prepared(root).agent;
    let seeds = Seeds {
        link_secret: None,
        local_secret: None,
        deployment: [7; 16],
        installation: [8; 16],
        seed: 7,
        crypto: Secret::new(Box::new([9; 32])),
    };
    let result = temper_service::startup::checked_start_with_limits(
        &document,
        &directories,
        &Files { entries },
        Some(&agent),
        &seeds,
        List::with_capacity(0),
        None,
        &temper_service::stock::Given::Stock,
        None,
        &figures.jig,
        &figures.limits,
    );
    if let Err(problems) = result {
        panic!("{}", String::from_utf8_lossy(&temper_service::startup::lines(&problems)));
    }
    scratch.remove();
}

#[test]
fn turn_configuration_declares_the_prepare_job() {
    support::turn_restart::tests::turn_configuration_declares_the_prepare_job();
}

// It reaches `forgejo_elsewhere`, whose capture witness uses Linux's abstract Unix sockets.
#[cfg(target_os = "linux")]
#[test]
fn elsewhere_settings_follow_the_connected_project() {
    support::forgejo_elsewhere::tests::settings_follow_the_connected_project();
}

#[test]
fn story_command_a_callback_command_captures_output_and_keeps_a_bad_exit() {
    support::story_command::tests::a_callback_command_captures_output_and_keeps_a_bad_exit();
}

#[test]
fn story_socket_bounded_reads_preserve_the_next_bytes_and_eof_is_an_error() {
    support::story_socket::tests::bounded_reads_preserve_the_next_bytes_and_eof_is_an_error();
}

#[test]
fn patience_starvation_disarms_the_inner_wait_without_passing_the_world() {
    support::patience::tests::starvation_disarms_the_inner_wait_without_passing_the_world();
}

#[test]
fn patience_a_failed_wait_keeps_its_figures_and_deadline() {
    support::patience::tests::a_failed_wait_keeps_its_figures_and_deadline();
}

#[test]
fn patience_socket_waits_round_up_and_refusals_never_classify_as_starvation() {
    support::patience::tests::socket_waits_round_up_and_refusals_never_classify_as_starvation();
}

#[test]
fn patience_an_interrupted_socket_retries_with_the_same_deadline() {
    support::patience::tests::an_interrupted_socket_retries_with_the_same_deadline();
}

#[test]
fn cluster_a_member_wait_can_defer_only_to_a_longer_world() {
    support::cluster::deadline_tests::a_member_wait_can_defer_only_to_a_longer_world();
}

#[test]
fn fixture_limits_are_applied_before_the_subprocess_restart() {
    let scratch = support::Scratch::startup("small-limits-settings");
    let mut configuration = support::configuration(scratch.path(), &support::Forge::Unreached);
    configuration.jig_over("tasks.project_goals", 2);
    configuration.temper_over("checkout.remote_timeout_ms", 7_000);
    support::limit_settings::remember(scratch.path(), &configuration);
    support::limit_settings::install(scratch.path());
    configuration.jig_over("tasks.project_goals", 1);
    configuration.temper_over("checkout.remote_timeout_ms", 8_000);
    support::limit_settings::remember(scratch.path(), &configuration);
    support::limit_settings::install(scratch.path());
    let seen = support::ctl::run(support::ctl::Story::new(
        scratch.path(),
        vec![
            support::ctl::Step::Run(support::ctl::Call::sign_in()),
            support::ctl::Step::Run(support::ctl::Call::new(&["settings", "show", "limits", "--json"])),
            support::ctl::Step::Run(support::ctl::Call::new(&["settings", "show", "temper-limits", "--json"])),
        ],
    ));
    assert_eq!(
        support::ctl::json(&seen.calls[1])["value"]
            .as_array()
            .expect("jig limits")
            .iter()
            .find(|item| item["name"] == "tasks.project_goals")
            .expect("project goal capacity")["value"],
        1
    );
    assert_eq!(
        support::ctl::json(&seen.calls[2])["value"]
            .as_array()
            .expect("temper limits")
            .iter()
            .find(|item| item["name"] == "checkout.remote_timeout_ms")
            .expect("worker checkout limit")["value"],
        8_000
    );
    scratch.remove();
}

#[test]
fn ctl_limit_settings_apply_both_kinds_in_one_revision() {
    support::ctl::read_condition_tests::limit_settings_apply_both_kinds_in_one_revision();
}

#[test]
fn ctl_a_failed_ctl_check_reaches_the_world_after_cleanup() {
    support::ctl::read_condition_tests::a_failed_ctl_check_reaches_the_world_after_cleanup();
}

#[test]
fn ctl_a_returned_visit_drains_without_another_process_or_a_poll() {
    support::ctl::read_condition_tests::a_returned_visit_drains_without_another_process_or_a_poll();
}

#[test]
fn forge_land_rung_an_expired_rung_phase_starts_no_further_command() {
    support::forge_land::rung::tests::an_expired_rung_phase_starts_no_further_command();
}

#[test]
fn forge_land_rung_rung_phases_keep_the_request_deadlines_when_selected_again() {
    support::forge_land::rung::tests::rung_phases_keep_the_request_deadlines_when_selected_again();
}

#[test]
fn github_file_scan_excludes_only_jigs_generation_secrets_store() {
    support::github::file_scan_excludes_only_jigs_generation_secrets_store();
}
