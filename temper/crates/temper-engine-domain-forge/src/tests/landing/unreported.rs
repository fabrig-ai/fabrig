//! Gate judgement and the failed receipt boundary (domain/landing-proof.md, 2.4).
use super::*;
use crate::history::{Event as HistoryEvent, GateVerdict, What};
use skein_forge::api::{self, Answer, Op, Read, Write};
use skein_lib::List;
use temper_engine_domain_forge_tiers::Format;

const FIRST: u64 = 91;
const SECOND: u64 = 92;
const ONE_RETRY: u32 = 1;
const TWO_RETRIES: u32 = 2;

fn gate(members: u16) -> (Domain, ReceiptKey, api::Run) {
    let (mut d, key, run) = prepared_gate(members, Format::Lines, true, TWO_RETRIES);
    begin_reporting_gate(&mut d, FIRST);
    (d, key, run)
}

fn prepared_gate(members: u16, format: Format, blocking: bool, retries: u32) -> (Domain, ReceiptKey, api::Run) {
    let (mut d, key, run) = reporting_gate(format);
    let project = d.landing.projects.get_mut(&5).unwrap();
    project.5.gate_retries = retries;
    project.1[0].gates[0].blocking = blocking;
    let mut row = d.landing(61).unwrap().clone();
    row.landing.round.branch = b"temper/1/l61".as_slice().into();
    row.landing.members[0].gates[0] = policy::GateProof {
        proof: policy::Proof::Dispatched { key: key.key, try_: 1 },
        posted: policy::Posted::Pending,
        faults: 0,
    };
    if members == 2 {
        let mut earlier = row.landing.members[0].clone();
        earlier.change = 52;
        earlier.unit = 1;
        earlier.pull = 3;
        earlier.candidate = Some([6; 32]);
        earlier.gates[0].proof = policy::Proof::Keyed { key: [6; 32] };
        row.landing.members = Box::new([earlier, row.landing.members[0].clone()]);

        let mut change = d.changes.get(&fixtures::CHANGE).unwrap().clone();
        change.task = 52;
        change.branch = b"temper/52".as_slice().into();
        change.pull = Some(3);
        d.changes.insert(52, change).unwrap();
    }
    let top = members.checked_sub(1).unwrap();
    row.follows.as_mut().unwrap()[0].entry.member = top;
    let preparation = row.landing.preparation;
    if members == 2 {
        restore_landing(&mut d, row);
    } else {
        assert!(d.landing.remember(row).is_some());
    }
    let flight = d.landing.flights.get_mut(&key).unwrap();
    flight.purpose =
        purpose::EffectPurpose::Landing { task: 61, member: top, gate: 0, operation: 1, preparation, attempt: 1 }
            .identity()
            .unwrap();
    flight.entry = 9;
    flight.inputs = Box::new([
        api::Input { name: b"temper_commit".as_slice().into(), value: commit_text(&run.commit) },
        api::Input { name: b"temper_key".as_slice().into(), value: commit_text(&key.key) },
    ]);
    (d, key, run)
}

fn read_answer(op: Op, run: &api::Run, report: &Result<Answer, api::Error>) -> Result<Answer, api::Error> {
    match op {
        Op::Read(Read::Branch { branch }) => {
            Ok(Answer::Commit(if branch.as_ref() == b"main" { fixtures::BASE } else { fixtures::HEAD }))
        }
        Op::Read(Read::Commits { .. }) => Ok(Answer::Commits { commits: Box::new([]), more: false }),
        Op::Read(Read::Pull { number, .. }) => {
            let mut pull = fixtures::open_pull();
            pull.number = number;
            if number == 3 {
                pull.head = b"temper/52".as_slice().into();
            }
            Ok(Answer::Pull(pull))
        }
        Op::Read(Read::Reviews { .. }) => Ok(Answer::Reviews { reviews: Box::new([]), more: false, cut: false }),
        Op::Read(Read::Runs { .. }) => Ok(Answer::Runs { runs: Box::new([run.clone()]), more: false }),
        Op::Read(Read::Run { run: id }) => {
            assert_eq!(id, run.id);
            Ok(Answer::Run(run.clone()))
        }
        Op::Read(Read::Job { attempt, .. }) => {
            for job in &run.jobs {
                if job.id == attempt.job {
                    assert_eq!(
                        attempt,
                        api::JobAttempt { head: run.commit, run: run.id, job: job.id, attempt: job.attempt }
                    );
                    return Err(api::Error::Missing);
                }
            }
            panic!("unexpected job {}", attempt.job)
        }
        Op::Read(Read::Artifact { run: id, name, .. }) => {
            assert_eq!((id, name.as_ref()), (run.id, b"report".as_slice()));
            report.clone()
        }
        Op::Read(_) | Op::Write(_) => panic!("unexpected report gather {op:?}"),
    }
}

