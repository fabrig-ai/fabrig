//! A gate's known-flake rerun (domain/landing-regate.md, 2, 4, 5).
use super::*;
use skein_forge::api::{self, Answer, Op, Read, Write};
use skein_lib::List;
use temper_engine_domain_forge_tiers::Format;

const TEST: &[u8] = b"test a::t x";

fn file(bytes: &[u8]) -> Answer {
    Answer::File { head: fixtures::HEAD, path: b"report".as_slice().into(), bytes: bytes.into(), truncated: false }
}
fn seed_flake(d: &mut Domain) {
    let result = d
        .prepare_check_flake(
            &LIMITS,
            env().wall,
            Filer::Task(51),
            5,
            FindingSignature {
                repository: REPO,
                source: b"check".as_slice().into(),
                ids: Box::new([TEST.into()]),
                first: Box::new([]),
                many: false,
            },
            b"flake".as_slice().into(),
            b"reproduce".as_slice().into(),
        )
        .unwrap();
    let CheckFlake::Filed(write) = result else { panic!("new finding") };
    d.commit_finding(&LIMITS, write, &mut Queue::uncapped());
}
fn gate() -> (Domain, ReceiptKey, api::Run) {
    let (mut d, key, run) = reporting_failed_gate(Format::Lines);
    seed_flake(&mut d);
    let row = d.landing.rows.get_mut(&61).unwrap();
    row.landing.members[0].gates[0] = policy::GateProof {
        proof: policy::Proof::Dispatched { key: key.key, try_: 1 },
        posted: policy::Posted::Pending,
        faults: 0,
    };
    row.entries = Box::new([LandingEntry {
        preparation: Some(row.landing.preparation),
        base: Some(fixtures::BASE),
        entry: 9,
        member: 0,
        gate: 0,
        operation: Operation::Dispatch,
        run: None,
    }]);
    let purpose = purpose::EffectPurpose::Landing {
        task: 61,
        member: 0,
        gate: 0,
        operation: 1,
        preparation: row.landing.preparation,
        attempt: 1,
    }
    .identity()
    .unwrap();
    let flight = d.landing.flights.get_mut(&key).unwrap();
    flight.purpose = purpose;
    flight.inputs = Box::new([
        api::Input { name: b"temper_commit".as_slice().into(), value: commit_text(&fixtures::HEAD) },
        api::Input { name: b"temper_key".as_slice().into(), value: commit_text(&key.key) },
    ]);
    let entry = skein_forge_client::Entry {
        number: 9,
        task: 61,
        repository: REPO,
        effect: skein_forge_client::Effect {
            write: Write::Dispatch {
                workflow: flight.workflow.clone(),
                branch: flight.branch.clone(),
                inputs: flight.inputs.clone(),
            },
            condition: skein_forge_client::Condition::None,
        },
        start: Some(skein_forge_client::Position { at: Time::from_nanos(10), comment: 0, review: 0, review_page: 1 }),
        attempt: None,
        failures: 0,
        queued: false,
    };
    // Retain the dispatch's start window without handing its write to the client again.
    outputs(&mut d, Event::Restore { record: Stored::Entry { entry, held: true } });
    (d, key, run)
}
fn begin(d: &mut Domain, entry: u64) {
    outputs(
        d,
        Event::StepLanding {
            task: 61,
            entry,
            heard: LandingHeard {
                job: None,
                outcomes: Box::new([]),
                dependencies: Box::new([]),
                held: Box::new([]),
                failure: Box::new([]),
                preparation: Some(policy::Preparation::Idle),
                kept: Box::new([]),
                released: false,
                cancelled: false,
            },
        },
    );
}
/// Finish one coherent gather, checking the attempts of every job/report it reads.
fn gather(d: &mut Domain, run: &api::Run, report: Result<Answer, api::Error>) -> (policy::Decision, Option<u64>) {
    for _ in 0..24_u32 {
        let (call, op) = fixtures::next_call(d);
        let answer = match op {
            Op::Read(Read::Branch { branch }) => {
                Ok(Answer::Commit(if branch.as_ref() == b"main" { fixtures::BASE } else { fixtures::HEAD }))
            }
            Op::Read(Read::Commits { .. }) => Ok(Answer::Commits { commits: Box::new([]), more: false }),
            Op::Read(Read::Pull { .. }) => Ok(Answer::Pull(fixtures::open_pull())),
            Op::Read(Read::Reviews { .. }) => Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }),
            Op::Read(Read::Runs { .. }) => Ok(Answer::Runs { runs: Box::new([run.clone()]), more: false }),
            Op::Read(Read::Run { run: id }) => {
                assert_eq!(id, run.id, "follow the rerun, not a fresh listing");
                Ok(Answer::Run(run.clone()))
            }
            Op::Read(Read::Job { attempt, .. }) => {
                let mut read_attempt = None;
                for job in &run.jobs {
                    if job.id == attempt.job {
                        read_attempt = Some(job.attempt);
                    }
                }
                assert_eq!((attempt.run, Some(attempt.attempt)), (run.id, read_attempt));
                Err(api::Error::Missing)
            }
            Op::Read(Read::Artifact { run: id, .. }) => {
                assert_eq!(id, run.id);
                report.clone()
            }
            Op::Read(_) | Op::Write(_) => panic!("unexpected gather {op:?}"),
        };
        for request in fixtures::answer(d, call, answer) {
            if let Request::LandingDecision { decision, entry, .. } = request {
                return (decision, entry);
            }
        }
    }
    panic!("gather must decide");
}
fn send_write(d: &mut Domain, entry: u64, expected: Write, answer: Result<Answer, api::Error>) {
    outputs(d, Event::Committed { entry });
    let (call, op) = fixtures::next_call(d);
    assert!(matches!(op, Op::Read(Read::Items { .. })), "write's clock: {op:?}");
    fixtures::answer(d, call, Ok(Answer::Items { items: Box::new([]), more: false, cut: false, now: env().now }));
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Write(expected));
    fixtures::answer(d, call, answer);
}
fn rerun(d: &mut Domain, entry: u64, expected: api::JobAttempt) {
    send_write(d, entry, Write::Rerun { attempt: expected }, Ok(Answer::Done));
}
fn passed(run: &api::Run) -> api::Run {
    let mut run = run.clone();
    run.conclusion = Some(api::Conclusion::Success);
    for job in &mut run.jobs {
        job.attempt = job.attempt.checked_add(1).unwrap();
        job.conclusion = Some(api::Conclusion::Success);
        job.step = None;
    }
    run
}
fn first_rerun(d: &mut Domain, key: &ReceiptKey, run: &api::Run) {
    let (decision, entry) = gather(d, run, Ok(file(TEST)));
    assert_eq!(entry, Some(91));
    assert_eq!(decision, policy::Decision::Retry { member: 0, gate: 0, key: key.key, try_: 1 });
    assert!(d.receipt(key).is_none(), "the first red has no receipt");
    let row = d.landing.rows.get(&61).unwrap();
    assert_eq!(row.flake_reruns.len(), 1);
    assert_eq!((row.flake_reruns[0].run, row.flake_reruns[0].findings.as_ref()), (run.id, [1].as_slice()));
    assert_eq!(row.landing.members[0].gates[0].faults, 0);
    assert_eq!(d.finding(5, 1).unwrap().seen, Some(1));
}

