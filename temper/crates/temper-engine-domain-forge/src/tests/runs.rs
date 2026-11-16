use crate::runs;
use alloc::boxed::Box;
use skein_forge::api as a;
use skein_lib::{Duration, Time, Wall};
use temper_engine_domain_forge_landing as policy;
const COMMIT: a::Commit = [3; 32];
fn run() -> a::Run {
    a::Run {
        id: 7,
        number: 8,
        workflow: Box::from(b"gate.yml".as_slice()),
        commit: COMMIT,
        event: a::Event::Dispatch,
        actor: 1,
        state: a::RunState::Completed,
        conclusion: Some(a::Conclusion::Success),
        url: Box::from(b"https://forge/run/7".as_slice()),
        created: Time::from_nanos(20),
        jobs: Box::new([]),
    }
}
pub(super) fn follow() -> runs::Follow {
    let mut follow = runs::follow(
        61,
        9,
        Box::new([6; 18]),
        super::REPO,
        Box::from(b"gate.yml".as_slice()),
        Box::from(b"temper/51".as_slice()),
        COMMIT,
        Box::new([]),
    );
    follow.writes = Box::new([]);
    follow.start = Some(Time::from_nanos(10));
    follow.since = Some(Time::from_nanos(10));
    follow.wall = Some(Wall::from_nanos(10));
    follow
}
#[test]
fn workflow_reservations_compare_repository_commit_and_inputs_through_fault_retries() {
    let mut d = super::domain();
    let key = crate::ReceiptKey {
        project: 5,
        repository: super::REPO,
        check: crate::Check::Gate(Box::from(b"gate".as_slice())),
        key: [8; 32],
    };
    let inputs =
        Box::new([a::Input { name: Box::from(b"temper_key".as_slice()), value: Box::from(b"other".as_slice()) }]);
    d.landing.flights.insert(key.clone(), follow()).expect("flight room");
    assert!(runs::busy(&d, super::REPO, b"gate.yml", COMMIT, inputs.as_ref()));
    assert!(!runs::busy(&d, super::REPO, b"gate.yml", COMMIT, &[]), "same inputs still share a flight");
    assert!(!runs::busy(&d, a::Repository { repository: 3, ..super::REPO }, b"gate.yml", COMMIT, inputs.as_ref()));
    assert!(!runs::busy(&d, super::REPO, b"other.yml", COMMIT, inputs.as_ref()));
    assert!(!runs::busy(&d, super::REPO, b"gate.yml", [4; 32], inputs.as_ref()));
    let mut flight = d.landing.flights.remove(&key).expect("known flight");
    flight.outcome = Some(policy::Verdict::Failed);
    d.landing.flights.insert(key.clone(), flight.clone()).expect("flight room");
    assert!(!runs::busy(&d, super::REPO, b"gate.yml", COMMIT, inputs.as_ref()), "an ended run releases its window");
    d.landing.flights.remove(&key);
    flight.outcome = Some(policy::Verdict::Fault(policy::Fault::Cancelled));
    d.landing.faults.insert((flight.task, 0, 0), flight.clone()).expect("fault room");
    assert!(
        runs::busy(&d, super::REPO, b"gate.yml", COMMIT, inputs.as_ref()),
        "a fault waiting for retry still owns its window"
    );
    d.landing.faults.remove(&(flight.task, 0, 0));
    runs::retry(&mut flight, 2);
    assert!(flight.start.is_none());
    d.landing.flights.insert(key, flight).expect("retry room");
    assert!(
        runs::busy(&d, super::REPO, b"gate.yml", COMMIT, inputs.as_ref()),
        "a fresh dispatch retry keeps its inputs reserved"
    );
}

