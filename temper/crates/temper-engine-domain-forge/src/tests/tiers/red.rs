//! A landing's red wakes its subscribed goals (temper/docs/design/domain/after-landing.md, 7).
use super::*;
use core::fmt::Write;

const GOAL: u64 = 91;
const OTHER_GOAL: u64 = 92;
const SECOND_CHANGE: u64 = 52;
const THIRD_CHANGE: u64 = 53;
const SUBSCRIPTION: u64 = 101;
const NEXT_SUBSCRIPTION: u64 = 102;
const SECOND_JOB: u64 = 82;
const FIRST_TEST: &[u8] = b"suite.first";
const SECOND_TEST: &[u8] = b"suite.second";
const URL: &[u8] = b"https://forge.example/run/71";
const LOG: &[u8] = b"first job failed\n";
const SECOND_LOG: &[u8] = b"second job failed\n";
const JOBS: u32 = 2; // The failed jobs in the run fixture.
const SHARE: u32 = LIMITS.brief_bytes / JOBS;
pub(super) const FIRST_ATTEMPT: api::JobAttempt = api::JobAttempt { head: NEXT_TIP, run: RUN, job: JOB, attempt: 2 };
pub(super) const SECOND_ATTEMPT: api::JobAttempt =
    api::JobAttempt { head: NEXT_TIP, run: RUN, job: SECOND_JOB, attempt: 3 };