#[test]
fn known_flake_reruns_before_receipt_and_green_counts_it_once() {
    let (mut d, key, run) = gate();
    let dispatched = d.landing.flights.get(&key).unwrap().clone();
    first_rerun(&mut d, &key, &run);
    rerun(&mut d, 91, api::JobAttempt { head: run.commit, run: run.id, job: 9, attempt: 2 });
    begin(&mut d, 92);
    assert!(matches!(gather(&mut d, &run, Ok(file(TEST))), (policy::Decision::Wait { .. }, None)));
    assert!(d.receipt(&key).is_none(), "the forge may still show the failed attempt after accepting a rerun");
    begin(&mut d, 93);
    let (decision, _) = gather(&mut d, &passed(&run), Ok(file(TEST)));
    assert!(d.receipt(&key).unwrap().passed);
    assert!(!matches!(
        decision,
        policy::Decision::Repair { .. } | policy::Decision::Retry { .. } | policy::Decision::Hold(_)
    ));
    assert_eq!(d.finding(5, 1).unwrap().seen, Some(2));
    begin(&mut d, 94);
    gather(&mut d, &passed(&run), Ok(file(TEST)));
    assert_eq!(d.finding(5, 1).unwrap().seen, Some(2), "a reused receipt does not count again");
    d.landing.receipts.remove(&key);
    d.landing.flights.insert(key.clone(), dispatched).unwrap();
    let mut fresh = passed(&run);
    fresh.id = 77;
    begin(&mut d, 95);
    gather(&mut d, &fresh, Ok(file(TEST)));
    assert_eq!(d.receipt(&key).unwrap().run.id, 77);
    assert_eq!(d.finding(5, 1).unwrap().seen, Some(2), "a later dispatch after eviction is not this flake rerun");
}

