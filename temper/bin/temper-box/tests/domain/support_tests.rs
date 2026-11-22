//! Harness contracts over values and scripted completions.
use crate::support;
use temper_box_kit::cluster_turn as cluster_turn_setup;

#[test]
fn an_explicit_program_supplies_its_own_death_signal_wrapper() {
    support::boxed::tests::an_explicit_program_supplies_its_own_death_signal_wrapper();
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
fn cluster_turn_proof_cut_refuses_to_pass_after_proving() {
    use serde_json::json;
    assert_eq!(cluster_turn_setup::proof_cut(&json!({"phase":"waiting"})), Ok(None));
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
