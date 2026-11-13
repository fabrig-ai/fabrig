//! Family 28's bounded policy record (temper/docs/design/protocol/store.md, 2).
use super::{FITS, answers, landing, names, verification, w};
use crate::Undecodable;
use alloc::boxed::Box;
use skein_lib::List;
use skein_lib::Time;
use temper_engine_domain_forge_vocabulary as d;
use temper_engine_domain_forge_tiers as p;

fn encode_tests(tests: &[Box<[u8]>]) -> List<w::FailingTest> {
    let mut values = List::with_capacity(u32::try_from(tests.len()).expect(FITS));
    for test in tests {
        values
            .push(w::FailingTest::new(&w::CEILINGS, w::FailingTestParts { test_id: test.clone() }).expect(FITS))
            .expect(FITS);
    }
    values
}
fn decode_tests(tests: List<w::FailingTest>) -> Result<Box<[Box<[u8]>]>, Undecodable> {
    let mut values = List::with_capacity(tests.len());
    for test in tests.into_boxed() {
        let test = test.into_parts().test_id;
        if test.is_empty() {
            return Err(Undecodable::Malformed);
        }
        if let Some(previous) = values.as_slice().last()
            && previous >= &test
        {
            return Err(Undecodable::Malformed);
        }
        values.push(test).expect(FITS);
    }
    Ok(values.into_boxed())
}
fn encode_landings(landings: &[p::Landing]) -> List<w::TierLanding> {
    let mut values = List::with_capacity(u32::try_from(landings.len()).expect(FITS));
    for landing in landings {
        values
            .push(
                w::TierLanding::new(&w::CEILINGS, w::TierLandingParts { commit: landing.commit, key: landing.key })
                    .expect(FITS),
            )
            .expect(FITS);
    }
    values
}
fn decode_landings(landings: List<w::TierLanding>) -> Box<[p::Landing]> {
    let mut values = List::with_capacity(landings.len());
    for landing in landings.into_boxed() {
        let w::TierLandingParts { commit, key } = landing.into_parts();
        values.push(p::Landing { commit, key }).expect(FITS);
    }
    values.into_boxed()
}
fn encode_finding(finding: &p::Finding) -> w::TierFinding {
    let mut blamed = List::with_capacity(u32::try_from(finding.blamed.len()).expect(FITS));
    for blame in &finding.blamed {
        blamed.push(verification::encode_blame(blame)).expect(FITS);
    }
    w::TierFinding::new(
        &w::CEILINGS,
        w::TierFindingParts {
            failing: verification::encode_failures(&finding.failing),
            flakes: encode_tests(&finding.flakes),
            blamed,
        },
    )
    .expect(FITS)
}
fn decode_finding(finding: w::TierFinding) -> Result<p::Finding, Undecodable> {
    let w::TierFindingParts { failing, flakes, blamed } = finding.into_parts();
    let mut values = List::with_capacity(blamed.len());
    for blame in blamed.into_boxed() {
        values.push(verification::decode_blame(blame)?).expect(FITS);
    }
    Ok(p::Finding {
        failing: verification::decode_failures(failing)?,
        flakes: decode_tests(flakes)?,
        blamed: values.into_boxed(),
    })
}
fn encode_run(run: &p::Run) -> w::TierRun {
    let part = match &run.part {
        p::Part::Whole => w::TierPart::Whole,
        p::Part::Rerun { failed } => w::TierPart::Rerun(
            w::TierTests::new(&w::CEILINGS, w::TierTestsParts { tests: encode_tests(failed) }).expect(FITS),
        ),
        p::Part::Probe { landing, tests, finding } => {
            let tests = match tests {
                p::Selection::All => w::TierSelection::All,
                p::Selection::Tests(tests) => w::TierSelection::Tests(
                    w::TierTests::new(&w::CEILINGS, w::TierTestsParts { tests: encode_tests(tests) }).expect(FITS),
                ),
            };
            w::TierPart::Probe(
                w::TierProbe::new(
                    &w::CEILINGS,
                    w::TierProbeParts { landing: *landing, tests, finding: encode_finding(finding) },
                )
                .expect(FITS),
            )
        }
    };
    w::TierRun::new(
        &w::CEILINGS,
        w::TierRunParts {
            number: run.number,
            tip: run.tip,
            key: run.key,
            landings: encode_landings(&run.landings),
            part,
            faults: run.faults,
        },
    )
    .expect(FITS)
}
fn decode_run(run: w::TierRun) -> Result<p::Run, Undecodable> {
    let w::TierRunParts { number, tip, key, landings, part, faults } = run.into_parts();
    let landings = decode_landings(landings);
    let part = match part {
        w::TierPart::Whole => p::Part::Whole,
        w::TierPart::Rerun(tests) => p::Part::Rerun { failed: decode_tests(tests.into_parts().tests)? },
        w::TierPart::Probe(probe) => {
            let w::TierProbeParts { landing, tests, finding } = probe.into_parts();
            let tests = match tests {
                w::TierSelection::All => p::Selection::All,
                w::TierSelection::Tests(tests) => p::Selection::Tests(decode_tests(tests.into_parts().tests)?),
            };
            let finding = decode_finding(finding)?;
            if usize::try_from(landing).or(Err(Undecodable::Malformed))? >= landings.len()
                || finding.blamed.len() != landings.len()
            {
                return Err(Undecodable::Malformed);
            }
            p::Part::Probe { landing, tests, finding }
        }
    };
    Ok(p::Run { number, tip, key, landings, part, faults })
}
pub(super) fn encode(row: &d::TierRow) -> w::TierRow {
    let record = encode_record(&row.record);
    w::TierRow::new(
        &w::CEILINGS,
        w::TierRowParts {
            observations: row.observations.clone(),
            repository: names::encode_repository(row.repository),
            tier: row.tier.clone(),
            record,
            task: row.task,
            requested: row.requested,
            cancelled: row.cancelled,
            purpose: match &row.purpose {
                Some(purpose) => Some(purpose.identity().expect(FITS).as_ref().try_into().expect(FITS)),
                None => None,
            },
            pending: encode_pending(&row.pending),
            baseline: match row.baseline {
                Some(tip) => Some(encode_tip(tip)),
                None => None,
            },
            branch: match row.branch {
                d::TierBranch::Absent => w::TierBranch::Absent,
                d::TierBranch::Made => w::TierBranch::Made,
                d::TierBranch::Pending { entry } => w::TierBranch::Pending(
                    w::TierBranchEntry::new(&w::CEILINGS, w::TierBranchEntryParts { entry }).expect(FITS),
                ),
            },
            probe: match &row.probe {
                Some(probe) => Some(
                    w::TierProbeBranch::new(
                        &w::CEILINGS,
                        w::TierProbeBranchParts { landing: probe.landing, branch: encode_branch(probe.branch) },
                    )
                    .expect(FITS),
                ),
                None => None,
            },
            flight: match &row.flight {
                Some(flight) => Some(encode_flight(flight)),
                None => None,
            },
            held: match row.held {
                Some(fault) => Some(encode_fault(fault)),
                None => None,
            },
            result: match &row.result {
                Some(receipt) => Some(landing::encode_receipt_value(receipt)),
                None => None,
            },
            received: row.received,
            goals: encode_goals(&row.goals),
            goal: row.goal,
            started: match row.started {
                Some(at) => Some(at.as_nanos()),
                None => None,
            },
        },
    )
    .expect(FITS)
}
pub(super) fn decode(row: w::TierRow) -> Result<d::TierRow, Undecodable> {
    let w::TierRowParts {
        observations,
        repository,
        tier,
        record,
        task,
        requested,
        cancelled,
        purpose,
        pending,
        baseline,
        branch,
        flight,
        probe,
        held,
        result,
        received,
        goals,
        goal,
        started,
    } = row.into_parts();
    if tier.is_empty() {
        return Err(Undecodable::Malformed);
    }
    for byte in &tier {
        if !byte.is_ascii_alphanumeric() && *byte != b'-' && *byte != b'_' {
            return Err(Undecodable::Malformed);
        }
    }
    let record = decode_record(record)?;
    let run = record.run.as_ref();
    Ok(d::TierRow {
        observations,
        repository: names::decode_repository(repository),
        tier,
        task,
        requested,
        cancelled,
        purpose: match purpose {
            Some(bytes) => match d::purpose::EffectPurpose::decode(&bytes) {
                Some(value @ d::purpose::EffectPurpose::Tier { .. }) => Some(value),
                _ => return Err(Undecodable::Malformed),
            },
            None => None,
        },
        pending: decode_pending(pending),
        baseline: match baseline {
            Some(tip) => Some(decode_tip(tip)),
            None => None,
        },
        branch: match branch {
            w::TierBranch::Absent => d::TierBranch::Absent,
            w::TierBranch::Made => d::TierBranch::Made,
            w::TierBranch::Pending(entry) => d::TierBranch::Pending { entry: entry.into_parts().entry },
        },
        probe: decode_probe(probe, run)?,
        flight: match flight {
            Some(flight) => Some(decode_flight(flight)),
            None => None,
        },
        held: match held {
            Some(fault) => Some(decode_fault(fault)),
            None => None,
        },
        result: match result {
            Some(receipt) => Some(landing::decode_receipt_value(receipt)?),
            None => None,
        },
        received,
        goals: decode_goals(goals)?,
        goal,
        started: match started {
            Some(at) => Some(Time::from_nanos(at)),
            None => None,
        },
        record,
    })
}

