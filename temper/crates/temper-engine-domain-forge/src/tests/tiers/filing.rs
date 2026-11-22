//! Landed projects file a verdict's unheld failures, not a red per landing (red-findings.md, 2–5).
use super::*;
pub(crate) mod runs;
use crate::findings::{BlameStrength, FindingBlame, FindingCleared, FindingSignature, VerdictFiling};
use red::{failures, pending, respond};

const A: &[u8] = b"a::one";
const B: &[u8] = b"a::two";
const C: &[u8] = b"a::three";
const OWN: &[u8] = b"z::red";
const MID: api::Commit = [8; 32];

fn landed(mut d: Domain) -> Domain {
    d.landing.projects.get_mut(&5).unwrap().2.wait = temper_engine_domain_forge_change::Wait::Landed;
    d
}
fn one() -> Domain {
    landed(pending(
        Box::new([p::Landing { commit: NEXT_TIP, key: NEXT_TIP }]),
        p::Part::Rerun { failed: Box::new([A.into(), B.into()]) },
        failures(&[A, B]),
    ))
}
pub(super) fn calls(out: &[Request]) -> Vec<VerdictFiling> {
    assert!(
        !out.iter().any(|request| matches!(
            request,
            Request::FileRedFinding { .. } | Request::News { news: News::Red(_), .. }
        ))
    );
    out.iter()
        .filter_map(|request| match request {
            Request::FileVerdict { task, filing } => {
                assert_eq!(*task, TIER_TASK);
                Some(filing.clone())
            }
            _ => None,
        })
        .collect()
}
fn judge(d: &mut Domain) -> (Vec<VerdictFiling>, Box<[Request]>) {
    let mut out = settled_step(d, RELEASE, false);
    if !d.tier_ops.reads.is_empty() {
        assert!(calls(&out).is_empty(), "filing waits for logs before keeping the verdict");
        respond(d, Err(api::Error::Missing));
        out = settled_step(d, RELEASE + 1, false);
    }
    (calls(&out), out)
}
fn strength(filing: &VerdictFiling) -> BlameStrength {
    filing.blame.as_ref().unwrap().strength
}
fn signature(ids: &[&[u8]], many: bool) -> FindingSignature {
    FindingSignature {
        repository: REPO,
        source: TIER.into(),
        ids: ids.iter().copied().map(Box::from).collect(),
        first: Box::new([]),
        many,
    }
}
fn kept(d: &mut Domain, signature: FindingSignature, kind: FindingKind) -> Finding {
    let filing = VerdictFiling {
        project: 5,
        kind,
        severity: Severity::High,
        title: b"Held".as_slice().into(),
        body: Box::new([]),
        signature,
        blame: if kind == FindingKind::Red {
            Some(FindingBlame { moved: None, strength: BlameStrength::Unblamed, landings: Box::new([]) })
        } else {
            None
        },
        follow: None,
    };
    let write = d.prepare_verdict_filing(&LIMITS, env().wall, Filer::Task(TIER_TASK), &filing).unwrap();
    let finding = write.row.clone();
    d.commit_finding(&LIMITS, write, &mut Queue::uncapped());
    finding
}

#[test]
fn logged_members_name_a_goal_once_with_all_its_pulls_and_many_uses_its_fallback_name() {
    let mut d =
        landed(pending(Box::new([p::Landing { commit: NEXT_TIP, key: NEXT_TIP }]), p::Part::Whole, p::Failures::Many));
    for (task, pull) in [(CHANGE, 401), (CHANGE + 1, 402)] {
        let mut change = fixtures::opening_row();
        change.task = task;
        change.repository = REPO;
        change.base = b"main".as_slice().into();
        change.pull = Some(pull);
        change.goal = Some(91);
        change.milestone = Some(81);
        change.person = Some(44);
        change.goal_title = Some(b"No leading id".as_slice().into());
        change.draft = Some(b"draft/path.md".as_slice().into());
        outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(change)) });
        landing_log::landed(&mut d, env().wall, NEXT_TIP, task, Box::new([]), &mut Queue::uncapped());
    }
    let (calls, _) = judge(&mut d);
    let [call] = calls.as_slice() else { panic!("one many owner") };
    assert_eq!(call.title.as_ref(), b"many tests fail in browser since goal 91 landed");
    assert_eq!(
        call.blame.as_ref().unwrap().landings[0].goals.as_ref(),
        [BlamedGoal { goal: 91, milestone: Some(81), person: Some(44) }]
    );
    let body = core::str::from_utf8(&call.body).unwrap();
    assert!(body.contains("Goal 91: No leading id.\nPull requests #401, #402.\nDraft: draft/path.md.\nPerson 44.\nFailing:\nmany tests"), "{body}");
}