pub(super) fn failures(tests: &[&[u8]]) -> p::Failures {
    p::Failures::Tests(tests.iter().copied().map(Box::from).collect())
}
pub(super) fn pending(landings: Box<[p::Landing]>, part: p::Part, failing: p::Failures) -> Domain {
    let mut d = dispatched();
    let kept = d.tiers.get_mut(&(REPO, TIER.into())).unwrap();
    kept.baseline = None;
    kept.record.judged = Some(p::Judged { tip: BASE, key: BASE });
    let run = kept.record.run.as_mut().unwrap();
    run.tip = NEXT_TIP;
    run.key = NEXT_TIP;
    run.landings = landings;
    run.part = part;
    kept.branch = TierBranch::Absent;
    kept.result = Some(Receipt {
        report: ReceiptReport::Unset,
        base: None,
        jobs: Box::new([]),
        logs: Box::new([]),
        failures: Some(failing),
        passed: false,
        run: ReceiptRun { id: RUN, number: 17, url: URL.into() },
        candidate: NEXT_TIP,
        ended: env().wall,
    });
    kept.flight = None;
    d.tier_ops.follows.remove(&(REPO, TIER.into()));
    d
}
fn verification(d: &mut Domain, commit: api::Commit, changes: &[u64]) {
    outputs(
        d,
        Event::Restore {
            record: Stored::Verification(Verification {
                repository: REPO,
                commit,
                changes: changes.into(),
                tiers: Box::new([TierVerification { tier: TIER.into(), verdict: None, repaired: None }]),
            }),
        },
    );
}
fn subscribe(d: &mut Domain, goal: u64, number: u64, branch: &[u8], changes: &[u64]) {
    outputs(
        d,
        Event::Subscribe {
            subscription: Subscriber {
                task: goal,
                number,
                topic: Topic::Landings { repository: REPO, branch: branch.into() },
                own_change: None,
                goal_tasks: Some(changes.into()),
                paths: Box::new([]),
            },
        },
    );
}
fn one() -> Domain {
    let mut d = pending(
        Box::new([p::Landing { commit: NEXT_TIP, key: NEXT_TIP }]),
        p::Part::Rerun { failed: Box::new([FIRST_TEST.into()]) },
        failures(&[FIRST_TEST]),
    );
    verification(&mut d, NEXT_TIP, &[CHANGE, SECOND_CHANGE]);
    subscribe(&mut d, GOAL, SUBSCRIPTION, b"main", &[CHANGE, SECOND_CHANGE]);
    d
}
fn run(d: &Domain) -> api::Run {
    let mut run = workflow(d, api::Conclusion::Failure, Some(1));
    run.jobs[0].attempt = FIRST_ATTEMPT.attempt;
    let mut second = run.jobs[0].clone();
    second.id = SECOND_JOB;
    second.name = b"second".as_slice().into();
    second.attempt = SECOND_ATTEMPT.attempt;
    let mut passed = second.clone();
    passed.id = SECOND_JOB + 1;
    passed.name = b"passed".as_slice().into();
    passed.conclusion = Some(api::Conclusion::Success);
    run.jobs = Box::new([run.jobs[0].clone(), passed, second]);
    run
}
fn read(d: &Domain) -> Token {
    *d.tier_ops.reads.iter().find(|(_, key)| key.tier == (REPO, TIER.into())).expect("pending red read").0
}
pub(super) fn respond(d: &mut Domain, answer: Result<api::Answer, api::Error>) -> Box<[Request]> {
    let owner = read(d);
    let mut out = Queue::uncapped();
    crate::tiers::answered(d, &env(), owner, answer, &mut out);
    collect(out)
}
pub(super) fn log(attempt: api::JobAttempt, bytes: &[u8]) -> api::Answer {
    api::Answer::Job { attempt, step: None, log: bytes.into(), truncated: false }
}
fn logs(d: &mut Domain, second: Result<api::Answer, api::Error>) -> Box<[Request]> {
    run_answer(d);
    respond(d, Ok(log(FIRST_ATTEMPT, LOG)));
    respond(d, second)
}
pub(super) fn run_answer(d: &mut Domain) {
    let run = run(d);
    respond(d, Ok(api::Answer::Run(run)));
}
fn news(out: &[Request]) -> Vec<Request> {
    out.iter()
        .filter_map(|request| match request {
            Request::News { task, subscription, topic, news: news @ News::Red(_), class } => Some(Request::News {
                task: *task,
                subscription: *subscription,
                topic: topic.clone(),
                news: news.clone(),
                class: *class,
            }),
            _ => None,
        })
        .collect()
}
fn expected(goal: u64, subscription: u64, commit: api::Commit, failing: p::Failures, logs: Box<[JobLog]>) -> Request {
    Request::News {
        task: goal,
        subscription,
        topic: Topic::Landings { repository: REPO, branch: b"main".as_slice().into() },
        news: News::Red(RedNews {
            many: None,
            tier: TIER.into(),
            run: 1,
            url: Some(URL.into()),
            on: RedOn::Landing { commit, branch: b"main".as_slice().into(), failing },
            logs,
        }),
        class: Class::Wakes,
    }
}
fn filings(out: &[Request]) -> Vec<FileFinding> {
    out.iter()
        .filter_map(|request| match request {
            Request::FileRedFinding { task, follow, finding } => {
                assert_eq!(*task, TIER_TASK);
                assert_eq!(*follow, None);
                Some(finding.clone())
            }
            _ => None,
        })
        .collect()
}
fn expected_finding() -> FileFinding {
    FileFinding {
        project: 5,
        kind: FindingKind::Red,
        severity: Severity::High,
        area: None,
        title: b"Red after landing on main: tier browser".as_slice().into(),
        body: format!(
            "Red after landing: tier browser run 1 blames landing {} on main: 1 failing.\nsuite.first\nRun: https://forge.example/run/71\nEnd of job tests's log:\nfirst job failed\n\nEnd of job second's log:\nsecond job failed\n",
            core::str::from_utf8(&commit_text(&NEXT_TIP)).unwrap()
        ).into_bytes().into(),
    }
}
fn job_logs(second: Option<&[u8]>) -> Box<[JobLog]> {
    Box::new([
        JobLog { name: b"tests".as_slice().into(), end: Some(LOG.into()) },
        JobLog { name: b"second".as_slice().into(), end: second.map(Box::from) },
    ])
}
#[test]
fn one_landings_red_wakes_its_goal_once_with_logs() {
    let mut d = one();
    // These subscriptions match either the change or the topic, but not both.
    subscribe(&mut d, OTHER_GOAL, NEXT_SUBSCRIPTION, b"release", &[CHANGE]);
    subscribe(&mut d, OTHER_GOAL, NEXT_SUBSCRIPTION, b"main", &[THIRD_CHANGE]);
    let waiting = settled_step(&mut d, RELEASE, false);
    assert_eq!(choice(&waiting), Some(&TierDecision::Wait { until: None }));
    assert_eq!(news(&waiting).len(), 0);
    assert!(d.verifications.get(&(REPO, NEXT_TIP)).unwrap().tiers[0].verdict.is_none());
    // The failed count excludes the successful job between the two failed jobs.
    run_answer(&mut d);
    assert_eq!(d.tier_ops.reads.len(), 1);
    respond(&mut d, Ok(log(FIRST_ATTEMPT, LOG)));
    let wake = respond(&mut d, Ok(log(SECOND_ATTEMPT, SECOND_LOG)));
    assert!(wake.contains(&Request::WakeProcedure { task: TIER_TASK }));
    let out = settled_step(&mut d, RELEASE + 1, false);
    assert_eq!(
        news(&out),
        [expected(GOAL, SUBSCRIPTION, NEXT_TIP, failures(&[FIRST_TEST]), job_logs(Some(SECOND_LOG)))]
    );
    assert!(row(&d).record.run.is_none());
    assert!(out.iter().any(|request| match request {
        Request::Save { record: Stored::Verification(row), .. } => {
            row.commit == NEXT_TIP
                && row.tiers[0]
                    .verdict
                    .as_ref()
                    .is_some_and(|verdict| verdict.blame == p::Blame::Owns(failures(&[FIRST_TEST])))
        }
        _ => false,
    }));
}

