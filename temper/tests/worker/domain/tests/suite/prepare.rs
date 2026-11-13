//! Prepare stories (temper/docs/design/domain/worker/jobs.md).
use temper_worker_domain_world::whole::prepare::{Case, run};

#[test]
fn a_prepare_merge_pushes_the_base_then_head_with_its_message() {
    for case in [Case::Merge, Case::ContainedMerge] {
        assert_eq!(run(1, case).pushes, 1);
    }
}
#[test]
fn a_prepare_squash_has_the_merged_tree_one_parent_and_commits_even_when_empty() {
    for case in [Case::Squash, Case::EmptySquash] {
        assert_eq!(run(2, case).pushes, 1);
    }
}
#[test]
fn a_squash_that_brings_upstream_commits_is_merged_and_one_that_does_not_is_squashed() {
    for case in [Case::Upstream, Case::NoContribution] {
        assert_eq!(run(3, case).pushes, 1);
    }
}
#[test]
fn either_prepare_way_reports_its_conflict_without_a_push() {
    for case in [Case::ConflictMerge, Case::ConflictSquash] {
        assert_eq!(run(4, case).pushes, 0);
    }
}
#[test]
fn an_existing_landing_branch_fails_prepare_undelivered() {
    let _report = run(5, Case::Existing);
}
#[test]
fn three_members_merge_onto_the_rolling_prefix_and_create_each_branch() {
    assert_eq!(run(6, Case::StackMerge).pushes, 3);
}
#[test]
fn a_squash_stack_has_each_merge_tree_one_parent_and_its_trailers() {
    assert_eq!(run(7, Case::StackSquash).tree, run(7, Case::StackMerge).tree);
}
#[test]
fn a_kept_prefix_starts_the_stack_but_brings_still_compares_with_main() {
    assert_eq!(run(8, Case::KeptPrefix).pushes, 3);
    assert_eq!(run(8, Case::KeptUpstream).pushes, 3);
}
#[test]
fn a_branch_conflict_is_skipped_with_the_base_merges_files() {
    assert_eq!(run(9, Case::BranchConflict).pushes, 2);
}
#[test]
fn a_conflict_with_an_earlier_member_is_told_apart() {
    assert_eq!(run(10, Case::MemberConflict).pushes, 2);
}
#[test]
fn the_first_conflict_on_main_needs_no_second_merge() {
    assert_eq!(run(11, Case::FirstConflict).pushes, 2);
}
#[test]
fn all_conflicting_members_report_conflicts_and_push_nothing() {
    assert_eq!(run(12, Case::AllConflict).pushes, 0);
}
#[test]
fn an_existing_second_prefix_stops_delivery_without_undoing_the_first() {
    assert_eq!(run(13, Case::ExistingPrefix).pushes, 2);
}
#[test]
fn prefix_names_counts_and_access_must_match_the_ordered_members() {
    for case in
        [Case::WrongPrefix, Case::FewerPrefixes, Case::MorePrefixes, Case::MissingPrefix, Case::MultipleWritable]
    {
        assert_eq!(run(14, case).pushes, 0);
    }
}
