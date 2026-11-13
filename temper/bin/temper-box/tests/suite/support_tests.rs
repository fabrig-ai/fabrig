//! Harness contracts run once in focused (skein/docs/foundation/testing-strategy.md, 8).
use crate::support;

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
#[ignore = "fails under Cranelift, which cannot unwind yet: run it on LLVM (2026-11-12)"]
fn printed_links_count_and_fail_without_disclosing_secrets() {
    support::printed_link_tests::printed_links_count_and_fail_without_disclosing_secrets();
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
fn turn_restart_keeps_the_moved_main_witness_on_retry() {
    support::turn_restart::tests::keeps_the_moved_main_witness_on_retry();
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
#[should_panic(expected = "task-list prerequisite")]
fn ctl_a_failed_child_prerequisite_stops_before_retrying() {
    support::ctl::read_condition_tests::a_failed_child_prerequisite_stops_before_retrying();
}

#[test]
#[should_panic(expected = "poll prerequisite")]
fn ctl_a_failed_poll_stops_before_retrying_an_unmet_condition() {
    support::ctl::read_condition_tests::a_failed_poll_stops_before_retrying_an_unmet_condition();
}

#[test]
#[should_panic(expected = "adoption ended refused: exit=Some(Code(2)); unreachable")]
fn ctl_a_refused_started_adoption_fails_before_polling_pending() {
    support::ctl::read_condition_tests::a_refused_started_adoption_fails_before_polling_pending();
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
#[ignore = "fails under Cranelift, which cannot unwind yet: run it on LLVM (2026-11-12)"]
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
