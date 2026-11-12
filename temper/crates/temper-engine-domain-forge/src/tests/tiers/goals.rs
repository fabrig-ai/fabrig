//! Goal branches run independently and tell their own reds (temper/docs/design/domain/after-landing.md, 3 and 7).
use super::*;

const GOAL: u64 = 1;
const OTHER: u64 = 2;
const BRANCH: &[u8] = b"temper/1/g1";
const OTHER_BRANCH: &[u8] = b"temper/1/g2";
const TEST: &[u8] = b"suite.first";
const FLAKE: &[u8] = b"suite.flake";
const SUBSCRIPTION: u64 = 101;
const LOG: &[u8] = b"failed again\n";
const URL: &[u8] = b"https://forge.example/run/71";
const SPAN: Duration = Duration::from_secs(1200);

fn name(branch: &[u8]) -> Name {
    Name {
        forge: REPO.forge,
        repository: REPO.repository,
        what: What::Branch(branch.split(|byte| *byte == b'/').map(Box::from).collect()),
    }
}
fn hold(d: &mut Domain, goal: u64, branch: &[u8]) {
    outputs(
        d,
        Event::Restore {
            record: Stored::Hold(Hold {
                design_writers: Box::new([]),
                name: name(branch),
                task: goal,
                writer: None,
                drift: None,
            }),
        },
    );
}
fn span(d: &mut Domain, span: Option<Duration>) {
    d.landing.projects.get_mut(&5).unwrap().1[0].tiers[0].span = span;
}
fn configured(span_value: Option<Duration>) -> Domain {
    let mut d = restoring_following(Follows::Goals);
    span(&mut d, span_value);
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    hold(&mut d, GOAL, BRANCH);
    moved(&mut d, &env(), BRANCH, None, BASE);
    d
}
fn moved(
    d: &mut Domain,
    env: &Env<Limits>,
    branch: &[u8],
    before: Option<api::Commit>,
    after: api::Commit,
) -> Box<[Request]> {
    let name = name(branch);
    d.heads.insert(name.clone(), BranchHead { name, commit: after, owned: true }).unwrap();
    let mut out = Queue::uncapped();
    crate::tiers::observed(d, env, REPO, branch, before, after, &mut out);
    collect(out)
}
fn later() -> Env<Limits> {
    Env { now: env().now.saturating_add(SPAN), ..env() }
}
fn fire(d: &mut Domain, env: &Env<Limits>) -> Box<[Request]> {
    let mut out = Queue::uncapped();
    crate::tiers::fire(d, env, &mut out);
    collect(out)
}
fn start(d: &mut Domain, tip: api::Commit) {
    let number = row(d).record.run.as_ref().unwrap().number;
    attach(d, number, TIER_TASK);
    tier_step(d, OPEN, false);
    made(d, OPEN, skein_forge_client::Made::Branch(tip));
    tier_step(d, DISPATCH, false);
    made(d, DISPATCH, skein_forge_client::Made::Set);
}
fn dispatched() -> Domain {
    let mut d = configured(None);
    moved(&mut d, &env(), BRANCH, Some(BASE), NEXT_TIP);
    start(&mut d, NEXT_TIP);
    d
}
fn failures(tests: &[&[u8]]) -> p::Failures {
    p::Failures::Tests(tests.iter().copied().map(Box::from).collect())
}
fn subscribe(d: &mut Domain, goal: u64, branch: &[u8]) {
    outputs(
        d,
        Event::Subscribe {
            subscription: Subscriber {
                task: goal,
                number: SUBSCRIPTION,
                topic: Topic::Landings { repository: REPO, branch: branch.into() },
                own_change: None,
                goal_tasks: Some(Box::new([])),
                paths: Box::new([]),
            },
        },
    );
}
fn news(out: &[Request]) -> Vec<(&RedNews, Class)> {
    out.iter()
        .filter_map(|request| match request {
            Request::News {
                task: GOAL,
                subscription: SUBSCRIPTION,
                topic: Topic::Landings { repository: REPO, branch },
                news: News::Red(red),
                class,
            } if branch.as_ref() == BRANCH => Some((red, *class)),
            _ => None,
        })
        .collect()
}
fn read(d: &mut Domain, answer: api::Answer) {
    let owner = *d.tier_ops.reads.iter().next().unwrap().0;
    let mut out = Queue::uncapped();
    crate::tiers::answered(d, &env(), owner, Ok(answer), &mut out);
}
fn logs(d: &mut Domain) {
    let run = workflow(d, api::Conclusion::Failure, Some(1));
    let attempt = api::JobAttempt { head: run.commit, run: RUN, job: JOB, attempt: 1 };
    read(d, api::Answer::Run(run));
    read(d, api::Answer::Job { attempt, step: None, log: LOG.into(), truncated: false });
}
fn report(d: &mut Domain, bytes: &[u8]) {
    answer(
        d,
        api::Answer::File {
            head: row(d).record.run.as_ref().unwrap().tip,
            path: b"failed.txt".as_slice().into(),
            bytes: bytes.into(),
            truncated: false,
        },
    );
}
fn failed_rerun(d: &mut Domain) {
    completed(d, api::Conclusion::Failure, Some(1));
    report(d, b"suite.first\nsuite.flake\n");
    tier_step(d, DISPATCH + 2, false);
    assert_eq!(
        row(d).record.run.as_ref().unwrap().part,
        p::Part::Rerun { failed: Box::new([TEST.into(), FLAKE.into()]) }
    );
    made(d, DISPATCH + 2, skein_forge_client::Made::Set);
    completed(d, api::Conclusion::Failure, Some(1));
    report(d, b"suite.first\n");
    tier_step(d, RELEASE + 10, false);
    made(d, RELEASE + 10, skein_forge_client::Made::Set);
}
fn kept_result(d: &mut Domain, failing: p::Failures, number: u64, tip: api::Commit) {
    let row = d.tiers.get_mut(&(REPO, TIER.into())).unwrap();
    row.task = Some(TIER_TASK);
    row.record.run = Some(p::Run {
        number,
        tip,
        key: tip,
        landings: Box::new([]),
        part: p::Part::Rerun { failed: Box::new([TEST.into()]) },
        faults: 0,
    });
    row.goal = Some(GOAL);
    row.branch = TierBranch::Absent;
    row.flight = None;
    row.result = Some(Receipt {
        logs: Box::new([]), passed: false,
        failures: Some(failing),
        run: ReceiptRun { id: RUN, number: 17, url: URL.into() },
        candidate: tip,
        ended: env().wall,
    });
    d.tier_ops.follows.remove(&(REPO, TIER.into()));
}

