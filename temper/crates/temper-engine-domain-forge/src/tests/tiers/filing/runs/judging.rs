//! End-of-goal judging, using real filing and tier decisions with fixture records.
use super::*;
use crate::findings::{FindingRepair, RepairJudging};
const GOAL: u64 = 91;

fn routed(d: &mut Domain, made: bool) -> Finding {
    let mut finding = kept(d, signature(&[A, B], false), FindingKind::Red);
    finding.state = FindingState::Routed;
    finding.goal = Some(GOAL);
    if made {
        finding.repair = Some(FindingRepair {
            goal: GOAL,
            made,
            attempts: 2,
            cancelled: false,
            failing: Box::new([]),
            until: Some(Wall::from_nanos(999)),
            judging: None,
        });
    }
    outputs(d, Event::Restore { record: Stored::Finding(finding.clone()) });
    let tier = d.tiers.get_mut(&(REPO, TIER.into())).unwrap();
    tier.record.last_run = Some(7);
    tier.record.failing = failures(&[B, C]);
    finding
}
fn returns(d: &mut Domain, out: &mut Queue<Request>) -> Vec<Finding> {
    let mut rows = Vec::new();
    while let Some(request) = out.pop() {
        if let Request::ReturnFinding(row) = request {
            rows.push(row.clone());
            d.commit_finding(&LIMITS, FindingWrite { row, erase: None }, &mut Queue::uncapped());
        }
    }
    rows
}
fn end(d: &mut Domain, ending: ReleaseEnding) -> Vec<Finding> {
    let mut out = Queue::uncapped();
    d.judge_goal_findings(&env(), 5, GOAL, ending, &mut out);
    returns(d, &mut out)
}
fn log(d: &mut Domain, commit: api::Commit, branch: &[u8], verdict: LoggedVerdict) {
    let mut change = fixtures::opening_row();
    change.goal = Some(GOAL);
    change.repository = REPO;
    change.base = branch.into();
    outputs(d, Event::Restore { record: Stored::Change(Box::new(change.clone())) });
    landing_log::landed(d, env().wall, commit, change.task, Box::new([]), &mut Queue::uncapped());
    let log = d.landing_logs.get_mut(&REPO).unwrap();
    log.landings.last_mut().unwrap().tiers = Box::new([LoggedTier { tier: TIER.into(), verdict }]);
}
fn waiting(d: &Domain, number: u32) -> Option<RepairJudging> {
    d.finding(5, number).unwrap().repair.as_ref().unwrap().judging
}

