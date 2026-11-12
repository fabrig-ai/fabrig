//! Baseline owner admission, coverage and the application's mark-only fixture.
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
