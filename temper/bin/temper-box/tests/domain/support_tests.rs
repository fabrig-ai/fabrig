//! Harness contracts over values and scripted completions.
use crate::support;
use serde_json::{Value, json};
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};
use temper_box_kit::cluster_turn as cluster_turn_setup;

#[test]
fn holding_fixture_limits_creates_a_private_data_directory() {
    use std::{fs, os::unix::fs::PermissionsExt};

    let scratch = skein_scratch::Scratch::new("fixture-limits-hold");
    let data = scratch.path().join("data");
    support::limit_settings::hold(&data, &[]);

    let directory = fs::symlink_metadata(&data).expect("fixture data directory");
    assert!(directory.is_dir());
    assert_eq!(directory.permissions().mode() & 0o777, 0o700);
    let held = fs::symlink_metadata(data.join("limits-held")).expect("fixture hold file");
    assert!(held.is_file());
    assert_eq!(held.permissions().mode() & 0o777, 0o600);
}

#[test]
fn readiness_eof_reports_child_status_and_stderr() {
    use std::os::unix::process::ExitStatusExt;
    let finding = temper_box_kit::fake_readiness::read_with(&mut b"".as_slice(), &"fake fixture", |kind| {
        assert_eq!(kind, std::io::ErrorKind::UnexpectedEof);
        (std::process::ExitStatus::from_raw(7 << 8), b"fixture refused".to_vec())
    })
    .expect_err("readiness EOF finding");
    assert!(finding.contains("status exit status: 7; stderr fixture refused"), "{finding}");
}

#[test]
fn an_explicit_program_supplies_its_own_death_signal_wrapper() {
    support::boxed::tests::an_explicit_program_supplies_its_own_death_signal_wrapper();
}

#[test]
fn freeing_the_forgejo_source_name_renames_the_public_source_by_id() {
    let forgejo = Path::new("/fixture/forgejo");
    let command = support::forgejo_sign_in::rename_public_source_command(forgejo, Path::new("/fixture"), "27");
    assert_eq!(command.get_program(), forgejo.as_os_str());
    assert_eq!(
        command.get_args().collect::<Vec<_>>(),
        [
            "--work-path",
            "/fixture/forge-1/forgejo",
            "--config",
            "/fixture/forge-1/forgejo/app.ini",
            "admin",
            "auth",
            "update-oauth",
            "--id",
            "27",
            "--name",
            "temper-public",
        ]
    );
}

#[test]
fn fixture_provider_edits_keep_routes_and_leave_demo_fixed() {
    use serde_json::json;
    let mut providers = temper_service::stock::providers().into_vec();
    providers.truncate(1);
    providers.push(temper_service::stock::scripted("127.0.0.1:19003".parse().expect("scripted address")));
    let mut wanted = temper_agent_fixtures::providers::flat(&providers);
    wanted["endpoints"][0]["address"] = json!({"socket":"127.0.0.1:19001"});
    let items = support::provider_setup::items(&wanted);
    assert_eq!(items.len(), 1, "set only the fixture provider, leaving other stock model references intact");
    assert_eq!(items[0]["set"]["endpoints"][0]["address"]["socket"], "127.0.0.1:19001");
    assert!(items.iter().all(|item| item["setting"]["number"] != 3), "demo is fixed, never a settings edit");
}

#[test]
fn cluster_turn_wrapper_holds_delivery_not_the_saved_push() {
    use std::path::Path;
    let script = cluster_turn_setup::wrapper(Path::new("/story"), Path::new("/story/git"));
    assert!(script.contains("if [ \"$arg\" = push ] && [ -f '/story/delivery.hold' ]"));
    assert!(script.contains("if mkdir '/story/saved-push'"), "save's first push passes the barrier");
    let entered = script.find(": > '/story/delivery.entered'").expect("entered before waiting");
    let waiting = script.find("while [ -f '/story/delivery.hold' ]").expect("wait on the hold file");
    assert!(entered < waiting);
    assert!(
        script.ends_with("exec /usr/bin/env -u PWD -u SHLVL -u _ '/story/git' \"$@\"\n"),
        "the wrapper always invokes fake git with the same arguments"
    );
}

#[test]
fn the_retained_cluster_charter_overrides_have_no_documents() {
    // The process story overlays these values on current settings before decoding.
    // Charter documents follow temper/docs/design/protocol/configuration/settings.md, 6.
    let fixture: serde_json::Value = serde_json::from_slice(include_bytes!("../fixtures/cluster/settings.json"))
        .expect("retained cluster settings overrides");
    let items = fixture["items"].as_array().expect("cluster settings items");
    let charters: Vec<_> = items.iter().filter(|item| item["setting"]["kind"] == "charter").collect();
    assert_eq!(charters.len(), 2);
    for item in charters {
        assert_eq!(item["set"]["documents"], serde_json::json!([]), "cluster charter: {}", item["setting"]);
    }
}

#[test]
fn printed_links_count_and_fail_without_disclosing_secrets() {
    support::printed_link_tests::printed_links_count_and_fail_without_disclosing_secrets();
}

#[test]
fn a_judge_keeps_an_action_finding_for_watch() {
    support::account_move::tests::a_judge_keeps_an_action_finding_for_watch();
}

#[test]
fn person_a_new_project_waits_for_its_applied_home() {
    support::person::a_new_project_waits_for_its_applied_home();
}

#[test]
fn person_a_chat_waits_for_the_model_watch_before_pressing_start() {
    support::person::a_chat_waits_for_the_model_watch_before_pressing_start();
}

