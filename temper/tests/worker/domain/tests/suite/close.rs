//! Close job stories (temper/docs/design/domain/worker/jobs.md).

use temper_worker_domain_world::whole::close::{Case, run};

/// temper/docs/design/domain/worker/jobs.md.
#[test]
fn a_close_job_commits_what_its_command_changed_and_pushes_it_with_one_parent() {
    let report = run(1, Case::Changed);
    assert_eq!(report.pushes, 1);
    assert_eq!(report.tree.get(b"docs/a.status.md".as_slice()).expect("settled status"), b"settled\n");
}

/// temper/docs/design/domain/worker/jobs.md.
#[test]
fn a_close_job_that_changed_nothing_finishes_without_a_push() {
    let report = run(2, Case::Unchanged);
    assert_eq!(report.pushes, 0);
    assert_eq!(report.tree.get(b"docs/a.status.md".as_slice()).expect("unchanged status"), b"in flight\n");
}

/// temper/docs/design/domain/worker/jobs.md.
#[test]
fn a_close_job_that_fails_pushes_nothing() {
    assert_eq!(run(3, Case::Failed).pushes, 0);
}

/// temper/docs/design/domain/worker/jobs.md.
#[test]
fn close_job_stories_replay() {
    for case in [Case::Changed, Case::Unchanged, Case::Failed] {
        assert_eq!(run(81, case), run(81, case));
    }
}
