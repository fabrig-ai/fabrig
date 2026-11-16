//! Context cues through the same CI and Actions reads as a client.
use super::*;
use crate::api::{CheckSummary, Conclusion, ContextCue, JobWait, RunState, StepStatus};

fn configured() -> (Config, Setup) {
    let config = Config { limits: Limits { name_bytes: 64, content_bytes: 128, ..LIMITS }, ..calm() };
    let mut setup = setup();
    setup.checks.contexts = names(&[b"check", b"changes"]);
    setup.checks.cues = Box::new([ContextCue {
        context: copy_of(b"changes"),
        cue: Cue::PullFile {
            path: copy_of(b"changes/{pull}.md"),
            no_pull: Check::Skipped,
            step: copy_of(b"changes"),
            log: copy_of(b"Missing changes/{pull}.md"),
        },
    }]);
    (config, setup)
}

fn assert_contexts(h: &mut Harness, pull: u64, check: Check, changes: Check) {
    let pull = h.pull(pull);
    for (name, expected) in [(b"check".as_slice(), check), (b"changes".as_slice(), changes)] {
        let mut found = false;
        for status in &pull.statuses {
            if status.context.as_ref() == name {
                assert_eq!(status.state, expected, "{name:?}");
                found = true;
            }
        }
        assert!(found, "named context {name:?}");
    }
}

fn named<'a>(rows: &'a [CheckSummary], context: &[u8]) -> &'a CheckSummary {
    for row in rows {
        if row.status.context.as_ref() == context {
            return row;
        }
    }
    panic!("named context {context:?}");
}

#[test]
fn contexts_choose_their_own_cue_and_inherit_the_repository_cue() {
    let mut configured = setup();
    configured.checks.contexts = names(&[b"check", b"changes"]);
    configured.checks.cue = Some(Cue::File { path: copy_of(b"ci"), green: copy_of(b"green") });
    configured.checks.cues =
        Box::new([ContextCue { context: copy_of(b"check"), cue: Cue::Message { fail: copy_of(b"FAIL") } }]);
    let mut h = Harness::with(calm(), configured);
    let head = h.commit(FIRST, &[(b"ci", b"red")]);
    h.push(ENGINE, b"work", head).unwrap();
    h.open(b"work").unwrap();
    h.settle();
    assert_contexts(&mut h, 1, Check::Passed, Check::Failed);
}

#[test]
fn a_file_cue_skips_a_push_then_judges_the_pull_on_the_same_head() {
    let (config, setup) = configured();
    let mut h = Harness::with(config, setup);
    let head = h.commit(FIRST, &[(b"src", b"one")]);
    h.push(ENGINE, b"work", head).unwrap();
    h.settle();
    let Answer::Checks(statuses) = h.ok(PERSON, read(Read::Checks { commit: head })) else { unreachable!("checks") };
    let check = named(&statuses, b"check");
    let changes = named(&statuses, b"changes");
    assert_eq!(check.status.state, Check::Passed, "an uncued context passes");
    assert_eq!(changes.status.state, Check::Skipped);
    assert_eq!(&*changes.description, b"CI skipped");
    assert_eq!(h.open(b"work"), Ok(Answer::Created(1)));
    h.settle();
    assert_contexts(&mut h, 1, Check::Passed, Check::Failed);
}

#[test]
fn a_file_cue_substitutes_the_pull_number_and_needs_only_file_presence() {
    let (config, setup) = configured();
    let mut h = Harness::with(config, setup);
    h.ok(ENGINE, create(b"issue", b"body", &[]));
    let wrong = h.commit(FIRST, &[(b"changes/1.md", b"not this pull")]);
    h.push(ENGINE, b"work", wrong).unwrap();
    assert_eq!(h.open(b"work"), Ok(Answer::Created(2)));
    h.settle();
    assert_contexts(&mut h, 2, Check::Passed, Check::Failed);
    let right = h.commit(wrong, &[(b"changes/2.md", b"")]);
    h.push(ENGINE, b"work", right).unwrap();
    h.settle();
    assert_contexts(&mut h, 2, Check::Passed, Check::Passed);
}