#[test]
fn terminal_a_command_waits_for_fresh_controls_when_returning_to_settings() {
    support::terminal::tests::a_command_waits_for_fresh_controls_when_returning_to_settings();
}

#[test]
fn gate_receipt_waits_for_an_unpublished_status_context() {
    support::batch_landing::tests::gate_receipt_waits_for_an_unpublished_status_context();
}

#[test]
fn cluster_stages_fixture_providers_after_the_engine_bootstrap() {
    support::cluster::bootstrap_tests::the_engine_stages_fixture_providers_after_bootstrap();
}

#[test]
fn cluster_passed_accepts_only_a_successful_settled_verdict() {
    support::cluster::verdict_tests::passed_accepts_only_a_successful_settled_verdict();
}

#[test]
fn cluster_the_retained_charter_overrides_name_no_documents() {
    // Charter documents follow temper/docs/design/protocol/configuration/settings.md, 6.
    // The process harness overlays these members on current settings before decoding whole values.
    let settings: serde_json::Value =
        serde_json::from_slice(include_bytes!("../fixtures/cluster/settings.json")).expect("cluster overrides JSON");
    let charters: Vec<_> = settings["items"]
        .as_array()
        .expect("settings items")
        .iter()
        .filter(|item| item["setting"]["kind"] == "charter")
        .collect();
    assert_eq!(charters.len(), 2, "chat and coding charter overrides");
    for charter in charters {
        assert_eq!(charter["set"]["documents"], serde_json::json!([]), "cluster charter names no documents");
    }
}

#[test]
fn ctl_a_box_call_consumes_captured_stderr_once_without_busy_waiting() {
    support::ctl::read_condition_tests::a_box_call_consumes_captured_stderr_once_without_busy_waiting();
}

#[test]
fn ctl_a_single_host_timeout_reports_its_pending_call() {
    support::ctl::read_condition_tests::a_single_host_timeout_reports_its_pending_call();
}

#[test]
fn ctl_a_child_prerequisite_reads_later_pages_and_polls_again_from_the_start() {
    support::ctl::read_condition_tests::a_child_prerequisite_reads_later_pages_and_polls_again_from_the_start();
}

#[test]
fn ctl_a_child_prerequisite_accepts_live_and_ended_identities_without_shifting_story_calls() {
    support::ctl::read_condition_tests::a_child_prerequisite_accepts_live_and_ended_identities_without_shifting_story_calls();
}

#[test]
fn ctl_a_failed_child_prerequisite_stops_before_retrying() {
    support::ctl::read_condition_tests::a_failed_child_prerequisite_stops_before_retrying();
}

#[test]
fn ctl_a_failed_poll_stops_before_retrying_an_unmet_condition() {
    support::ctl::read_condition_tests::a_failed_poll_stops_before_retrying_an_unmet_condition();
}

#[test]
fn ctl_a_refused_started_adoption_fails_before_polling_pending() {
    support::ctl::read_condition_tests::a_refused_started_adoption_fails_before_polling_pending();
}

#[test]
fn ctl_process_writing_task_resubscribes_from_the_subject_digest_without_a_poll_delay() {
    support::ctl::read_condition_tests::process_writing_task_resubscribes_from_the_subject_digest_without_a_poll_delay(
    );
}

#[test]
fn ctl_process_until_retains_matching_bytes_and_their_completion_wall() {
    support::ctl::read_condition_tests::process_until_retains_matching_bytes_and_their_completion_wall();
}

#[test]
fn person_a_native_adoption_refusal_names_the_alert_without_a_wait() {
    support::person::a_native_adoption_refusal_names_the_alert_without_a_wait();
}

#[test]
fn terminal_a_terminal_adoption_refusal_uses_the_current_frame() {
    support::terminal::a_terminal_adoption_refusal_uses_the_current_frame();
}

#[test]
fn github_stopped_members_details_include_exits_and_bounded_stderr() {
    support::github::stopped_members_details_include_exits_and_bounded_stderr();
}

#[test]
fn survivor_lines_name_each_process_its_parent_and_group() {
    support::forge_land::tests::survivor_lines_name_each_process_its_parent_and_group();
}

#[test]
fn a_kept_deployment_with_no_merged_pulls_can_find_its_project() {
    support::forge_land::tests::a_kept_deployment_with_no_merged_pulls_can_find_its_project();
}

#[test]
fn a_kept_project_matches_the_owned_adoptions_identity() {
    support::forge_land::tests::a_kept_project_matches_the_owned_adoptions_identity();
}

#[test]
fn a_kept_project_requires_complete_visible_unambiguous_adoptions() {
    support::forge_land::tests::a_kept_project_requires_complete_visible_unambiguous_adoptions();
}

#[test]
fn the_twin_waits_for_the_childs_result_after_a_forge_notification() {
    support::forge_land::rung::twin_tests::the_twin_waits_for_the_childs_result_after_a_forge_notification();
}

