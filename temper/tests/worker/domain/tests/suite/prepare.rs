//! One-member prepare stories (temper/docs/design/domain/worker/jobs.md).
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
fn two_prepare_members_are_refused_as_invalid() {
    let _report = run(6, Case::TwoMembers);
}