fn gather(
    d: &mut Domain,
    run: &api::Run,
    report: Result<Answer, api::Error>,
) -> (policy::Decision, Box<[HistoryEvent]>) {
    let requests = gather_requests(d, run, report);
    let mut history = List::uncapped();
    let mut decision = None;
    for request in requests {
        if let Request::Save { record: Stored::HistoryEvent(event), .. } = &request {
            history.push(event.clone()).unwrap();
        }
        if let Request::LandingDecision { decision: next, .. } = request {
            decision = Some(next);
        }
    }
    (decision.expect("report gather decides"), history.into_boxed())
}

pub(super) fn gather_requests(d: &mut Domain, run: &api::Run, report: Result<Answer, api::Error>) -> Box<[Request]> {
    let mut requests = List::uncapped();
    for _ in 0..32_u32 {
        let (call, op) = fixtures::next_call(d);
        let answer = read_answer(op, run, &report);
        let mut decided = false;
        for request in fixtures::answer(d, call, answer) {
            decided |= matches!(request, Request::LandingDecision { .. });
            requests.push(request).unwrap();
        }
        if decided {
            return requests.into_boxed();
        }
    }
    panic!("report gather decides");
}

fn unblamed(d: &Domain, key: &ReceiptKey) {
    assert!(d.receipt(key).is_none(), "a gate without a report has no failed receipt");
    let row = d.landing.rows.get(&61).unwrap();
    assert!(row.landing.round.search.is_none(), "no search for a runner fault");
    for member in &row.landing.members {
        assert!(member.left.is_none(), "no member leaves or is blamed");
    }
    for member in &row.landing.members {
        let change = d.change(member.change).unwrap();
        assert!(change.asks.is_empty(), "no repair ask");
        assert_eq!(change.change.repairs, 0, "no repair spent");
    }
    assert!(row.flake_reruns.is_empty(), "an absent report cannot start a flake rerun");
    for (_, entry) in &d.entries {
        if let Write::Dispatch { inputs, .. } = &entry.entry.effect.write {
            for input in inputs {
                assert_ne!(input.name.as_ref(), b"temper_probe", "no probe dispatched");
            }
        }
    }
}

fn ended(history: &[HistoryEvent], members: u16, run: u64, attempt: u32, verdict: GateVerdict) {
    for change in [fixtures::CHANGE, 52].into_iter().take(usize::from(members)) {
        let mut ends = 0_u32;
        for event in history {
            if event.change == change && matches!(event.what, What::GateEnded { .. }) {
                ends = ends.checked_add(1).unwrap();
                assert_eq!(event.what, What::GateEnded { gate: 0, key: [7; 32], verdict });
                assert_eq!((event.operation, event.attempt), (run, attempt));
            }
        }
        assert_eq!(ends, 1, "one end per run attempt and member");
    }
}

