//! A landing gate with no uploaded verdict is the runner's (domain/landing-proof.md, 2.4).
use super::*;
use crate::history::{Event as HistoryEvent, GateVerdict, What};
use skein_forge::api::{self, Answer, Op, Read, Write};
use skein_lib::List;
use temper_engine_domain_forge_tiers::Format;

fn gate(members: u16) -> (Domain, ReceiptKey, api::Run) {
    let (mut d, key, run) = reporting_gate(Format::Lines);
    d.landing.projects.get_mut(&5).unwrap().5.gate_retries = 2;
    let row = d.landing.rows.get_mut(&61).unwrap();
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
        row.asks = Box::new([None, None]);
        let mut change = d.changes.get(&fixtures::CHANGE).unwrap().clone();
        change.task = 52;
        change.branch = b"temper/52".as_slice().into();
        change.pull = Some(3);
        d.changes.insert(52, change).unwrap();
    }
    let top = members.checked_sub(1).unwrap();
    row.entries = Box::new([LandingEntry {
        preparation: Some(row.landing.preparation),
        base: Some(fixtures::BASE),
        entry: 9,
        member: top,
        gate: 0,
        operation: Operation::Dispatch,
        run: None,
    }]);
    let flight = d.landing.flights.get_mut(&key).unwrap();
    flight.purpose = purpose::EffectPurpose::Landing {
        task: 61,
        member: top,
        gate: 0,
        operation: 1,
        preparation: row.landing.preparation,
        attempt: 1,
    }
    .identity()
    .unwrap();
    flight.entry = 9;
    flight.inputs = Box::new([
        api::Input { name: b"temper_commit".as_slice().into(), value: commit_text(&run.commit) },
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
    outputs(&mut d, Event::Restore { record: Stored::Entry { entry, held: true } });
    begin_reporting_gate(&mut d, 91);
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
            assert_eq!(
                attempt,
                api::JobAttempt { head: run.commit, run: run.id, job: 9, attempt: run.jobs[0].attempt }
            );
            Err(api::Error::Missing)
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
    let mut history = List::uncapped();
    for _ in 0..32_u32 {
        let (call, op) = fixtures::next_call(d);
        let answer = read_answer(op, run, &report);
        let mut decision = None;
        for request in fixtures::answer(d, call, answer) {
            if let Request::Save { record: Stored::HistoryEvent(event), .. } = &request {
                history.push(event.clone()).unwrap();
            }
            if let Request::LandingDecision { decision: next, .. } = request {
                decision = Some(next);
            }
        }
        if let Some(decision) = decision {
            return (decision, history.into_boxed());
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
    for ask in &row.asks {
        assert!(ask.is_none(), "no repair ask");
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

fn ended(history: &[HistoryEvent], members: u16, attempt: u32, verdict: GateVerdict) {
    for change in [fixtures::CHANGE, 52].into_iter().take(usize::from(members)) {
        let mut ends = 0_u32;
        for event in history {
            if event.change == change && matches!(event.what, What::GateEnded { .. }) {
                ends = ends.checked_add(1).unwrap();
                assert_eq!(event.what, What::GateEnded { gate: 0, key: [7; 32], verdict });
                assert_eq!((event.operation, event.attempt), (7, attempt));
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
    Answer::File {
        head: run.commit,
        path: b"report".as_slice().into(),
        bytes: b"test binary broken\n".as_slice().into(),
        truncated: false,
    }
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
    ended(&history, 2, 2, GateVerdict::Failed);
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
    for ask in &row.asks {
        assert!(ask.is_none());
    }
    assert_eq!(d.receipt(&key).unwrap().run.id, 7, "the top's real red is unchanged");
}

#[test]
fn restarting_after_an_absent_report_finds_the_unsent_rerun_without_a_receipt_or_repair() {
    let (mut d, key, run) = gate(2);
    let (decision, history) = gather(&mut d, &run, Err(api::Error::Missing));
    assert_eq!(decision, policy::Decision::Retry { member: 1, gate: 0, key: key.key, try_: 2 });
    ended(&history, 2, 2, GateVerdict::Fault);
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
            ended(&history, members, run.jobs[0].attempt, GateVerdict::Fault);
            if retry == 2 {
                assert_eq!(
                    decision,
                    policy::Decision::Hold(policy::Hold::Faults { member: members.checked_sub(1).unwrap(), gate: 0 })
                );
                let row = d.landing.rows.get(&61).unwrap();
                let why = d.landing_hold_reason(row).unwrap();
                assert_eq!(why.as_ref(), b"gate gate of org/repo failed on its runner past its retries");
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
