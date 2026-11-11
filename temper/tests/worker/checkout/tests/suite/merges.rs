use temper_worker_checkout_world::merges::{self, Case, PinCase};

#[test]
fn a_repair_replaces_the_cached_producers_pin_and_freezes_it_for_deliveries() {
    merges::replaced_pin(PinCase::Repair);
}

#[test]
fn a_saved_repair_pins_the_push_head_not_its_saved_start() {
    merges::replaced_pin(PinCase::Saved);
}

#[test]
fn a_resolution_pins_the_push_head_before_merging() {
    merges::replaced_pin(PinCase::Resolution);
}

#[test]
fn a_missing_push_branch_pins_the_start_commit() {
    merges::replaced_pin(PinCase::NewBranch);
}

#[test]
fn a_read_only_item_leaves_the_cached_pin_untouched() {
    merges::replaced_pin(PinCase::ReadOnly);
}

#[test]
fn merges_conflicts_saving_head_movement_and_ambiguous_pushes_settle() {
    for case in
        [Case::Clean, Case::Conflict, Case::PastTheBound, Case::Unchanged, Case::Moved, Case::Ambiguous, Case::Saved]
    {
        let _ = merges::run(73, case, 64);
    }
}

#[test]
fn merge_scenarios_replay_and_facts_do_not_change_requests_or_effects() {
    for case in [
        Case::Clean,
        Case::Resumed,
        Case::Conflict,
        Case::PastTheBound,
        Case::Unchanged,
        Case::Moved,
        Case::Ambiguous,
        Case::Saved,
        Case::Contained,
    ] {
        let expected = merges::run(119, case, 64);
        assert_eq!(merges::run(119, case, 64), expected);
        assert_eq!(merges::run(119, case, 0), expected);
    }
}

#[test]
fn a_base_with_two_heads_merged_in_pushes_one_history_holding_all() {
    merges::merged_heads();
}

#[test]
fn a_conflicting_head_fails_the_prepare_and_the_next_starts_clean() {
    merges::conflicting_head();
}

#[test]
fn a_resumed_producer_holding_the_tip_pushes_no_merge() {
    let _ = merges::run(73, Case::Contained, 64);
}

#[test]
fn a_resumed_producers_clean_merge_is_committed_before_its_run() {
    for case in [Case::Resumed, Case::Conflict, Case::Contained] {
        let _ = merges::run(73, case, 64);
    }
}

#[test]
fn a_goals_later_run_holding_its_run_branch_in_g_starts_on_gs_tip() {
    merges::goal_later_run(Case::Contained);
}

#[test]
fn a_goals_later_run_merges_its_run_branch_into_gs_tip_and_fast_forwards_it() {
    merges::goal_later_run(Case::Resumed);
}

#[test]
fn a_goals_later_run_keeps_a_conflict_with_its_run_branch_in_progress() {
    merges::goal_later_run(Case::Conflict);
}