#[test]
fn a_goals_landing_red_reaches_its_goal_branch_without_a_main_subscription() {
    let mut d = one();
    d.subscriptions.remove(&(GOAL, Topic::Landings { repository: REPO, branch: b"main".as_slice().into() }));
    let branch = b"temper/91/g91";
    let mut landing = fixtures::opening_row();
    landing.repository = REPO;
    landing.branch = branch.as_slice().into();
    landing.goal = Some(GOAL);
    outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(landing)) });
    // The branch topic knows the work landed into it, not the closer's landing out of it.
    subscribe(&mut d, GOAL, SUBSCRIPTION, branch, &[THIRD_CHANGE]);
    subscribe(&mut d, OTHER_GOAL, NEXT_SUBSCRIPTION, b"release", &[CHANGE]);
    settled_step(&mut d, RELEASE, false);
    logs(&mut d, Ok(log(SECOND_ATTEMPT, SECOND_LOG)));
    let out = settled_step(&mut d, RELEASE + 1, false);
    let mut wanted = expected(GOAL, SUBSCRIPTION, NEXT_TIP, failures(&[FIRST_TEST]), job_logs(Some(SECOND_LOG)));
    let Request::News { topic, .. } = &mut wanted else { panic!("expected news") };
    *topic = Topic::Landings { repository: REPO, branch: branch.as_slice().into() };
    assert_eq!(news(&out), [wanted]);
}
#[test]
fn two_red_landings_tell_each_goal_its_own_failures() {
    let mut d = pending(
        Box::new([p::Landing { commit: LAST_TIP, key: LAST_TIP }, p::Landing { commit: NEXT_TIP, key: NEXT_TIP }]),
        p::Part::Probe {
            landing: 1,
            low: 1,
            high: 1,
            waiting: Box::new([]),
            tests: p::Selection::Tests(Box::new([SECOND_TEST.into()])),
            finding: p::Finding {
                failing: failures(&[FIRST_TEST, SECOND_TEST]),
                flakes: Box::new([]),
                blamed: Box::new([p::Blame::Owns(failures(&[FIRST_TEST])), p::Blame::Cleared]),
            },
        },
        failures(&[SECOND_TEST]),
    );
    verification(&mut d, LAST_TIP, &[CHANGE]);
    verification(&mut d, NEXT_TIP, &[SECOND_CHANGE]);
    subscribe(&mut d, GOAL, SUBSCRIPTION, b"main", &[CHANGE]);
    subscribe(&mut d, OTHER_GOAL, NEXT_SUBSCRIPTION, b"main", &[SECOND_CHANGE]);
    settled_step(&mut d, RELEASE, false);
    logs(&mut d, Ok(log(SECOND_ATTEMPT, SECOND_LOG)));
    let out = settled_step(&mut d, RELEASE + 1, false);
    assert_eq!(
        news(&out),
        [
            expected(GOAL, SUBSCRIPTION, LAST_TIP, failures(&[FIRST_TEST]), job_logs(Some(SECOND_LOG))),
            expected(OTHER_GOAL, NEXT_SUBSCRIPTION, NEXT_TIP, failures(&[SECOND_TEST]), job_logs(Some(SECOND_LOG))),
        ]
    );
}
#[test]
fn inherited_failures_tell_no_goal_again() {
    let mut d = one();
    settled_step(&mut d, RELEASE, false);
    logs(&mut d, Ok(log(SECOND_ATTEMPT, SECOND_LOG)));
    settled_step(&mut d, RELEASE + 1, false);
    let receipt = row(&one()).result.clone();
    let kept = d.tiers.get_mut(&(REPO, TIER.into())).unwrap();
    kept.task = Some(NEXT_TASK);
    kept.record.run = Some(p::Run {
        number: 2,
        tip: LAST_TIP,
        key: LAST_TIP,
        landings: Box::new([p::Landing { commit: LAST_TIP, key: LAST_TIP }]),
        part: p::Part::Rerun { failed: Box::new([FIRST_TEST.into()]) },
        faults: 0,
    });
    kept.result = receipt;
    verification(&mut d, LAST_TIP, &[SECOND_CHANGE]);
    let out = settled_step(&mut d, RELEASE + 2, false);
    assert_eq!(news(&out).len(), 0);
    assert!(d.tier_ops.reads.is_empty());
    assert!(row(&d).record.run.is_none());
    assert_eq!(d.verifications.get(&(REPO, NEXT_TIP)).unwrap().tiers[0].verdict.as_ref().unwrap().run, 1);
}
#[test]
fn unavailable_and_empty_job_logs_are_named_without_ends() {
    for answer in [Err(api::Error::Missing), Ok(log(SECOND_ATTEMPT, b""))] {
        let mut d = one();
        settled_step(&mut d, RELEASE, false);
        logs(&mut d, answer);
        let out = settled_step(&mut d, RELEASE + 1, false);
        assert_eq!(news(&out), [expected(GOAL, SUBSCRIPTION, NEXT_TIP, failures(&[FIRST_TEST]), job_logs(None))]);
    }
}
#[test]
fn restart_repeats_unfinished_reads_and_tells_once() {
    let mut d = one();
    settled_step(&mut d, RELEASE, false);
    run_answer(&mut d);
    respond(&mut d, Ok(log(FIRST_ATTEMPT, LOG)));
    let mut cold = restoring(None);
    outputs(&mut cold, Event::Restore { record: Stored::Tier(row(&d).clone()) });
    for (_, verification) in &d.verifications {
        outputs(&mut cold, Event::Restore { record: Stored::Verification(verification.clone()) });
    }
    for (_, subscriber) in &d.subscriptions {
        outputs(&mut cold, Event::Restore { record: Stored::Subscription(subscriber.clone()) });
    }
    outputs(&mut cold, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    let wake = outputs(&mut cold, Event::ReadAfresh);
    assert!(wake.contains(&Request::WakeProcedure { task: TIER_TASK }));
    fresh_reads(&mut cold, BASE);
    let waiting = settled_step(&mut cold, RELEASE + 1, false);
    assert_eq!(choice(&waiting), Some(&TierDecision::Wait { until: None }));
    logs(&mut cold, Ok(log(SECOND_ATTEMPT, SECOND_LOG)));
    let out = settled_step(&mut cold, RELEASE + 2, false);
    assert_eq!(
        news(&out),
        [expected(GOAL, SUBSCRIPTION, NEXT_TIP, failures(&[FIRST_TEST]), job_logs(Some(SECOND_LOG)))]
    );
    let saved = row(&cold).clone();
    let mut again = restoring(None);
    outputs(&mut again, Event::Restore { record: Stored::Tier(saved) });
    let out = outputs(&mut again, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    assert_eq!(news(&out).len(), 0);
}

#[test]
fn failed_jobs_share_the_log_budget_and_keep_ends() {
    let mut d = one();
    settled_step(&mut d, RELEASE, false);
    run_answer(&mut d);
    let mut bytes: Vec<_> = core::iter::repeat_n(b'x', usize::try_from(SHARE).unwrap() + 1).collect();
    bytes[0] = b'y';
    respond(&mut d, Ok(log(FIRST_ATTEMPT, &bytes)));
    respond(&mut d, Ok(log(SECOND_ATTEMPT, &bytes)));
    let out = settled_step(&mut d, RELEASE + 1, false);
    let end: Box<[u8]> = Box::from(&bytes[1..]);
    assert_eq!(
        news(&out),
        [expected(
            GOAL,
            SUBSCRIPTION,
            NEXT_TIP,
            failures(&[FIRST_TEST]),
            Box::new([
                JobLog { name: b"tests".as_slice().into(), end: Some(end.clone()) },
                JobLog { name: b"second".as_slice().into(), end: Some(end) },
            ])
        )]
    );
}

fn many_report() -> Domain {
    let mut d = reports::one();
    verification(&mut d, NEXT_TIP, &[CHANGE]);
    completed(&mut d, api::Conclusion::Failure, Some(1));
    // Multiplication by 13 permutes the 37 distinct ids.
    let mut report = String::new();
    for i in 0_u32..37 {
        writeln!(report, "suite::t{:02}", i.checked_mul(13).unwrap() % 37).unwrap();
    }
    reports::report(&mut d, report.as_bytes(), false);
    d
}
fn many_words() -> Vec<u8> {
    let mut words = format!(
        "Red after landing: tier browser run 1 blames landing {} on main: many failing.\nmany; its report names:\n",
        core::str::from_utf8(&commit_text(&NEXT_TIP)).unwrap()
    );
    for i in 0_u32..16 {
        writeln!(words, "suite::t{i:02}").unwrap();
    }
    words.push_str("and 21 more\nRun: https://forge.example/run/71");
    words.into_bytes()
}
#[test]
fn a_many_report_names_its_tests_in_a_finding() {
    let mut d = many_report();
    let out = finish(&mut d);
    assert_eq!(news(&out).len(), 0);
    assert_eq!(filings(&out)[0].body.as_ref(), many_words());
}
#[test]
fn a_many_report_names_its_tests_in_goal_news() {
    let mut d = many_report();
    subscribe(&mut d, GOAL, SUBSCRIPTION, b"main", &[CHANGE]);
    let out = finish(&mut d);
    let news = news(&out);
    let Request::News { news: News::Red(red), .. } = &news[0] else { panic!("red news") };
    assert_eq!(red_news_words(red, 4096).as_ref(), many_words());
}
#[test]
fn a_new_run_drops_many_names_even_when_a_receipt_is_reused() {
    let mut d = many_report();
    finish(&mut d);
    assert!(row(&d).many.is_some());
    d.tiers.get_mut(&(REPO, TIER.into())).unwrap().pending =
        Box::new([TierLanding { commit: LAST_TIP, key: Some(NEXT_TIP) }]);
    let mut out = Queue::uncapped();
    crate::tiers::start_due(&mut d, &env(), REPO, &mut out);
    assert_eq!(row(&d).many, None);
    assert!(row(&d).received);
    assert_eq!(row(&d).result.as_ref().unwrap().failures, Some(p::Failures::Many));
}

#[test]
fn a_red_without_a_subscribed_goal_still_judges() {
    let mut d = one();
    d.subscriptions.remove(&(GOAL, Topic::Landings { repository: REPO, branch: b"main".as_slice().into() }));
    settled_step(&mut d, RELEASE, false);
    logs(&mut d, Ok(log(SECOND_ATTEMPT, SECOND_LOG)));
    let out = settled_step(&mut d, RELEASE + 1, false);
    assert_eq!(news(&out).len(), 0);
    assert!(row(&d).record.run.is_none());
    assert_eq!(filings(&out), [expected_finding()]);
    assert_eq!(
        d.verifications.get(&(REPO, NEXT_TIP)).unwrap().tiers[0].verdict,
        Some(TierVerdict { run: 1, blame: p::Blame::Owns(failures(&[FIRST_TEST])) })
    );
}

#[test]
fn a_landing_temper_did_not_make_asks_to_file_its_red() {
    let mut d = one();
    d.verifications.remove(&(REPO, NEXT_TIP));
    settled_step(&mut d, RELEASE, false);
    logs(&mut d, Ok(log(SECOND_ATTEMPT, SECOND_LOG)));
    let out = settled_step(&mut d, RELEASE + 1, false);
    assert_eq!(news(&out).len(), 0);
    assert!(row(&d).record.run.is_none());
    assert_eq!(filings(&out), [expected_finding()]);
}

#[test]
fn a_project_ending_at_landing_files_red_even_while_a_subscribed_verification_survives() {
    let mut d = one();
    d.landing.projects.get_mut(&5).unwrap().2.wait = temper_engine_domain_forge_change::Wait::Landed;
    settled_step(&mut d, RELEASE, false);
    logs(&mut d, Ok(log(SECOND_ATTEMPT, SECOND_LOG)));
    let out = settled_step(&mut d, RELEASE + 1, false);
    assert!(news(&out).is_empty(), "no goal hears a main red under landed, even before a switched row closes");
    let calls = filing::calls(&out);
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].title.as_ref(), b"1 test fails in browser since 060606060606 landed");
    assert_eq!(calls[0].signature.ids.as_ref(), [Box::from(FIRST_TEST)]);
    assert_eq!(calls[0].signature.first.as_ref(), b"first job failed");
    let write = d.prepare_verdict_filing(&LIMITS, env().wall, Filer::Task(TIER_TASK), &calls[0]).unwrap();
    let mut later = one();
    later.landing.projects.get_mut(&5).unwrap().2.wait = temper_engine_domain_forge_change::Wait::Landed;
    later.verifications.remove(&(REPO, NEXT_TIP));
    outputs(&mut later, Event::Restore { record: Stored::Finding(write.row) });
    later.tiers.get_mut(&(REPO, TIER.into())).unwrap().record.run.as_mut().unwrap().number = 2;
    let out = settled_step(&mut later, RELEASE + 2, false);
    assert!(filing::calls(&out).is_empty(), "a second red blaming the landing files none");
    assert!(later.tier_ops.reads.is_empty(), "held failures need no logs");
    assert_eq!(news(&out).len(), 0);
}