#[test]
fn a_second_red_reads_new_attempts_and_repairs_without_a_third_run() {
    let (mut d, key, mut run) = gate();
    d.finding_rows_mut(&LIMITS).get_mut(&(5, 1)).unwrap().signature.as_mut().unwrap().ids =
        Box::new([TEST.into(), b"test a::t y".as_slice().into()]);
    first_rerun(&mut d, &key, &run);
    rerun(&mut d, 91, api::JobAttempt { head: run.commit, run: run.id, job: 9, attempt: 2 });
    run.jobs[0].attempt = 3;
    begin(&mut d, 92);
    let (decision, _) = gather(&mut d, &run, Ok(file(b"test a::t y")));
    let receipt = d.receipt(&key).unwrap();
    assert!(!receipt.passed);
    assert_eq!(receipt.jobs[0].attempt, 3);
    assert_eq!(
        receipt.report,
        ReceiptReport::Tests(Box::new([ReceiptTest { key: b"test a::t y".as_slice().into(), fuzzy: false }]))
    );
    assert_eq!(
        decision,
        policy::Decision::Leave { member: 0, why: policy::Leave::Failed { gate: 0, run: run.number } }
    );
    assert_eq!(
        d.landing.rows.get(&61).unwrap().asks[0],
        Some(temper_engine_domain_forge_change::Ask::Repair { gate: 0, candidate: run.commit, below: None })
    );
    assert_eq!(d.finding(5, 1).unwrap().seen, Some(1));
    assert!(!d.landing.flights.contains_key(&key));
    for (_, entry) in &d.entries {
        assert!(!matches!(entry.entry.effect.write, Write::Rerun { .. }));
    }
}

#[test]
fn incomplete_or_unmatched_reports_publish_the_failure_without_rerunning() {
    for case in 0..11_u32 {
        let (mut d, key, run) = gate();
        let mut report = Ok(file(TEST));
        let finding = d.finding_rows_mut(&LIMITS).get_mut(&(5, 1)).unwrap();
        match case {
            0 => report = Ok(file(b"test a::t x\ntest a::t unknown")),
            1 => finding.state = FindingState::Dropped,
            2 => finding.signature.as_mut().unwrap().repository.repository = 99,
            3 => finding.signature.as_mut().unwrap().source = b"other-gate".as_slice().into(),
            4 => report = Err(api::Error::Forbidden),
            5 => {
                let Ok(Answer::File { truncated, .. }) = &mut report else { unreachable!() };
                *truncated = true;
            }
            6 => {
                let mut tests = List::with_capacity(4096);
                for number in 0..65_u32 {
                    for part in
                        [b"test binary t".as_slice(), skein_lib::Decimal::of(u64::from(number)).as_bytes(), b"\n"]
                    {
                        for byte in part {
                            tests.push(*byte).unwrap();
                        }
                    }
                }
                report = Ok(file(tests.as_slice()));
            }
            7 => report = Ok(file(b"")),
            8 => finding.kind = FindingKind::Red,
            9 => finding.signature = None,
            10 => {
                let project = d.landing.projects.get_mut(&5).unwrap();
                project.1[0].gates[0].report = None;
            }
            _ => unreachable!(),
        }
        gather(&mut d, &run, report);
        assert!(!d.receipt(&key).unwrap().passed, "case {case}");
        assert!(d.landing.rows.get(&61).unwrap().flake_reruns.is_empty(), "case {case}");
    }
}

/// Restore the landing, outbox and findings, then reconstruct only derived flights.
fn restore_gate(d: &Domain, key: &ReceiptKey) -> Domain {
    let mut restored = connected_member_landing();
    outputs(
        &mut restored,
        Event::Restore { record: Stored::Change(Box::new(d.changes.get(&fixtures::CHANGE).unwrap().clone())) },
    );
    restored.landing.projects.insert(5, d.landing.projects.get(&5).unwrap().clone()).unwrap();
    restored.landing.main_keys.insert((REPO, b"gate".as_slice().into()), (fixtures::BASE, key.key)).unwrap();
    outputs(&mut restored, Event::Restore { record: Stored::Landing(d.landing.rows.get(&61).unwrap().clone()) });
    for (_, row) in d.findings(5) {
        outputs(&mut restored, Event::Restore { record: Stored::Finding(row.clone()) });
    }
    for (_, entry) in &d.entries {
        // These writes have settled; their retained windows, not another write, are followed.
        outputs(&mut restored, Event::Restore { record: Stored::Entry { entry: entry.entry.clone(), held: true } });
    }
    landing::rebuild(&mut restored);
    restored
}

