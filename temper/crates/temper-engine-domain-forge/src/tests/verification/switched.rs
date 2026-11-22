//! A switched project's verification files at its last release (red-findings.md, 8).
#![expect(clippy::disallowed_methods, reason = "ordinary assertions inspect captured filing words")]
use super::*;

fn failures(ids: &[&[u8]]) -> policy::Failures {
    let mut tests = List::uncapped();
    for id in ids {
        tests.push(Box::from(*id)).unwrap();
    }
    policy::Failures::Tests(tests.into_boxed())
}
fn switched(blame: policy::Blame) -> Domain {
    let mut d = verifying();
    judge(&mut d, TIER, 5, &[FIRST, LAST], Box::new([policy::Blame::Cleared, blame]), failures(&[b"x", b"y"]));
    d.landing.projects.get_mut(&PROJECT).unwrap().2.wait = change::Wait::Landed;
    d
}
fn release(d: &mut Domain, task: u64) -> Box<[VerdictFiling]> {
    let mut out = Queue::uncapped();
    verification::released(d, task, &mut out);
    let mut calls = List::uncapped();
    while let Some(request) = out.pop() {
        if let Request::FileVerdict { task: filer, filing } = request {
            assert_eq!(filer, task);
            calls.push(filing).unwrap();
        }
    }
    calls.into_boxed()
}
fn commit(d: &mut Domain, filing: &VerdictFiling) {
    let write = d.prepare_verdict_filing(&LIMITS, env().wall, Filer::Task(SECOND), filing).unwrap();
    d.commit_finding(&LIMITS, write, &mut Queue::uncapped());
}

#[test]
fn suspected_and_many_verdicts_name_only_the_released_landing() {
    for (blame, strength, many) in [
        (policy::Blame::Suspect(failures(&[b"x", b"y"])), BlameStrength::Suspected, false),
        (policy::Blame::Owns(policy::Failures::Many), BlameStrength::Owned, true),
    ] {
        let mut d = switched(blame);
        let calls = release(&mut d, SECOND);
        let [call] = calls.as_ref() else { panic!("one switched filing: {calls:?}") };
        assert_eq!(
            call.blame,
            Some(FindingBlame {
                moved: None,
                strength,
                landings: Box::new([BlamedLanding { commit: LAST, goals: Box::new([]) }])
            })
        );
        assert_eq!(call.signature.many, many);
        if many {
            assert_eq!(call.signature.ids.as_ref(), []);
        } else {
            assert_eq!(call.signature.ids.as_ref(), [Box::from(b"x".as_slice()), Box::from(b"y".as_slice())]);
        }
        assert_eq!(
            call.title.as_ref(),
            if many {
                b"many tests fail in browser since 070707070707 landed".as_slice()
            } else {
                b"2 tests fail in browser since 070707070707 landed".as_slice()
            }
        );
        assert!(call.body.starts_with(
            b"browser run 5 blamed landing 070707070707 on main before its project ended changes at landing.\n"
        ));
        assert!(!call.body.windows(5).any(|text| text == b"Run: "));
        assert!(!call.body.windows(11).any(|text| text == b"End of job "));
        assert!(call.signature.first.is_empty());
    }
}

#[test]
fn repaired_cleared_unjudged_and_cleared_policy_release_file_nothing() {
    for state in 0_u8..4 {
        let mut d = switched(policy::Blame::Owns(failures(&[b"x", b"y"])));
        match state {
            0 => {
                d.verifications.get_mut(&(REPO, LAST)).unwrap().tiers[0].repaired =
                    Some(TierRepair { run: 6, tip: REPAIR, by: None });
            }
            1 => {
                d.verifications.get_mut(&(REPO, LAST)).unwrap().tiers[0].verdict.as_mut().unwrap().blame =
                    policy::Blame::Cleared;
            }
            2 => d.verifications.get_mut(&(REPO, LAST)).unwrap().tiers[0].verdict = None,
            3 => d.landing.projects.get_mut(&PROJECT).unwrap().2.wait = change::Wait::Cleared,
            _ => unreachable!(),
        }
        assert!(release(&mut d, SECOND).is_empty(), "case {state}");
        assert!(d.landing_verification(REPO, &LAST).is_none());
    }
}

