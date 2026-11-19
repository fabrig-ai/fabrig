//! Clearing, many ids and durable waiting, in judged-run order (red-findings.md, 2, 5–7).
use super::*;

fn next(d: &mut Domain, mut failing: p::Failures, originally: &[&[u8]]) {
    if let p::Failures::Tests(ids) = &mut failing {
        ids.sort();
    }
    let mut originally: Box<[Box<[u8]>]> = originally.iter().map(|id| Box::from(*id)).collect();
    originally.sort();
    let old = row(d).clone();
    let mut fresh = row(&one()).clone();
    fresh.waiting = old.waiting;
    fresh.unkept = old.unkept;
    fresh.record.sequence = old
        .record
        .run
        .as_ref()
        .map_or(old.record.sequence, |run| old.record.sequence.max(run.number))
        .checked_add(1)
        .unwrap();
    fresh.record.failing = old.record.failing;
    let run = fresh.record.run.as_mut().unwrap();
    run.number = fresh.record.sequence;
    run.part = match &failing {
        p::Failures::Many => p::Part::Whole,
        p::Failures::Tests(_) => p::Part::Rerun { failed: originally },
    };
    let result = fresh.result.as_mut().unwrap();
    result.passed = failing == failures(&[]);
    result.failures = Some(failing);
    d.tiers.insert((REPO, TIER.into()), fresh).unwrap();
}
fn judged(d: &mut Domain, env: &Env<Limits>) -> (Vec<VerdictFiling>, Vec<Request>) {
    let mut out = Queue::uncapped();
    step(
        d,
        env,
        Event::StepTier { task: TIER_TASK, entry: RELEASE, heard: TierHeard { released: false, message_bytes: 4096 } },
        &mut out,
    );
    let mut all = collect(out).into_vec();
    if !d.tier_ops.reads.is_empty() {
        for _ in 0..env.limits.client.pending {
            let (call, _) = fixtures::next_at(d, env);
            all.extend(collect(fixtures::answer_at(d, env, call, Err(api::Error::Missing))).into_vec());
            if d.tier_ops.reads.is_empty() {
                break;
            }
        }
        assert!(d.tier_ops.reads.is_empty(), "log read answered");
        let mut out = Queue::uncapped();
        step(
            d,
            env,
            Event::StepTier {
                task: TIER_TASK,
                entry: RELEASE + 1,
                heard: TierHeard { released: false, message_bytes: 4096 },
            },
            &mut out,
        );
        all.extend(collect(out).into_vec());
    }
    (calls(&all), all)
}
fn take(d: &mut Domain, env: &Env<Limits>, calls: Vec<VerdictFiling>) -> Vec<Finding> {
    calls
        .into_iter()
        .map(|filing| {
            let write = d
                .prepare_verdict_filing(&env.limits, env.wall, Filer::Task(TIER_TASK), &filing)
                .expect("every requested filing has room");
            if let Some(erase) = write.erase {
                assert!(!d.finding(5, erase).unwrap().uncleared_red(), "never erase an uncleared red");
            }
            let finding = write.row.clone();
            d.commit_finding(&env.limits, write, &mut Queue::uncapped());
            finding
        })
        .collect()
}
#[test]
fn enumerated_runs_clear_passed_reds_in_every_state_and_later_file_them_anew() {
    for state in [FindingState::New, FindingState::Routed, FindingState::Backlog, FindingState::Dropped] {
        let mut d = one();
        let mut finding = kept(&mut d, signature(&[A, B], false), FindingKind::Red);
        finding.state = state;
        outputs(&mut d, Event::Restore { record: Stored::Finding(finding.clone()) });
        next(&mut d, failures(&[A]), &[A, B]);
        let (calls, _) = judged(&mut d, &env());
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].kind, FindingKind::Flake);
        assert!(d.finding(5, finding.number).unwrap().cleared.is_none());
        next(&mut d, failures(&[OWN]), &[B, OWN]);
        let mut env = env();
        env.wall = Wall::from_nanos(321);
        let (calls, out) = judged(&mut d, &env);
        let cleared = d.finding(5, finding.number).unwrap();
        assert_eq!(cleared.cleared, Some(FindingCleared { run: 3, at: env.wall }));
        assert_eq!(cleared.state, state);
        assert!(
            out.iter()
                .any(|request| matches!(request, Request::Save { record: Stored::Finding(row), .. } if row == cleared))
        );
        assert_eq!(
            calls
                .iter()
                .filter(|call| call.kind == FindingKind::Red)
                .map(|call| call.signature.ids.as_ref())
                .collect::<Vec<_>>(),
            [&[Box::from(OWN)][..]]
        );
        next(&mut d, failures(&[A]), &[A]);
        let (calls, _) = judged(&mut d, &env);
        assert_eq!(calls[0].signature.ids.as_ref(), [Box::from(A)]);
        assert_eq!(d.finding(5, finding.number).unwrap().cleared, Some(FindingCleared { run: 3, at: env.wall }));
    }
}
#[test]
fn many_runs_neither_clear_nor_assign_ids() {
    let mut d = one();
    let enumerated = kept(&mut d, signature(&[A], false), FindingKind::Red);
    let many = kept(&mut d, signature(&[], true), FindingKind::Red);
    next(&mut d, p::Failures::Many, &[]);
    let (calls, _) = judged(&mut d, &env());
    assert_eq!(calls.len(), 0);
    assert_eq!(d.finding(5, enumerated.number), Some(&enumerated));
    assert_eq!(d.finding(5, many.number), Some(&many));
}
#[test]
fn enumerated_runs_give_only_unheld_ids_to_the_oldest_many() {
    let mut d = one();
    kept(&mut d, signature(&[A], false), FindingKind::Red);
    let mut first = kept(&mut d, signature(&[], true), FindingKind::Red);
    first.title = b"many tests fail in browser since R3 landed".as_slice().into();
    first.blame = Some(FindingBlame {
        moved: None,
        strength: BlameStrength::Owned,
        landings: Box::new([BlamedLanding {
            commit: NEXT_TIP,
            goals: Box::new([BlamedGoal { goal: 91, milestone: None, person: None }]),
        }]),
    });
    outputs(&mut d, Event::Restore { record: Stored::Finding(first.clone()) });
    let second = kept(&mut d, signature(&[], true), FindingKind::Red);
    next(&mut d, failures(&[A, B, C]), &[A, B, C]);
    let (calls, _) = judged(&mut d, &env());
    assert_eq!(calls.len(), 0);
    let given = d.finding(5, first.number).unwrap();
    assert_eq!(given.signature, Some(signature(&[C, B], false)));
    assert_eq!(given.blame.as_ref().unwrap().strength, BlameStrength::Suspected);
    assert_eq!(given.title.as_ref(), b"2 tests fail in browser since R3 landed");
    assert_eq!(given.body, first.body);
    assert_eq!(d.finding(5, second.number).unwrap().cleared, Some(FindingCleared { run: 2, at: env().wall }));
    let third = kept(&mut d, signature(&[], true), FindingKind::Red);
    let fourth = kept(&mut d, signature(&[], true), FindingKind::Red);
    let waiting = VerdictFiling {
        project: 5,
        kind: FindingKind::Red,
        severity: Severity::High,
        title: b"Waiting".as_slice().into(),
        body: b"Whole".as_slice().into(),
        signature: signature(&[OWN], false),
        blame: Some(FindingBlame { moved: None, strength: BlameStrength::Unblamed, landings: Box::new([]) }),
        follow: Some(9),
    };
    d.tiers.get_mut(&(REPO, TIER.into())).unwrap().waiting = Box::new([waiting.clone()]);
    next(&mut d, failures(&[A, B, OWN]), &[A, B, OWN]);
    assert_eq!(judged(&mut d, &env()).0, [waiting], "waiting holds its id before it is filed");
    for number in [third.number, fourth.number] {
        assert_eq!(d.finding(5, number).unwrap().cleared, Some(FindingCleared { run: 3, at: env().wall }));
    }
}
#[test]
fn red_filings_wait_whole_in_order_and_only_eight_are_kept() {
    let mut d = one();
    let mut env = env();
    env.limits.findings_per_project = 1;
    let block = kept(&mut d, signature(&[b"occupied"], false), FindingKind::Qa);
    let mut expected = Vec::new();
    for n in 0_usize..10 {
        let id = format!("failure::{n}");
        next(&mut d, failures(&[id.as_bytes()]), &[id.as_bytes(), b"flake"]);
        let (calls, out) = judged(&mut d, &env);
        assert!(calls.is_empty(), "full new table");
        if n == 9 {
            let out = posted(&mut d, out.into_boxed_slice());
            let Some(TierDecision::Done { words }) = choice(&out) else { panic!("completed report") };
            assert!(words.ends_with(
                b"8 red filings wait for room in the project's findings.\n2 red filings were dropped for want of room."
            ));
        }
        if n < 8 {
            expected.push(row(&d).waiting.last().unwrap().clone());
        }
        assert_eq!(row(&d).waiting.len(), (n + 1).min(8));
        assert_eq!(row(&d).unkept, u32::try_from((n + 1).saturating_sub(8)).unwrap());
    }
    assert_eq!(row(&d).waiting.as_ref(), expected);
    next(&mut d, failures(&[b"failure::8"]), &[b"failure::8"]);
    assert_eq!(judged(&mut d, &env).0.len(), 0);
    assert_eq!(row(&d).unkept, 3, "the ninth filing holds no id, so it is tried again");
    next(&mut d, failures(&[b"failure::0"]), &[b"failure::0"]);
    assert_eq!(judged(&mut d, &env).0.len(), 0);
    assert_eq!(row(&d).waiting.as_ref(), expected, "waiting ids hold across runs");
    // A backlog is no room either. Routing or clearing makes the same place erasable.
    for state in [FindingState::Backlog, FindingState::Routed] {
        let mut block = block.clone();
        block.state = state;
        outputs(&mut d, Event::Restore { record: Stored::Finding(block) });
        next(&mut d, failures(&[b"failure::0"]), &[b"failure::0"]);
        let (calls, _) = judged(&mut d, &env);
        if state == FindingState::Backlog {
            assert_eq!(calls.len(), 0);
        } else {
            assert_eq!(calls, [expected[0].clone()], "oldest before run's filings, with its blame");
            take(&mut d, &env, calls);
            assert_eq!(row(&d).waiting.as_ref(), &expected[1..]);
        }
    }
    // The next enumerated run clears that holder, admitting the next waiting filing first.
    next(&mut d, failures(&[b"own"]), &[b"own"]);
    let (calls, _) = judged(&mut d, &env);
    assert_eq!(calls, [expected[1].clone()]);
    take(&mut d, &env, calls);
    assert_eq!(row(&d).waiting.last().unwrap().signature.ids.as_ref(), [Box::from(b"own".as_slice())]);
}