#[test]
fn restarting_a_multi_job_flake_rerun_drains_only_pending_jobs_and_follows_its_run() {
    let (mut d, key, mut run) = gate();
    run.jobs = Box::new([run.jobs[0].clone(), api::RunJob { id: 10, ..run.jobs[0].clone() }]);
    first_rerun(&mut d, &key, &run);
    assert_eq!(d.landing.rows.get(&61).unwrap().flake_reruns[0].pending.len(), 1);
    rerun(&mut d, 91, api::JobAttempt { head: run.commit, run: run.id, job: 9, attempt: 2 });
    run.jobs[0].attempt = 3;
    d.landing.rows.get_mut(&61).unwrap().landing.preparation = 1;
    let mut restored = restore_gate(&d, &key);
    assert_eq!(restored.landing.flights.get(&key).unwrap().run.as_ref().unwrap().id, run.id);
    begin(&mut restored, 92);
    let (decision, entry) = gather(&mut restored, &run, Ok(file(TEST)));
    assert_eq!((decision, entry), (policy::Decision::Retry { member: 0, gate: 0, key: key.key, try_: 1 }, Some(92)));
    assert_eq!(restored.landing.rows.get(&61).unwrap().flake_reruns.len(), 1);
    rerun(&mut restored, 92, api::JobAttempt { head: run.commit, run: run.id, job: 10, attempt: 2 });
    let mut restored = restore_gate(&restored, &key);
    assert_eq!(
        restored.landing.flights.get(&key).unwrap().entry,
        92,
        "a later round still follows the newest rerun of the old key"
    );
    assert!(restored.landing.flights.get(&key).unwrap().writes.is_empty());
    assert_eq!(restored.landing.flights.get(&key).unwrap().read(), Some(Read::Run { run: run.id }));
    begin(&mut restored, 93);
    assert!(matches!(gather(&mut restored, &run, Ok(file(TEST))), (policy::Decision::Wait { .. }, None)));
    assert!(restored.receipt(&key).is_none(), "restart still fences the second job's old terminal attempt");
    begin(&mut restored, 94);
    gather(&mut restored, &passed(&run), Ok(file(TEST)));
    assert!(restored.receipt(&key).unwrap().passed);
    assert_eq!(restored.finding(5, 1).unwrap().seen, Some(2));
}

#[test]
fn a_refused_flake_rerun_dispatches_the_same_inputs_without_spending_fault_retries() {
    let (mut d, key, mut run) = gate();
    first_rerun(&mut d, &key, &run);
    let flight = d.landing.flights.get(&key).unwrap();
    let dispatch = Write::Dispatch {
        workflow: flight.workflow.clone(),
        branch: flight.branch.clone(),
        inputs: flight.inputs.clone(),
    };
    send_write(
        &mut d,
        91,
        Write::Rerun { attempt: api::JobAttempt { head: run.commit, run: run.id, job: 9, attempt: 2 } },
        Err(api::Error::Refused),
    );
    assert!(d.landing.rows.get(&61).unwrap().flake_reruns[0].redispatch);
    let mut d = restore_gate(&d, &key);
    begin(&mut d, 92);
    let (decision, entry) = gather(&mut d, &run, Ok(file(TEST)));
    assert_eq!((decision, entry), (policy::Decision::Retry { member: 0, gate: 0, key: key.key, try_: 1 }, Some(92)));
    send_write(&mut d, 92, dispatch, Ok(Answer::Done));
    run.id = 77;
    run.number = 78;
    run.created = env().now;
    begin(&mut d, 93);
    gather(&mut d, &passed(&run), Ok(file(TEST)));
    assert_eq!(d.receipt(&key).unwrap().run.id, 77);
    assert_eq!(d.finding(5, 1).unwrap().seen, Some(2));
    assert_eq!(d.landing.rows.get(&61).unwrap().landing.members[0].gates[0].faults, 0);
}