#[test]
fn a_cued_failure_is_read_back_through_actions_job_step_and_log() {
    let (config, setup) = configured();
    let mut h = Harness::with(config, setup);
    h.ok(ENGINE, create(b"issue", b"body", &[]));
    let head = h.commit(FIRST, &[(b"src", b"one")]);
    h.push(ENGINE, b"work", head).unwrap();
    h.open(b"work").unwrap();
    h.settle();
    let Answer::Checks(statuses) = h.ok(PERSON, read(Read::Checks { commit: head })) else { unreachable!("checks") };
    let reference = named(&statuses, b"changes").job.unwrap();
    let Answer::Jobs(jobs) = h.ok(PERSON, read(Read::Jobs { run: reference.run })) else { unreachable!("jobs") };
    assert_eq!(jobs[1].id, reference.job);
    let Answer::Job { job, steps } = h.ok(PERSON, read(Read::Job { job: reference.job })) else { unreachable!("job") };
    assert_eq!(job.attempt, reference.attempt);
    assert_eq!(&*steps[1].name, b"changes");
    assert_eq!(steps[1].status, StepStatus::Failure);
    assert!(steps[1].log.ends_with(b"Missing changes/2.md\n"));
    assert_eq!(
        h.ok(PERSON, read(Read::Log { job: job.id, attempt: job.attempt, step: Some(1) })),
        Answer::File(steps[1].log.clone())
    );
    assert_eq!(
        h.call(PERSON, read(Read::Log { job: job.id, attempt: 99, step: Some(1) })),
        Err(Error::Missing(What::Job))
    );
}

#[test]
fn templates_expand_every_occurrence_and_price_twenty_digit_numbers() {
    assert_eq!(&*crate::ci::template(b"{pull}/{pull}", Some(u64::MAX)), b"18446744073709551615/18446744073709551615");
    assert_eq!(crate::ci::template_bound(b"{pull}/{pull}"), 41);
    let (config, mut setup) = configured();
    setup.checks.cues = Box::new([ContextCue {
        context: copy_of(b"changes"),
        cue: Cue::PullFile {
            path: copy_of(b"changes/{pull}.md"),
            no_pull: Check::Skipped,
            step: copy_of(b"changes"),
            log: copy_of(b"Missing changes/{pull}.md"),
        },
    }]);
    let config = Config { limits: Limits { name_bytes: 31, ..config.limits }, ..config };
    let _h = Harness::with(config, setup);
}

fn declared() -> (Config, Setup) {
    let mut config = calm();
    config.limits.name_bytes = 64;
    config.limits.content_bytes = 128;
    config.limits.items = 8;
    let mut setup = setup();
    setup.tree = files(&[]);
    setup.checks.contexts = names(&[b"check"]);
    setup.checks.cue = Some(Cue::Declared { path: copy_of(b".fake-forge/ci") });
    setup.checks.passes = 0;
    (config, setup)
}

