use super::*;
use crate::api::{Conclusion, Job, JobRequirements, JobWait, Run, RunState, Workflow};
use crate::{Fault, Fell};

fn fixture(rule: Option<&[u8]>, jobs: &[&[u8]]) -> Setup {
    let mut configured = setup();
    configured.checks.contexts = Box::new([]);
    configured.hooked = false;
    configured.workflows = Box::new([Workflow {
        file: copy_of(b"gate"),
        jobs: names(jobs),
        requirements: Box::new([]),
        latency_min: Duration::from_secs(10),
        latency_max: Duration::from_secs(10),
        silent: 0,
        passes: 1000,
        cue: if rule.is_some() { Some(Cue::Declared { path: copy_of(b"rule") }) } else { None },
        fails: Box::new([]),
        flaky: Box::new([]),
        select: None,
        report: None,
    }]);
    if let Some(rule) = rule {
        configured.tree = Box::new([File { path: copy_of(b"rule"), content: copy_of(rule) }]);
    }
    configured
}
fn harness(rule: Option<&[u8]>, jobs: &[&[u8]]) -> Harness {
    let config = Config { latency_min: Duration::ZERO, latency_max: Duration::ZERO, ..calm() };
    Harness::with(config, fixture(rule, jobs))
}
fn dispatch(h: &mut Harness, repository: &[u8]) {
    h.call_on(
        repository,
        ENGINE,
        write(Write::Dispatch { workflow: copy_of(b"gate"), branch: copy_of(MAIN), inputs: Box::new([]) }),
    )
    .unwrap();
}
fn run(h: &Harness, repository: &[u8], run: u64) -> Run {
    let Answer::Run(run) = h.domain.inspect(&h.env.limits, repository, &Read::Run { run }).unwrap() else {
        panic!("run")
    };
    run
}
fn job(h: &Harness, repository: &[u8], run: u64, index: usize) -> Job {
    let Answer::Jobs(jobs) = h.domain.inspect(&h.env.limits, repository, &Read::Jobs { run }).unwrap() else {
        panic!("jobs")
    };
    jobs[index].clone()
}
fn at(h: &mut Harness, seconds: u64) {
    h.until(Time::ZERO.saturating_add(Duration::from_secs(seconds)));
}
fn wait_fell(at: u64, until: u64, run: u64, repository: &[u8], commit: u64, runner: u32) -> Fell {
    Fell {
        at: Time::ZERO.saturating_add(Duration::from_secs(at)),
        until: Some(Time::ZERO.saturating_add(Duration::from_secs(until))),
        fault: Fault::RunnerWait,
        repository: Some(copy_of(repository)),
        user: None,
        call: None,
        reply_to: None,
        pull: None,
        commit: Some(commit),
        run: Some(run),
        runner: Some(runner),
    }
}

#[test]
fn one_runner_is_shared_across_repositories_and_starts_the_whole_span() {
    let mut h = harness(None, &[b"ci"]);
    let mut other = fixture(None, &[b"ci"]);
    other.id = 3;
    other.name = copy_of(b"ai/other");
    let commit = crate::repository(&mut h.domain, &h.env.limits, other);
    crate::grant(&mut h.domain, b"ai/other", ENGINE, Permission::Write);
    crate::runners(&mut h.domain, Some(1));
    dispatch(&mut h, REPOSITORY);
    dispatch(&mut h, b"ai/other");
    assert_eq!(job(&h, b"ai/other", 2, 0).wait, Some(JobWait::Runner));
    assert!(job(&h, b"ai/other", 2, 0).steps.is_empty());
    assert_eq!(run(&h, b"ai/other", 2).wait, Some(JobWait::Runner));
    assert!(h.domain.pop_fell().is_none());
    at(&mut h, 10);
    assert_eq!(job(&h, REPOSITORY, 1, 0).conclusion, Some(Conclusion::Success));
    assert_eq!(job(&h, b"ai/other", 2, 0).wait, None);
    assert_eq!(run(&h, b"ai/other", 2).wait, None);
    assert_eq!(h.domain.pop_fell(), Some(wait_fell(0, 10, 2, b"ai/other", commit, 1)));
    assert!(h.domain.pop_fell().is_none());
    at(&mut h, 19);
    assert_eq!(run(&h, b"ai/other", 2).state, RunState::Running);
    at(&mut h, 20);
    assert_eq!(run(&h, b"ai/other", 2).conclusion, Some(Conclusion::Success));
}