#[test]
fn landing_into_a_held_goal_branch_runs_without_following_the_change() {
    let mut d = configured(None);
    let out = moved(&mut d, &env(), BRANCH, Some(BASE), NEXT_TIP);
    assert_eq!(requests(&out), [(1, NEXT_TIP)]);
    let row = row(&d);
    assert_eq!(row.goal, Some(GOAL));
    assert_eq!(row.goals[0].started, Some(env().now));
    assert!(row.pending.is_empty());
    assert!(row.record.run.as_ref().unwrap().landings.is_empty());
    let mut change = fixtures::opening_row();
    change.base = BRANCH.into();
    outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(change)) });
    let mut out = Queue::uncapped();
    verification::landed(&mut d, NEXT_TIP, CHANGE, &mut out);
    assert_eq!(d.change_verification(CHANGE), temper_engine_domain_forge_change::Verification::Unfollowed);
    assert!(d.verifications.is_empty());
    let words = d.tier_specification(REPO, TIER, 1, NEXT_TIP).unwrap();
    assert!(skein_lib::bytes::find(&words, b"Repository: org/repo\nGoal branch: temper/1/g1\nJudged run:").is_some());
    let main = super::dispatched();
    assert!(skein_lib::bytes::find(&main.tier_specification(REPO, TIER, 1, BASE).unwrap(), b"Goal branch:").is_none());
}