#[test]
fn declared_rules_judge_the_heads_first_line_and_expose_unreadable_logs() {
    let oversized = skein_lib::bytes::concat(&[b"pass\n", &[b'x'; 128]]);
    for (rule, expected, unreadable) in [
        (None, Check::Passed, false),
        (Some(b"fail\n".as_slice()), Check::Failed, false),
        (Some(b"pass".as_slice()), Check::Passed, false),
        (Some(b"pass\nfail\n".as_slice()), Check::Passed, false),
        (Some(oversized.as_ref()), Check::Failed, true),
        (Some(b"maybe".as_slice()), Check::Failed, true),
        (Some(b"wait 40 pass".as_slice()), Check::Failed, true),
        (Some(b"wait 25h pass".as_slice()), Check::Failed, true),
        (Some(b"wait 123456s pass".as_slice()), Check::Failed, true),
        (Some(b"run 5m wait 40m pass".as_slice()), Check::Failed, true),
        (Some(b"wait 40m".as_slice()), Check::Failed, true),
        (Some(b"wait  40m pass".as_slice()), Check::Failed, true),
        (Some(b"pull-file ".as_slice()), Check::Failed, true),
        (Some(b"pull-file {pull}{pull}{pull}{pull}".as_slice()), Check::Failed, true),
    ] {
        let (mut config, setup) = declared();
        config.limits.content_bytes = 256;
        let mut h = Harness::with(config, setup);
        let changes = match rule {
            Some(rule) => files(&[(b".fake-forge/ci", rule)]),
            None => files(&[(b"src", b"one")]),
        };
        let head = crate::commit(&mut h.domain, &h.env.limits, FIRST, changes, b"").unwrap().unwrap();
        h.env.limits.limits.content_bytes = 128;
        h.push(ENGINE, b"work", head).unwrap();
        h.settle();
        let Answer::Statuses(statuses) = h.ok(PERSON, read(Read::Statuses { commit: head })) else {
            panic!("statuses")
        };
        assert_eq!(statuses[0].state, expected, "{rule:?}");
        let Answer::Runs(runs) =
            h.ok(PERSON, read(Read::Runs { workflow: None, commit: Some(head), number: None, page: 1, limit: 2 }))
        else {
            panic!("runs")
        };
        let Answer::Jobs(jobs) = h.ok(PERSON, read(Read::Jobs { run: runs[0].id })) else { panic!("jobs") };
        let Answer::Job { job, steps } = h.ok(PERSON, read(Read::Job { job: jobs[0].id })) else { panic!("job") };
        assert_eq!(steps, jobs[0].steps);
        assert_eq!(steps[1].status, if expected == Check::Failed { StepStatus::Failure } else { StepStatus::Success });
        let log = h.ok(PERSON, read(Read::Log { job: job.id, attempt: job.attempt, step: Some(1) }));
        assert_eq!(log, Answer::File(steps[1].log.clone()));
        if unreadable {
            assert!(steps[1].log.ends_with(b"unreadable .fake-forge/ci\n"));
        }
    }
}

#[test]
fn declared_pull_file_rebinds_a_pushed_head_to_pull_seven() {
    let (config, setup) = declared();
    let mut h = Harness::with(config, setup);
    for _ in 0_u32..6 {
        h.ok(ENGINE, create(b"issue", b"body", &[]));
    }
    let head =
        h.commit(FIRST, &[(b".fake-forge/ci", b"pull-file changes/{pull}.md\n"), (b"src/main.rs", b"fn main() {}")]);
    h.push(ENGINE, b"work", head).unwrap();
    h.settle();
    let Answer::Statuses(statuses) = h.ok(PERSON, read(Read::Statuses { commit: head })) else { panic!("statuses") };
    assert_eq!(statuses[0].state, Check::Skipped);
    assert_eq!(h.open(b"work"), Ok(Answer::Created(7)));
    h.settle();
    let Answer::Checks(checks) = h.ok(PERSON, read(Read::Checks { commit: head })) else { panic!("checks") };
    assert_eq!(checks[0].status.state, Check::Failed);
    let reference = checks[0].job.unwrap();
    let Answer::Job { job, steps } = h.ok(PERSON, read(Read::Job { job: reference.job })) else { panic!("job") };
    assert_eq!(steps[1].name.as_ref(), b"check");
    assert_eq!(steps[1].status, StepStatus::Failure);
    assert!(steps[1].log.ends_with(b"Missing changes/7.md\n"));
    assert_eq!(
        h.ok(PERSON, read(Read::Log { job: job.id, attempt: job.attempt, step: Some(1) })),
        Answer::File(steps[1].log.clone())
    );
    let right = h.commit(head, &[(b"changes/7.md", b"")]);
    h.push(ENGINE, b"work", right).unwrap();
    h.settle();
    assert_eq!(h.pull(7).statuses[0].state, Check::Passed);
}