#[test]
fn only_our_dispatch_at_the_commit_after_start_is_followed() {
    let mut follow = follow();
    let valid = run();
    let mut other_actor = valid.clone();
    other_actor.actor = 2;
    let mut other_event = valid.clone();
    other_event.event = a::Event::Other;
    let mut other_commit = valid.clone();
    other_commit.commit = [4; 32];
    let mut old = valid.clone();
    old.created = Time::from_nanos(9);
    assert!(
        follow
            .answered(
                Ok(a::Answer::Runs {
                    runs: Box::new([other_actor, other_event, other_commit, old, valid]),
                    more: false
                }),
                1,
                Time::from_nanos(20),
                Wall::from_nanos(20),
                Duration::from_secs(10)
            )
            .is_none()
    );
    assert_eq!(follow.read(), Some(a::Read::Run { run: 7 }));
    let ended = follow
        .answered(Ok(a::Answer::Run(run())), 1, Time::from_nanos(20), Wall::from_nanos(20), Duration::from_secs(10))
        .expect("one end");
    assert_eq!(ended.outcome, policy::Verdict::Passed);
    assert!(
        follow
            .answered(Ok(a::Answer::Run(run())), 1, Time::from_nanos(20), Wall::from_nanos(20), Duration::from_secs(10))
            .is_none()
    );
}
#[test]
fn a_missing_run_is_lost_once_at_its_local_lifetime() {
    let mut follow = follow();
    follow.start = Some(Time::from_nanos(1_000));
    let empty = a::Answer::Runs { runs: Box::new([]), more: false };
    assert!(
        follow
            .answered(Ok(empty.clone()), 1, Time::from_nanos(19), Wall::from_nanos(19), Duration::from_nanos(10))
            .is_none()
    );
    assert_eq!(
        follow
            .answered(Ok(empty.clone()), 1, Time::from_nanos(20), Wall::from_nanos(20), Duration::from_nanos(10))
            .expect("lost")
            .outcome,
        policy::Verdict::Fault(policy::Fault::Lost)
    );
    assert!(
        follow.answered(Ok(empty), 1, Time::from_nanos(21), Wall::from_nanos(21), Duration::from_nanos(10)).is_none()
    );
}
#[test]
fn an_unseen_run_is_not_lost_until_a_read_answers_without_it() {
    for error in [a::Error::Unavailable, a::Error::Timeout] {
        let mut follow = follow();
        for now in [20, 40, 80] {
            assert!(
                follow
                    .answered(Err(error), 1, Time::from_nanos(now), Wall::from_nanos(now), Duration::from_nanos(10))
                    .is_none(),
                "{error:?} is not a runner fault"
            );
            assert_eq!(follow.outcome, None);
            assert!(matches!(follow.read(), Some(a::Read::Runs { .. })));
        }
        let empty = a::Answer::Runs { runs: Box::new([]), more: false };
        assert_eq!(
            follow
                .answered(Ok(empty), 1, Time::from_nanos(81), Wall::from_nanos(81), Duration::from_nanos(10))
                .expect("answered absence past lifetime")
                .outcome,
            policy::Verdict::Fault(policy::Fault::Lost)
        );
    }
}

#[test]
fn the_execution_clock_counts_from_the_first_running_answer_and_ignores_completion() {
    let mut follow = follow();
    let mut run = run();
    run.state = a::RunState::Waiting;
    run.conclusion = None;
    follow.run = Some(run.clone());
    for now in [20, 40, 80] {
        assert!(
            follow
                .answered(
                    Ok(a::Answer::Run(run.clone())),
                    1,
                    Time::from_nanos(now),
                    Wall::from_nanos(now),
                    Duration::from_secs(10)
                )
                .is_none()
        );
        let clock = follow.clock.expect("a waiting reading");
        assert_eq!(clock.ran(Wall::from_nanos(now)), Duration::ZERO);
        assert_eq!(clock.waited(Wall::from_nanos(now)), Duration::from_nanos(now - 20));
    }
    run.state = a::RunState::Running;
    for now in [100, 120, 140] {
        assert!(
            follow
                .answered(
                    Ok(a::Answer::Run(run.clone())),
                    1,
                    Time::from_nanos(now),
                    Wall::from_nanos(now),
                    Duration::from_secs(10)
                )
                .is_none()
        );
        let clock = follow.clock.expect("a running reading");
        assert_eq!(clock.ran(Wall::from_nanos(now)), Duration::from_nanos(now - 100));
        assert_eq!(clock.waited(Wall::from_nanos(now)), Duration::from_nanos(80));
    }
    let clock = follow.clock;
    run.state = a::RunState::Completed;
    run.conclusion = Some(a::Conclusion::Success);
    assert!(
        follow
            .answered(Ok(a::Answer::Run(run)), 1, Time::from_nanos(160), Wall::from_nanos(160), Duration::from_secs(10))
            .is_some()
    );
    assert_eq!(follow.clock, clock, "a completed answer adds no reading");
}