fn encode_tip(tip: d::TierLanding) -> w::TierPending {
    w::TierPending::new(&w::CEILINGS, w::TierPendingParts { commit: tip.commit, key: tip.key }).expect(FITS)
}
fn decode_tip(tip: w::TierPending) -> d::TierLanding {
    let w::TierPendingParts { commit, key } = tip.into_parts();
    d::TierLanding { commit, key }
}
fn encode_pending(tips: &[d::TierLanding]) -> List<w::TierPending> {
    let mut values = List::with_capacity(u32::try_from(tips.len()).expect(FITS));
    for tip in tips {
        values.push(encode_tip(*tip)).expect(FITS);
    }
    values
}
fn decode_pending(tips: List<w::TierPending>) -> Box<[d::TierLanding]> {
    let mut values = List::with_capacity(tips.len());
    for tip in tips.into_boxed() {
        values.push(decode_tip(tip)).expect(FITS);
    }
    values.into_boxed()
}
fn encode_fault(fault: temper_engine_domain_forge_landing::Fault) -> w::TierFault {
    use temper_engine_domain_forge_landing::Fault;
    match fault {
        Fault::SetUp => w::TierFault::Setup,
        Fault::Completion => w::TierFault::Completion,
        Fault::NoStep => w::TierFault::NoStep,
        Fault::Cancelled => w::TierFault::Cancelled,
        Fault::OtherCommit => w::TierFault::OtherCommit,
        Fault::Lost => w::TierFault::Lost,
        Fault::MissingWorkflow => w::TierFault::MissingWorkflow,
    }
}
fn decode_fault(fault: w::TierFault) -> temper_engine_domain_forge_landing::Fault {
    use temper_engine_domain_forge_landing::Fault;
    match fault {
        w::TierFault::Setup => Fault::SetUp,
        w::TierFault::Completion => Fault::Completion,
        w::TierFault::NoStep => Fault::NoStep,
        w::TierFault::Cancelled => Fault::Cancelled,
        w::TierFault::OtherCommit => Fault::OtherCommit,
        w::TierFault::Lost => Fault::Lost,
        w::TierFault::MissingWorkflow => Fault::MissingWorkflow,
    }
}
pub(super) fn encode_flight(flight: &d::TierFlight) -> w::TierFlight {
    let mut pending = List::with_capacity(u32::try_from(flight.pending.len()).expect(FITS));
    for attempt in &flight.pending {
        pending.push(answers::encode_job_attempt(attempt)).expect(FITS);
    }
    w::TierFlight::new(
        &w::CEILINGS,
        w::TierFlightParts {
            entry: flight.entry,
            start: match flight.start {
                Some(at) => Some(at.as_nanos()),
                None => None,
            },
            since: match flight.since {
                Some(at) => Some(at.as_nanos()),
                None => None,
            },
            waiting: flight.waiting,
            next: match flight.next {
                Some(at) => Some(at.as_nanos()),
                None => None,
            },
            report: match &flight.report {
                Some(run) => Some(
                    w::ReceiptRun::new(
                        &w::CEILINGS,
                        w::ReceiptRunParts { id: run.id, number: run.number, url: run.url.clone() },
                    )
                    .expect(FITS),
                ),
                None => None,
            },
            run: flight.run,
            pending,
            fault: match flight.fault {
                Some(fault) => Some(encode_fault(fault)),
                None => None,
            },
        },
    )
    .expect(FITS)
}
pub(super) fn decode_flight(flight: w::TierFlight) -> d::TierFlight {
    let w::TierFlightParts { entry, start, since, waiting, next, run, pending, fault, report } = flight.into_parts();
    let mut attempts = List::with_capacity(pending.len());
    for attempt in pending.into_boxed() {
        attempts.push(answers::decode_job_attempt(attempt)).expect(FITS);
    }
    d::TierFlight {
        entry,
        start: match start {
            Some(at) => Some(Time::from_nanos(at)),
            None => None,
        },
        since: match since {
            Some(at) => Some(Time::from_nanos(at)),
            None => None,
        },
        waiting,
        next: match next {
            Some(at) => Some(Time::from_nanos(at)),
            None => None,
        },
        report: match report {
            Some(run) => {
                let w::ReceiptRunParts { id, number, url } = run.into_parts();
                Some(d::ReceiptRun { id, number, url })
            }
            None => None,
        },
        run,
        pending: attempts.into_boxed(),
        fault: match fault {
            Some(fault) => Some(decode_fault(fault)),
            None => None,
        },
    }
}