fn rerun(d: &mut Domain, entry: u64, run: &api::Run) {
    outputs(d, Event::Committed { entry });
    let attempt = api::JobAttempt { head: run.commit, run: run.id, job: 9, attempt: run.jobs[0].attempt };
    for _ in 0..32_u32 {
        let (call, op) = fixtures::next_call(d);
        match op {
            Op::Write(write) => {
                assert_eq!(write, Write::Rerun { attempt });
                fixtures::answer(d, call, Ok(Answer::Done));
                return;
            }
            Op::Read(Read::Items { .. }) => {
                fixtures::answer(
                    d,
                    call,
                    Ok(Answer::Items { items: Box::new([]), more: false, cut: false, now: env().now }),
                );
            }
            Op::Read(_) => {
                let answer = read_answer(op, run, &Err(api::Error::Missing));
                fixtures::answer(d, call, answer);
            }
        }
    }
    panic!("rerun written");
}

fn report(run: &api::Run) -> Answer {
    file(run.commit, b"test binary broken\n")
}

fn file(head: api::Commit, bytes: &[u8]) -> Answer {
    Answer::File { head, path: b"report".as_slice().into(), bytes: bytes.into(), truncated: false }
}

fn dispatch_probe(d: &mut Domain, run: &api::Run) {
    begin_reporting_gate(d, 92);
    let (decision, _) = gather(d, run, Ok(report(run)));
    assert_eq!(
        decision,
        policy::Decision::Post { member: 1, gate: 0, posting: policy::Posting::Ran { passed: false } }
    );
    commit_gate_effect(d, 92);
    begin_reporting_gate(d, 93);
    let (decision, _) = gather(d, run, Ok(report(run)));
    assert_eq!(decision, policy::Decision::Dispatch { member: 0, gate: 0, key: [6; 32], try_: 1 });
}

#[test]
fn an_uploaded_gate_failure_keeps_its_test_and_dispatches_a_search_probe() {
    let (mut d, key, run) = gate(2);
    let (decision, history) = gather(&mut d, &run, Ok(report(&run)));
    let receipt = d.receipt(&key).expect("uploaded gate verdict makes a failed receipt");
    assert!(!receipt.passed);
    assert_eq!(
        receipt.failures,
        Some(temper_engine_domain_forge_tiers::Failures::Tests(Box::new([b"test binary broken".as_slice().into()])))
    );
    ended(&history, 2, run.id, 2, GateVerdict::Failed);
    assert_eq!(decision, policy::Decision::None);
    assert!(d.landing.rows.get(&61).unwrap().landing.round.search.is_some());
    dispatch_probe(&mut d, &run);
    let Write::Dispatch { inputs, .. } = &d.entries.get(&93).unwrap().entry.effect.write else {
        panic!("probe dispatched")
    };
    let mut probe = false;
    let mut tests = false;
    for input in inputs {
        probe |= input.name.as_ref() == b"temper_probe";
        tests |= input.name.as_ref() == b"temper_tests" && input.value.as_ref() == b"test binary broken\n";
    }
    assert!(probe && tests, "probe dispatched with the failed test's id");
}

#[test]
fn a_selected_probe_without_a_report_retries_without_moving_the_search_or_blaming_a_member() {
    let (mut d, key, mut run) = gate(2);
    gather(&mut d, &run, Ok(report(&run)));
    dispatch_probe(&mut d, &run);
    commit_gate_effect(&mut d, 93);
    let search = d.landing.rows.get(&61).unwrap().landing.round.search.clone();
    run.id = 8;
    run.number = 9;
    run.commit = [6; 32];
    run.created = env().now;
    let probe_key = ReceiptKey { key: [6; 32], ..key.clone() };
    begin_reporting_gate(&mut d, 94);
    let (decision, _) = gather(&mut d, &run, Err(api::Error::Missing));
    assert!(d.receipt(&probe_key).is_none(), "an unreported probe has no receipt");
    assert_eq!(decision, policy::Decision::Retry { member: 0, gate: 0, key: probe_key.key, try_: 2 });
    assert_eq!(
        d.entries.get(&94).unwrap().entry.effect.write,
        Write::Rerun { attempt: api::JobAttempt { head: run.commit, run: 8, job: 9, attempt: 2 } }
    );
    let row = d.landing.rows.get(&61).unwrap();
    assert_eq!(row.landing.round.search, search, "no verdict from a missing report");
    for member in &row.landing.members {
        assert!(member.left.is_none());
    }
    for member in &row.landing.members {
        assert!(d.change(member.change).unwrap().asks.is_empty(), "no repair ask");
    }
    assert_eq!(d.receipt(&key).unwrap().run.id, 7, "the top's real red is unchanged");
}

