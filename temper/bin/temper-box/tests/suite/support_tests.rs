//! Harness contracts run once in focused (skein/docs/foundation/testing-strategy.md, 8).
use crate::support;
use temper_box_kit::cluster_turn as cluster_turn_setup;

#[test]
fn cluster_turn_setup_declares_the_paced_gate_and_prepare_job() {
    use serde_json::json;
    use temper_service::startup;
    let scratch = support::Scratch::startup("cluster-turn-documents");
    let mut cluster = support::cluster::Cluster::description_from(scratch.path(), &cluster_turn_setup::fixture());
    cluster_turn_setup::prepare(&mut cluster, scratch.path(), &support::free_addresses(&[]));
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
    let engine = cluster_turn_setup::read(&cluster.dir("engine").join("config/temper.json"));
    let project = &engine["application"]["landing"]["projects"][0];
    assert_eq!(project["by"], "turn");
    assert_eq!(project["batch"], 1);
    assert_eq!(project["gates"][0]["gates"][0]["workflow"], "gate.yml");
    let limits = jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 65_536, depth: 32 };
    let bytes = serde_json::to_vec(&engine["engine"]).expect("engine JSON");
    let document = jig_protocol_config::document(&bytes, &limits).expect("engine document");
    let engine = startup::engine(&document, 0, &jig_protocol_config::Directories::working()).expect("initial engine");
    assert!(
        temper_service::limits::prepare_jobs(&engine.limits).contains(&temper_service::limits::prepare_bound()),
        "the worker's merge train must be declared"
    );
    support::write_private(&cluster.dir("llm").join("root.der"), skein_tls_kit::pki::ROOT);
    let agent = temper_worker_shell::agent::read(&cluster.dir("w1").join("config/agent.json")).expect("agent profile");
    for process in ["engine", "w1"] {
        let root = cluster.dir(process);
        let directories = temper_real_loop_world::cluster::directories_in([
            &root.join("config"),
            &root.join("data"),
            &root.join("cache"),
        ]);
        let directories = temper_box_shell::directories::jig_directories(&directories);
        let bytes = std::fs::read(root.join("config/temper.json")).expect("deployment document");
        if process == "engine" {
            startup::store_limits(&bytes, &directories, Some(&agent.declaration()))
                .unwrap_or_else(|why| panic!("{process}: {}", String::from_utf8_lossy(&startup::lines(&why))));
        }
        match startup::shape(&bytes, &directories).expect("separate shape") {
            startup::Shape::EngineAlone => {
                startup::named(&bytes, &directories).expect("checked engine alone");
            }
            startup::Shape::WorkerAlone => {
                startup::worker_part(&bytes, &directories).expect("checked worker alone");
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
        json!({"setting":{"kind":"charter","number":1},"value":{"documents":[1],"kept":"chat"}}),
        json!({"setting":{"kind":"charter","number":2},"value":{"documents":[2],"kept":"producer"}}),
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
    assert_eq!(settings["items"][3]["set"]["gates"], json!([]), "turn's test gate is the only gate");
    scratch.remove();
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
        script.ends_with("exec '/story/git' \"$@\"\n"),
        "the wrapper always invokes fake git with the same arguments"
    );
}

#[test]
fn cluster_turn_proof_cut_refuses_to_pass_after_proving() {
    use serde_json::json;
    assert_eq!(cluster_turn_setup::proof_cut(&json!({"phase":"preparing"})), Ok(None));
    assert_eq!(
        cluster_turn_setup::proof_cut(&json!({"phase":"proving","members":[{"gates":[{"proof":{
        "dispatched":{"attempt":1,"run":3}}}]}]})),
        Ok(Some(3))
    );
    for panel in [
        json!({"phase":"turn"}),
        json!({"phase":"landed"}),
        json!({"phase":"proving","members":[{"gates":[{"proof":{"passed":{"run":3}}}]}]}),
    ] {
        let finding = cluster_turn_setup::proof_cut(&panel).expect_err("a missed cut never passes");
        assert!(finding.contains(&format!("pace.ci_ms={}", cluster_turn_setup::CI_MS)), "{finding}");
    }
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

fn landing_configuration_admission(case: &str, configure: fn(&std::path::Path, u64)) {
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
    configure(root, 1);
    let document = std::fs::read(root.join("temper.json")).expect("story configuration");
    let directories = jig_protocol_config::Directories::working();
    let named = temper_service::named(&document, &directories).expect("named story files");
    let mut entries = List::with_capacity(u32::try_from(named.files.len()).expect("file count"));
    for file in named.files {
        let path = std::str::from_utf8(&file.path).expect("fixture path");
        let bytes = std::fs::read(root.join(path)).expect("fixture file");
        entries.push((file.path, Secret::new(bytes.into_boxed_slice()))).expect("named file count");
    }
    let agent = temper_worker_shell::agent::read(&root.join("agent.json")).expect("story agent");
    let seeds = Seeds {
        link_secret: None,
        local_secret: None,
        deployment: [7; 16],
        installation: [8; 16],
        seed: 7,
        crypto: Secret::new(Box::new([9; 32])),
    };
    let result = temper_service::startup::check(
        &document,
        &directories,
        &Files { entries },
        &agent,
        &seeds,
        None,
        List::with_capacity(0),
        None,
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
fn cluster_passed_accepts_only_a_successful_settled_verdict() {
    support::cluster::verdict_tests::passed_accepts_only_a_successful_settled_verdict();
}

#[test]
fn cluster_a_member_wait_can_defer_only_to_a_longer_world() {
    support::cluster::deadline_tests::a_member_wait_can_defer_only_to_a_longer_world();
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
fn ctl_a_failed_ctl_check_reaches_the_world_after_cleanup() {
    support::ctl::read_condition_tests::a_failed_ctl_check_reaches_the_world_after_cleanup();
}

#[test]
fn ctl_a_returned_visit_drains_without_another_process_or_a_poll() {
    support::ctl::read_condition_tests::a_returned_visit_drains_without_another_process_or_a_poll();
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
fn forge_land_rung_an_expired_rung_phase_starts_no_further_command() {
    support::forge_land::rung::tests::an_expired_rung_phase_starts_no_further_command();
}

#[test]
fn forge_land_rung_rung_phases_keep_the_request_deadlines_when_selected_again() {
    support::forge_land::rung::tests::rung_phases_keep_the_request_deadlines_when_selected_again();
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
fn github_file_scan_excludes_only_jigs_generation_secrets_store() {
    support::github::file_scan_excludes_only_jigs_generation_secrets_store();
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