#[test]
fn signature_uses_the_last_nonblank_line_of_the_first_job_within_512_bytes() {
    let mut d = one();
    settled_step(&mut d, RELEASE, false);
    red::run_answer(&mut d);
    let text = format!("earlier\n{}\n \t\n", "é".repeat(257));
    respond(&mut d, Ok(red::log(red::FIRST_ATTEMPT, text.as_bytes())));
    respond(&mut d, Ok(red::log(red::SECOND_ATTEMPT, b"second job's end")));
    let calls = calls(&settled_step(&mut d, RELEASE + 1, false));
    let [call] = calls.as_slice() else { panic!("one logged finding") };
    assert_eq!(call.signature.first.len(), 512);
    assert_eq!(core::str::from_utf8(&call.signature.first).unwrap(), "é".repeat(256));
    assert!(call.body.ends_with(b"End of job second's log:\nsecond job's end"));
    assert!(call.body.len() <= 4096);
}

#[test]
fn body_keeps_whole_ids_then_counts_the_rest_while_signature_keeps_every_id() {
    let ids: Box<[Box<[u8]>]> =
        (0_u8..16).map(|n| format!("{n:02}{}", "x".repeat(254)).into_bytes().into_boxed_slice()).collect();
    let mut d = landed(pending(
        Box::new([p::Landing { commit: NEXT_TIP, key: NEXT_TIP }]),
        p::Part::Rerun { failed: ids.clone() },
        p::Failures::Tests(ids.clone()),
    ));
    let (calls, _) = judge(&mut d);
    let [call] = calls.as_slice() else { panic!("one bounded filing") };
    assert_eq!(call.signature.ids, ids);
    assert!(call.body.len() <= 4096);
    let body = core::str::from_utf8(&call.body).unwrap();
    let lines: Vec<_> = body.lines().collect();
    let failing = lines.iter().position(|line| *line == "Failing:").unwrap();
    let shown = &lines[failing + 1..lines.len() - 1];
    for (line, id) in shown.iter().zip(&ids) {
        assert_eq!(line.as_bytes(), id.as_ref(), "an id is never partial");
    }
    assert_eq!(lines.last().unwrap(), &format!("and {} more tests", ids.len() - shown.len()));
    assert_ne!(shown.len(), 0);
    assert!(shown.len() < ids.len());
}

#[test]
fn bisected_failures_file_each_owner_oldest_first_with_only_its_ids() {
    let mut d = landed(pending(
        Box::new([
            p::Landing { commit: LAST_TIP, key: LAST_TIP },
            p::Landing { commit: MID, key: MID },
            p::Landing { commit: NEXT_TIP, key: NEXT_TIP },
        ]),
        p::Part::Probe {
            landing: 2,
            low: 2,
            high: 2,
            tests: p::Selection::Tests(Box::new([B.into()])),
            waiting: Box::new([]),
            finding: p::Finding {
                failing: failures(&[A, B]),
                flakes: Box::new([]),
                blamed: Box::new([p::Blame::Owns(failures(&[A])), p::Blame::Cleared, p::Blame::Cleared]),
            },
        },
        failures(&[B]),
    ));
    let (calls, _) = judge(&mut d);
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].signature, signature(&[A], false));
    assert_eq!(calls[1].signature, signature(&[B], false));
    for (call, commit) in calls.iter().zip([LAST_TIP, NEXT_TIP]) {
        assert_eq!(strength(call), BlameStrength::Owned);
        assert_eq!(call.blame.as_ref().unwrap().landings[0].commit, commit);
    }
}