#[test]
fn the_sign_in_fixture_reads_the_initialized_listeners_without_an_llm_member() {
    use temper_config_shell::{FILE, Shape};
    let root = Path::new("/fixture/sign-in-listeners");
    let start = support::configuration::parse(
        &temper_config_shell::render(&support::configuration::file(), Shape::Standalone),
        Shape::Standalone,
    );
    let document: Value = serde_json::from_slice(&start.document).expect("initialized document");
    assert!(document["engine"].get("llm").is_none());
    let drawn = [local(19_001), local(19_002)];
    let mut file = None;
    let (setting, _, _) =
        support::sign_in::patch_configuration_with(root, start, drawn, local(19_003), |path, bytes, providers| {
            assert_eq!(path, root.join(FILE));
            let kept = support::configuration::parse(bytes, Shape::Standalone);
            file = Some(kept.file);
            let start = json_bytes(&kept.document);
            let agent = temper_agent_fixtures::with_figures(&start, &temper_service::stock::agents(), root, providers);
            (start, agent)
        });
    let file = file.expect("captured fixture file");
    assert_eq!(file.listen, Some(drawn[0]));
    assert!(file.webhooks.is_some());
    let mut copies = ProviderCopies::default();
    let _ = temper_agent_fixtures::providers::remember_with(root, &setting.providers, copies.writer());
    let providers = copies.startup(root);
    assert_eq!(providers["providers"][0]["number"], 1);
    let sign_in = &providers["providers"][0]["sources"]["sign_in"];
    let issuer = sign_in["issuer"].as_str().expect("patched issuer");
    assert!(issuer.starts_with("http://127.0.0.1:"), "independent local issuer: {issuer}");
    assert_eq!(issuer, "http://127.0.0.1:19003");
    assert_eq!(sign_in["transport"], "plaintext");
}

#[test]
fn moving_listeners_preserves_the_retained_files_explicit_keys() {
    use temper_config_shell::{File, Internal, Shape};
    // The world supplies the allocator's cutoff rather than consulting the host.
    const EPHEMERAL_FIRST: u16 = 32_768;
    let init = temper_box_shell::testing::init_fixtures::configuration(
        8367.try_into().expect("init's preferred clients port"),
        8368.try_into().expect("the adjacent webhooks port"),
    );
    let explicit = File {
        listen: Some(local(7100)),
        webhooks: Some(local(7101)),
        public: Some("https://temper.example.invalid".into()),
        owner: Some("owner@example.invalid".into()),
        internal: Internal { bundle: Some("/fixture/retained-file-edit/bundle".into()) },
        environment: vec![("FIXTURE".into(), "retained".into())],
        ..Default::default()
    };
    // Init omits the adjacent webhooks port. A non-adjacent draw must be written explicitly.
    for (row, file, drawn) in
        [("init", init, [local(19_001), local(19_003)]), ("explicit keys", explicit, [local(19_001), local(19_002)])]
    {
        let read =
            support::configuration::parse(&temper_config_shell::render(&file, Shape::Standalone), Shape::Standalone);
        let moved = support::move_listener_file(read, drawn);
        let moved =
            support::configuration::parse(&temper_config_shell::render(&moved, Shape::Standalone), Shape::Standalone);
        let document = json_bytes(&moved.document);
        assert!(document["application"].get("machine").is_none(), "{row}: a forge chooses its listener at set-up");
        for (pointer, wanted) in
            ["/application/clients/listener/socket", "/application/webhooks/listener/socket"].into_iter().zip(drawn)
        {
            let address: SocketAddr =
                document.pointer(pointer).expect("listener").as_str().expect("socket").parse().expect("address");
            assert_eq!(address, wanted, "{row}: {pointer}");
            assert!(address.port() < EPHEMERAL_FIRST, "{row}: {pointer}: {address}");
        }

        assert_ne!(moved.file.listen, file.listen, "{row}");
        assert_ne!(moved.file.webhooks, file.webhooks, "{row}");
        assert_ne!(moved.file.listen, moved.file.webhooks, "{row}");
        let expected = File { listen: Some(drawn[0]), webhooks: Some(drawn[1]), ..file };
        assert_eq!(moved.file, expected, "{row}: all non-listener keys are retained");
    }
}

#[test]
fn rebinding_the_fake_model_keeps_the_split_deployments_issuer() {
    let root = Path::new("/fixture/split-provider-rebind");
    let config = root.join("config");
    let data = root.join("data");
    let mut providers = temper_agent_fixtures::providers::stock();
    providers["providers"][0]["sources"]["sign_in"]["issuer"] = "http://127.0.0.1:19002".into();
    let address = local(19_003);
    let mut copies = ProviderCopies::default();
    support::llm::configure_with(
        &config,
        address,
        |path| path == data,
        |path| {
            assert_eq!(path, data);
            providers.clone()
        },
        copies.writer(),
    );
    let rebound = copies.startup(&data);
    assert_eq!(
        rebound["providers"][0]["sources"]["sign_in"]["issuer"],
        providers["providers"][0]["sources"]["sign_in"]["issuer"]
    );
    assert_eq!(rebound["endpoints"][0]["address"]["socket"], address.to_string());
}

// It reaches `forgejo_elsewhere`, whose capture witness uses Linux's abstract Unix sockets.
#[cfg(target_os = "linux")]
#[test]
fn the_elsewhere_deployment_keeps_the_whole_fixture_provider_in_its_data_directory() {
    let root = Path::new("/fixture/elsewhere-split-provider");
    let config = root.join("config");
    let data = root.join("data");
    let recipe = support::recipe(&config, &support::Forge::Unreached);
    let before = temper_agent_fixtures::providers::flat(&recipe.agent_setting().providers);
    let address = local(19_003);
    let mut copies = ProviderCopies::default();
    let mut settings = None;
    support::forgejo_elsewhere::configure_provider_with(
        root,
        address,
        before.clone(),
        copies.writer(),
        |path, bytes| {
            assert_eq!(path, config.join("elsewhere-providers.json"));
            settings = Some(json_bytes(bytes));
        },
    );

    // Read the startup copy through the product's reader, not the world's retained inputs.
    let running = copies.startup(&data);
    assert_eq!(running["endpoints"][0]["address"]["socket"], address.to_string());
    assert_eq!(running["endpoints"][0]["transport"], "tls");
    assert_eq!(running["endpoints"][0]["server_name"], "skein.test");
    assert_eq!(running["endpoints"][0]["trust"], json!(config.join("root.der")));
    let mut expected = before;
    expected["endpoints"][0]["address"] = json!({"socket": address.to_string()});
    assert_eq!(running, expected, "rebinding preserves the complete provider, not stock with a new socket");
    let settings = settings.expect("captured provider settings");
    let edits = support::provider_setup::items(&running);
    assert_eq!(settings["items"], json!(edits), "ctl commits the running providers");
}

