use skein_fake_forge_forgejo::Route;
use skein_forge::api::{Answer, Error, Op, Read, Write};
use skein_forge_kit::joint::{Joint, REPO};
use skein_lib::{Duration, Time};
#[test]
fn the_component_starts_and_lists_items_through_the_fakes_bytes() {
    let mut joint = Joint::new(3);
    joint.issue(b"first", b"body");
    joint.run_for(Duration::from_secs(1));
    joint.call(1, REPO, Op::Read(Read::Items { single: false, since: Time::ZERO, page: 1, kind: None }));
    joint.run_for(Duration::from_secs(1));
    assert!(joint.answered(1).is_some(), "tally {:?}", joint.tally());
    let answered = joint.answered(1).expect("one answer");
    assert_eq!(answered.cost, 2, "repository resolved, then listed");
    let Ok(Answer::Items { items, more: false, .. }) = &answered.result else {
        panic!("one page: {:?}", answered.result)
    };
    assert_eq!(items.len(), 1);
    assert_eq!(&*items[0].title, b"first");
    assert_eq!(joint.tally().total, 4, "start two reads, then call two");
}
#[test]
fn pull_for_answers_the_open_pull_when_the_lookup_names_a_closed_one() {
    let mut joint = Joint::new(3);
    joint.push(b"work", b"change");
    let first = joint.pull(b"work", b"main");
    joint.close(first);
    let second = joint.pull(b"work", b"main");
    joint.run_for(Duration::from_secs(1));
    joint.call(
        1,
        REPO,
        Op::Read(Read::PullFor { head: Box::from(b"work".as_slice()), base: Box::from(b"main".as_slice()) }),
    );
    joint.run_for(Duration::from_secs(1));
    let answered = joint.answered(1).expect("answer");
    assert_eq!(answered.cost, 3, "resolve, oldest lookup, open page");
    let Ok(Answer::Pull(pull)) = &answered.result else { panic!("pull: {:?}", answered.result) };
    assert_eq!(pull.number, second);
    assert_eq!(joint.tally().routes.get(&Route::PullFor), Some(&1));
    assert_eq!(joint.tally().routes.get(&Route::Pulls), Some(&1));
}
#[test]
fn opening_with_nothing_to_merge_writes_no_pull_request() {
    let mut joint = Joint::new(3);
    joint.branch(b"same", 1);
    joint.run_for(Duration::from_secs(1));
    joint.call(
        1,
        REPO,
        Op::Write(Write::OpenPull {
            title: Box::from(b"empty".as_slice()),
            body: Box::from(b"body".as_slice()),
            head: Box::from(b"same".as_slice()),
            base: Box::from(b"main".as_slice()),
        }),
    );
    joint.run_for(Duration::from_secs(1));
    let answered = joint.answered(1).expect("answer");
    assert_eq!(answered.result, Err(Error::NothingToMerge));
    assert_eq!(answered.cost, 4, "resolve, two branches, compare");
    assert_eq!(joint.tally().routes.get(&Route::OpenPull), None);
    let pulls = joint
        .direct(skein_fake_forge_domain::api::Op::Read(skein_fake_forge_domain::api::Read::OpenPulls {
            head: None,
            base: None,
            page: 1,
            limit: 2,
        }))
        .expect("pulls");
    assert_eq!(pulls, skein_fake_forge_domain::api::Answer::Pulls(Box::from([])));
}

const BUILD_STEP_NAME: &[u8] = b"Run build";
const BUILD_STEP_LOG: &[u8] =
    b"failing check build 0\nfailing check build 1\nfailing check build 2\nfailing check build 3\n";
#[test]
fn a_failed_job_read_names_its_failed_step_and_reads_its_log() {
    let mut joint = Joint::failing(5);
    let pushed = joint.push(b"work", b"change");
    joint.run_for(Duration::from_secs(1));
    let head = skein_forge_kit::client::commit(pushed);
    joint.call(1, REPO, Op::Read(Read::Checks { commit: head }));
    joint.run_for(Duration::from_secs(1));
    let Some(Ok(Answer::Checks(statuses))) = joint.answered(1).map(|a| &a.result) else {
        panic!("Checks: {:?}", joint.answered(1))
    };
    let attempt = statuses[0].job.expect("The failed status links its job");
    joint.call(2, REPO, Op::Read(Read::Job { part: skein_forge::api::LogPart::End, attempt, max_bytes: 4096 }));
    joint.run_for(Duration::from_secs(1));
    let answered = joint.answered(2).expect("Job answer");
    assert_eq!(answered.cost, 3, "Run, job, the step log");
    let step = Some(skein_forge::api::Step { number: 1, name: Box::from(BUILD_STEP_NAME) });
    assert_eq!(answered.result, Ok(Answer::Job { attempt, step, log: Box::from(BUILD_STEP_LOG), truncated: false }));
}