#[test]
fn a_listing_reads_the_highest_attempt_and_a_rerun_write_starts_a_fresh_clock() {
    let mut follow = follow();
    let mut run = run();
    run.state = a::RunState::Running;
    run.conclusion = None;
    let job = a::RunJob {
        id: 11,
        name: b"test".as_slice().into(),
        attempt: 3,
        state: a::RunState::Running,
        conclusion: None,
        step: None,
        steps: 0,
    };
    run.jobs = Box::new([job.clone(), a::RunJob { id: 12, attempt: 2, ..job }]);
    assert!(
        follow
            .answered(
                Ok(a::Answer::Runs { runs: Box::new([run.clone()]), more: false }),
                1,
                Time::from_nanos(20),
                Wall::from_nanos(20),
                Duration::from_secs(10)
            )
            .is_none()
    );
    let clock = follow.clock.expect("a listing reads execution too");
    assert_eq!(clock.attempt, 3);
    assert_eq!(clock.ran(Wall::from_nanos(80)), Duration::from_nanos(60));
    follow.writes =
        Box::new([a::Write::Rerun { attempt: a::JobAttempt { head: COMMIT, run: 7, job: 11, attempt: 3 } }]);
    assert!(follow.write(10).is_some());
    follow.wall = Some(Wall::from_nanos(80));
    follow.waiting = false;
    assert!(
        follow
            .answered(Ok(a::Answer::Run(run)), 1, Time::from_nanos(100), Wall::from_nanos(100), Duration::from_secs(10))
            .is_none()
    );
    let clock = follow.clock.expect("the rerun's first answer");
    assert_eq!(clock.since, Wall::from_nanos(80));
    assert_eq!(
        clock.ran(Wall::from_nanos(120)),
        Duration::from_nanos(20),
        "even an unchanged forge attempt starts afresh after our write"
    );
}

#[test]
fn a_job_fault_reruns_each_failed_attempt_and_clears_its_old_verdict() {
    let mut follow = follow();
    let mut run = run();
    run.conclusion = Some(a::Conclusion::Failure);
    let job = a::RunJob {
        id: 11,
        name: Box::from(b"test".as_slice()),
        attempt: 2,
        state: a::RunState::Completed,
        conclusion: Some(a::Conclusion::Failure),
        step: Some(a::Step { number: 0, name: Box::from(b"Set up".as_slice()) }),
        steps: 3,
    };
    let mut second = job.clone();
    second.id = 12;
    run.jobs = Box::new([job, second]);
    follow.outcome = Some(runs::verdict(&run, COMMIT));
    follow.run = Some(run);
    runs::retry(&mut follow, 2);
    assert_eq!(follow.outcome, None);
    assert_eq!(
        follow.writes.as_ref(),
        &[
            a::Write::Rerun { attempt: a::JobAttempt { head: COMMIT, run: 7, job: 11, attempt: 2 } },
            a::Write::Rerun { attempt: a::JobAttempt { head: COMMIT, run: 7, job: 12, attempt: 2 } }
        ]
    );
    assert!(follow.write(10).is_some());
    assert_eq!(follow.read(), None);
}
