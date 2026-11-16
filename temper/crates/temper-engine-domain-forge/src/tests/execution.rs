//! A reading follows pending checks' linked jobs, never the run's aggregate state.
use super::*;
use crate::execution::{Gather, fold};
use skein_forge::api::{self, RunState};
use skein_lib::{List, Map};
use temper_engine_domain_forge_change::{Execution, Reading};

#[test]
fn pending_checks_fold_linked_runs_into_execution() {
    let link = api::JobAttempt { head: fixtures::HEAD, run: 7, job: 41, attempt: 2 };
    let api::Answer::Checks(mut checks) = fixtures::checks(api::Check::Pending) else { panic!("checks") };
    checks[0].job = Some(link);
    let mut run = fixtures::failed_run(link, None);
    for (name, check, answer, want, newest, attempt) in [
        ("running", checks.clone(), Some(RunState::Running), Execution::Running, 7, 2),
        ("waiting", checks.clone(), Some(RunState::Waiting), Execution::Waiting, 7, 2),
        ("completed", checks.clone(), Some(RunState::Completed), Execution::Unknown, 7, 2),
        ("unread", checks.clone(), None, Execution::Unknown, 0, 0),
        ("no pending", Box::new([]), Some(RunState::Waiting), Execution::Unknown, 7, 0),
    ] {
        let mut runs = Map::uncapped();
        runs.insert(
            link.run,
            match answer {
                Some(state) => {
                    run.jobs[0].state = state;
                    Some(run.clone())
                }
                None => None,
            },
        )
        .unwrap();
        assert_eq!(fold(&check, &runs), Reading { execution: want, run: newest, attempt }, "{name}");
    }
    let mut runs = Map::uncapped();
    run.jobs[0].state = RunState::Waiting;
    runs.insert(7, Some(run.clone())).unwrap();
    checks[0].job = None;
    assert_eq!(fold(&checks, &runs).execution, Execution::Unknown, "no job linked");
    checks[0].job = Some(api::JobAttempt { job: 42, ..link });
    assert_eq!(fold(&checks, &runs).execution, Execution::Unknown, "linked job absent");
    checks[0].check = api::Check::Passed;
    assert_eq!(fold(&checks, &runs).execution, Execution::Unknown, "only completed checks");
}

#[test]
fn a_reading_names_newest_run_and_highest_linked_job_attempt() {
    let api::Answer::Checks(mut checks) = fixtures::checks(api::Check::Pending) else { panic!("checks") };
    let mut rows = List::uncapped();
    let mut runs = Map::uncapped();
    for (id, attempt, state) in [(7, 4, RunState::Running), (9, 2, RunState::Waiting)] {
        let link = api::JobAttempt { head: fixtures::HEAD, run: id, job: 41, attempt: 1 };
        checks[0].job = Some(link);
        rows.push(checks[0].clone()).unwrap();
        let mut run = fixtures::failed_run(link, None);
        run.jobs[0].state = state;
        run.jobs[0].attempt = attempt;
        // A running unlinked job must not turn this linked waiting job into running.
        run.jobs = Box::new([
            run.jobs[0].clone(),
            api::RunJob { id: 99, attempt: 20, state: RunState::Running, ..run.jobs[0].clone() },
        ]);
        runs.insert(id, Some(run)).unwrap();
    }
    assert_eq!(fold(rows.as_slice(), &runs), Reading { execution: Execution::Running, run: 9, attempt: 4 });
    assert_eq!(fold(&rows.as_slice()[1..], &runs), Reading { execution: Execution::Waiting, run: 9, attempt: 2 });
}

#[test]
fn execution_gather_reads_eight_distinct_runs_newest_first() {
    let api::Answer::Checks(mut checks) = fixtures::checks(api::Check::Pending) else { panic!("checks") };
    let mut rows = List::uncapped();
    for id in 1..=9 {
        checks[0].job = Some(api::JobAttempt { head: fixtures::HEAD, run: id, job: 41, attempt: 2 });
        rows.push(checks[0].clone()).unwrap();
    }
    rows.push(rows.as_slice()[8].clone()).unwrap(); // Two checks of one run cost one read.
    let mut gather = Gather::new(rows.into_boxed());
    for id in (2..=9).rev() {
        assert_eq!(gather.next(), Some(api::Read::Run { run: id }));
        let link = api::JobAttempt { head: fixtures::HEAD, run: id, job: 41, attempt: 2 };
        let mut run = fixtures::failed_run(link, None);
        run.jobs[0].state = RunState::Waiting;
        gather.answered(id, Ok(api::Answer::Run(run)));
    }
    assert_eq!(gather.next(), None);
    assert_eq!(gather.reading(), Reading { execution: Execution::Unknown, run: 9, attempt: 2 });
    let mut gather = Gather::new(checks);
    assert_eq!(gather.next(), Some(api::Read::Run { run: 9 }));
    gather.answered(9, Err(api::Error::Refused));
    assert_eq!(gather.next(), None, "a failed read stays unread, never fails the step");
    assert_eq!(gather.reading(), Reading { execution: Execution::Unknown, run: 0, attempt: 0 });
}
