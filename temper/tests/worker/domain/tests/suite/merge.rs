//! Merge job stories (temper/docs/design/domain/worker/jobs.md).

use temper_worker_domain_world::whole::merge::{Case, run};

/// temper/docs/design/domain/worker/jobs.md.
#[test]
fn a_merge_job_settles_its_conflict_and_pushes_a_merge_with_both_parents() {
    let report = run(1, Case::Settled);
    assert_eq!(report.pushes, 1);
    assert_eq!(report.tree.get(b"code".as_slice()).expect("settled file"), b"settled\n");
}

/// temper/docs/design/domain/worker/jobs.md.
#[test]
fn a_merge_job_that_leaves_conflicts_pushes_nothing() {
    assert_eq!(run(2, Case::Left).pushes, 0);
}

/// temper/docs/design/domain/worker/jobs.md.
#[test]
fn a_merge_job_that_leaves_a_file_marked_fails_undelivered() {
    assert_eq!(run(3, Case::Marked).pushes, 0);
}

/// temper/docs/design/domain/worker/jobs.md.
#[test]
fn merge_job_stories_replay() {
    for case in [Case::Settled, Case::Left, Case::Marked] {
        assert_eq!(run(81, case), run(81, case));
    }
}
