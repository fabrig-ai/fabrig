//! Failed reports and partial workflows (temper/docs/design/domain/after-landing.md, 4–5.2).
use super::*;
const FAILURES: &[u8] = b"suite.first\nsuite.second\n";
const PROBE_OPEN: u64 = 301;
const PROBE_DISPATCH: u64 = 302;
fn tests() -> Box<[p::TestId]> {
    Box::new([b"suite.first".as_slice().into(), b"suite.second".as_slice().into()])
}
fn failures() -> p::Failures {
    p::Failures::Tests(tests())
}
fn range(landings: Box<[p::Landing]>) -> Domain {
    let mut d = dispatched();
    let row = d.tiers.get_mut(&(REPO, TIER.into())).unwrap();
    row.record.judged = Some(p::Judged { tip: BASE, key: BASE });
    let run = row.record.run.as_mut().unwrap();
    run.tip = landings.last().unwrap().commit;
    run.key = landings.last().unwrap().key;
    run.landings = landings;
    row.flight = None;
    d.tier_ops.follows.remove(&(REPO, TIER.into()));
    tier_step(&mut d, DISPATCH, false);
    made(&mut d, DISPATCH, skein_forge_client::Made::Set);
    d
}
fn one() -> Domain {
    range(Box::new([p::Landing { commit: NEXT_TIP, key: NEXT_TIP }]))
}
fn three() -> Domain {
    range(Box::new([
        p::Landing { commit: HEAD, key: BASE },
        p::Landing { commit: NEXT_TIP, key: NEXT_TIP },
        p::Landing { commit: LAST_TIP, key: LAST_TIP },
    ]))
}
fn report(d: &mut Domain, bytes: &[u8], truncated: bool) {
    answer(
        d,
        api::Answer::File {
            head: workflow(d, api::Conclusion::Failure, Some(1)).commit,
            path: b"failed.txt".as_slice().into(),
            bytes: bytes.into(),
            truncated,
        },
    );
}
fn red(d: &mut Domain) {
    completed(d, api::Conclusion::Failure, Some(1));
    report(d, FAILURES, false);
}
fn send(d: &mut Domain, entry: u64) -> api::Write {
    tier_step(d, entry, false);
    let write = d.entries.get(&entry).unwrap().entry.effect.write.clone();
    made(
        d,
        entry,
        match &write {
            api::Write::CreateBranch { commit, .. } => skein_forge_client::Made::Branch(*commit),
            _ => skein_forge_client::Made::Set,
        },
    );
    write
}
fn judgement(d: &Domain) -> p::Verdict {
    let row = row(d);
    let result = row.result.as_ref().expect("final observation");
    let outcome = if result.passed { p::Outcome::Passed } else { p::Outcome::Failed(result.failures.clone().unwrap()) };
    let p::Next::Judged(verdict) = p::step(
        &row.record,
        row.record.run.as_ref().unwrap(),
        outcome,
        &AfterLanding {
            verdict: p::Settings { retries: RETRIES, suspects: p::Suspects::Bisect },
            bound: Duration::ZERO,
        }
        .verdict,
    ) else {
        panic!("final judgement")
    };
    verdict
}
fn confirmed(d: &mut Domain) {
    red(d);
    send(d, DISPATCH + 1);
    red(d);
}
fn failed_rerun() -> Domain {
    let mut d = one();
    completed(&mut d, api::Conclusion::Failure, Some(1));
    report(&mut d, b"design-listed\n", false);
    send(&mut d, DISPATCH + 1);
    let mut run = workflow(&d, api::Conclusion::Failure, Some(1));
    run.id = RUN + 1;
    run.number = 2;
    run.url = b"https://forge.example/run/72".as_slice().into();
    answer(&mut d, api::Answer::Runs { runs: Box::new([run.clone()]), more: false });
    answer(&mut d, api::Answer::Run(run));
    d
}
fn report_error(d: &mut Domain, error: api::Error) -> Box<[Request]> {
    d.tier_ops.reads.insert(READ, crate::tiers::Read { tier: (REPO, TIER.into()), baseline: false }).unwrap();
    let mut out = Queue::uncapped();
    crate::tiers::answered(d, &env(), READ, Err(error), &mut out);
    collect(out)
}
#[test]
fn a_busy_rerun_report_is_read_again_without_repeating_tests() {
    let mut d = failed_rerun();
    let saved = row(&d).clone();
    let out = report_error(&mut d, api::Error::Busy);
    assert_eq!(row(&d).record.run, saved.record.run, "Busy must not consume a runner-fault allowance");
    assert_eq!(row(&d).flight.as_ref().unwrap().report, saved.flight.as_ref().unwrap().report);
    let at = env().now.saturating_add(LIMITS.client.poll);
    assert_eq!(row(&d).flight.as_ref().unwrap().next, Some(at));
    assert!(!out.contains(&Request::WakeProcedure { task: TIER_TASK }));
    let mut clock = env();
    clock.now = at;
    let mut out = Queue::uncapped();
    crate::tiers::fire(&mut d, &clock, &mut out);
    assert!(collect(out).contains(&Request::WakeProcedure { task: TIER_TASK }));
    tier_step(&mut d, DISPATCH + 2, false);
    assert!(!d.entries.contains_key(&(DISPATCH + 2)), "the report poll must not dispatch or rerun tests");
    let follow = d.tier_ops.follows.get(&(REPO, TIER.into())).unwrap();
    assert!(follow.writes.is_empty());
    assert_eq!(row(&d).flight.as_ref().unwrap().report.as_ref().unwrap().id, RUN + 1);
    report(&mut d, b"design-listed\n", false);
    assert_eq!(row(&d).record.run.as_ref().unwrap().faults, 0);
    assert_eq!(
        judgement(&d).landings.as_ref(),
        &[p::Blame::Owns(p::Failures::Tests(Box::new([b"design-listed".as_slice().into()])))]
    );
    let out = finish(&mut d);
    let Some(TierDecision::Done { words }) = choice(&out) else { panic!("completed red") };
    assert!(words.starts_with(b"Tier browser run 1: red, 1 test failing"));
}
#[test]
fn a_missing_rerun_report_records_why_before_retrying() {
    let mut d = failed_rerun();
    let out = report_error(&mut d, api::Error::Missing);
    let reason = b"forge run 2 https://forge.example/run/72; artifact report: target missing\n";
    let saved = out
        .iter()
        .find_map(|request| match request {
            Request::Save { record: Stored::Tier(row), .. }
                if skein_lib::bytes::find(&row.observations, reason).is_some() =>
            {
                Some(row.clone())
            }
            _ => None,
        })
        .expect("save the report failure before retrying");
    assert!(skein_lib::bytes::find(&saved.observations, b"Runner fault: Failed-test rerun: commit ").is_some());
    assert_eq!(saved.record.run.as_ref().unwrap().faults, 1);
    let mut cold = restoring(None);
    outputs(&mut cold, Event::Restore { record: Stored::Tier(saved) });
    outputs(&mut cold, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut cold, Event::ReadAfresh);
    let words = cold.tier_execution_words(TIER_TASK).unwrap();
    assert!(skein_lib::bytes::find(&words, reason).is_some());
    send(&mut cold, DISPATCH + 2);
    completed(&mut cold, api::Conclusion::Success, None);
    let out = finish(&mut cold);
    let projection = out
        .iter()
        .find_map(|request| match request {
            Request::TierDecision { decision: TierDecision::Done { .. }, projection, .. } => projection.as_deref(),
            _ => None,
        })
        .expect("completed task keeps its execution facts");
    assert!(skein_lib::bytes::find(projection, reason).is_some());
}
#[test]
fn an_unavailable_artifact_is_read_again_without_a_fault_or_retry() {
    for error in [api::Error::Unavailable, api::Error::Timeout] {
        let mut d = one();
        let mut env = env();
        env.limits.client.read_attempts = 1;
        completed(&mut d, api::Conclusion::Failure, Some(1));
        tier_step(&mut d, DISPATCH + 1, false);
        let mut artifact = None;
        for _ in 0..LIMITS.client.pending {
            let (call, op) = fixtures::next_at(&mut d, &env);
            match op {
                api::Op::Read(api::Read::Artifact { .. }) => {
                    artifact = Some((call, op));
                    break;
                }
                api::Op::Read(api::Read::Tree { .. }) => {
                    fixtures::answer_at(
                        &mut d,
                        &env,
                        call,
                        Ok(api::Answer::Tree { entries: Box::new([]), more: false }),
                    );
                }
                _ => panic!("artifact read: {op:?}"),
            }
        }
        let (call, op) = artifact.expect("artifact read");
        let saved = row(&d).record.run.clone();
        fixtures::answer_at(&mut d, &env, call, Err(error));
        assert_eq!(row(&d).record.run, saved, "an outage spends no retry");
        assert_eq!(row(&d).held, None);
        fixtures::advance(&mut d, &mut env, CLIENT.backoff);
        let mut retried = None;
        for _ in 0..LIMITS.client.pending {
            let (call, asked) = fixtures::next_at(&mut d, &env);
            if asked == op {
                retried = Some(call);
                break;
            }
            if let api::Op::Read(api::Read::Items { .. }) = asked {
                fixtures::answer_at(
                    &mut d,
                    &env,
                    call,
                    Ok(api::Answer::Items { cut: false, items: Box::new([]), more: false, now: env.now }),
                );
            } else if let api::Op::Read(api::Read::Branch { .. }) = asked {
                fixtures::answer_at(&mut d, &env, call, Ok(api::Answer::Commit(BASE)));
            } else {
                panic!("artifact re-ask: {asked:?}");
            }
        }
        let again = retried.expect("the same artifact is asked again");
        let head = workflow(&d, api::Conclusion::Failure, Some(1)).commit;
        fixtures::answer_at(
            &mut d,
            &env,
            again,
            Ok(api::Answer::File {
                head,
                path: b"failed.txt".as_slice().into(),
                bytes: FAILURES.into(),
                truncated: false,
            }),
        );
        assert_eq!(row(&d).record.run.as_ref().expect("rerun").faults, 0);
        assert_eq!(row(&d).record.run.as_ref().expect("rerun").part, p::Part::Rerun { failed: tests() });
    }
}