#[test]
fn a_red_reads_the_lowest_member_follow_from_the_landing_log() {
    let mut d = one();
    d.landing.projects.get_mut(&5).unwrap().2.wait = temper_engine_domain_forge_change::Wait::Landed;
    d.verifications.remove(&(REPO, NEXT_TIP));
    for (task, follow) in [(THIRD_CHANGE, 9), (SECOND_CHANGE, 4)] {
        let mut change = fixtures::opening_row();
        change.task = task;
        change.repository = REPO;
        change.base = b"main".as_slice().into();
        change.follows = Some(ChangeFollows { follow, commits: Box::new([BASE]) });
        outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(change)) });
        landing_log::landed(&mut d, env().wall, NEXT_TIP, task, Box::new([]), &mut Queue::uncapped());
    }
    settled_step(&mut d, RELEASE, false);
    logs(&mut d, Ok(log(SECOND_ATTEMPT, SECOND_LOG)));
    let out = settled_step(&mut d, RELEASE + 1, false);
    assert_eq!(news(&out).len(), 0);
    let filings: Vec<_> = out
        .iter()
        .filter_map(|request| match request {
            Request::FileVerdict { filing, .. } => Some(filing),
            _ => None,
        })
        .collect();
    assert_eq!(filings.len(), 1);
    assert_eq!(filings[0].follow, Some(4));
    assert_eq!(filings[0].title.as_ref(), b"1 test fails in browser since 060606060606 landed");
}