pub(crate) fn random_runs(seed: u64) -> (Vec<Finding>, Box<[VerdictFiling]>, u32) {
    use alloc::collections::BTreeSet;
    let mut rng = skein_lib::Rng::new(seed);
    let mut d = one();
    let mut env = env();
    env.limits.findings_per_project = 3;
    for turn in 0..32 {
        let mut failed = Vec::new();
        let mut originally = Vec::new();
        let mut previous = Vec::new();
        for id in [A, B, C, OWN, b"extra::a", b"extra::b"] {
            if rng.chance(500) {
                originally.push(id);
                if rng.chance(750) {
                    failed.push(id);
                }
            }
            if rng.chance(500) {
                previous.push(id);
            }
        }
        let failing = if rng.chance(150) { p::Failures::Many } else { failures(&failed) };
        next(&mut d, failing.clone(), &originally);
        let range: Box<[p::Landing]> = (0..rng.below(4))
            .map(|n| p::Landing {
                commit: [u8::try_from(n.checked_add(20).unwrap()).unwrap(); 32],
                key: [u8::try_from(n.checked_add(20).unwrap()).unwrap(); 32],
            })
            .collect();
        let tier = d.tiers.get_mut(&(REPO, TIER.into())).unwrap();
        tier.record.run.as_mut().unwrap().landings = range;
        previous.sort();
        tier.record.failing = failures(&previous);
        d.landing.projects.get_mut(&5).unwrap().2.verdict.suspects = p::Suspects::TellAll;
        env.wall = Wall::from_nanos(turn);
        let (calls, _) = judged(&mut d, &env);
        take(&mut d, &env, calls);
        let selected = d.findings(5).nth(usize::try_from(rng.below(4)).unwrap()).map(|(_, finding)| finding.clone());
        if let Some(mut finding) = selected {
            let act = match rng.below(3) {
                0 => Triage::Route(91),
                1 => Triage::Backlog,
                _ => Triage::Drop(b"triaged".as_slice().into()),
            };
            if !finding.state.ended() {
                let write =
                    d.prepare_triage_finding(TriageFinding { project: 5, number: finding.number, act }).unwrap();
                finding = write.row.clone();
                d.commit_finding(&env.limits, write, &mut Queue::uncapped());
            }
            assert_eq!(d.finding(5, finding.number), Some(&finding));
        }
        let mut held = BTreeSet::new();
        for (_, finding) in d.findings(5) {
            if finding.uncleared_red() {
                for id in &finding.signature.as_ref().unwrap().ids {
                    assert!(held.insert(id.clone()), "duplicate id seed {seed} turn {turn}");
                }
            }
        }
        for filing in &row(&d).waiting {
            for id in &filing.signature.ids {
                assert!(held.insert(id.clone()), "waiting duplicate seed {seed} turn {turn}");
            }
        }
        assert!(row(&d).waiting.len() <= 8, "seed {seed} turn {turn}");
        if rng.chance(250) {
            let tier = row(&d).clone();
            let findings: Vec<_> = d.findings(5).map(|(_, finding)| finding.clone()).collect();
            d = one();
            for finding in findings {
                outputs(&mut d, Event::Restore { record: Stored::Finding(finding) });
            }
            outputs(&mut d, Event::Restore { record: Stored::Tier(tier) });
        }
    }
    (d.findings(5).map(|(_, finding)| finding.clone()).collect(), row(&d).waiting.clone(), row(&d).unkept)
}

#[test]
fn enumerating_a_many_finding_keeps_moved_blame() {
    use crate::findings::BlameMove;
    let mut d = one();
    let mut first = kept(&mut d, signature(&[], true), FindingKind::Red);
    first.title = b"many tests fail in browser since R3 landed".as_slice().into();
    first.blame = Some(FindingBlame {
        strength: BlameStrength::Moved,
        landings: Box::new([BlamedLanding { commit: NEXT_TIP, goals: Box::new([]) }]),
        moved: Some(BlameMove { by: Filer::Party(8), why: b"the landing log".as_slice().into() }),
    });
    outputs(&mut d, Event::Restore { record: Stored::Finding(first.clone()) });
    next(&mut d, failures(&[B, C]), &[B, C]);
    let (calls, _) = judged(&mut d, &env());
    assert_eq!(calls.len(), 0);
    let given = d.finding(5, first.number).expect("enumerated finding");
    assert_eq!(given.signature, Some(signature(&[C, B], false)));
    assert_eq!(given.blame, first.blame);
    assert_eq!(given.body, first.body);
    assert_eq!(given.title.as_ref(), b"2 tests fail in browser since R3 landed");
}