#[test]
fn a_runner_fault_in_the_flake_rerun_spends_only_the_runner_retry() {
    let (mut d, key, mut run) = gate();
    first_rerun(&mut d, &key, &run);
    rerun(&mut d, 91, api::JobAttempt { head: run.commit, run: run.id, job: 9, attempt: 2 });
    run.jobs[0].attempt = 3;
    run.jobs[0].step.as_mut().unwrap().number = 0;
    begin(&mut d, 92);
    let (decision, entry) = gather(&mut d, &run, Ok(file(TEST)));
    assert_eq!((decision, entry), (policy::Decision::Retry { member: 0, gate: 0, key: key.key, try_: 2 }, Some(92)));
    assert!(d.receipt(&key).is_none());
    assert_eq!(d.landing.rows.get(&61).unwrap().flake_reruns.len(), 1);
    assert_eq!(d.landing.rows.get(&61).unwrap().landing.members[0].gates[0].faults, 1);
    rerun(&mut d, 92, api::JobAttempt { head: run.commit, run: run.id, job: 9, attempt: 3 });
    begin(&mut d, 93);
    gather(&mut d, &passed(&run), Ok(file(TEST)));
    assert!(d.receipt(&key).unwrap().passed);
    assert_eq!(d.finding(5, 1).unwrap().seen, Some(2));
}

#[test]
fn gate_flakes_match_each_test_at_the_bound_and_choose_each_finding_once() {
    let (mut d, key, run) = gate();
    let mut tests = List::with_capacity(65);
    for number in 0..65_u32 {
        tests
            .push(ReceiptTest {
                key: skein_lib::bytes::concat(&[b"test binary ", skein_lib::Decimal::of(u64::from(number)).as_bytes()]),
                fuzzy: false,
            })
            .unwrap();
    }
    let mut ids = List::with_capacity(64);
    for test in tests.as_slice().get(..64).unwrap() {
        ids.push(test.key.clone()).unwrap();
    }
    let first = d.finding_rows_mut(&LIMITS).get_mut(&(5, 1)).unwrap();
    first.state = FindingState::Routed;
    let mut ids = ids.into_boxed();
    ids.sort();
    first.signature.as_mut().unwrap().ids = ids;
    let mut second = first.clone();
    second.number = 2;
    second.state = FindingState::Backlog;
    second.signature.as_mut().unwrap().source = b"gate".as_slice().into();
    second.signature.as_mut().unwrap().ids = Box::new([tests.as_slice().last().unwrap().key.clone()]);
    d.finding_rows_mut(&LIMITS).insert((5, 2), second).unwrap();
    assert_eq!(
        d.gate_flakes(5, REPO, b"gate", tests.as_slice().get(..64).unwrap()),
        Some(Box::from([1_u32].as_slice()))
    );
    assert_eq!(d.gate_flakes(5, REPO, b"gate", tests.as_slice()), None, "65 fully named tests still exceed the bound");
    assert_eq!(d.gate_flakes(5, REPO, b"gate", &[]), None);
    let pair = Box::new([tests.as_slice().first().unwrap().clone(), tests.as_slice().last().unwrap().clone()]);
    assert_eq!(d.gate_flakes(5, REPO, b"gate", pair.as_ref()), Some(Box::from([1_u32, 2].as_slice())));
    assert_eq!(
        d.gate_flakes(6, REPO, b"gate", pair.as_ref()),
        None,
        "another project's flakes do not name this gate's tests"
    );
    let report = skein_lib::bytes::concat(&[
        &tests.as_slice()[0].key,
        b"\n",
        &tests.as_slice()[1].key,
        b"\n",
        &tests.as_slice()[64].key,
    ]);
    let (decision, entry) = gather(&mut d, &run, Ok(file(&report)));
    assert_eq!((decision, entry), (policy::Decision::Retry { member: 0, gate: 0, key: key.key, try_: 1 }, Some(91)));
    assert_eq!(d.landing.rows.get(&61).unwrap().flake_reruns[0].findings.as_ref(), &[1, 2]);
    rerun(&mut d, 91, api::JobAttempt { head: run.commit, run: run.id, job: 9, attempt: 2 });
    begin(&mut d, 92);
    gather(&mut d, &passed(&run), Ok(file(&report)));
    assert!(d.receipt(&key).unwrap().passed);
    assert_eq!(d.finding(5, 1).unwrap().seen, Some(2), "two tests count their shared finding once");
    assert_eq!(d.finding(5, 2).unwrap().seen, Some(2), "the gate-sourced finding counts too");
}