#[test]
fn tell_all_files_one_suspected_finding_naming_every_landing() {
    let mut d = one();
    d.landing.projects.get_mut(&5).unwrap().2.verdict.suspects = p::Suspects::TellAll;
    let range = Box::new([
        p::Landing { commit: LAST_TIP, key: LAST_TIP },
        p::Landing { commit: MID, key: MID },
        p::Landing { commit: NEXT_TIP, key: NEXT_TIP },
    ]);
    d.tiers.get_mut(&(REPO, TIER.into())).unwrap().record.run.as_mut().unwrap().landings = range;
    let (calls, _) = judge(&mut d);
    let [call] = calls.as_slice() else { panic!("one suspected filing: {calls:?}") };
    assert_eq!(call.title.as_ref(), b"2 tests fail in browser since one of 3 landings");
    assert_eq!(call.signature, signature(&[A, B], false));
    assert_eq!(strength(call), BlameStrength::Suspected);
    assert_eq!(
        call.blame.as_ref().unwrap().landings.iter().map(|row| row.commit).collect::<Vec<_>>(),
        [LAST_TIP, MID, NEXT_TIP]
    );
}

#[test]
fn first_landed_run_files_previous_failures_unblamed_and_new_failures_owned() {
    let mut d = one();
    d.tiers.get_mut(&(REPO, TIER.into())).unwrap().record.failing = failures(&[A]);
    let (calls, _) = judge(&mut d);
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].signature, signature(&[B], false));
    assert_eq!(strength(&calls[0]), BlameStrength::Owned);
    assert_eq!(calls[1].signature, signature(&[A], false));
    assert_eq!(calls[1].title.as_ref(), b"1 test fails in browser from before run 1");
    assert_eq!(strength(&calls[1]), BlameStrength::Unblamed);
    assert!(calls[1].blame.as_ref().unwrap().landings.is_empty());
}

#[test]
fn baseline_failures_make_one_unblamed_finding() {
    let mut d =
        landed(pending(Box::new([]), p::Part::Rerun { failed: Box::new([A.into(), B.into()]) }, failures(&[A, B])));
    d.tiers.get_mut(&(REPO, TIER.into())).unwrap().record.judged = None;
    let (calls, _) = judge(&mut d);
    let [call] = calls.as_slice() else { panic!("one baseline finding: {calls:?}") };
    assert_eq!(call.signature, signature(&[A, B], false));
    assert_eq!(call.title.as_ref(), b"2 tests fail in browser from before run 1");
    assert_eq!(strength(call), BlameStrength::Unblamed);
    assert!(call.blame.as_ref().unwrap().landings.is_empty());
}

#[test]
fn uncleared_reds_hold_ids_in_every_state_without_reading_logs() {
    for state in [FindingState::New, FindingState::Routed, FindingState::Backlog, FindingState::Dropped] {
        let mut d = one();
        let mut finding = kept(&mut d, signature(&[A, B], false), FindingKind::Red);
        finding.state = state;
        outputs(&mut d, Event::Restore { record: Stored::Finding(finding) });
        let out = settled_step(&mut d, RELEASE, false);
        assert!(calls(&out).is_empty(), "held in {state:?}");
        assert!(d.tier_ops.reads.is_empty());
        assert!(row(&d).record.run.is_none());
    }
}

#[test]
fn held_ids_are_only_uncleared_reds_of_this_repository_and_tier() {
    for different in 0_u8..5 {
        let mut d = one();
        let mut finding = kept(&mut d, signature(&[A], false), FindingKind::Red);
        match different {
            0 => finding.cleared = Some(FindingCleared { run: 1, at: env().wall }),
            1 => finding.signature.as_mut().unwrap().source = b"another".as_slice().into(),
            2 => finding.signature.as_mut().unwrap().repository.repository += 1,
            3 => finding.blame = None,
            4 => finding.kind = FindingKind::Flake,
            _ => unreachable!(),
        }
        outputs(&mut d, Event::Restore { record: Stored::Finding(finding) });
        let (calls, _) = judge(&mut d);
        assert_eq!(calls.len(), 1, "case {different}");
        assert_eq!(calls[0].signature, signature(&[A, B], false), "case {different}");
    }
    let mut d = one();
    kept(&mut d, signature(&[A], false), FindingKind::Red);
    let (calls, _) = judge(&mut d);
    assert_eq!(calls[0].signature, signature(&[B], false), "the held id is subtracted");
}