#[derive(Default)]
struct ProviderCopies(Vec<(PathBuf, Box<[u8]>)>);
impl ProviderCopies {
    fn writer(&mut self) -> impl FnMut(&Path, &[jig_protocol_config::ProviderSetting]) -> std::io::Result<bool> + '_ {
        |path, values| {
            self.0.push((path.into(), temper_service_shell::providers::render(values)));
            Ok(true)
        }
    }

    fn startup(self, data: &Path) -> Value {
        let mut log = Vec::new();
        assert_eq!(self.0.len(), 2, "fixture input and startup copies");
        assert_eq!(self.0[0].0, data.join("world-providers.json"));
        let values = temper_service_shell::providers::read_using(
            &data.join("providers.setting"),
            temper_service::stock::providers(),
            &mut log,
            |path| {
                let (kept, bytes) = self.0.into_iter().nth(1).expect("startup copy");
                assert_eq!(path, kept);
                Ok(Some(bytes.into_vec()))
            },
        );
        assert!(log.is_empty(), "{}", String::from_utf8_lossy(&log));
        temper_agent_fixtures::providers::flat(&values)
    }
}

pub(super) fn cluster_turn_description(root: &Path) -> support::cluster::Cluster {
    let mut cluster = support::cluster::Cluster::description_from(root, &cluster_turn_setup::fixture());
    cluster_turn_setup::bind_addresses(
        &mut cluster,
        &[local(7100), local(7101), local(7102), local(3000), local(8371)],
    );
    cluster
}

fn cluster_turn_with_spend(
    cluster: &mut support::cluster::Cluster,
    rendered: &support::cluster::Rendered,
) -> temper_playground::documents::Configuration {
    let engine = rendered.engine_configuration();
    let providers = temper_agent_fixtures::providers::values(&engine["world_providers"]);
    let agent = temper_agent_fixtures::with_figures(
        &engine,
        &temper_service::stock::agents(),
        &cluster.dir("engine").join("data"),
        &providers,
    );
    let composed = cluster_turn_setup::compose_engine(engine, &agent);
    cluster.bind("story.change_spend", &composed.change_spend().to_string());
    composed.configuration
}

#[test]
fn deployment_binaries_render_every_member_of_both_clusters() {
    use support::cluster::Cluster;
    use temper_service::startup::Shape::{EngineAlone, WorkerAlone};

    let cluster = Cluster::documents(
        Path::new("/fixture/deployment-cluster"),
        &[("forge", "address", "127.0.0.1:3"), ("llm", "address", "127.0.0.1:1"), ("engine", "workers", "127.0.0.1:2")],
    );
    let turn_root = Path::new("/fixture/deployment-cluster-turn");
    let mut turn = cluster_turn_description(turn_root);
    turn.git_on_path("engine", turn_root);
    let rendered = turn.render_files("engine");
    cluster_turn_with_spend(&mut turn, &rendered);

    let directory = Path::new("/fixture/deployment-bin");
    let common = [
        ("forge", PathBuf::from("/fixture/temper-box"), None),
        ("llm", PathBuf::from("/fixture/temper-box"), None),
        ("engine", directory.join("temper-engine"), Some(EngineAlone)),
        ("w1", directory.join("temper-worker"), Some(WorkerAlone)),
    ];
    for (mut description, extra) in
        [(cluster, Some(("w2", directory.join("temper-worker"), Some(WorkerAlone)))), (turn, None)]
    {
        description.deployment_binaries_from(directory);
        for (binding, file) in [
            ("{artifact.temper-engine}", "temper-engine"),
            ("{artifact.temper-worker}", "temper-worker"),
            ("{artifact.temper-agent}", "temper-agent"),
        ] {
            assert_eq!(Path::new(&description.render(binding)), directory.join(file), "{binding}");
        }
        for (process, program, shape) in common.iter().chain(extra.iter()) {
            let command = description.render_command(process);
            assert_eq!(&command.program, program, "{process} executable");
            let rendered = description.render_files(process);
            assert_eq!(rendered.start.is_some(), shape.is_some(), "{process} start document presence");
            if let Some(shape) = shape {
                let start = rendered.start.expect("rendered start document");
                admit_cluster_start(process, &start.document, &cluster_directories(&description.dir(process)), *shape);
            }
        }
    }
}

#[test]
fn a_story_binding_is_refused_a_second_time() {
    let mut cluster =
        support::cluster::Cluster::description_from(Path::new("/fixture/duplicate"), &cluster_turn_setup::fixture());
    cluster.try_bind("story.address", "127.0.0.1:1").expect("first story binding");
    assert_eq!(cluster.try_bind("story.address", "127.0.0.1:2"), Err("duplicate binding story.address".into()));
    assert_eq!(cluster.render("{story.address}"), "127.0.0.1:1");
}