#[test]
fn every_job_end_frees_the_runner_in_ready_order() {
    for (lost, setup, end) in [(1000, 0, Some(Fault::Lost)), (0, 1000, Some(Fault::SetUp)), (0, 0, None)] {
        let mut h = harness(None, &[b"ci"]);
        crate::runners(&mut h.domain, Some(1));
        h.env.limits.lost = lost;
        h.env.limits.setup_faults = setup;
        for _ in 0u32..3 {
            dispatch(&mut h, REPOSITORY);
        }
        let freed = if end.is_none() {
            at(&mut h, 3);
            h.ok(MAINTAINER, write(Write::Cancel { run: 1 }));
            assert_eq!(run(&h, REPOSITORY, 1).conclusion, Some(Conclusion::Cancelled));
            3
        } else {
            at(&mut h, 10);
            let fell = h.domain.pop_fell().unwrap();
            assert_eq!(Some(fell.fault), end);
            assert_eq!((fell.runner, fell.run, fell.commit), (Some(1), Some(1), Some(FIRST)));
            10
        };
        assert_eq!(job(&h, REPOSITORY, 2, 0).wait, None);
        assert_eq!(job(&h, REPOSITORY, 3, 0).wait, Some(JobWait::Runner));
        assert_eq!(h.domain.pop_fell(), Some(wait_fell(0, freed, 2, REPOSITORY, FIRST, 1)));
        h.env.limits.lost = 0;
        h.env.limits.setup_faults = 0;
        at(&mut h, freed + 10);
        assert_eq!(run(&h, REPOSITORY, 2).conclusion, Some(Conclusion::Success));
        assert_eq!(job(&h, REPOSITORY, 3, 0).wait, None);
        assert_eq!(h.domain.pop_fell(), Some(wait_fell(0, freed + 10, 3, REPOSITORY, FIRST, 1)));
        at(&mut h, freed + 20);
        assert_eq!(run(&h, REPOSITORY, 3).conclusion, Some(Conclusion::Success));
        assert!(h.domain.pop_fell().is_none());
    }
}

#[test]
fn a_zero_pool_waits_until_disabled() {
    let mut h = harness(None, &[b"ci", b"fmt"]);
    crate::runners(&mut h.domain, Some(0));
    dispatch(&mut h, REPOSITORY);
    at(&mut h, 30);
    for index in 0..2 {
        assert_eq!(job(&h, REPOSITORY, 1, index).wait, Some(JobWait::Runner));
        assert!(job(&h, REPOSITORY, 1, index).steps.is_empty());
    }
    assert_eq!(run(&h, REPOSITORY, 1).wait, Some(JobWait::Runner));
    assert!(h.domain.pop_fell().is_none());
    crate::runners(&mut h.domain, None);
    at(&mut h, 40);
    assert_eq!(run(&h, REPOSITORY, 1).wait, None);
    at(&mut h, 50);
    assert_eq!(run(&h, REPOSITORY, 1).conclusion, Some(Conclusion::Success));
}

#[test]
fn declared_waits_labels_and_needs_precede_pool_admission() {
    let mut h = harness(Some(b"wait 3s run 4s pass"), &[b"child", b"parent"]);
    let id = h.domain.id(REPOSITORY);
    h.domain.repositories.get_mut(id).unwrap().workflows[0].requirements =
        Box::new([JobRequirements { job: copy_of(b"child"), labels: names(&[b"linux"]), needs: names(&[b"parent"]) }]);
    h.env.limits.runner_labels = Some(names(&[b"linux"]));
    crate::runners(&mut h.domain, Some(1));
    dispatch(&mut h, REPOSITORY);
    assert_eq!(job(&h, REPOSITORY, 1, 0).wait, Some(JobWait::Needs));
    assert_eq!(job(&h, REPOSITORY, 1, 1).wait, Some(JobWait::Runner));
    at(&mut h, 2);
    assert!(job(&h, REPOSITORY, 1, 1).steps.is_empty());
    at(&mut h, 3);
    assert_eq!(job(&h, REPOSITORY, 1, 1).wait, None);
    dispatch(&mut h, REPOSITORY);
    at(&mut h, 6);
    assert_eq!(job(&h, REPOSITORY, 2, 1).wait, Some(JobWait::Runner));
    at(&mut h, 7);
    assert_eq!(job(&h, REPOSITORY, 2, 1).wait, None);
    assert_eq!(h.domain.pop_fell(), Some(wait_fell(6, 7, 2, REPOSITORY, FIRST, 1)));
    assert!(h.domain.pop_fell().is_none(), "declared and needs waits do not join the pool early");
    h.env.limits.runner_labels = Some(names(&[b"other"]));
    at(&mut h, 30);
    assert_eq!(job(&h, REPOSITORY, 1, 0).wait, Some(JobWait::Runner));
    assert!(h.domain.pop_fell().is_none(), "unserved labels never take the pool");
    h.env.limits.runner_labels = Some(names(&[b"linux"]));
    at(&mut h, 33);
    assert_eq!(job(&h, REPOSITORY, 1, 0).wait, None);
    at(&mut h, 37);
    assert_eq!(run(&h, REPOSITORY, 1).conclusion, Some(Conclusion::Success));
    h.settle();
    assert_eq!(run(&h, REPOSITORY, 2).conclusion, Some(Conclusion::Success));
}