#[test]
fn a_red_names_the_lowest_kept_member_follow() {
    let mut d = one();
    d.subscriptions.remove(&(GOAL, Topic::Landings { repository: REPO, branch: b"main".as_slice().into() }));
    d.changes.remove(&CHANGE);
    // Deliberately not in member order; CHANGE's row has already been forgotten.
    verification(&mut d, NEXT_TIP, &[THIRD_CHANGE, CHANGE, SECOND_CHANGE, 1]);
    let mut ordinary = fixtures::opening_row();
    ordinary.task = 1;
    ordinary.follows = None;
    outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(ordinary)) });
    for (task, follow) in [(THIRD_CHANGE, 9), (SECOND_CHANGE, 4)] {
        let mut change = fixtures::opening_row();
        change.task = task;
        change.follows = Some(ChangeFollows { follow, commits: Box::new([BASE]) });
        outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(change)) });
    }
    settled_step(&mut d, RELEASE, false);
    logs(&mut d, Ok(log(SECOND_ATTEMPT, SECOND_LOG)));
    let out = settled_step(&mut d, RELEASE + 1, false);
    let filings: Vec<_> = out
        .iter()
        .filter_map(|request| match request {
            Request::FileRedFinding { follow, finding, .. } => Some((*follow, finding)),
            _ => None,
        })
        .collect();
    assert_eq!(filings, [(Some(4), &expected_finding())]);
}