#[test]
fn cluster_turn_documents_admit_the_worker_and_script_delivery() {
    use temper_service::startup::{
        self,
        Shape::{EngineAlone, WorkerAlone},
    };
    let mut cluster = cluster_turn_description(Path::new("/fixture/cluster-turn-documents"));
    let rendered = cluster.render_files("engine");
    let figures = cluster_turn_with_spend(&mut cluster, &rendered);
    let forge = rendered_json(&cluster, "forge", "forge.json");
    temper_fake::forge::Document::decode(&serde_json::to_vec(&forge).expect("forge JSON")).expect("forge document");
    let llm = rendered_json(&cluster, "llm", "llm.json");
    temper_fake::llm::Document::decode(&serde_json::to_vec(&llm).expect("LLM JSON"))
        .expect("scripted producer and chat");
    let producer = &llm["cued"][0]["turns"];
    assert_eq!(producer[2]["lines"][0]["call"], "save");
    assert_eq!(
        producer[3]["lines"][0]["arguments"]["content"].as_str().expect("both lines").as_bytes(),
        cluster_turn_setup::CONTENT
    );
    assert_eq!(producer[4]["lines"][0]["call"], "deliver");
    let engine_start = rendered.start.expect("rendered engine").document;
    let engine = json_bytes(&engine_start);
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
    let bytes = serde_json::to_vec(&support::cluster::agent_configuration(
        &json_bytes(&engine_start),
        &cluster.dir("w1").join("data"),
    ))
    .expect("agent JSON");
    let agent = support::prepare_agent(bytes.into()).agent;
    for (process, shape) in [("engine", EngineAlone), ("w1", WorkerAlone)] {
        let directories = cluster_directories(&cluster.dir(process));
        let bytes = cluster.render_files(process).start.expect("rendered process").document;
        if shape == EngineAlone {
            startup::store_limits_with_limits(
                &bytes,
                &directories,
                Some(&agent.declaration()),
                &figures.jig,
                &figures.limits,
            )
            .unwrap_or_else(|why| panic!("{process}: {}", String::from_utf8_lossy(&startup::lines(&why))));
        }
        admit_cluster_start(process, &bytes, &directories, shape);
    }
    let core = jig_protocol_config::core(&engine, [0; 16], 1);
    let needs = jig_domain_core::settings::fleet_needs::configured(&core, &engine.limits.core);
    assert_eq!(
        jig_domain_fleet::shortfall(&needs, &agent.bounds, jig_domain_fleet::HostKind::Worker),
        None,
        "the sole worker admits the stock"
    );
}

fn cluster_directories(root: &Path) -> jig_protocol_config::Directories {
    let directories = temper_real_loop_world::cluster::directories_in([
        &root.join("config"),
        &root.join("data"),
        &root.join("cache"),
    ]);
    temper_box_shell::directories::jig_directories(&directories)
}

fn admit_cluster_start(
    process: &str,
    bytes: &[u8],
    directories: &jig_protocol_config::Directories,
    expected: temper_service::startup::Shape,
) {
    use temper_service::startup::{self, Shape};
    let shape = startup::shape(bytes, directories).expect("separate shape");
    assert_eq!(shape, expected, "{process} start shape");
    match shape {
        Shape::EngineAlone => {
            startup::named(bytes, directories).expect("checked engine alone");
        }
        Shape::WorkerAlone => {
            startup::worker_part(bytes, directories, None, &[]).expect("checked worker alone");
        }
        Shape::Standalone => panic!("separate deployment processes"),
    }
}

#[test]
fn cluster_turn_worker_inputs_exclude_the_forge_admin_token() {
    let cluster = cluster_turn_description(Path::new("/fixture/cluster-worker-inputs"));
    let worker = cluster.dir("w1");
    let rendered = cluster.render_files("w1");
    let start = rendered_file(&rendered, &worker.join("config").join(temper_config_shell::FILE));
    let engine = cluster.render_files("engine").start.expect("rendered engine").document;
    let figures = support::cluster::figures(&json_bytes(&engine));
    let agent = temper_worker_shell::agent::figures::configuration(&figures, None, &worker.join("data"), &[])
        .expect("generated worker agent");
    let token = temper_fake::credential::credential(17, temper_fake::credential::ADMIN_TOKEN);
    for (name, bytes) in [("worker start", start), ("worker agent", agent.as_ref())] {
        assert!(!bytes.windows(token.len()).any(|window| window == token), "forge admin token in {name}");
    }
    let secret = rendered_file(&rendered, &worker.join("data/secrets/worker.secret"));
    assert!(secret.len() < token.len(), "the fixture's worker secret is too short to contain the forge admin token");
}