#[test]
fn a_pull_requests_link_numbers_resolve_to_forge_ids_against_the_fake() {
    use skein_fake_forge_domain::{self as fake, api as raw};
    use skein_forge_kit::fixtures;
    let checks = raw::Checks {
        cues: Box::new([]),
        contexts: Box::new([Box::from(*b"build"), Box::from(*b"fmt")]),
        latency_min: Duration::ZERO,
        latency_max: Duration::ZERO,
        silent: 0,
        passes: 0,
        reruns: 0,
        cue: Some(raw::Cue::PullFile {
            path: Box::from(*b"changes/{pull}"),
            no_pull: raw::Check::Skipped,
            step: Box::from(*b"changes"),
            log: Box::from(*b"missing changes/{pull}"),
        }),
    };
    let mut settings = fixtures::settings();
    settings.forges.get_mut(0).unwrap().access.as_mut().unwrap().user = 1;
    let mut joint = Joint::with(
        5,
        0,
        fixtures::protocol_config(),
        fixtures::limits(),
        settings,
        fixtures::shared(),
        checks.clone(),
    );
    joint.fake_env.limits.limits.repositories = 2;
    joint.domain = fake::Domain::new(&joint.fake_env.limits, 5);
    let setup = raw::Setup {
        id: 2,
        name: Box::from(*b"org/repo"),
        default: Box::from(*b"main"),
        tree: Box::new([]),
        labels: Box::new([]),
        checks,
        workflows: Box::new([]),
        protection: None,
        hooked: false,
    };
    fake::repository(&mut joint.domain, &joint.fake_env.limits, setup.clone());
    fake::grant(&mut joint.domain, b"org/repo", 1, raw::Permission::Admin);
    fake::grant(&mut joint.domain, b"org/repo", 2, raw::Permission::Write);
    fake::repository(
        &mut joint.domain,
        &joint.fake_env.limits,
        raw::Setup { id: 3, name: Box::from(*b"org/other"), ..setup },
    );
    for n in 0..6 {
        fake::advance(&mut joint.domain, &joint.fake_env, b"org/other", b"main", b"file", &[n], 1).expect("other run");
    }
    let head = joint.push(b"work", b"change");
    joint.pull(b"work", b"main");
    joint.run_for(Duration::from_secs(1));
    joint.call(1, REPO, Op::Read(Read::Checks { commit: skein_forge_kit::client::commit(head) }));
    joint.run_for(Duration::from_secs(1));
    let Ok(Answer::Checks(checks)) = &joint.answered(1).expect("checks").result else { panic!("checks") };
    let check = checks.iter().find(|check| check.context.as_ref() == b"build").expect("build");
    assert!(check.url.ends_with(b"/actions/runs/2/jobs/0"));
    let attempt = check.job.expect("resolved attempt");
    assert_eq!((attempt.run, attempt.job, attempt.attempt), (8, 15, 1));
    joint.call(2, REPO, Op::Read(Read::Job { attempt, max_bytes: 18, part: skein_forge::api::LogPart::End }));
    joint.run_for(Duration::from_secs(1));
    assert_eq!(
        joint.answered(2).expect("log").result,
        Ok(Answer::Job {
            attempt,
            step: Some(skein_forge::api::Step { number: 1, name: Box::from(*b"changes") }),
            log: Box::from(*b"missing changes/1\n"),
            truncated: true,
        })
    );
}

#[test]
fn a_failed_jobs_log_end_comes_through_a_suffix_range() {
    let mut joint = Joint::failing(5);
    let pushed = joint.push(b"work", b"change");
    joint.run_for(Duration::from_secs(1));
    let head = skein_forge_kit::client::commit(pushed);
    joint.call(1, REPO, Op::Read(Read::Checks { commit: head }));
    joint.run_for(Duration::from_secs(1));
    let Some(Ok(Answer::Checks(statuses))) = joint.answered(1).map(|a| &a.result) else {
        panic!("Checks: {:?}", joint.answered(1))
    };
    let attempt = statuses[0].job.expect("The failed status links its job");
    joint.call(2, REPO, Op::Read(Read::Job { part: skein_forge::api::LogPart::End, attempt, max_bytes: 8 }));
    joint.run_for(Duration::from_secs(1));
    let answered = joint.answered(2).expect("Job answer");
    assert_eq!(answered.cost, 3, "Run, job, the step log");
    let step = Some(skein_forge::api::Step { number: 1, name: Box::from(BUILD_STEP_NAME) });
    assert_eq!(answered.result, Ok(Answer::Job { attempt, step, log: Box::from(&b"build 3\n"[..]), truncated: true }));
}

#[test]
fn a_start_read_reads_the_fake_logs_first_bytes() {
    let mut joint = Joint::failing(5);
    let pushed = joint.push(b"work", b"change");
    joint.run_for(Duration::from_secs(1));
    let head = skein_forge_kit::client::commit(pushed);
    joint.call(1, REPO, Op::Read(Read::Checks { commit: head }));
    joint.run_for(Duration::from_secs(1));
    let Some(Ok(Answer::Checks(statuses))) = joint.answered(1).map(|a| &a.result) else {
        panic!("Checks: {:?}", joint.answered(1))
    };
    let attempt = statuses[0].job.expect("The failed status links its job");
    joint.call(2, REPO, Op::Read(Read::Job { attempt, max_bytes: 8, part: skein_forge::api::LogPart::Start }));
    joint.run_for(Duration::from_secs(1));
    let answered = joint.answered(2).expect("Job answer");
    assert_eq!(answered.cost, 3, "Run metadata, job metadata, whole job log");
    let step = Some(skein_forge::api::Step { number: 1, name: Box::from(BUILD_STEP_NAME) });
    assert_eq!(answered.result, Ok(Answer::Job { attempt, step, log: Box::from(&b"set up b"[..]), truncated: true }));
}