#[test]
fn made_repositories_keep_workflows_and_merge_settings_across_an_image() {
    for enabled in [false, true] {
        for mode in [0_u32, 1, 2] {
            let mut config = calm();
            if enabled {
                config.made_workflows = Box::new([crate::api::Workflow {
                    file: copy_of(b"gate.yml"),
                    jobs: names(&[b"gate"]),
                    latency_min: Duration::ZERO,
                    latency_max: Duration::ZERO,
                    silent: 0,
                    passes: 1000,
                    cue: None,
                    fails: Box::new([]),
                    flaky: Box::new([]),
                    report: None,
                    select: None,
                    requirements: Box::new([]),
                }]);
                config.made_manual_merges =
                    crate::api::ManualMerges { allow_manual_merge: true, autodetect_manual_merge: true };
            }
            let mut h = Harness::new(config.clone());
            let name = b"org/made";
            match mode {
                0 => {
                    crate::create_repository(&mut h.domain, &config, 3, name, b"made", ENGINE).unwrap();
                }
                1 => {
                    crate::create_empty_repository(&mut h.domain, &config, name, ENGINE).unwrap();
                }
                2 => {
                    h.ok(
                        ENGINE,
                        write(Write::Copy {
                            owner: copy_of(b"org"),
                            name: copy_of(b"made"),
                            fill: crate::api::Fill::Whole,
                        }),
                    );
                    h.settle();
                }
                _ => unreachable!("three creation paths"),
            }
            for restored in [false, true] {
                if restored {
                    let image = h.domain.image().unwrap();
                    let mut config = config.clone();
                    config.made_workflows = Box::new([]);
                    config.made_manual_merges = crate::api::ManualMerges::OFF;
                    h.domain = Domain::from_image(&config, &image).unwrap();
                }
                let repository = h.domain.repositories.get(h.domain.id(name)).unwrap();
                assert_eq!(repository.workflows, config.made_workflows, "mode {mode}, restored {restored}");
                assert_eq!(repository.manual_merges, config.made_manual_merges, "mode {mode}, restored {restored}");
            }
        }
    }
}

#[test]
fn made_repositories_keep_their_checks_across_an_image() {
    for checked in [false, true] {
        for mode in [0_u32, 1, 2] {
            let (mut config, setup) = declared();
            if checked {
                config.made_checks = setup.checks.clone();
            }
            let mut h = Harness::with(config.clone(), setup);
            let name = b"org/made";
            match mode {
                0 => {
                    crate::create_repository(&mut h.domain, &config, 3, name, b"made", ENGINE).unwrap();
                }
                1 => {
                    crate::create_empty_repository(&mut h.domain, &config, name, ENGINE).unwrap();
                }
                2 => {
                    h.ok(
                        ENGINE,
                        write(Write::Copy {
                            owner: copy_of(b"org"),
                            name: copy_of(b"made"),
                            fill: crate::api::Fill::Whole,
                        }),
                    );
                    h.settle();
                }
                _ => unreachable!("three creation paths"),
            }
            let mut parent = FIRST;
            for restored in [false, true] {
                if restored {
                    // The image keeps the repository's checks, not the new creation default.
                    let image = h.domain.image().unwrap();
                    let mut restore_config = config.clone();
                    restore_config.made_checks = calm().made_checks;
                    h.domain = Domain::from_image(&restore_config, &image).unwrap();
                }
                let rule = if restored { b"pass".as_slice() } else { b"fail\n" };
                let head = h.commit(parent, &[(b".fake-forge/ci", rule)]);
                assert_eq!(
                    h.call_on(
                        name,
                        ENGINE,
                        Op::Git(Git::Push { branch: copy_of(b"work"), commit: head, expected: None })
                    ),
                    Ok(Answer::Pushed(Pushed::Pushed))
                );
                parent = head;
                h.settle();
                let Answer::Statuses(statuses) =
                    h.call_on(name, ENGINE, read(Read::Statuses { commit: head })).unwrap()
                else {
                    panic!("statuses")
                };
                assert_eq!(statuses.len(), usize::from(checked));
                if checked {
                    assert_eq!(statuses[0].context.as_ref(), b"check");
                    assert_eq!(statuses[0].state, if restored { Check::Passed } else { Check::Failed });
                }
            }
        }
    }
}

fn phased(rule: &[u8]) -> (Harness, u64) {
    let (mut config, setup) = declared();
    config.latency_min = Duration::ZERO;
    config.latency_max = Duration::ZERO;
    let mut h = Harness::with(config, setup);
    let head = h.commit(FIRST, &[(b".fake-forge/ci", rule)]);
    h.push(ENGINE, b"work", head).unwrap();
    (h, head)
}