fn encode_branch(branch: d::TierBranch) -> w::TierBranch {
    match branch {
        d::TierBranch::Absent => w::TierBranch::Absent,
        d::TierBranch::Made => w::TierBranch::Made,
        d::TierBranch::Pending { entry } => w::TierBranch::Pending(
            w::TierBranchEntry::new(&w::CEILINGS, w::TierBranchEntryParts { entry }).expect(FITS),
        ),
    }
}
fn decode_branch(branch: w::TierBranch) -> d::TierBranch {
    match branch {
        w::TierBranch::Absent => d::TierBranch::Absent,
        w::TierBranch::Made => d::TierBranch::Made,
        w::TierBranch::Pending(entry) => d::TierBranch::Pending { entry: entry.into_parts().entry },
    }
}

fn decode_probe(
    probe: Option<w::TierProbeBranch>,
    run: Option<&p::Run>,
) -> Result<Option<d::TierProbeBranch>, Undecodable> {
    match probe {
        Some(probe) => {
            let w::TierProbeBranchParts { landing, branch } = probe.into_parts();
            if usize::try_from(landing).or(Err(Undecodable::Malformed))?
                >= run.ok_or(Undecodable::Malformed)?.landings.len()
            {
                return Err(Undecodable::Malformed);
            }
            Ok(Some(d::TierProbeBranch { landing, branch: decode_branch(branch) }))
        }
        None => Ok(None),
    }
}

