//! Job capability composition (temper/docs/design/domain/worker.md, 4.4).

use temper_worker_domain_world::whole::jobs::{Case, run};

/// temper/docs/design/domain/worker.md, 4.4.
#[test]
fn the_hello_states_the_job_capabilitys_kinds() {
    let report = run(1, Case::Hello);
    assert_eq!(report.kinds[0].kind.0, 9);
    assert!(report.answer.is_none());
}

/// temper/docs/design/domain/worker.md, 4.4.
#[test]
fn a_job_runs_its_command_in_its_prepared_checkout_and_answers_its_result() {
    assert_eq!(run(2, Case::Finished).pushes, 0);
}

/// temper/docs/design/domain/worker.md, 4.4.
#[test]
fn a_delivering_kinds_job_pushes_its_writable_repositorys_head_before_answering() {
    let report = run(3, Case::Delivered);
    assert_eq!(report.pushes, 1);
    assert_eq!(report.tree.get(b"code".as_slice()).expect("delivered code"), b"checked\n");
}

/// temper/docs/design/domain/worker.md, 4.4.
#[test]
fn a_delivering_jobs_stale_push_answers_failed_undelivered() {
    let report = run(4, Case::Stale);
    assert_eq!(report.pushes, 1);
    assert_eq!(report.tree.get(b"code".as_slice()).expect("concurrent code"), b"another writer\n");
}

/// temper/docs/design/domain/worker.md, 4.4.
#[test]
fn a_cancelled_job_is_killed_and_answered_once_its_processes_are_gone() {
    let report = run(5, Case::Cancelled);
    assert_eq!(report.kills, 1);
    assert_eq!(report.pushes, 0);
}

/// temper/docs/design/domain/worker.md, 4.4.
#[test]
fn a_job_whose_spec_did_not_decode_is_refused_its_start() {
    assert_eq!(run(6, Case::Undecodable).pushes, 0);
}

/// temper/docs/design/domain/worker.md, 4.4.
#[test]
fn a_job_waits_for_its_result_read_after_its_processes_are_gone() {
    assert_eq!(run(7, Case::GoneBeforeRead).pushes, 0);
}

/// temper/docs/design/domain/worker.md, 4.4.
#[test]
fn job_stories_replay() {
    for case in [
        Case::Hello,
        Case::Finished,
        Case::Delivered,
        Case::Stale,
        Case::Cancelled,
        Case::Undecodable,
        Case::GoneBeforeRead,
    ] {
        assert_eq!(run(81, case), run(81, case));
    }
}