#[test]
fn a_dropped_goal_run_keeps_its_branch_history_and_is_due_again() {
    let mut d = dispatched();
    completed(&mut d, api::Conclusion::Success, None);
    finish(&mut d);
    moved(&mut d, &env(), BRANCH, Some(NEXT_TIP), LAST_TIP);
    start(&mut d, LAST_TIP);
    span(&mut d, Some(SPAN));
    let goals = row(&d).goals.clone();
    let out = close_tier(&mut d, TIER_TASK, ReleaseEnding::Failed);
    assert!(requests(&out).is_empty());
    assert!(row(&d).record.run.is_none());
    assert!(row(&d).goal.is_none());
    assert_eq!(row(&d).record.sequence, 2);
    assert_eq!(row(&d).goals, goals);
    let out = fire(&mut d, &later());
    assert_eq!(requests(&out), [(3, LAST_TIP)]);
    assert_eq!(row(&d).goal, Some(GOAL));
}

#[test]
fn goal_watch_uses_only_the_repositories_matching_live_holds() {
    let mut d = configured(None);
    hold(&mut d, OTHER, OTHER_BRANCH);
    hold(&mut d, 3, b"temper/1/g4");
    hold(&mut d, 4, b"temper/4");
    assert_eq!(crate::tiers::watched(&d).as_slice(), &[name(BRANCH), name(OTHER_BRANCH)]);
    let mut watches = skein_lib::Map::with_capacity(LIMITS.client.resources);
    assert!(crate::tiers::add_watches(&d, &env(), &mut watches));
    assert_eq!(watches.len(), 2);
}

#[test]
fn a_goal_branch_with_no_landing_is_never_run() {
    let mut d = configured(Some(SPAN));
    assert_eq!(row(&d).goals[0].ran, Some(BASE));
    assert_eq!(row(&d).goals[0].tip, Some(BASE));
    assert!(row(&d).record.run.is_none());
    assert!(requests(&moved(&mut d, &env(), BRANCH, Some(BASE), BASE)).is_empty());
    assert!(requests(&fire(&mut d, &later())).is_empty());
    assert_eq!(crate::tiers::deadline(&d), None);
}

#[test]
fn the_makings_tip_is_the_baseline_even_before_a_watch_sees_it() {
    let mut d = configured(None);
    d.tiers.remove(&(REPO, TIER.into()));
    let entry = skein_forge_client::Entry {
        number: OPEN,
        task: GOAL,
        repository: REPO,
        effect: skein_forge_client::Effect {
            write: api::Write::CreateBranch { branch: BRANCH.into(), commit: BASE },
            condition: skein_forge_client::Condition::None,
        },
        start: None,
        attempt: None,
        failures: 0,
        queued: false,
    };
    outputs(&mut d, Event::Restore { record: Stored::Entry { entry, held: false } });
    made(&mut d, OPEN, skein_forge_client::Made::Branch(BASE));
    assert_eq!(row(&d).goals[0].ran, Some(BASE));
    assert!(row(&d).record.run.is_none());
    assert_eq!(requests(&moved(&mut d, &env(), BRANCH, Some(BASE), NEXT_TIP)), [(1, NEXT_TIP)]);
}

#[test]
fn moves_within_the_span_wait_and_take_the_newest_tip() {
    let mut d = configured(Some(SPAN));
    moved(&mut d, &env(), BRANCH, Some(BASE), NEXT_TIP);
    start(&mut d, NEXT_TIP);
    completed(&mut d, api::Conclusion::Success, None);
    finish(&mut d);
    assert!(d.landing.receipts.is_empty());
    assert!(requests(&moved(&mut d, &env(), BRANCH, Some(NEXT_TIP), HEAD)).is_empty());
    assert!(requests(&moved(&mut d, &env(), BRANCH, Some(HEAD), LAST_TIP)).is_empty());
    assert_eq!(crate::tiers::deadline(&d), Some(later().now));
    assert_eq!(d.next_deadline(), Some(later().now));
    assert!(requests(&fire(&mut d, &Env { now: Time::from_nanos(later().now.as_nanos() - 1), ..env() })).is_empty());
    assert_eq!(requests(&fire(&mut d, &later())), [(2, LAST_TIP)]);
    assert_eq!(row(&d).goals[0].started, Some(later().now));
}