#[test]
fn done_and_failed_unlanded_goals_return_only_the_ids_still_failing_and_name_the_run() {
    for (ending, made, numbered) in
        [(ReleaseEnding::Done, false, true), (ReleaseEnding::Failed, true, true), (ReleaseEnding::Done, false, false)]
    {
        let mut d = one();
        let finding = routed(&mut d, made);
        if !numbered {
            d.tiers.get_mut(&(REPO, TIER.into())).unwrap().record.last_run = None;
        }
        let rows = end(&mut d, ending);
        let [back] = rows.as_slice() else { panic!("one returned finding") };
        assert_eq!(back.state, FindingState::New);
        assert_eq!(back.goal, None);
        assert_eq!(back.hander, None);
        let repair = back.repair.as_ref().unwrap();
        assert_eq!(repair.goal, GOAL);
        assert_eq!(repair.failing.as_ref(), [Box::from(B)]);
        assert_eq!(
            (repair.made, repair.attempts, repair.cancelled, repair.until, repair.judging),
            (made, if made { 2 } else { 0 }, false, None, None)
        );
        let prefix = if made { "repair goal" } else { "goal" };
        let run = if numbered { "browser run 7" } else { "browser's last run" };
        assert_eq!(
            back.why.as_deref(),
            Some(format!("{prefix} {GOAL} ended and {run} still fails 1 of its tests").as_bytes())
        );
        assert_eq!(d.finding(5, finding.number), Some(back));
    }
}
#[test]
fn a_last_judged_run_that_passes_all_ids_clears_at_goal_end() {
    let mut d = one();
    let finding = routed(&mut d, false);
    d.tiers.get_mut(&(REPO, TIER.into())).unwrap().record.failing = failures(&[C]);
    assert_eq!(end(&mut d, ReleaseEnding::Done), Vec::<Finding>::new());
    let found = d.finding(5, finding.number).unwrap();
    assert_eq!(found.cleared, Some(FindingCleared { run: 7, at: env().wall }));
    assert_eq!(found.repair.as_ref().unwrap().until, None);
}
#[test]
fn the_newest_landing_waits_across_restart_for_its_first_covering_run() {
    let mut d = one();
    let finding = routed(&mut d, true);
    log(&mut d, MID, b"main", LoggedVerdict::Cleared { run: 6 });
    log(&mut d, NEXT_TIP, b"main", LoggedVerdict::Verifying);
    assert_eq!(end(&mut d, ReleaseEnding::Done), Vec::<Finding>::new());
    assert_eq!(waiting(&d, finding.number), Some(RepairJudging::Landing(NEXT_TIP)));
    assert_eq!(d.finding(5, finding.number).unwrap().repair.as_ref().unwrap().until, None);
    let saved = d.finding(5, finding.number).unwrap().clone();
    let mut restarted = one();
    outputs(&mut restarted, Event::Restore { record: Stored::Finding(saved) });
    next(&mut restarted, failures(&[B]), &[B]);
    // The run with an unrelated range must not judge this wait.
    restarted.tiers.get_mut(&(REPO, TIER.into())).unwrap().record.run.as_mut().unwrap().landings =
        Box::new([p::Landing { commit: MID, key: MID }]);
    let (_, out) = judged(&mut restarted, &env());
    assert!(!out.iter().any(|r| matches!(r, Request::ReturnFinding(_))));
    next(&mut restarted, failures(&[B]), &[B]);
    let (_, out) = judged(&mut restarted, &env());
    let mut requests = Queue::uncapped();
    for request in out {
        requests.push(request);
    }
    let mut out = requests;
    let rows = returns(&mut restarted, &mut out);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].repair.as_ref().unwrap().failing.as_ref(), [Box::from(B)]);
    assert!(rows[0].why.as_ref().unwrap().windows(b"browser run 3".len()).any(|w| w == b"browser run 3"));
}
#[test]
fn an_already_judged_landing_uses_the_last_run_at_once() {
    let mut d = one();
    routed(&mut d, false);
    log(
        &mut d,
        NEXT_TIP,
        b"main",
        LoggedVerdict::Red { run: 6, failing: Some(1), tests: Box::new([B.into()]), many: None },
    );
    let rows = end(&mut d, ReleaseEnding::Done);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].why.as_deref(), Some(b"goal 91 ended and browser run 7 still fails 1 of its tests".as_slice()));
}
#[test]
fn many_waits_for_the_next_enumerated_run_even_when_it_covers_the_landing() {
    for landed in [false, true] {
        let mut d = one();
        let finding = routed(&mut d, false);
        if landed {
            log(&mut d, NEXT_TIP, b"main", LoggedVerdict::Verifying);
        } else {
            d.tiers.get_mut(&(REPO, TIER.into())).unwrap().record.failing = p::Failures::Many;
        }
        assert_eq!(end(&mut d, ReleaseEnding::Done), Vec::<Finding>::new());
        if landed {
            next(&mut d, p::Failures::Many, &[]);
            let (_, out) = judged(&mut d, &env());
            assert!(!out.iter().any(|r| matches!(r, Request::ReturnFinding(_))));
        }
        let Some(RepairJudging::After(number)) = waiting(&d, finding.number) else { panic!("many waits After") };
        // A numbered run later than the many judgement is enumerated.
        d.tiers.get_mut(&(REPO, TIER.into())).unwrap().record.sequence = number;
        next(&mut d, failures(&[B]), &[B]);
        let (_, out) = judged(&mut d, &env());
        assert!(out.iter().any(|r| matches!(r, Request::ReturnFinding(row) if row.number == finding.number)));
    }
}
#[test]
fn a_persons_hold_waits_until_release_and_uses_the_latest_judged_run() {
    let mut d = one();
    let finding = routed(&mut d, true);
    d.tiers.get_mut(&(REPO, TIER.into())).unwrap().hold =
        Some(TierHold { party: 44, at: env().wall, reason: b"hold".as_slice().into() });
    assert_eq!(end(&mut d, ReleaseEnding::Failed), Vec::<Finding>::new());
    assert_eq!(waiting(&d, finding.number), Some(RepairJudging::Release));
    let row = d.tiers.get_mut(&(REPO, TIER.into())).unwrap();
    row.record.last_run = Some(8);
    row.record.failing = failures(&[A]);
    let out = outputs(
        &mut d,
        Event::TierAct {
            request: Token::new(70),
            token: Token::new(71),
            party: 44,
            call: TierActRequest { project: 5, repository: None, tier: TIER.into(), act: TierAct::Release },
        },
    );
    assert!(out.iter().any(|r| matches!(r, Request::ReturnFinding(row) if row.why.as_deref() == Some(b"repair goal 91 ended and browser run 8 still fails 1 of its tests".as_slice()))), "{out:?}");
}
#[test]
fn a_landing_outside_the_tiers_branch_or_let_go_waits_for_a_run_started_after_the_end() {
    for forgotten in [false, true] {
        let mut d = one();
        let finding = routed(&mut d, false);
        log(&mut d, NEXT_TIP, b"other", LoggedVerdict::Verifying);
        if forgotten {
            d.landing_logs.remove(&REPO);
            let row = d.changes.get_mut(&fixtures::opening_row().task).unwrap();
            row.change.state = temper_engine_domain_forge_change::State::Landed { merge: NEXT_TIP, since: env().wall };
        }
        // The already started run must not judge the wait, even if it finishes after the end.
        let sequence = row(&d).record.run.as_ref().unwrap().number;
        assert_eq!(end(&mut d, ReleaseEnding::Done), Vec::<Finding>::new());
        assert_eq!(waiting(&d, finding.number), Some(RepairJudging::After(sequence)));
        let (_, out) = judged(&mut d, &env());
        assert!(!out.iter().any(|r| matches!(r, Request::ReturnFinding(_))));
        next(&mut d, failures(&[B]), &[B]);
        let (_, out) = judged(&mut d, &env());
        assert!(out.iter().any(|r| matches!(r, Request::ReturnFinding(row) if row.number == finding.number)));
    }
}
#[test]
fn a_tier_with_no_judgement_waits_for_its_first_run() {
    let mut d = one();
    let finding = routed(&mut d, false);
    d.tiers.get_mut(&(REPO, TIER.into())).unwrap().record.judged = None;
    assert_eq!(end(&mut d, ReleaseEnding::Failed), Vec::<Finding>::new());
    assert_eq!(waiting(&d, finding.number), Some(RepairJudging::After(0)));
    let (_, out) = judged(&mut d, &env());
    assert!(out.iter().any(|r| matches!(r, Request::ReturnFinding(row) if row.number == finding.number)));
}
#[test]
fn cancellation_returns_at_once_and_every_ending_releases_even_a_cleared_findings_cap() {
    let mut d = one();
    let finding = routed(&mut d, true);
    let rows = end(&mut d, ReleaseEnding::Cancelled);
    assert_eq!(rows[0].why.as_deref(), Some(b"repair goal 91 was cancelled".as_slice()));
    assert!(rows[0].repair.as_ref().unwrap().cancelled);
    assert_eq!(rows[0].repair.as_ref().unwrap().until, None);
    let mut cleared = finding;
    cleared.cleared = Some(FindingCleared { run: 5, at: env().wall });
    outputs(&mut d, Event::Restore { record: Stored::Finding(cleared.clone()) });
    assert_eq!(end(&mut d, ReleaseEnding::Done), Vec::<Finding>::new());
    assert_eq!(d.finding(5, cleared.number).unwrap().repair.as_ref().unwrap().until, None);
}
#[test]
fn repair_deadlines_wake_once_per_start_and_restart_rearms_them() {
    let mut d = one();
    let mut finding = routed(&mut d, true);
    let until = Wall::from_nanos(1);
    finding.repair.as_mut().unwrap().until = Some(until);
    outputs(&mut d, Event::Restore { record: Stored::Finding(finding.clone()) });
    assert_eq!(d.next_wall_deadline(), Some(until));
    let mut before = env();
    before.wall = Wall::EPOCH;
    let mut out = Queue::uncapped();
    fire(&mut d, &before, &mut out);
    assert!(!out.iter().any(|r| matches!(r, Request::FailRepair { .. })));
    let mut due = env();
    due.wall = until;
    fire(&mut d, &due, &mut out);
    fire(&mut d, &due, &mut out);
    fire(&mut d, &before, &mut out);
    fire(&mut d, &due, &mut out);
    assert_eq!(
        out.iter()
            .filter(|r| matches!(r, Request::FailRepair { project: 5, task: GOAL, until: at } if *at == until))
            .count(),
        1
    );
    assert_ne!(d.next_wall_deadline(), Some(until));
    let mut restarted = one();
    outputs(&mut restarted, Event::Restore { record: Stored::Finding(finding) });
    let mut out = Queue::uncapped();
    fire(&mut restarted, &due, &mut out);
    assert_eq!(out.iter().filter(|r| matches!(r, Request::FailRepair { task: GOAL, .. })).count(), 1);
}