#[test]
fn a_filed_red_is_not_filed_again_in_a_later_run() {
    let mut d = one();
    d.verifications.remove(&(REPO, NEXT_TIP));
    settled_step(&mut d, RELEASE, false);
    logs(&mut d, Ok(log(SECOND_ATTEMPT, SECOND_LOG)));
    let out = settled_step(&mut d, RELEASE + 1, false);
    let calls = filings(&out);
    let [call] = calls.as_slice() else { panic!("one red filing") };
    let write = d.prepare_file_finding(&LIMITS, env().wall, Filer::Task(TIER_TASK), call.clone()).unwrap();
    let saved = Stored::Finding(write.row);
    // A restart keeps the finding, not the disposed run or any Verification of temper's.
    let mut d = one();
    d.verifications.remove(&(REPO, NEXT_TIP));
    outputs(&mut d, Event::Restore { record: saved });
    d.tiers.get_mut(&(REPO, TIER.into())).unwrap().record.run.as_mut().unwrap().number = 2;
    settled_step(&mut d, RELEASE + 2, false);
    logs(&mut d, Ok(log(SECOND_ATTEMPT, SECOND_LOG)));
    let out = settled_step(&mut d, RELEASE + 3, false);
    assert_eq!(filings(&out).len(), 0);
    assert!(row(&d).record.run.is_none());
}