#[test]
fn one_run_at_a_time_and_unstarted_goals_run_before_started_ones() {
    let mut d = dispatched();
    hold(&mut d, OTHER, OTHER_BRANCH);
    moved(&mut d, &env(), OTHER_BRANCH, None, BASE);
    assert!(requests(&moved(&mut d, &env(), OTHER_BRANCH, Some(BASE), HEAD)).is_empty());
    moved(&mut d, &env(), BRANCH, Some(NEXT_TIP), LAST_TIP);
    assert_eq!(row(&d).goal, Some(GOAL));
    completed(&mut d, api::Conclusion::Success, None);
    assert_eq!(requests(&finish(&mut d)), [(2, HEAD)]);
    assert_eq!(row(&d).goal, Some(OTHER));
    start(&mut d, HEAD);
    completed(&mut d, api::Conclusion::Success, None);
    assert_eq!(requests(&finish(&mut d)), [(3, LAST_TIP)]);
    assert_eq!(row(&d).goal, Some(GOAL));
    assert_eq!(row(&d).goals.iter().map(|g| g.goal).collect::<Vec<_>>(), [GOAL, OTHER]);
}

#[test]
fn confirmed_red_is_new_then_still_and_green_tells_nothing() {
    let mut d = dispatched();
    subscribe(&mut d, GOAL, BRANCH);
    subscribe(&mut d, OTHER, BRANCH);
    failed_rerun(&mut d);
    assert!(news(&tier_step(&mut d, RELEASE + 11, false)).is_empty());
    logs(&mut d);
    let out = tier_step(&mut d, RELEASE + 12, false);
    let reds = news(&out);
    assert_eq!(reds.len(), 1);
    assert_eq!(reds[0].1, Class::Wakes);
    assert_eq!(
        reds[0].0.on,
        RedOn::Goal { branch: BRANCH.into(), tip: NEXT_TIP, new: failures(&[TEST]), still: failures(&[]) }
    );
    assert_eq!(reds[0].0.run, 1);
    assert_eq!(reds[0].0.url, Some(URL.into()));
    assert_eq!(reds[0].0.logs.as_ref(), &[JobLog { name: b"tests".as_slice().into(), end: Some(LOG.into()) }]);
    assert_eq!(
        out.iter()
            .filter_map(|r| match r {
                Request::News { news: News::Red(red), .. } => Some(red),
                _ => None,
            })
            .count(),
        1
    );
    assert_eq!(row(&d).goals[0].failing, failures(&[TEST]));
    assert_eq!(row(&d).record.flakes.as_ref(), &[p::Flake { test: FLAKE.into(), count: 1 }]);
    assert!(d.verifications.is_empty());
    assert!(d.landing.receipts.is_empty());
    moved(&mut d, &env(), BRANCH, Some(NEXT_TIP), LAST_TIP);
    kept_result(&mut d, failures(&[TEST]), 2, LAST_TIP);
    tier_step(&mut d, RELEASE + 13, false);
    logs(&mut d);
    let out = tier_step(&mut d, RELEASE + 14, false);
    let reds = news(&out);
    assert_eq!(reds.len(), 1);
    assert_eq!(reds[0].1, Class::Kept);
    assert_eq!(
        reds[0].0.on,
        RedOn::Goal { branch: BRANCH.into(), tip: LAST_TIP, new: failures(&[]), still: failures(&[TEST]) }
    );
    moved(&mut d, &env(), BRANCH, Some(LAST_TIP), HEAD);
    start(&mut d, HEAD);
    completed(&mut d, api::Conclusion::Success, None);
    assert!(news(&finish(&mut d)).is_empty());
    assert_eq!(row(&d).goals[0].failing, failures(&[]));
}

#[test]
fn released_hold_drops_the_entry_and_in_flight_red_tells_nothing() {
    let mut d = dispatched();
    subscribe(&mut d, GOAL, BRANCH);
    let out = outputs(
        &mut d,
        Event::Release { task: GOAL, root: GOAL, ending: ReleaseEnding::Done, reason: None, entry: 40 },
    );
    assert!(out.contains(&Request::Released { task: GOAL }));
    assert!(d.hold(&name(BRANCH)).is_none());
    assert!(row(&d).goals.is_empty());
    assert!(crate::tiers::watched(&d).is_empty());
    failed_rerun(&mut d);
    let out = tier_step(&mut d, RELEASE + 11, false);
    assert!(news(&out).is_empty());
    let Some(TierDecision::Done { .. }) = choice(&out) else { panic!("released goal run ends") };
    assert!(row(&d).record.run.is_none());
    assert_eq!(row(&d).goal, None);
    assert!(row(&d).goals.is_empty());
    assert!(d.tier_ops.reads.is_empty());
    assert!(d.landing.receipts.is_empty());
}