#[test]
fn restarting_after_an_absent_report_finds_the_unsent_rerun_without_a_receipt_or_repair() {
    let (mut d, key, run) = gate(2);
    let (decision, history) = gather(&mut d, &run, Err(api::Error::Missing));
    assert_eq!(decision, policy::Decision::Retry { member: 1, gate: 0, key: key.key, try_: 2 });
    ended(&history, 2, run.id, 2, GateVerdict::Fault);
    let mut restored = landing_fixture();
    for (_, change) in &d.changes {
        outputs(&mut restored, Event::Restore { record: Stored::Change(Box::new(change.clone())) });
    }
    restored.landing.projects.insert(5, d.landing.projects.get(&5).unwrap().clone()).unwrap();
    outputs(&mut restored, Event::Restore { record: Stored::Landing(d.landing.rows.get(&61).unwrap().clone()) });
    for (_, entry) in &d.entries {
        outputs(
            &mut restored,
            Event::Restore { record: Stored::Entry { entry: entry.entry.clone(), held: entry.held } },
        );
    }
    restored.landing.main_keys.insert((REPO, b"gate".as_slice().into()), (fixtures::BASE, key.key)).unwrap();
    fixtures::connect(&mut restored);
    outputs(&mut restored, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut restored, Event::CatchUp);
    unblamed(&restored, &key);
    assert_eq!(restored.landing.flights.get(&key).unwrap().entry, 91, "find the rerun kept before its write");

    rerun(&mut restored, 91, &run);
    unblamed(&restored, &key);
}

#[test]
fn unreported_gate_retries_then_holds_without_receipt_search_or_repair() {
    for members in [2, 1] {
        let (mut d, key, mut run) = gate(members);
        for retry in 0..=2_u16 {
            let (decision, history) = gather(&mut d, &run, Err(api::Error::Missing));
            unblamed(&d, &key);
            assert!(
                matches!(decision, policy::Decision::Retry { .. } | policy::Decision::Hold(_)),
                "expected runner fault, got {decision:?}"
            );
            ended(&history, members, run.id, run.jobs[0].attempt, GateVerdict::Fault);
            if retry == 2 {
                assert_eq!(
                    decision,
                    policy::Decision::Hold(policy::Hold::Faults { member: members.checked_sub(1).unwrap(), gate: 0 })
                );
                hold_names(
                    &d,
                    &["gate gate", "check broke", "report was missing", "change wasn't judged", "retries: 2"],
                );
            } else {
                assert_eq!(
                    decision,
                    policy::Decision::Retry {
                        member: members.checked_sub(1).unwrap(),
                        gate: 0,
                        key: key.key,
                        try_: retry.checked_add(2).unwrap()
                    }
                );
                let entry = 91_u64.checked_add(u64::from(retry)).unwrap();
                rerun(&mut d, entry, &run);
                run.jobs[0].attempt = run.jobs[0].attempt.checked_add(1).unwrap();
                begin_reporting_gate(&mut d, entry.checked_add(1).unwrap());
            }
        }
    }
}

fn hold_names(d: &Domain, names: &[&str]) {
    let words = d.landing_hold_words(61).expect("gate held with a reason");
    for name in names {
        assert!(skein_lib::bytes::find(&words, name.as_bytes()).is_some(), "missing {name:?}: {words:?}");
    }
}