#[test]
fn a_workflow_fault_records_why_before_retrying() {
    let mut d = one();
    completed(&mut d, api::Conclusion::Failure, Some(0));
    let words = d.tier_execution_words(TIER_TASK).unwrap();
    assert!(skein_lib::bytes::find(&words, b"Runner fault: Judged run: commit ").is_some());
    assert!(skein_lib::bytes::find(&words, b"forge run 1 https://forge.example/run/71; set-up\n").is_some());
    assert_eq!(row(&d).record.run.as_ref().unwrap().faults, 1);
}
#[test]
fn failure_reads_the_named_artifact_before_rerunning() {
    let mut d = one();
    completed(&mut d, api::Conclusion::Failure, Some(1));
    tier_step(&mut d, DISPATCH + 1, false);
    let mut artifact = None;
    for _ in 0..LIMITS.client.pending {
        let (call, op) = fixtures::next_call(&mut d);
        match op {
            api::Op::Read(api::Read::Artifact { .. }) => {
                artifact = Some(op);
                break;
            }
            api::Op::Read(api::Read::Tree { .. }) => {
                fixtures::answer(&mut d, call, Ok(api::Answer::Tree { entries: Box::new([]), more: false }));
            }
            _ => panic!("unexpected call: {op:?}"),
        }
    }
    let op = artifact.expect("artifact read");
    // The fixture client caps the 16 * (256 + CRLF) report at its answer bound.
    assert_eq!(op, api::Op::Read(api::Read::Artifact { run: RUN, name: b"report".as_slice().into(), max_bytes: 4096 }));
    report(&mut d, FAILURES, false);
    assert_eq!(row(&d).record.run.as_ref().unwrap().part, p::Part::Rerun { failed: tests() });
    assert_eq!(row(&d).flight.as_ref().unwrap().report, None);
}
#[test]
fn junit_failure_and_error_ids_are_selected() {
    let mut d = one();
    d.landing.projects.get_mut(&5).unwrap().1[0].tiers[0].format = p::Format::Junit;
    completed(&mut d, api::Conclusion::Failure, Some(1));
    report(&mut d, b"<testsuite><testcase classname='suite' name='first'><failure/></testcase><testcase classname='suite' name='second'><error/></testcase></testsuite>", false);
    assert_eq!(row(&d).record.run.as_ref().unwrap().part, p::Part::Rerun { failed: tests() });
}
#[test]
fn an_empty_report_is_a_fault() {
    let mut d = one();
    completed(&mut d, api::Conclusion::Failure, Some(1));
    report(&mut d, b"", false);
    assert_eq!(row(&d).record.run.as_ref().unwrap().faults, 1);
    assert!(d.landing.receipts.is_empty());
}
#[test]
fn malformed_junit_is_a_fault() {
    let mut d = one();
    d.landing.projects.get_mut(&5).unwrap().1[0].tiers[0].format = p::Format::Junit;
    completed(&mut d, api::Conclusion::Failure, Some(1));
    report(&mut d, b"<testsuite>", false);
    assert_eq!(row(&d).record.run.as_ref().unwrap().faults, 1);
}
#[test]
fn selected_reruns_keep_the_tip_and_make_no_receipts() {
    let mut d = one();
    red(&mut d);
    let receipt = d.landing.receipts.iter().next().unwrap().1.clone();
    let write = send(&mut d, DISPATCH + 1);
    let api::Write::Dispatch { branch, inputs, .. } = write else { panic!("selected dispatch") };
    assert_eq!(branch.as_ref(), b"temper/tier/browser.1");
    assert_eq!(inputs[2], api::Input { name: b"temper_tests".as_slice().into(), value: FAILURES.into() });
    completed(&mut d, api::Conclusion::Success, None);
    assert_eq!(judgement(&d).landings.as_ref(), &[p::Blame::Cleared]);
    finish(&mut d);
    assert_eq!(d.landing.receipts.iter().next().unwrap().1, &receipt);
    assert!(!receipt.passed);
    assert_eq!(receipt.failures, Some(failures()));
    assert_eq!(
        row(&d).record.flakes.as_ref(),
        &[
            p::Flake { test: b"suite.first".as_slice().into(), count: 1 },
            p::Flake { test: b"suite.second".as_slice().into(), count: 1 },
        ]
    );
}
#[test]
fn inherited_failures_clear_the_new_range() {
    let mut d = one();
    d.tiers.get_mut(&(REPO, TIER.into())).unwrap().record.failing = failures();
    confirmed(&mut d);
    assert_eq!(judgement(&d).landings.as_ref(), &[p::Blame::Cleared]);
    assert_eq!(judgement(&d).failing, failures());
}
#[test]
fn probes_skip_unchanged_keys_and_blame_the_first_failure() {
    let mut d = three();
    confirmed(&mut d);
    assert_eq!(
        send(&mut d, PROBE_OPEN),
        api::Write::CreateBranch { branch: b"temper/tier/browser.1.1".as_slice().into(), commit: NEXT_TIP }
    );
    tier_step(&mut d, PROBE_DISPATCH, false);
    assert_eq!(d.describe_tier_effect(TIER_TASK, PROBE_DISPATCH).unwrap().2.kind, 16);
    let api::Write::Dispatch { inputs, .. } = &d.entries.get(&PROBE_DISPATCH).unwrap().entry.effect.write else {
        panic!("probe dispatch")
    };
    assert_eq!(inputs[2].value.as_ref(), FAILURES);
    made(&mut d, PROBE_DISPATCH, skein_forge_client::Made::Set);
    red(&mut d);
    assert_eq!(judgement(&d).landings.as_ref(), &[p::Blame::Cleared, p::Blame::Owns(failures()), p::Blame::Cleared]);
    assert_eq!(
        send(&mut d, PROBE_DISPATCH + 1),
        api::Write::DeleteBranch { branch: b"temper/tier/browser.1.1".as_slice().into() }
    );
    assert!(row(&d).probe.is_none());
    finish(&mut d);
    assert!(row(&d).record.run.is_none());
    assert_eq!(d.landing.receipts.len(), 1);
}
#[test]
fn a_missing_probe_workflow_names_the_probes_commit_when_held() {
    let mut d = three();
    confirmed(&mut d);
    send(&mut d, PROBE_OPEN);
    tier_step(&mut d, PROBE_DISPATCH, false);
    let kind = b"workflow tier.yml not found at 0606060606060606060606060606060606060606060606060606060606060606";
    for fault in 0..=RETRIES {
        let entry = PROBE_DISPATCH + u64::from(fault);
        outcome(&mut d, entry, skein_forge_client::Outcome::Failed(api::Error::Missing));
        let out = tier_step(&mut d, entry + 1, false);
        if fault == RETRIES {
            assert_eq!(
                choice(&out),
                Some(&TierDecision::Held {
                    words: join_words(&[b"Tier browser run 1 held after 3 runner faults, last: ", kind]),
                })
            );
        }
    }
    let words = d.tier_execution_words(TIER_TASK).unwrap();
    let line = join_words(&[
        b"Runner fault: Bisection probe: commit 0606060606060606060606060606060606060606060606060606060606060606, branch temper/tier/browser.1.1; ",
        kind, b"\n",
    ]);
    assert_eq!(skein_lib::bytes::count(&words, &line, 4), 3);
    assert_eq!(row(&d).record.run.as_ref().unwrap().tip, LAST_TIP);
}
#[test]
fn many_failures_probe_the_whole_tier() {
    let mut d = three();
    completed(&mut d, api::Conclusion::Failure, Some(1));
    report(&mut d, b"suite.first", true);
    send(&mut d, PROBE_OPEN);
    let api::Write::Dispatch { inputs, .. } = send(&mut d, PROBE_DISPATCH) else { panic!("whole probe") };
    assert_eq!(inputs.len(), 2);
    completed(&mut d, api::Conclusion::Success, None);
    assert_eq!(
        judgement(&d).landings.as_ref(),
        &[p::Blame::Cleared, p::Blame::Cleared, p::Blame::Owns(p::Failures::Many)]
    );
}
#[test]
fn probe_branches_are_released_before_the_next_probe() {
    let mut d = range(Box::new([
        p::Landing { commit: HEAD, key: HEAD },
        p::Landing { commit: NEXT_TIP, key: NEXT_TIP },
        p::Landing { commit: LAST_TIP, key: LAST_TIP },
    ]));
    confirmed(&mut d);
    send(&mut d, PROBE_OPEN);
    send(&mut d, PROBE_DISPATCH);
    completed(&mut d, api::Conclusion::Success, None);
    assert_eq!(
        send(&mut d, PROBE_DISPATCH + 1),
        api::Write::DeleteBranch { branch: b"temper/tier/browser.1.0".as_slice().into() }
    );
    assert_eq!(
        send(&mut d, PROBE_DISPATCH + 2),
        api::Write::CreateBranch { branch: b"temper/tier/browser.1.1".as_slice().into(), commit: NEXT_TIP }
    );
}
#[test]
fn a_failed_part_reads_its_report_after_restart() {
    let mut d = one();
    completed(&mut d, api::Conclusion::Failure, Some(1));
    let mut cold = restoring(None);
    outputs(&mut cold, Event::Restore { record: Stored::Tier(row(&d).clone()) });
    outputs(&mut cold, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut cold, Event::ReadAfresh);
    report(&mut cold, FAILURES, false);
    assert_eq!(row(&cold).record.run.as_ref().unwrap().part, p::Part::Rerun { failed: tests() });
}
#[test]
fn selected_rerun_and_probe_flights_survive_restart() {
    let mut d = three();
    red(&mut d);
    send(&mut d, DISPATCH + 1);
    for probe in [false, true] {
        if probe {
            red(&mut d);
            send(&mut d, PROBE_OPEN);
            send(&mut d, PROBE_DISPATCH);
        }
        let saved = row(&d).clone();
        let mut cold = restoring(None);
        outputs(&mut cold, Event::Restore { record: Stored::Tier(saved.clone()) });
        outputs(&mut cold, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        outputs(&mut cold, Event::ReadAfresh);
        let follow = cold.tier_ops.follows.get(&(REPO, TIER.into())).unwrap();
        assert_eq!(follow.commit, if probe { NEXT_TIP } else { LAST_TIP });
        assert_eq!(follow.inputs[2].value.as_ref(), FAILURES);
        assert!(follow.writes.is_empty());
        completed(&mut cold, api::Conclusion::Success, None);
        assert!(row(&cold).result.is_some());
    }
}
#[test]
fn a_failed_receipt_starts_probes_without_a_selected_rerun() {
    let mut d = three();
    red(&mut d);
    let receipt = d.landing.receipts.iter().next().unwrap().1.clone();
    let kept = d.tiers.get_mut(&(REPO, TIER.into())).unwrap();
    kept.branch = TierBranch::Absent;
    kept.record.run.as_mut().unwrap().part = p::Part::Whole;
    kept.flight = None;
    kept.result = Some(receipt);
    kept.received = true;
    d.tier_ops.follows.remove(&(REPO, TIER.into()));
    tier_step(&mut d, OPEN, false);
    assert_eq!(d.describe_tier_effect(TIER_TASK, OPEN).unwrap().2.state, LAST_TIP);
    made(&mut d, OPEN, skein_forge_client::Made::Branch(LAST_TIP));
    send(&mut d, PROBE_OPEN);
    send(&mut d, PROBE_DISPATCH);
    red(&mut d);
    assert_eq!(judgement(&d).landings.as_ref(), &[p::Blame::Cleared, p::Blame::Owns(failures()), p::Blame::Cleared]);
}
#[test]
fn probe_effects_name_their_commit_resource_and_purpose() {
    use crate::purpose::EffectPurpose;
    let mut d = three();
    confirmed(&mut d);
    tier_step(&mut d, PROBE_OPEN, false);
    let (_, what, effect) = d.describe_tier_effect(TIER_TASK, PROBE_OPEN).unwrap();
    assert_eq!((effect.kind, effect.state), (9, NEXT_TIP));
    assert_eq!(
        what,
        What::Branch(Box::new([
            b"temper".as_slice().into(),
            b"tier".as_slice().into(),
            b"browser.1.1".as_slice().into()
        ]))
    );
    assert_eq!(row(&d).purpose, Some(EffectPurpose::Tier { repository: REPO, tier: 0, run: 1, operation: 3, part: 3 }));
    made(&mut d, PROBE_OPEN, skein_forge_client::Made::Branch(NEXT_TIP));
    tier_step(&mut d, PROBE_DISPATCH, false);
    assert_eq!(row(&d).purpose, Some(EffectPurpose::Tier { repository: REPO, tier: 0, run: 1, operation: 4, part: 3 }));
    made(&mut d, PROBE_DISPATCH, skein_forge_client::Made::Set);
    red(&mut d);
    let out = tier_step(&mut d, PROBE_DISPATCH + 1, false);
    assert_eq!(row(&d).purpose, Some(EffectPurpose::Tier { repository: REPO, tier: 0, run: 1, operation: 5, part: 3 }));
    assert!(out.iter().any(|request| match request {
        Request::Save { release: Some(entry), .. } => *entry == PROBE_DISPATCH + 1,
        _ => false,
    }));
    assert!(d.describe_tier_effect(TIER_TASK, PROBE_DISPATCH + 1).is_none());
}

#[test]
fn a_completed_verdict_pairs_each_failed_id_with_its_own_landing() {
    let mut d = one();
    confirmed(&mut d);
    let out = finish(&mut d);
    let Some(TierDecision::Done { words }) = choice(&out) else { panic!("completed red") };
    let commit = commit_text(&NEXT_TIP);
    let mut expected = Vec::new();
    for part in [
        b"Tier browser run 1: red, 2 tests failing, blamed on 1 landing\n".as_slice(),
        &commit,
        b" owns: \"suite.first\"\n",
        &commit,
        b" owns: \"suite.second\"",
    ] {
        expected.extend_from_slice(part);
    }
    assert_eq!(words.as_ref(), expected.as_slice());
}