#[test]
fn cluster_turn_settings_point_open_and_coding_runs_at_the_worker() {
    let root = Path::new("/fixture/cluster-turn-settings");
    let calls: Vec<_> = [
        json!({}),
        json!({"newest":7}),
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
    let mut settings = None;
    let _apply =
        cluster_turn_setup::settings_with(root, &calls, &temper_agent_fixtures::providers::stock(), |path, bytes| {
            assert_eq!(path, root.join("turn-settings.json"));
            settings = Some(json_bytes(bytes));
        });
    let settings = settings.expect("captured turn settings");
    for (index, kept) in ["chat", "producer"].into_iter().enumerate() {
        let item = &settings["items"][index]["set"];
        assert_eq!(item["kept"], kept);
        assert_eq!(item["documents"], json!([]));
        assert_eq!(item["hosts"], "workers");
        assert_eq!(item["model"]["provider"], 1, "use the fake provider for {kept}");
    }
}

fn rendered_json(cluster: &support::cluster::Cluster, process: &str, file: &str) -> Value {
    let wanted = cluster.dir(process).join(file);
    let rendered = cluster.render_files(process);
    json_bytes(rendered_file(&rendered, &wanted))
}

fn rendered_file<'a>(rendered: &'a support::cluster::Rendered, wanted: &Path) -> &'a [u8] {
    let (_, bytes) = rendered.files.iter().find(|(path, _)| path == wanted).expect("rendered file");
    bytes
}

#[test]
fn the_scripted_batch_fits_its_budget_and_call() {
    use skein_fake_llm_domain::api::Line;
    use support::turn_restart::Profile;
    let root = Path::new("/fixture/batch-admission");
    let fixture = landing_fixture(root, Some(Profile::NarrowedBatch));
    let value = fixture.retained_configuration();
    let engine = temper_real_loop_world::settings::configuration(&value);
    let stock = temper_service::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    let scripts = support::batch_landing::scripted_batch(
        support::forgejo::native_scripts(b"hello", local(3000), 1, b"README").into_vec(),
        stock.rules.maximum_run_spend,
    );
    let owner = scripts.iter().find(|script| script.cue.as_ref() == b"@forge_chat").expect("owner");
    let Line::Call { arguments, .. } = &owner.turns[0].lines[0] else { panic!("delegate") };
    let delegate: serde_json::Value = serde_json::from_slice(arguments).expect("delegate JSON");
    let tracked = delegate["batch"].as_array().expect("batch").iter().filter(|task| !task["tracked"].is_null()).count();
    assert!(
        u32::try_from(tracked).expect("tracked changes") <= engine.limits.core.tasks.project_goals,
        "the independent changes fit the project's goal limit"
    );
    let result_credits = u32::try_from(delegate["batch"].as_array().expect("batch").len())
        .expect("batch members")
        .checked_mul(engine.limits.core.tasks.result_bytes)
        .expect("batch result credits");
    assert!(result_credits <= engine.limits.core.tasks.inbox_bytes, "the batch's result credits fit its inbox");
    let spend: u64 = delegate["batch"]
        .as_array()
        .expect("batch")
        .iter()
        .map(|change| change["authority"]["budget"]["spend"].as_u64().expect("spend"))
        .sum();
    let policy = &stock.policies.get(&1).expect("stock policy").policy;
    let owner = policy.roles.iter().find(|role| role.number == 0).expect("owner role");
    assert!(
        spend + stock.rules.maximum_run_spend <= owner.authority.budget.spend,
        "the chat funds every delegate and retains one run for itself"
    );
    let agent = json_bytes(&fixture.configured.agent.bytes);
    assert!(
        u64::try_from(arguments.len()).expect("call length")
            <= agent["profile"]["declared"]["tool_payload"].as_u64().expect("payload"),
        "one admitted model call carries the whole batch"
    );
}

#[test]
fn native_forgejo_settings_preserve_the_coding_producer() {
    let root = Path::new("/fixture/native-forgejo-producer");
    let api = local(3000);
    let recipe = support::recipe(root, &forge(api));
    let settings = json_bytes(recipe.landing_settings.as_ref().expect("native landing settings"));
    let producer = &settings["items"][1]["set"];
    assert_eq!(producer, 2, "native scripts delegate to charter 2");
    assert_eq!(settings["items"][0]["set"]["gates"], json!([]));
    let scripts = support::forgejo::native_scripts(b"hello", api, 1, b"README");
    let skein_fake_llm_domain::api::Line::Call { name, arguments } = &scripts[0].turns[0].lines[0] else {
        panic!("delegate call")
    };
    assert_eq!(name.as_ref(), b"delegate");
    let delegate: Value = serde_json::from_slice(arguments).expect("delegate JSON");
    let (_, deployment) = support::configuration::documents(root, "/fixture/temper");
    let deployment = temper_playground::documents::Configuration::from_jig(
        serde_json::from_slice(&deployment).expect("deployment JSON"),
    );
    let engine = temper_real_loop_world::settings::configuration(&deployment);
    let stock = temper_service::stock::defaults(&engine.llm, engine.limits.core.tasks.result_bytes);
    let spend = delegate["batch"][0]["authority"]["budget"]["spend"].as_u64().expect("change spend");
    assert!(
        spend >= stock.rules.maximum_run_spend * 2,
        "the native change funds stock model reserves and its production and review turns"
    );
    let kinds = delegate["batch"][0]["authority"]["delegation"]["kinds"].as_array().expect("kinds");
    assert!(kinds.contains(&json!({"kind":"agent", "number":producer})));
}

#[test]
fn owned_settings_preserve_the_ordinary_producer_without_review() {
    let owned = support::owned::Owned { review: false, ..support::owned::Owned::example() };
    let settings = json_bytes(&support::owned::owned_landing_settings(&owned).expect("no-review settings"));
    assert_eq!(settings["items"][1]["set"], 1, "owned fixture uses the ordinary charter");
    assert_eq!(settings["items"][0]["set"]["gates"], serde_json::json!([]));
}