#[test]
fn only_the_last_release_files_once_with_change_rows_when_the_log_is_gone() {
    let mut d = switched(policy::Blame::Owns(failures(&[b"x", b"y"])));
    d.landing_logs.remove(&REPO);
    let row = d.changes.get_mut(&SECOND).unwrap();
    row.goal = Some(91);
    row.goal_title = Some(b"R2 Reds blamed".as_slice().into());
    row.milestone = Some(81);
    row.person = Some(44);
    row.draft = Some(b"draft/path.md".as_slice().into());
    row.pull = Some(402);
    d.verifications.remove(&(REPO, FIRST));
    d.verifications.get_mut(&(REPO, LAST)).unwrap().changes = Box::new([CHANGE, SECOND]);
    assert_eq!(release(&mut d, CHANGE).as_ref(), []);
    assert!(d.landing_verification(REPO, &LAST).is_some());
    let calls = release(&mut d, SECOND);
    let [call] = calls.as_ref() else { panic!("one final filing: {calls:?}") };
    assert_eq!(call.title.as_ref(), b"2 tests fail in browser since R2 landed");
    assert_eq!(
        call.blame.as_ref().unwrap().landings[0].goals.as_ref(),
        [BlamedGoal { goal: 91, milestone: Some(81), person: Some(44) }]
    );
    assert!(
        core::str::from_utf8(&call.body).unwrap().contains(
            "Goal 91: R2 Reds blamed.\nPull request #402.\nDraft: draft/path.md.\nPerson 44.\nFailing:\nx\ny"
        )
    );
    assert!(d.landing_verification(REPO, &LAST).is_none());
    assert_eq!(release(&mut d, SECOND).as_ref(), []);
}

#[test]
fn held_ids_and_waiting_filings_are_not_filed_again() {
    let mut d = switched(policy::Blame::Owns(failures(&[b"x"])));
    let calls = release(&mut d, SECOND);
    commit(&mut d, &calls[0]);
    let finding = d.finding(PROJECT, 1).unwrap().clone();
    let mut dropped = finding;
    dropped.state = FindingState::Dropped;
    d.commit_finding(&LIMITS, FindingWrite { row: dropped, erase: None }, &mut Queue::uncapped());
    let row = d.verifications.get_mut(&(REPO, FIRST)).unwrap();
    row.tiers[0].verdict = Some(TierVerdict { run: 5, blame: policy::Blame::Owns(failures(&[b"x", b"y"])) });
    d.landing.limits.findings_per_project = 1;
    let mut out = Queue::uncapped();
    verification::released(&mut d, CHANGE, &mut out);
    assert!(!out.iter().any(|request| matches!(request, Request::FileVerdict { .. })), "no room");
    let mut saved = None;
    for request in &out {
        if let Request::Save { record: Stored::Tier(row), .. } = request {
            saved = Some(row);
        }
    }
    let saved = saved.expect("the waiting filing is saved before verification is erased");
    let waiting = &d.tiers.get(&(REPO, TIER.into())).unwrap().waiting;
    assert_eq!(saved.waiting, *waiting);
    assert_eq!(waiting.len(), 1);
    assert_eq!(waiting[0].signature.ids.as_ref(), [Box::from(b"y".as_slice())]);
    // A later release of the same failures sees both the dropped red and the waiting filing.
    d.verifications
        .insert(
            (REPO, LAST),
            Verification {
                repository: REPO,
                commit: LAST,
                changes: Box::new([SECOND]),
                tiers: Box::new([TierVerification {
                    tier: TIER.into(),
                    verdict: Some(TierVerdict { run: 5, blame: policy::Blame::Owns(failures(&[b"x", b"y"])) }),
                    repaired: None,
                }]),
            },
        )
        .unwrap();
    assert_eq!(release(&mut d, SECOND).as_ref(), []);
    assert_eq!(d.tiers.get(&(REPO, TIER.into())).unwrap().waiting.len(), 1);
}