fn at(seconds: u64) -> Time {
    Time::from_nanos(Duration::from_secs(seconds).as_nanos())
}

fn phase_read(h: &Harness, head: u64, wait: Option<JobWait>, state: RunState, verdict: Check, attempt: u32) {
    let Answer::Checks(checks) = h.inspect(&Read::Checks { commit: head }).unwrap() else { panic!("checks") };
    let check = &checks[0];
    assert_eq!(check.status.state, verdict);
    let Answer::Runs(runs) =
        h.inspect(&Read::Runs { workflow: None, commit: Some(head), number: None, page: 1, limit: 2 }).unwrap()
    else {
        panic!("runs")
    };
    let Answer::Jobs(jobs) = h.inspect(&Read::Jobs { run: runs[0].id }).unwrap() else { panic!("jobs") };
    let reference = JobRef { run: runs[0].id, job: jobs[0].id, attempt: jobs[0].attempt };
    assert_eq!(reference.attempt, attempt);
    if verdict == Check::Pending {
        assert_eq!(check.job, Some(reference));
        assert!(!check.link.is_empty());
    }
    let Answer::Run(run) = h.inspect(&Read::Run { run: reference.run }).unwrap() else { panic!("run") };
    assert_eq!((run.wait, run.state), (wait, state));
    let Answer::Job { job, steps } = h.inspect(&Read::Job { job: reference.job }).unwrap() else { panic!("job") };
    assert_eq!(job, jobs[0]);
    assert_eq!(job.steps, steps);
    assert_eq!((job.wait, job.state), (wait, state));
    if wait.is_some() {
        assert!(steps.is_empty());
    } else if state == RunState::Running {
        assert_eq!(steps.len(), 2);
        assert_eq!((steps[0].name.as_ref(), steps[0].status), (b"Set up job".as_slice(), StepStatus::Success));
        assert_eq!((steps[1].name.as_ref(), steps[1].status), (b"Run check".as_slice(), StepStatus::Running));
    } else {
        let conclusion = Some(if verdict == Check::Passed { Conclusion::Success } else { Conclusion::Failure });
        assert_eq!(job.conclusion, conclusion);
        assert_eq!(run.conclusion, conclusion);
    }
}

#[test]
fn a_declared_head_waits_then_runs_then_reports_its_own_spans() {
    let (mut h, head) = phased(b"wait 40m run 5m pass");
    phase_read(&h, head, Some(JobWait::Runner), RunState::Running, Check::Pending, 1);
    h.until(at(2399));
    phase_read(&h, head, Some(JobWait::Runner), RunState::Running, Check::Pending, 1);
    h.until(at(2400));
    phase_read(&h, head, None, RunState::Running, Check::Pending, 1);
    h.until(at(2699));
    phase_read(&h, head, None, RunState::Running, Check::Pending, 1);
    h.until(at(2700));
    phase_read(&h, head, None, RunState::Completed, Check::Passed, 1);
}

#[test]
fn optional_phases_use_the_drawn_latency_or_start_running_at_once() {
    for (rule, waiting, done, verdict) in [
        (b"wait 2m pass".as_slice(), 120, 125, Check::Passed),
        (b"run 2m fail".as_slice(), 0, 120, Check::Failed),
        (b"wait 0s run 0m pass".as_slice(), 0, 0, Check::Passed),
        (b"run 00090s fail".as_slice(), 0, 90, Check::Failed),
        (b"run 24h pass".as_slice(), 0, 86400, Check::Passed),
    ] {
        let (mut h, head) = phased(rule);
        if waiting > 0 {
            phase_read(&h, head, Some(JobWait::Runner), RunState::Running, Check::Pending, 1);
        }
        h.until(at(waiting));
        if done > waiting {
            phase_read(&h, head, None, RunState::Running, Check::Pending, 1);
            h.until(at(done - 1));
            phase_read(&h, head, None, RunState::Running, Check::Pending, 1);
        }
        h.until(at(done));
        phase_read(&h, head, None, RunState::Completed, verdict, 1);
    }
}