#[test]
fn native_landing_rules_replace_the_projects_stock_review() {
    for producer in [1, 2] {
        let settings = json_bytes(&support::owned::producer_landing_settings_bytes(producer));
        assert_eq!(
            settings["items"][0],
            serde_json::json!({"setting":{"kind":"landing-rule","project":1,"number":1},"set":{
                "pattern":{"segments":["6272616e6368"],"last":{"open":""}},
                "enforce":true,"ci":true,"up_to_date":true,"gates":[],"approvals":[]}}),
            "no-review fixtures replace project rule 1 and keep CI and freshness"
        );
        assert_eq!(settings["items"][1]["set"], producer);
    }
}

#[test]
fn turn_restart_configuration_passes_startup_admission() {
    landing_configuration_admission("/fixture/turn-restart-admission", support::turn_restart::Profile::Turn);
}

#[test]
fn batch_landing_configuration_passes_startup_admission() {
    landing_configuration_admission("/fixture/batch-landing-admission", support::turn_restart::Profile::NarrowedBatch);
}

fn landing_configuration_admission(root: &str, profile: support::turn_restart::Profile) {
    use skein_lib::{List, Secret};
    use temper_service::settings::{Files, Seeds};

    let fixture = landing_fixture(Path::new(root), Some(profile));
    let figures = &fixture.configured.configuration;
    let document = &fixture.recipe.read_start().document;
    let directories = jig_protocol_config::Directories::working();
    let named = temper_service::named(document, &directories).expect("named story files");
    let mut entries = List::with_capacity(u32::try_from(named.files.len()).expect("file count"));
    for file in named.files {
        let bytes = fixture
            .recipe
            .files
            .entries
            .iter()
            .find(|(path, _)| *path == file.path)
            .unwrap_or_else(|| panic!("missing composed file {}", String::from_utf8_lossy(&file.path)))
            .1
            .expose();
        entries.push((file.path, Secret::new(bytes.into()))).expect("named file count");
    }
    let seeds = Seeds {
        link_secret: None,
        local_secret: None,
        deployment: [7; 16],
        installation: [8; 16],
        seed: 7,
        crypto: Secret::new(Box::new([9; 32])),
    };
    let result = temper_service::startup::checked_start_with_limits(
        document,
        &directories,
        &Files { entries },
        Some(&fixture.configured.agent.agent),
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
}

#[test]
fn turn_configuration_declares_the_prepare_job() {
    let fixture = landing_fixture(Path::new("/fixture/turn-prepare-carriers"), None);
    let config = fixture.retained_configuration();
    let engine = temper_real_loop_world::settings::configuration(&config);
    let prepare = temper_service::limits::prepare_bound();
    assert!(
        temper_service::limits::prepare_jobs(&engine.limits).contains(&prepare),
        "the turn declares the prepare job"
    );
    assert!(engine.limits.core.tasks.result_bytes >= prepare.result, "the worker carries the prepare result");
}

struct LandingFixture {
    recipe: support::Recipe,
    configured: support::turn_restart::Configured,
    figures: Vec<u8>,
}
impl LandingFixture {
    fn retained_configuration(&self) -> temper_playground::documents::Configuration {
        support::limit_settings::configuration_over(
            self.recipe.start_document(),
            temper_agent_fixtures::providers::flat(&self.configured.setting.providers),
            Some(&self.figures),
        )
    }
}

fn landing_fixture(root: &Path, profile: Option<support::turn_restart::Profile>) -> LandingFixture {
    use support::turn_restart::{self, Inputs, Profile, Stage};
    let recipe = support::recipe(root, &forge(local(3000)));
    let start = recipe.start_document();
    let configuration = support::limit_settings::configuration_over(
        start.clone(),
        temper_agent_fixtures::providers::flat(&recipe.agent_setting().providers),
        Some(&support::limit_settings::figures_bytes(&recipe.configuration)),
    );
    let inputs = Inputs {
        configuration,
        agent: json_bytes(&recipe.agent.bytes),
        native_settings: json_bytes(recipe.landing_settings.as_ref().expect("native settings")),
    };
    let prepare = |setting: &temper_agent_fixtures::Setting| {
        let agent = temper_agent_fixtures::with_figures(&start, &setting.figures, root, &setting.providers);
        support::prepare_agent(serde_json::to_vec(&agent).expect("agent JSON").into())
    };
    let mut seeded = Vec::new();
    let mut figures = Vec::new();
    let stage = |step: Stage<'_>| match step {
        Stage::Landing(bytes) => turn_restart::stage_landing_with(
            root,
            bytes,
            |path| path == root.join("state"),
            |path, _| assert_eq!(path, root.join("landing-settings.json")),
            |path, bytes| {
                temper_real_loop_world::fixtures::seed_settings_file_with(path, bytes, |path, writes| {
                    assert_eq!(path, root.join("state"));
                    assert_eq!(writes.len(), 4, "metadata and data for both landing settings");
                });
                seeded.push(json_bytes(bytes));
            },
        ),
        Stage::Figures(value) => figures = support::limit_settings::figures_bytes(value),
        Stage::LocalPlaces(_) | Stage::Native(_) => {}
    };
    let configured = match profile {
        Some(profile) => turn_restart::configure_with(inputs, 1, profile, prepare, stage),
        None => turn_restart::configure_batch_with(inputs, 1, 3, prepare, stage),
    };
    let narrowed = matches!(profile, Some(Profile::NarrowedBatch));
    assert_eq!(seeded.len(), if narrowed { 2 } else { 1 });
    let last = seeded.last().expect("landing settings seed");
    let batch = if matches!(profile, Some(Profile::Turn)) { 1 } else { 3 };
    assert_eq!(last["items"][0]["set"], json!({"by":"turn","batch":batch}));
    if narrowed {
        assert_eq!(
            last["items"][1]["set"]["repositories"][0]["gates"][0]["report"],
            json!({"artifact":"failed-tests","format":"lines"}),
        );
    }
    LandingFixture { recipe, configured, figures }
}