#[test]
fn a_landing_that_reaches_a_goal_files_no_finding() {
    let mut d = one();
    settled_step(&mut d, RELEASE, false);
    logs(&mut d, Ok(log(SECOND_ATTEMPT, SECOND_LOG)));
    let out = settled_step(&mut d, RELEASE + 1, false);
    assert_eq!(news(&out).len(), 1);
    assert_eq!(filings(&out).len(), 0);
}

#[test]
fn an_already_red_landing_needs_no_reads_or_news() {
    let mut d = one();
    d.tiers.get_mut(&(REPO, TIER.into())).unwrap().record.run.as_mut().unwrap().number = 2;
    d.verifications.get_mut(&(REPO, NEXT_TIP)).unwrap().tiers[0].verdict =
        Some(TierVerdict { run: 1, blame: p::Blame::Suspect(failures(&[FIRST_TEST])) });
    let out = settled_step(&mut d, RELEASE, false);
    assert_eq!(news(&out).len(), 0);
    assert!(d.tier_ops.reads.is_empty());
    assert!(row(&d).record.run.is_none());
}

#[test]
fn a_run_read_failure_keeps_the_red_without_logs() {
    let mut d = one();
    settled_step(&mut d, RELEASE, false);
    respond(&mut d, Err(api::Error::Missing));
    let out = settled_step(&mut d, RELEASE + 1, false);
    assert_eq!(news(&out), [expected(GOAL, SUBSCRIPTION, NEXT_TIP, failures(&[FIRST_TEST]), Box::new([]))]);
}

#[test]
fn cancelling_during_log_reads_discards_the_pending_news() {
    let mut d = one();
    settled_step(&mut d, RELEASE, false);
    let owner = read(&d);
    let run = run(&d);
    let out = close_tier(&mut d, TIER_TASK, ReleaseEnding::Cancelled);
    let mut late = Queue::uncapped();
    crate::tiers::answered(&mut d, &env(), owner, Ok(api::Answer::Run(run)), &mut late);
    assert_eq!(news(&out).len(), 0);
    assert!(collect(late).is_empty());
    assert!(row(&d).task.is_none());
    assert_eq!(row(&d).record.sequence, 1);
}
