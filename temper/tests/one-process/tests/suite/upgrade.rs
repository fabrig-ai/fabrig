//! Baseline owner admission, coverage and a registry and job upgrade.
mod live_opening;

#[test]
fn an_opened_worker_may_stay_after_its_deployment_stops() {
    temper_one_process_world::upgrade::check::opened_worker_rehearsal();
}
#[test]
fn prebaseline_rehearsals_without_wires_report_the_older_layout() {
    temper_one_process_world::upgrade::check::prebaseline_rehearsals();
}
#[test]
fn pre_settings_refusals_leave_the_whole_directory_unchanged() {
    temper_one_process_world::testing::pre_settings_refusals_leave_the_whole_directory_unchanged();
}
#[test]
fn a_corrupt_store_open_reaches_a_report_and_empties_scratch() {
    temper_one_process_world::upgrade::check::corrupt_open_rehearsal();
}
#[test]
fn rehearsal_preserves_the_deployment_then_start_and_back_obey_the_first_commit() {
    temper_one_process_world::upgrade::check::rehearsal_then_start_and_back();
}
#[test]
fn power_lost_at_the_copys_first_write_keeps_a_whole_generation() {
    temper_one_process_world::upgrade::story::power_loss(0);
}
#[test]
fn power_lost_at_the_seal_keeps_a_whole_generation() {
    temper_one_process_world::upgrade::story::power_loss(1);
}
#[test]
fn power_lost_at_the_switch_rename_keeps_a_whole_generation() {
    temper_one_process_world::upgrade::story::power_loss(2);
}
#[test]
fn power_lost_at_the_wires_commit_keeps_a_whole_generation() {
    temper_one_process_world::upgrade::story::power_loss(3);
}
#[test]
fn power_lost_at_a_drawn_moment_keeps_a_whole_generation() {
    temper_one_process_world::upgrade::story::drawn_power_loss(5);
}
#[test]
fn a_story_keeps_its_history_through_a_settled_upgrade() {
    temper_one_process_world::upgrade::story::settled();
}
#[test]
fn a_stopped_deployments_project_forge_settings_and_landings_read_the_same_on_its_next_start() {
    temper_one_process_world::testing::restored_deployment_reads();
}
#[test]
fn golden_rehearsal_decodes_every_record_and_restores_without_pending_work() {
    temper_one_process_world::upgrade::rehearsal::golden_check(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("golden/floor"),
    );
}

#[test]
fn deployment_rehearsal_with_remote_workers_and_forge_starts_only_the_store() {
    temper_one_process_world::upgrade::rehearsal::deployment_check(
        &skein_scratch::paths::tree().join("tests/one-process/golden/floor"),
    );
}

#[test]
fn configured_accounts_rehearsal_starts_only_the_store_and_reaches_its_report() {
    temper_one_process_world::upgrade::rehearsal::accounts_check(
        &skein_scratch::paths::tree().join("tests/one-process/golden/floor"),
    );
}

#[test]
fn a_held_source_refuses_a_rehearsal_after_its_scratch_is_made() {
    live_opening::rehearsal().held_source(live_opening::settled(true));
}

#[test]
fn a_configured_floor_goes_back_to_no_previous_generation() {
    live_opening::rehearsal().no_previous(live_opening::settled(true));
}

#[test]
fn every_golden_key_has_exactly_one_patch_family_or_a_reserved_component_row() {
    temper_one_process_world::upgrade::golden_ranges(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("golden/floor"),
    );
}

#[test]
fn a_killed_services_in_flight_write_recovers_through_the_upgrade_once() {
    temper_one_process_world::upgrade::story::unsettled();
}

#[test]
fn golden_rehearsal_names_the_broken_record() {
    temper_one_process_world::upgrade::rehearsal::broken_check(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("golden/floor"),
    );
}
#[test]
fn floorless_golden_rehearsal_refuses_before_decoding() {
    temper_one_process_world::upgrade::rehearsal::floorless_check(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("golden/floor"),
    );
}

#[test]
fn the_settled_image_matches_the_story() {
    let scratch = skein_scratch::Scratch::new("settled-upgrade-image");
    temper_one_process_world::upgrade::story::settled_image(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("golden/settled"),
        scratch.path(),
    );
}