fn encode_record(record: &p::Record) -> w::TierRecord {
    let mut flakes = List::with_capacity(u32::try_from(record.flakes.len()).expect(FITS));
    for flake in &record.flakes {
        flakes
            .push(
                w::TierFlake::new(&w::CEILINGS, w::TierFlakeParts { test: flake.test.clone(), count: flake.count })
                    .expect(FITS),
            )
            .expect(FITS);
    }
    let judged = match record.judged {
        Some(j) => Some(w::TierJudged::new(&w::CEILINGS, w::TierJudgedParts { tip: j.tip, key: j.key }).expect(FITS)),
        None => None,
    };
    let run = match &record.run {
        Some(run) => Some(encode_run(run)),
        None => None,
    };
    w::TierRecord::new(
        &w::CEILINGS,
        w::TierRecordParts {
            sequence: record.sequence,
            judged,
            failing: verification::encode_failures(&record.failing),
            flakes,
            run,
        },
    )
    .expect(FITS)
}

fn decode_record(record: w::TierRecord) -> Result<p::Record, Undecodable> {
    let w::TierRecordParts { sequence, judged, failing, flakes, run } = record.into_parts();
    let judged = match judged {
        Some(j) => {
            let w::TierJudgedParts { tip, key } = j.into_parts();
            Some(p::Judged { tip, key })
        }
        None => None,
    };
    let run = match run {
        Some(run) => Some(decode_run(run)?),
        None => None,
    };
    let mut values = List::with_capacity(flakes.len());
    for flake in flakes.into_boxed() {
        let w::TierFlakeParts { test, count } = flake.into_parts();
        if let Some(previous) = values.as_slice().last() {
            let previous: &p::Flake = previous;
            if previous.test >= test {
                return Err(Undecodable::Malformed);
            }
        }
        values.push(p::Flake { test, count }).expect(FITS);
    }
    Ok(p::Record {
        sequence,
        judged,
        failing: verification::decode_failures(failing)?,
        flakes: values.into_boxed(),
        run,
    })
}

fn encode_goals(goals: &[d::TierGoal]) -> List<w::TierGoal> {
    let mut values = List::with_capacity(u32::try_from(goals.len()).expect(FITS));
    for goal in goals {
        values
            .push(
                w::TierGoal::new(
                    &w::CEILINGS,
                    w::TierGoalParts {
                        goal: goal.goal,
                        branch: goal.branch.clone(),
                        tip: goal.tip,
                        ran: goal.ran,
                        failing: verification::encode_failures(&goal.failing),
                        started: match goal.started {
                            Some(at) => Some(at.as_nanos()),
                            None => None,
                        },
                    },
                )
                .expect(FITS),
            )
            .expect(FITS);
    }
    values
}
fn decode_goals(goals: List<w::TierGoal>) -> Result<Box<[d::TierGoal]>, Undecodable> {
    let mut values = List::with_capacity(goals.len());
    for goal in goals.into_boxed() {
        let w::TierGoalParts { goal, branch, tip, ran, failing, started } = goal.into_parts();
        if branch.is_empty() {
            return Err(Undecodable::Malformed);
        }
        if let Some(previous) = values.as_slice().last() {
            let previous: &d::TierGoal = previous;
            if previous.goal >= goal {
                return Err(Undecodable::Malformed);
            }
        }
        values
            .push(d::TierGoal {
                goal,
                branch,
                tip,
                ran,
                failing: verification::decode_failures(failing)?,
                started: match started {
                    Some(at) => Some(Time::from_nanos(at)),
                    None => None,
                },
            })
            .expect(FITS);
    }
    Ok(values.into_boxed())
}