#[test]
fn each_forgejo_write_completes_with_its_own_status() {
    const REVIEWER: u64 = 1; // The fixture's engine user can review the person's pull.
    const SEED: u64 = 3;
    const ISSUE_WRITES: u64 = 3; // Title edit, close and reopen share EditIssue.
    const REVIEWER_WRITES: u64 = 1; // One request and one removal.
    const BRANCH_DELETIONS: u64 = 1; // The seeded work branch is deleted once.
    const SETTLE: Duration = Duration::from_secs(1); // Fault-free calls settle before their ten-second lifetime.
    let mut joint = Joint::new(SEED);
    let issue = joint.issue(b"before", b"body");
    joint.push(b"work", b"change");
    let pull = joint.pull(b"work", b"main");
    joint.run_for(SETTLE);
    for (call, write) in [
        (1, Write::Edit { number: issue, title: Some(Box::from(b"after".as_slice())), body: None }),
        (2, Write::Close { number: issue }),
        (3, Write::Reopen { number: issue }),
        (4, Write::SetReviewers { number: pull, reviewers: Box::new([REVIEWER]) }),
        (5, Write::SetReviewers { number: pull, reviewers: Box::from([]) }),
        (6, Write::DeleteBranch { branch: Box::from(b"work".as_slice()) }),
    ] {
        joint.call(call, REPO, Op::Write(write));
        joint.run_for(SETTLE);
        let answered = joint.answered(call).expect("write completes before its lifetime");
        let Ok(Answer::Done) = &answered.result else { panic!("write {call}: {:?}", answered.result) };
    }
    assert_eq!(joint.tally().routes.get(&Route::EditIssue), Some(&ISSUE_WRITES));
    assert_eq!(joint.tally().routes.get(&Route::RequestReviewers), Some(&REVIEWER_WRITES));
    assert_eq!(joint.tally().routes.get(&Route::RemoveReviewers), Some(&REVIEWER_WRITES));
    assert_eq!(joint.tally().routes.get(&Route::DeleteBranch), Some(&BRANCH_DELETIONS));
}

fn pull_with_statuses(states: &[skein_fake_forge_domain::api::Check]) -> (Joint, u64) {
    use skein_fake_forge_domain::api as raw;
    let mut joint = Joint::new(3);
    let head = joint.push(b"work", b"change");
    let number = joint.pull(b"work", b"main");
    joint.run_for(Duration::from_secs(1));
    for (index, state) in states.iter().enumerate() {
        let context = if index == 0 { "build".to_owned() } else { format!("context-{index}") };
        joint
            .direct(raw::Op::Write(raw::Write::Status {
                commit: head,
                context: context.into_bytes().into_boxed_slice(),
                state: *state,
                description: Box::new([]),
                link: Box::new([]),
            }))
            .expect("status");
    }
    (joint, number)
}

#[test]
fn a_pulls_ci_is_pending_while_a_later_context_is_pending() {
    use skein_fake_forge_domain::api::Check;
    let (mut joint, number) = pull_with_statuses(&[Check::Passed, Check::Pending]);
    for (call, read, ci) in [
        (1, Read::Pull { number, ci: true }, skein_forge::api::Ci::Pending),
        (
            2,
            Read::PullFor { head: Box::from(b"work".as_slice()), base: Box::from(b"main".as_slice()) },
            skein_forge::api::Ci::None,
        ),
    ] {
        joint.call(call, REPO, Op::Read(read));
        joint.run_for(Duration::from_secs(1));
        let answered = joint.answered(call).expect("pull answer");
        let Ok(Answer::Pull(pull)) = &answered.result else { panic!("pull: {:?}", answered.result) };
        assert_eq!(pull.ci, ci);
    }
}

#[test]
fn a_pulls_ci_folds_every_status_page() {
    use skein_fake_forge_domain::api::Check;
    let (mut joint, number) = pull_with_statuses(&[Check::Passed, Check::Passed, Check::Failed]);
    joint.call(1, REPO, Op::Read(Read::Pull { number, ci: true }));
    joint.run_for(Duration::from_secs(1));
    let answered = joint.answered(1).expect("pull answer");
    let Ok(Answer::Pull(pull)) = &answered.result else { panic!("pull: {:?}", answered.result) };
    assert_eq!(pull.ci, skein_forge::api::Ci::Failed);
    assert_eq!(answered.cost, 4, "resolve, pull, two status pages with the failure on the last");
    assert_eq!(joint.tally().routes.get(&Route::CombinedStatus), Some(&2));
}