fn before_and_after_retry(
    d: &mut Domain,
    key: &ReceiptKey,
    mut run: api::Run,
    answer: Result<Answer, api::Error>,
    redispatch: bool,
) -> policy::Decision {
    begin_reporting_gate(d, FIRST);
    let (decision, history) = gather(d, &run, answer.clone());
    assert_eq!(decision, policy::Decision::Retry { member: 0, gate: 0, key: key.key, try_: 2 });
    ended(&history, 1, run.id, run.jobs[0].attempt, GateVerdict::Fault);
    unblamed(d, key);
    if redispatch {
        assert!(matches!(d.entries.get(&FIRST).unwrap().entry.effect.write, Write::Dispatch { .. }));
        commit_gate_effect(d, FIRST);
        run.id = run.id.checked_add(1).unwrap();
        run.number = run.number.checked_add(1).unwrap();
        run.created = env().now;
    } else {
        rerun(d, FIRST, &run);
        run.jobs[0].attempt = run.jobs[0].attempt.checked_add(1).unwrap();
    }
    begin_reporting_gate(d, SECOND);
    let (decision, history) = gather(d, &run, answer);
    ended(&history, 1, run.id, run.jobs[0].attempt, GateVerdict::Fault);
    unblamed(d, key);
    decision
}

#[test]
fn an_unreadable_report_retries_once_then_holds_without_a_receipt() {
    for answer in [Ok(file(fixtures::HEAD, b"not XML")), Err(api::Error::Forbidden)] {
        let (mut d, key, run) = prepared_gate(1, Format::Junit, true, ONE_RETRY);
        let decision = before_and_after_retry(&mut d, &key, run, answer, false);
        assert_eq!(decision, policy::Decision::Hold(policy::Hold::Faults { member: 0, gate: 0 }));
        hold_names(&d, &["gate gate", "check broke", "report could not be read", "change wasn't judged", "retries: 1"]);
    }
}

#[test]
fn a_missing_report_with_no_runner_fact_holds_as_broke() {
    let (mut d, key, run) = prepared_gate(1, Format::Lines, true, ONE_RETRY);
    let decision = before_and_after_retry(&mut d, &key, run, Err(api::Error::Missing), false);
    assert_eq!(decision, policy::Decision::Hold(policy::Hold::Faults { member: 0, gate: 0 }));
    hold_names(&d, &["gate gate", "check broke", "report was missing", "change wasn't judged", "retries: 1"]);
}

fn add_job(run: &mut api::Run, name: &[u8], conclusion: api::Conclusion, step: Option<api::Step>) {
    const SECOND_JOB: u64 = 10;
    let job =
        api::RunJob { id: SECOND_JOB, name: name.into(), conclusion: Some(conclusion), step, ..run.jobs[0].clone() };
    run.jobs = Box::new([run.jobs[0].clone(), job]);
}

#[test]
fn a_cancelled_job_names_the_runner_instead_of_broke() {
    let (mut d, key, mut run) = prepared_gate(1, Format::Lines, true, ONE_RETRY);
    add_job(&mut run, b"build matrix", api::Conclusion::Cancelled, None);
    let decision = before_and_after_retry(&mut d, &key, run, Err(api::Error::Missing), true);
    assert_eq!(decision, policy::Decision::Hold(policy::Hold::Faults { member: 0, gate: 0 }));
    hold_names(
        &d,
        &["gate gate", "runner's fault", "job build matrix was cancelled", "change wasn't judged", "retries: 1"],
    );
}

fn refused(d: &Domain, key: &ReceiptKey, decision: policy::Decision, report: &ReceiptReport) {
    let receipt = d.receipt(key).expect("refusal has a receipt");
    assert!(!receipt.passed);
    assert_eq!(&receipt.report, report);
    assert_eq!(
        decision,
        policy::Decision::Leave { member: 0, why: policy::Leave::Failed { gate: 0, run: receipt.run.number } }
    );
    let change = d.change(fixtures::CHANGE).unwrap();
    assert!(matches!(change.asks.first().unwrap().ask, temper_engine_domain_forge_change::Ask::Repair { gate: 0, .. }));
}

