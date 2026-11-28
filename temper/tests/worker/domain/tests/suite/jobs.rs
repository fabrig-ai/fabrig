//! Job capability composition (temper/docs/design/domain/worker/jobs.md).

use temper_worker_domain_world::whole::jobs::{Case, run};

/// temper/docs/design/domain/worker/jobs.md.
#[test]
fn the_hello_states_the_job_capabilitys_kinds() {
    let report = run(1, Case::Hello).expect("world checks");
    assert_eq!(report.kinds[0].kind.0, 9);
    assert!(report.answer.is_none());
}

/// temper/docs/design/domain/worker/jobs.md.
#[test]
fn a_job_runs_its_command_in_its_prepared_checkout_and_answers_its_result() {
    assert_eq!(run(2, Case::Finished).expect("world checks").pushes, 0);
}

/// temper/docs/design/protocol/hooks.md, 4.3: missing checkout hooks are command failures, not start refusals.
#[test]
fn a_missing_close_hook_answers_as_a_command_failure() {
    use temper_worker_domain_world::whole::close;
    assert_eq!(close::run(8, close::Case::MissingHook).expect("world checks").pushes, 0);
}

/// temper/docs/design/protocol/hooks.md, 4.3: an unmerged hook fails before it can be run.
#[test]
fn an_unmerged_hook_does_not_reach_the_spawn_boundary() {
    use temper_worker_domain_world::whole::merge;
    assert_eq!(merge::run(9, merge::Case::ConflictedHook).expect("world checks").pushes, 0);
}

/// temper/docs/design/protocol/hooks.md, 4.2: no hook does not deliver a conflict.
#[test]
fn an_absent_merge_hook_leaves_conflicts_undelivered() {
    use temper_worker_domain_world::whole::merge;
    assert_eq!(merge::run(10, merge::Case::NoHookConflicted).expect("world checks").pushes, 0);
}

/// temper/docs/design/protocol/hooks.md, 4.2: no hook still delivers a clean merge.
#[test]
fn an_absent_merge_hook_delivers_a_clean_catch_up() {
    use temper_worker_domain_world::whole::merge;
    assert_eq!(merge::run(11, merge::Case::NoHookClean).expect("world checks").pushes, 1);
}

/// temper/docs/design/protocol/hooks.md, 4.2: finding no close hook is a finished job, not a failure.
#[test]
fn an_absent_close_hook_finishes_without_delivering() {
    use temper_worker_domain_world::whole::close;
    assert_eq!(close::run(12, close::Case::NoHook).expect("world checks").pushes, 0);
}

/// temper/docs/design/domain/worker/jobs.md.
#[test]
fn a_delivering_kinds_job_pushes_its_writable_repositorys_head_before_answering() {
    let report = run(3, Case::Delivered).expect("world checks");
    assert_eq!(report.pushes, 1);
    assert_eq!(report.tree.get(b"code".as_slice()).expect("delivered code"), b"checked\n");
}

/// temper/docs/design/domain/worker/jobs.md.
#[test]
fn a_delivering_jobs_stale_push_answers_failed_undelivered() {
    let report = run(4, Case::Stale).expect("world checks");
    assert_eq!(report.pushes, 1);
    assert_eq!(report.tree.get(b"code".as_slice()).expect("concurrent code"), b"another writer\n");
}

/// temper/docs/design/domain/worker/jobs.md.
#[test]
fn a_cancelled_job_is_killed_and_answered_once_its_processes_are_gone() {
    let report = run(5, Case::Cancelled).expect("world checks");
    assert_eq!(report.kills, 1);
    assert_eq!(report.pushes, 0);
}

/// temper/docs/design/domain/worker/jobs.md.
#[test]
fn a_job_whose_spec_did_not_decode_is_refused_its_start() {
    assert_eq!(run(6, Case::Undecodable).expect("world checks").pushes, 0);
}

/// temper/docs/design/domain/worker/jobs.md.
#[test]
fn a_job_waits_for_its_result_read_after_its_processes_are_gone() {
    assert_eq!(run(7, Case::GoneBeforeRead).expect("world checks").pushes, 0);
}

/// temper/docs/design/domain/worker/jobs.md.
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
        assert_eq!(run(81, case).expect("world checks"), run(81, case).expect("world checks"));
    }
}