#[test]
fn a_run_waits_only_when_all_its_unfinished_contexts_wait() {
    let (mut config, mut setup) = declared();
    config.latency_min = Duration::ZERO;
    config.latency_max = Duration::ZERO;
    setup.checks.contexts = names(&[b"check", b"changes"]);
    setup.checks.cues =
        Box::new([ContextCue { context: copy_of(b"changes"), cue: Cue::Declared { path: copy_of(b"other") } }]);
    let mut h = Harness::with(config, setup);
    let head = h.commit(FIRST, &[(b".fake-forge/ci", b"wait 1m run 1m pass"), (b"other", b"wait 3m run 1m pass")]);
    h.push(ENGINE, b"work", head).unwrap();
    for (seconds, wait) in [(0, Some(JobWait::Runner)), (60, None), (120, Some(JobWait::Runner)), (180, None)] {
        h.until(at(seconds));
        let Answer::Run(run) = h.inspect(&Read::Run { run: 1 }).unwrap() else { panic!("run") };
        assert_eq!(run.wait, wait, "at {seconds}");
        let Answer::Jobs(jobs) = h.inspect(&Read::Jobs { run: 1 }).unwrap() else { panic!("jobs") };
        assert_eq!(jobs[1].wait, if seconds < 180 { Some(JobWait::Runner) } else { None });
        assert_eq!(jobs[0].state, if seconds < 120 { RunState::Running } else { RunState::Completed });
    }
    h.until(at(240));
    let Answer::Run(run) = h.inspect(&Read::Run { run: 1 }).unwrap() else { panic!("run") };
    assert_eq!((run.wait, run.conclusion), (None, Some(Conclusion::Success)));
}

#[test]
fn a_declared_contexts_rerun_waits_afresh_at_attempt_two() {
    let (mut config, mut setup) = declared();
    config.latency_min = Duration::ZERO;
    config.latency_max = Duration::ZERO;
    setup.checks.reruns = 1000;
    let mut h = Harness::with(config, setup);
    let head = h.commit(FIRST, &[(b".fake-forge/ci", b"wait 2m run 1m pass")]);
    h.push(ENGINE, b"work", head).unwrap();
    h.until(at(180));
    phase_read(&h, head, None, RunState::Completed, Check::Passed, 1);
    h.until(at(185));
    phase_read(&h, head, Some(JobWait::Runner), RunState::Running, Check::Pending, 2);
    h.until(at(304));
    phase_read(&h, head, Some(JobWait::Runner), RunState::Running, Check::Pending, 2);
    h.until(at(305));
    phase_read(&h, head, None, RunState::Running, Check::Pending, 2);
    h.until(at(365));
    phase_read(&h, head, None, RunState::Completed, Check::Passed, 2);
}

#[test]
fn a_pull_bindings_new_declared_run_waits_afresh() {
    let (mut h, head) = phased(b"wait 2m run 1m pass");
    h.until(at(130));
    h.open(b"work").unwrap();
    phase_read(&h, head, Some(JobWait::Runner), RunState::Running, Check::Pending, 1);
    let Answer::Run(run) = h.inspect(&Read::Run { run: 2 }).unwrap() else { panic!("run") };
    assert_eq!(run.event, crate::api::RunEvent::PullRequest);
    h.until(at(249));
    phase_read(&h, head, Some(JobWait::Runner), RunState::Running, Check::Pending, 1);
    h.until(at(250));
    phase_read(&h, head, None, RunState::Running, Check::Pending, 1);
    h.until(at(310));
    phase_read(&h, head, None, RunState::Completed, Check::Passed, 1);
}

#[test]
fn a_silent_declared_job_starts_but_never_reports() {
    let (mut config, mut setup) = declared();
    config.latency_min = Duration::ZERO;
    config.latency_max = Duration::ZERO;
    setup.checks.silent = 1000;
    let mut h = Harness::with(config, setup);
    let head = h.commit(FIRST, &[(b".fake-forge/ci", b"wait 2m run 1m pass")]);
    h.push(ENGINE, b"work", head).unwrap();
    h.until(at(180));
    phase_read(&h, head, None, RunState::Running, Check::Pending, 1);
    h.settle();
    phase_read(&h, head, None, RunState::Running, Check::Pending, 1);
}