#[test]
fn a_reported_red_stands_beside_another_jobs_setup_fault() {
    let (mut d, key, mut run) = prepared_gate(1, Format::Lines, true, ONE_RETRY);
    add_job(
        &mut run,
        b"matrix",
        api::Conclusion::Failure,
        Some(api::Step { number: 0, name: b"set-up".as_slice().into() }),
    );
    begin_reporting_gate(&mut d, FIRST);
    let (decision, _) = gather(&mut d, &run, Ok(report(&run)));
    refused(
        &d,
        &key,
        decision,
        &ReceiptReport::Tests(ReceiptTests {
            tests: Box::new([ReceiptTest { key: b"test binary broken".as_slice().into(), matched: false }]),
            matched: None,
            rest: None,
        }),
    );
}

#[test]
fn a_reportless_gate_can_refuse_by_its_workflow_step() {
    let (mut d, key, run) = prepared_gate(1, Format::Lines, true, ONE_RETRY);
    d.landing.projects.get_mut(&5).unwrap().1[0].gates[0].report = None;
    begin_reporting_gate(&mut d, FIRST);
    let (decision, _) = gather(&mut d, &run, Err(api::Error::Missing));
    refused(&d, &key, decision, &ReceiptReport::Unset);
}

#[test]
fn an_empty_advisory_gate_report_does_not_hold_proof() {
    let (mut d, key, run) = prepared_gate(1, Format::Lines, false, ONE_RETRY);
    let answer = Ok(file(run.commit, b""));
    let decision = before_and_after_retry(&mut d, &key, run, answer, false);
    assert_eq!(decision, policy::Decision::Proved);
}

fn stored_gate(report: ReceiptReport) -> (Domain, ReceiptKey, api::Run) {
    let (mut d, key, run) = prepared_gate(1, Format::Lines, true, ONE_RETRY);
    d.landing.flights.remove(&key);
    let row = d.landing.rows.get_mut(&61).unwrap();
    row.follows = None;
    row.landing.members[0].gates[0].proof = policy::Proof::Keyed { key: key.key };
    row.landing.members[0].gates[0].posted = policy::Posted::Nothing;
    let receipt = Receipt {
        report,
        base: Some(fixtures::BASE),
        jobs: Box::new([]),
        logs: Box::new([]),
        failures: None,
        passed: false,
        run: ReceiptRun { id: run.id, number: run.number, url: run.url.clone() },
        candidate: run.commit,
        ended: env().wall,
    };
    outputs(&mut d, Event::Restore { record: Stored::Receipt { key: key.clone(), receipt } });
    begin_reporting_gate(&mut d, FIRST);
    (d, key, run)
}

#[test]
fn a_retained_gate_receipt_naming_nothing_is_erased_and_redispatched() {
    for report in
        [ReceiptReport::Tests(ReceiptTests { tests: Box::new([]), matched: None, rest: None }), ReceiptReport::Unread]
    {
        let (mut d, key, run) = stored_gate(report);
        let requests = gather_requests(&mut d, &run, Err(api::Error::Missing));
        let mut erased = false;
        let mut dispatched = false;
        for request in &requests {
            erased |= matches!(request, Request::Erase { key: Key::Receipt(saved) } if *saved == key);
            dispatched |= matches!(request, Request::LandingDecision {
                decision: policy::Decision::Dispatch { member: 0, gate: 0, key: dispatched, try_: 1 }, ..
            } if *dispatched == key.key);
        }
        assert!(erased);
        assert!(!d.landing.receipts.contains_key(&key));
        assert!(dispatched);
        assert!(matches!(d.entries.get(&FIRST).unwrap().entry.effect.write, Write::Dispatch { .. }));
    }
}

#[test]
fn retained_reportless_and_many_failures_are_reused_as_red() {
    for report in [ReceiptReport::Unset, ReceiptReport::Many { bound: LIMITS.tiers.failing_tests }] {
        let (mut d, key, run) = stored_gate(report.clone());
        let (decision, _) = gather(&mut d, &run, Err(api::Error::Missing));
        refused(&d, &key, decision, &report);
    }
}
