//! Baseline owner admission, coverage and the application's mark-only fixture.
#[test]
fn power_lost_at_the_copys_first_write_keeps_a_whole_generation() {
    temper_one_process_world::upgrade::story::power_loss(0);
}
#[test]
fn power_lost_at_the_patch_commit_keeps_a_whole_generation() {
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
fn a_story_keeps_its_history_through_a_settled_upgrade() {
    temper_one_process_world::upgrade::story::settled();
}
#[test]
fn every_golden_key_has_exactly_one_patch_family_or_a_reserved_component_row() {
    temper_one_process_world::upgrade::golden_ranges(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("golden/baseline"),
    );
}

#[test]
fn a_mark_only_upgrade_closes_a_signal_stopped_chats_window_and_opens_again_at_equal_levels() {
    temper_one_process_world::upgrade::fresh_windows();
}

#[test]
fn a_killed_services_in_flight_write_recovers_through_the_upgrade_once() {
    temper_one_process_world::upgrade::story::unsettled();
}