#[test]
fn a_main_tier_waits_its_span_between_moves() {
    let mut d = super::dispatched();
    span(&mut d, Some(SPAN));
    assert_eq!(row(&d).started, Some(env().now));
    completed(&mut d, api::Conclusion::Success, None);
    finish(&mut d);
    seen(&mut d, Some(BASE), NEXT_TIP);
    assert!(requests(&key_trees(&mut d)).is_empty());
    seen(&mut d, Some(NEXT_TIP), LAST_TIP);
    assert!(requests(&key_trees(&mut d)).is_empty());
    assert_eq!(crate::tiers::deadline(&d), Some(later().now));
    assert_eq!(requests(&fire(&mut d, &later())), [(2, LAST_TIP)]);
    assert_eq!(row(&d).started, Some(later().now));
}

#[test]
fn restart_resumes_the_goal_run_and_tells_once() {
    let mut d = dispatched();
    subscribe(&mut d, GOAL, BRANCH);
    let run = workflow(&d, api::Conclusion::Failure, Some(1));
    answer(&mut d, api::Answer::Runs { runs: Box::new([run]), more: false });
    let mut cold = restoring_following(Follows::Goals);
    outputs(&mut cold, Event::Restore { record: Stored::Tier(row(&d).clone()) });
    hold(&mut cold, GOAL, BRANCH);
    for (_, subscriber) in &d.subscriptions {
        outputs(&mut cold, Event::Restore { record: Stored::Subscription(subscriber.clone()) });
    }
    outputs(&mut cold, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    let out = outputs(&mut cold, Event::ReadAfresh);
    assert!(requests(&out).is_empty());
    assert_eq!(row(&cold).goal, Some(GOAL));
    assert!(cold.tier_ops.follows.contains_key(&(REPO, TIER.into())));
    failed_rerun(&mut cold);
    tier_step(&mut cold, RELEASE + 11, false);
    logs(&mut cold);
    let out = tier_step(&mut cold, RELEASE + 12, false);
    assert_eq!(news(&out).len(), 1);
    assert_eq!(row(&cold).record.sequence, 1);
    assert!(row(&cold).record.run.is_none());
    let mut again = restoring_following(Follows::Goals);
    outputs(&mut again, Event::Restore { record: Stored::Tier(row(&cold).clone()) });
    hold(&mut again, GOAL, BRANCH);
    outputs(&mut again, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    assert!(news(&outputs(&mut again, Event::ReadAfresh)).is_empty());
    assert!(row(&again).record.run.is_none());
}

#[test]
fn restart_keeps_pending_span_deadlines_for_goal_and_main_tiers() {
    for goals in [true, false] {
        let mut d = if goals { dispatched() } else { super::dispatched() };
        span(&mut d, Some(SPAN));
        completed(&mut d, api::Conclusion::Success, None);
        finish(&mut d);
        if goals {
            moved(&mut d, &env(), BRANCH, Some(NEXT_TIP), LAST_TIP);
        } else {
            seen(&mut d, Some(BASE), LAST_TIP);
            key_trees(&mut d);
        }
        let mut cold = if goals { restoring_following(Follows::Goals) } else { restoring(None) };
        span(&mut cold, Some(SPAN));
        outputs(&mut cold, Event::Restore { record: Stored::Tier(row(&d).clone()) });
        for (_, head) in &d.heads {
            outputs(&mut cold, Event::Restore { record: Stored::BranchHead(head.clone()) });
        }
        if goals {
            hold(&mut cold, GOAL, BRANCH);
        }
        outputs(&mut cold, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        let out = outputs(&mut cold, Event::ReadAfresh);
        assert!(requests(&out).is_empty());
        assert_eq!(crate::tiers::deadline(&cold), Some(later().now));
        assert_eq!(requests(&fire(&mut cold, &later())), [(2, LAST_TIP)]);
    }
}

#[test]
fn byte_accounting_includes_goal_entries_and_failure_sets() {
    let d = configured(None);
    let mut kept = row(&d).clone();
    let bytes = crate::tiers::row_bytes(&kept).unwrap();
    kept.goals[0].failing = failures(&[TEST]);
    assert_eq!(
        crate::tiers::row_bytes(&kept).unwrap() - bytes,
        u64::try_from(size_of::<Box<[u8]>>() + TEST.len()).unwrap()
    );
    let bytes = crate::tiers::row_bytes(&kept).unwrap();
    kept.goals = Box::new([]);
    assert_eq!(
        bytes - crate::tiers::row_bytes(&kept).unwrap(),
        u64::try_from(size_of::<TierGoal>() + BRANCH.len() + size_of::<Box<[u8]>>() + TEST.len()).unwrap()
    );
}

#[test]
fn many_is_still_only_against_many_and_named_tests_split_by_membership() {
    for (failing, previous, new, still) in [
        (p::Failures::Many, p::Failures::Many, failures(&[]), p::Failures::Many),
        (p::Failures::Many, failures(&[TEST]), p::Failures::Many, failures(&[])),
        (failures(&[TEST]), p::Failures::Many, failures(&[TEST]), failures(&[])),
        (failures(&[TEST, FLAKE]), failures(&[TEST]), failures(&[FLAKE]), failures(&[TEST])),
    ] {
        let mut d = dispatched();
        subscribe(&mut d, GOAL, BRANCH);
        kept_result(&mut d, failing.clone(), 1, NEXT_TIP);
        let kept = d.tiers.get_mut(&(REPO, TIER.into())).unwrap();
        kept.goals[0].failing = previous;
        kept.record.run.as_mut().unwrap().part = match failing {
            p::Failures::Many => p::Part::Whole,
            p::Failures::Tests(tests) => p::Part::Rerun { failed: tests },
        };
        tier_step(&mut d, RELEASE, false);
        logs(&mut d);
        let out = tier_step(&mut d, RELEASE + 1, false);
        assert_eq!(news(&out)[0].0.on, RedOn::Goal { branch: BRANCH.into(), tip: NEXT_TIP, new, still });
    }
}

#[test]
fn a_goal_run_dispatches_even_with_a_receipt_for_its_tip() {
    let mut d = configured(None);
    outputs(
        &mut d,
        Event::Restore {
            record: Stored::Receipt {
                key: ReceiptKey { project: 5, repository: REPO, check: Check::Tier(TIER.into()), key: NEXT_TIP },
                receipt: Receipt {
                    logs: Box::new([]), passed: true,
                    failures: Some(failures(&[])),
                    candidate: NEXT_TIP,
                    ended: env().wall,
                    run: ReceiptRun { id: RUN, number: 17, url: URL.into() },
                },
            },
        },
    );
    assert_eq!(requests(&moved(&mut d, &env(), BRANCH, Some(BASE), NEXT_TIP)), [(1, NEXT_TIP)]);
    assert!(row(&d).result.is_none());
    assert!(!row(&d).received);
    start(&mut d, NEXT_TIP);
    assert!(row(&d).flight.is_some());
}

#[test]
fn due_goals_break_start_time_ties_by_the_lowest_goal() {
    let mut d = dispatched();
    for (goal, branch) in [(3, b"temper/1/g3".as_slice()), (OTHER, OTHER_BRANCH)] {
        hold(&mut d, goal, branch);
        moved(&mut d, &env(), branch, None, BASE);
        moved(&mut d, &env(), branch, Some(BASE), HEAD);
    }
    completed(&mut d, api::Conclusion::Success, None);
    finish(&mut d);
    assert_eq!(row(&d).goal, Some(OTHER));
    start(&mut d, HEAD);
    completed(&mut d, api::Conclusion::Success, None);
    finish(&mut d);
    assert_eq!(row(&d).goal, Some(3));
    assert_eq!(row(&d).goals.iter().map(|g| g.goal).collect::<Vec<_>>(), [GOAL, OTHER, 3]);
}