#[test]
fn ready_order_not_job_id_controls_the_pool() {
    let mut configured = fixture(Some(b"wait 3s run 4s pass"), &[b"ci"]);
    let mut delayed = configured.workflows[0].clone();
    delayed.file = copy_of(b"delayed");
    configured.workflows[0].cue = None;
    configured.workflows = Box::new([configured.workflows[0].clone(), delayed]);
    let config = Config { latency_min: Duration::ZERO, latency_max: Duration::ZERO, ..calm() };
    let mut h = Harness::with(config, configured);
    crate::runners(&mut h.domain, Some(1));
    dispatch(&mut h, REPOSITORY);
    h.ok(ENGINE, write(Write::Dispatch { workflow: copy_of(b"delayed"), branch: copy_of(MAIN), inputs: Box::new([]) }));
    at(&mut h, 1);
    dispatch(&mut h, REPOSITORY);
    at(&mut h, 3);
    at(&mut h, 10);
    assert_eq!(job(&h, REPOSITORY, 2, 0).wait, Some(JobWait::Runner));
    assert_eq!(job(&h, REPOSITORY, 3, 0).wait, None);
    assert_eq!(h.domain.pop_fell(), Some(wait_fell(1, 10, 3, REPOSITORY, FIRST, 1)));
    at(&mut h, 20);
    assert_eq!(h.domain.pop_fell(), Some(wait_fell(3, 20, 2, REPOSITORY, FIRST, 1)));
    at(&mut h, 23);
    assert_eq!(run(&h, REPOSITORY, 2).state, RunState::Running);
    at(&mut h, 24);
    assert_eq!(run(&h, REPOSITORY, 2).conclusion, Some(Conclusion::Success));
}

#[test]
fn cancelling_a_queued_job_records_its_wait_without_a_runner() {
    let mut h = harness(None, &[b"ci"]);
    crate::runners(&mut h.domain, Some(1));
    dispatch(&mut h, REPOSITORY);
    at(&mut h, 2);
    dispatch(&mut h, REPOSITORY);
    at(&mut h, 4);
    h.ok(MAINTAINER, write(Write::Cancel { run: 2 }));
    let mut expected = wait_fell(2, 4, 2, REPOSITORY, FIRST, 1);
    expected.runner = None;
    assert_eq!(h.domain.pop_fell(), Some(expected));
    assert_eq!(run(&h, REPOSITORY, 2).conclusion, Some(Conclusion::Cancelled));
    at(&mut h, 10);
    dispatch(&mut h, REPOSITORY);
    assert_eq!(job(&h, REPOSITORY, 3, 0).wait, None);
    at(&mut h, 20);
    assert_eq!(run(&h, REPOSITORY, 3).conclusion, Some(Conclusion::Success));
    assert!(h.domain.pop_fell().is_none());
}