#[test]
fn outside_landing_names_only_the_commit_and_uses_twelve_digits() {
    let mut d = one();
    let (calls, _) = judge(&mut d);
    let [call] = calls.as_slice() else { panic!("one outside finding") };
    assert_eq!(call.title.as_ref(), b"2 tests fail in browser since 060606060606 landed");
    assert_eq!(call.blame.as_ref().unwrap().landings[0].commit, NEXT_TIP);
    assert!(call.blame.as_ref().unwrap().landings[0].goals.is_empty());
    assert!(call.body.starts_with(b"browser run 1 on main: landing 060606060606.\nArtifact: report\nOwned by landing 060606060606.\nLanding 060606060606:\nFailing:\na::one\na::two"));
}

#[test]
fn flakes_are_counted_once_per_open_holder_and_the_rest_filed_not_red() {
    let mut d = landed(pending(
        Box::new([p::Landing { commit: NEXT_TIP, key: NEXT_TIP }]),
        p::Part::Rerun { failed: Box::new([A.into(), C.into(), B.into(), OWN.into()]) },
        failures(&[OWN]),
    ));
    let open = kept(&mut d, signature(&[A, B], false), FindingKind::Flake);
    let mut dropped = kept(&mut d, signature(&[C], false), FindingKind::Flake);
    dropped.state = FindingState::Dropped;
    outputs(&mut d, Event::Restore { record: Stored::Finding(dropped.clone()) });
    let (calls, out) = judge(&mut d);
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].kind, FindingKind::Red);
    assert_eq!(calls[0].signature, signature(&[OWN], false));
    assert_eq!(calls[1].kind, FindingKind::Flake);
    assert_eq!(calls[1].severity, Severity::Low);
    assert_eq!(calls[1].title.as_ref(), b"1 test flaked in browser run 1");
    assert_eq!(calls[1].signature, signature(&[C], false));
    assert_eq!(calls[1].blame, None);
    assert_eq!(calls[1].follow, None);
    assert_eq!(d.finding(5, open.number).unwrap().seen, Some(2));
    assert_eq!(d.finding(5, dropped.number).unwrap().seen, Some(1));
    assert!(out.iter().any(|request| matches!(request, Request::Save { record: Stored::Finding(row), .. } if row.number == open.number && row.seen == Some(2))));
}

#[test]
fn a_run_that_only_flakes_files_without_log_reads() {
    let mut d = one();
    d.tiers.get_mut(&(REPO, TIER.into())).unwrap().result.as_mut().unwrap().failures = Some(failures(&[]));
    d.tiers.get_mut(&(REPO, TIER.into())).unwrap().result.as_mut().unwrap().passed = true;
    let out = settled_step(&mut d, RELEASE, false);
    assert!(d.tier_ops.reads.is_empty());
    let calls = calls(&out);
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].kind, FindingKind::Flake);
    assert_eq!(calls[0].title.as_ref(), b"2 tests flaked in browser run 1");
    assert_eq!(calls[0].signature, signature(&[A, B], false));
}

#[test]
fn many_files_owned_or_unblamed_but_not_while_a_many_finding_holds_it() {
    for baseline in [false, true] {
        let range: Box<[p::Landing]> =
            if baseline { Box::new([]) } else { Box::new([p::Landing { commit: NEXT_TIP, key: NEXT_TIP }]) };
        let mut d = landed(pending(range, p::Part::Whole, p::Failures::Many));
        let (calls, _) = judge(&mut d);
        let [call] = calls.as_slice() else { panic!("one many finding") };
        assert_eq!(call.signature, signature(&[], true));
        assert_eq!(strength(call), if baseline { BlameStrength::Unblamed } else { BlameStrength::Owned });
        assert_eq!(
            call.title.as_ref(),
            if baseline {
                b"many tests fail in browser from before run 1".as_slice()
            } else {
                b"many tests fail in browser since 060606060606 landed".as_slice()
            }
        );
        let mut again = landed(pending(Box::new([]), p::Part::Whole, p::Failures::Many));
        kept(&mut again, call.signature.clone(), FindingKind::Red);
        let out = settled_step(&mut again, RELEASE, false);
        assert_eq!(self::calls(&out).len(), 0);
        assert!(again.tier_ops.reads.is_empty());
    }
}