// It reaches `forgejo_elsewhere`, whose capture witness uses Linux's abstract Unix sockets.
#[cfg(target_os = "linux")]
#[test]
fn elsewhere_reads_the_connected_project() {
    use support::forgejo_elsewhere::check_adopted_project;
    support::forgejo_elsewhere::tests::reads_the_connected_project();
    let reply = |owner, name| support::Ended {
        exit: Some(skein_io::kernel::Exit::Code(0)),
        stdout: format!(
            "{{\"connecting\":{{}}}}\n{}\n",
            json!({"project":7,"outcome":{"adopted":{"owner":owner,"name":name}}})
        )
        .into_bytes(),
        stderr: Vec::new(),
    };
    let project = check_adopted_project(&reply("elsewhere", "first")).expect("adopted repository");
    assert_eq!(project, 7, "the project is read from the connect answer");
    assert!(check_adopted_project(&reply("other", "first")).is_err(), "owner must match");
    assert!(check_adopted_project(&reply("elsewhere", "other")).is_err(), "repository must match");
}

#[test]
fn ctl_limit_settings_apply_both_kinds_in_one_revision() {
    let root = Path::new("/fixture/limit-settings-step");
    let reads = limit_reads(&json!([]), &json!([]));
    let overrides =
        [("temper-limits", "tool_payload", 2048, "small agent"), ("limits", "tasks.tasks", 8, "small task set")];
    assert_eq!(support::limit_settings::held_text(&overrides), "tool_payload 2048\ntasks.tasks 8\n");
    let file = root.join("limit-settings.json");
    let mut captured = None;
    let call = support::ctl::limit_settings_apply_with(root, &overrides, &reads, None, |path, bytes| {
        assert_eq!(path, file);
        captured = Some(json_bytes(bytes));
    })
    .expect("figures change the fresh settings");
    assert_eq!(
        call.arguments(),
        support::ctl::Call::new(&[
            "settings",
            "apply",
            file.to_str().expect("FILE path"),
            "--against",
            "7",
            "--key",
            "fixture-limits-7"
        ])
        .arguments()
    );
    let value: serde_json::Value = captured.expect("one settings FILE");
    assert_eq!(
        value,
        serde_json::json!({"items":[
            {"setting":{"kind":"limits"},"set":[{"name":"tasks.tasks","value":8,"reason":"small task set"}]},
            {"setting":{"kind":"temper-limits"},"set":[{"name":"tool_payload","value":2048,"reason":"small agent"}]}
        ]})
    );
}

#[test]
fn ctl_skips_a_limit_change_already_held() {
    const TASKS: u64 = 8; // The cluster fixture's tasks.tasks.
    let current = json!([{"name":"tasks.tasks","value":TASKS,"reason":"small task set"}]);
    let reads = limit_reads(&current, &json!([]));
    let overrides = [("limits", "tasks.tasks", TASKS, "small task set")];
    let mut writes = 0;
    let call = support::ctl::limit_settings_apply_with(
        Path::new("/fixture/limit-settings-step"),
        &overrides,
        &reads,
        None,
        |_, _| writes += 1,
    );
    assert_eq!((call.is_none(), writes), (true, 0));
}

pub(super) fn limit_reads(jig: &Value, temper: &Value) -> Vec<support::Ended> {
    [
        json!({"newest":7}),
        json!({"setting":{"kind":"limits"},"value":jig}),
        json!({"setting":{"kind":"temper-limits"},"value":temper}),
    ]
    .into_iter()
    .map(|value| support::Ended {
        exit: Some(skein_io::kernel::Exit::Code(0)),
        stdout: serde_json::to_vec(&value).expect("read JSON"),
        stderr: Vec::new(),
    })
    .collect()
}

fn json_bytes(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("fixture JSON")
}

fn local(port: u16) -> SocketAddr {
    ([127, 0, 0, 1], port).into()
}

fn forge(api: SocketAddr) -> support::Forge<'static> {
    support::Forge::Throwaway { forgejo: Path::new("/fixture/forgejo"), api, git: format!("http://{api}/") }
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
fn cluster_stop_bounds_read_the_printed_seconds_and_need_a_complete_line() {
    support::cluster::deadline_tests::stop_bounds_read_the_printed_seconds_and_need_a_complete_line();
}

#[test]
fn cluster_a_member_wait_can_defer_only_to_a_longer_world() {
    support::cluster::deadline_tests::a_member_wait_can_defer_only_to_a_longer_world();
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
    support::github::file_scan_tests::file_scan_excludes_only_jigs_generation_secrets_store();
}

#[test]
fn serve_listening_requires_the_clients_listener_in_this_runs_log() {
    support::boxed::serve_log_tests::serve_listening_requires_the_clients_listener_in_this_runs_log();
}

#[test]
fn ctl_link_sign_in_waits_for_serve_before_starting_its_bound() {
    support::ctl::serve_gate_tests::link_sign_in_waits_for_serve_before_starting_its_bound();
}

#[test]
fn native_sign_in_keeps_its_start_bound_until_serve_listens() {
    support::person::serve_gate_tests::native_sign_in_keeps_its_start_bound_until_serve_listens();
}