#[test]
fn lowest_free_runner_is_taken_and_cancelled_waiters_leave_no_ghost() {
    let mut h = harness(None, &[b"ci"]);
    crate::runners(&mut h.domain, Some(2));
    for _ in 0u32..4 {
        dispatch(&mut h, REPOSITORY);
    }
    h.ok(MAINTAINER, write(Write::Cancel { run: 3 }));
    let mut cancelled = wait_fell(0, 0, 3, REPOSITORY, FIRST, 1);
    cancelled.runner = None;
    assert_eq!(h.domain.pop_fell(), Some(cancelled));
    at(&mut h, 2);
    h.ok(MAINTAINER, write(Write::Cancel { run: 2 }));
    assert_eq!(h.domain.pop_fell(), Some(wait_fell(0, 2, 4, REPOSITORY, FIRST, 2)));
    assert!(h.domain.pop_fell().is_none());
    h.settle();
    assert_eq!(run(&h, REPOSITORY, 4).conclusion, Some(Conclusion::Success));
    assert_eq!(run(&h, REPOSITORY, 3).conclusion, Some(Conclusion::Cancelled));
}

#[test]
fn no_pool_and_explicit_none_replay_the_same_draws() {
    for seed in 0..8 {
        let mut untouched = harness(None, &[b"ci", b"fmt"]);
        let mut explicit = harness(None, &[b"ci", b"fmt"]);
        untouched.domain.rng = skein_lib::Rng::new(seed);
        explicit.domain.rng = skein_lib::Rng::new(seed);
        crate::runners(&mut explicit.domain, None);
        assert_eq!(untouched.domain.rng.state(), explicit.domain.rng.state());
        for h in [&mut untouched, &mut explicit] {
            h.env.limits.lost = 250;
            h.env.limits.setup_faults = 350;
            dispatch(h, REPOSITORY);
            dispatch(h, REPOSITORY);
            h.settle();
        }
        assert_eq!(untouched.domain.image().unwrap(), explicit.domain.image().unwrap());
        assert_eq!(untouched.domain.rng.state(), explicit.domain.rng.state());
        for run in 1..=2 {
            for index in 0..2 {
                assert_eq!(job(&untouched, REPOSITORY, run, index), job(&explicit, REPOSITORY, run, index));
            }
        }
    }
}

#[test]
fn commit_status_ci_is_outside_the_pool() {
    let mut configured = fixture(None, &[b"ci"]);
    configured.checks = setup().checks;
    let mut h = Harness::with(calm(), configured);
    crate::runners(&mut h.domain, Some(0));
    let head = h.commit(FIRST, &[(b"src", b"new")]);
    h.push(ENGINE, b"work", head).unwrap();
    dispatch(&mut h, REPOSITORY);
    at(&mut h, 30);
    let Answer::Checks(checks) = h.inspect(&Read::Checks { commit: head }).unwrap() else { panic!("checks") };
    assert_eq!(checks[0].status.state, Check::Passed);
    assert_eq!(job(&h, REPOSITORY, 2, 0).wait, Some(JobWait::Runner));
}

#[test]
fn restored_silent_job_does_not_keep_the_pool_poll_timer() {
    let mut h = harness(None, &[b"ci"]);
    let id = h.domain.id(REPOSITORY);
    h.domain.repositories.get_mut(id).unwrap().workflows[0].silent = 1000;
    crate::runners(&mut h.domain, Some(0));
    dispatch(&mut h, REPOSITORY);
    at(&mut h, 2);
    let image = h.domain.image().unwrap();
    h.domain = Domain::from_image(&h.env.limits, &image).unwrap();
    assert_eq!(job(&h, REPOSITORY, 1, 0).wait, None);
    assert_eq!(run(&h, REPOSITORY, 1).state, RunState::Running);
    assert_eq!(h.domain.next_deadline(), None, "a silent job never reports");
}

#[test]
fn an_image_restores_a_pool_waiter_runnable_without_a_pool() {
    let mut h = harness(Some(b"run 4s pass"), &[b"ci"]);
    crate::runners(&mut h.domain, Some(1));
    dispatch(&mut h, REPOSITORY);
    dispatch(&mut h, REPOSITORY);
    let image = h.domain.image().unwrap();
    h.domain = Domain::from_image(&h.env.limits, &image).unwrap();
    assert_eq!(job(&h, REPOSITORY, 2, 0).wait, None);
    assert!(!job(&h, REPOSITORY, 2, 0).steps.is_empty());
    assert_eq!(run(&h, REPOSITORY, 2).wait, None);
    assert!(h.domain.pop_fell().is_none());
    at(&mut h, 3);
    assert_eq!(run(&h, REPOSITORY, 2).state, RunState::Running);
    at(&mut h, 4);
    assert_eq!(run(&h, REPOSITORY, 2).conclusion, Some(Conclusion::Success));
}
