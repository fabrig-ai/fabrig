//! Family 28's bounded policy record (temper/docs/design/protocol/store.md, 2).
use super::{FITS, answers, landing, names, verification, w};
use crate::Undecodable;
use alloc::boxed::Box;
use skein_lib::List;
use skein_lib::Time;
use temper_engine_domain_forge_tiers as p;
use temper_engine_domain_forge_vocabulary as d;
mod post;

pub(super) fn encode_tests(tests: &[Box<[u8]>]) -> List<w::FailingTest> {
    let mut values = List::with_capacity(u32::try_from(tests.len()).expect(FITS));
    for test in tests {
        values.push(w::FailingTest { test_id: test.clone() }).expect(FITS);
    }
    values
}
pub(super) fn decode_tests(tests: List<w::FailingTest>) -> Result<Box<[Box<[u8]>]>, Undecodable> {
    let mut values = List::with_capacity(tests.len());
    for test in tests.into_boxed() {
        let test = test.test_id;
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
fn encode_shown(tests: &[Box<[u8]>]) -> (List<w::ShownTest>, u32) {
    let capacity = u32::try_from(tests.len()).expect(FITS).min(w::CEILINGS.many_tests_tests);
    let mut values = List::with_capacity(capacity);
    let mut dropped = 0_u32;
    for test in tests {
        if values.len() == capacity {
            dropped = dropped.saturating_add(1);
        } else {
            values.push(w::ShownTest { test_id: test.clone() }).expect(FITS);
        }
    }
    (values, dropped)
}
fn decode_shown(tests: List<w::ShownTest>) -> Result<Box<[Box<[u8]>]>, Undecodable> {
    let mut values = List::with_capacity(tests.len());
    for test in tests.into_boxed() {
        let test = test.test_id;
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
        values.push(w::TierLanding { commit: landing.commit, key: landing.key }).expect(FITS);
    }
    values
}
fn decode_landings(landings: List<w::TierLanding>) -> Box<[p::Landing]> {
    let mut values = List::with_capacity(landings.len());
    for landing in landings.into_boxed() {
        let w::TierLanding { commit, key } = landing;
        values.push(p::Landing { commit, key }).expect(FITS);
    }
    values.into_boxed()
}
fn encode_finding(finding: &p::Finding) -> w::TierFinding {
    let mut blamed = List::with_capacity(u32::try_from(finding.blamed.len()).expect(FITS));
    for blame in &finding.blamed {
        blamed.push(verification::encode_blame(blame)).expect(FITS);
    }
    w::TierFinding {
        failing: verification::encode_failures(&finding.failing),
        flakes: encode_tests(&finding.flakes),
        blamed,
    }
}
fn decode_finding(finding: w::TierFinding) -> Result<p::Finding, Undecodable> {
    let w::TierFinding { failing, flakes, blamed } = finding;
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
fn encode_selection(tests: &p::Selection) -> w::TierSelection {
    match tests {
        p::Selection::All => w::TierSelection::All,
        p::Selection::Tests(tests) => w::TierSelection::Tests(w::TierTests { tests: encode_tests(tests) }),
    }
}
fn decode_selection(tests: w::TierSelection) -> Result<p::Selection, Undecodable> {
    Ok(match tests {
        w::TierSelection::All => p::Selection::All,
        w::TierSelection::Tests(tests) => p::Selection::Tests(decode_tests(tests.tests)?),
    })
}
fn encode_groups(groups: &[p::Group]) -> List<w::TierGroup> {
    let mut values = List::with_capacity(u32::try_from(groups.len()).expect(FITS));
    for group in groups {
        let mut tests = List::with_capacity(u32::try_from(group.tests.len()).expect(FITS));
        for place in &group.tests {
            tests.push(*place).expect(FITS);
        }
        values.push(w::TierGroup { low: group.low, high: group.high, tests }).expect(FITS);
    }
    values
}
fn group_range(low: u32, high: u32, landings: usize) -> Result<(), Undecodable> {
    if low > high || usize::try_from(high).or(Err(Undecodable::Malformed))? >= landings {
        return Err(Undecodable::Malformed);
    }
    Ok(())
}
fn decode_groups(
    groups: List<w::TierGroup>,
    landings: usize,
    finding: &p::Finding,
) -> Result<Box<[p::Group]>, Undecodable> {
    let mut values = List::with_capacity(groups.len());
    for group in groups.into_boxed() {
        let w::TierGroup { low, high, tests } = group;
        group_range(low, high, landings)?;
        let p::Failures::Tests(failing) = &finding.failing else { return Err(Undecodable::Malformed) };
        let mut previous = None;
        for place in tests.as_slice() {
            if usize::from(*place) >= failing.len() || previous >= Some(*place) {
                return Err(Undecodable::Malformed);
            }
            previous = Some(*place);
        }
        values.push(p::Group { low, high, tests: tests.into_boxed() }).expect(FITS);
    }
    Ok(values.into_boxed())
}
fn encode_run(run: &p::Run) -> w::TierRun {
    let part = match &run.part {
        p::Part::Whole => w::TierPart::Whole,
        p::Part::Rerun { failed } => w::TierPart::Rerun(w::TierTests { tests: encode_tests(failed) }),
        p::Part::Probe { landing, low, high, tests, waiting, finding } => {
            let tests = encode_selection(tests);
            w::TierPart::Probe(w::TierProbe {
                landing: *landing,
                low: *low,
                high: *high,
                tests,
                waiting: encode_groups(waiting),
                finding: encode_finding(finding),
            })
        }
    };
    w::TierRun {
        number: run.number,
        tip: run.tip,
        key: run.key,
        landings: encode_landings(&run.landings),
        part,
        faults: run.faults,
    }
}
fn decode_run(run: w::TierRun) -> Result<p::Run, Undecodable> {
    let w::TierRun { number, tip, key, landings, part, faults } = run;
    let landings = decode_landings(landings);
    let part = match part {
        w::TierPart::Whole => p::Part::Whole,
        w::TierPart::Rerun(tests) => p::Part::Rerun { failed: decode_tests(tests.tests)? },
        w::TierPart::Probe(probe) => {
            let w::TierProbe { landing, low, high, tests, waiting, finding } = probe;
            group_range(low, high, landings.len())?;
            let tests = decode_selection(tests);
            let finding = decode_finding(finding)?;
            if landing < low || landing > high || finding.blamed.len() != landings.len() {
                return Err(Undecodable::Malformed);
            }
            let waiting = decode_groups(waiting, landings.len(), &finding)?;
            p::Part::Probe { landing, low, high, tests: tests?, waiting, finding }
        }
    };
    Ok(p::Run { number, tip, key, landings, part, faults })
}
pub(super) fn encode(row: &d::TierRow) -> w::TierRow {
    let record = encode_record(&row.record);
    w::TierRow {
        many: match &row.many {
            Some(many) => {
                let (tests, dropped) = encode_shown(&many.tests);
                Some(w::ManyTests { tests, more: many.more.saturating_add(dropped), cut: many.cut })
            }
            None => None,
        },
        observations: row.observations.clone(),
        waiting: super::findings::encode_filings(&row.waiting),
        unkept: row.unkept,
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
            d::TierBranch::Pending { entry } => w::TierBranch::Pending(w::TierBranchEntry { entry }),
        },
        probe: match &row.probe {
            Some(probe) => Some(w::TierProbeBranch { landing: probe.landing, branch: encode_branch(probe.branch) }),
            None => None,
        },
        flight: match &row.flight {
            Some(flight) => Some(encode_flight(flight)),
            None => None,
        },
        hold: match &row.hold {
            Some(hold) => Some(w::TierHold { party: hold.party, at: hold.at.as_nanos(), reason: hold.reason.clone() }),
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
        posting: match &row.posting {
            Some(posting) => Some(post::encode(posting, &row.record.failing)),
            None => None,
        },
        received: row.received,
        goals: encode_goals(&row.goals),
        goal: row.goal,
        started: match row.started {
            Some(at) => Some(at.as_nanos()),
            None => None,
        },
    }
}
pub(super) fn decode(row: w::TierRow) -> Result<d::TierRow, Undecodable> {
    let w::TierRow {
        many,
        observations,
        waiting,
        unkept,
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
        hold,
        result,
        received,
        posting,
        goals,
        goal,
        started,
    } = row;
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
        many: decode_many(many)?,
        observations,
        waiting: super::findings::decode_filings(waiting)?,
        unkept,
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
            w::TierBranch::Pending(entry) => d::TierBranch::Pending { entry: entry.entry },
        },
        probe: decode_probe(probe, run)?,
        flight: match flight {
            Some(flight) => Some(decode_flight(flight)),
            None => None,
        },
        hold: match hold {
            Some(hold) => {
                let w::TierHold { party, at, reason } = hold;
                Some(d::TierHold { party, at: skein_lib::Wall::from_nanos(at), reason })
            }
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
        posting: match posting {
            Some(posting) => Some(Box::new(post::decode(posting, &record.failing)?)),
            None => None,
        },
        goals: decode_goals(goals)?,
        goal,
        started: match started {
            Some(at) => Some(Time::from_nanos(at)),
            None => None,
        },
        record,
    })
}

fn decode_many(many: Option<w::ManyTests>) -> Result<Option<p::ManyTests>, Undecodable> {
    Ok(match many {
        Some(many) => {
            let w::ManyTests { tests, more, cut } = many;
            Some(p::ManyTests { tests: decode_shown(tests)?, more, cut })
        }
        None => None,
    })
}

fn encode_tip(tip: d::TierLanding) -> w::TierPending {
    w::TierPending { commit: tip.commit, key: tip.key }
}
fn decode_tip(tip: w::TierPending) -> d::TierLanding {
    let w::TierPending { commit, key } = tip;
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
        Fault::Unreported => w::TierFault::Unreported,
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
        w::TierFault::Unreported => Fault::Unreported,
    }
}
pub(super) fn encode_flight(flight: &d::TierFlight) -> w::TierFlight {
    let mut pending = List::with_capacity(u32::try_from(flight.pending.len()).expect(FITS));
    for attempt in &flight.pending {
        pending.push(answers::encode_job_attempt(attempt)).expect(FITS);
    }
    w::TierFlight {
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
            Some(run) => Some(w::ReceiptRun { id: run.id, number: run.number, url: run.url.clone() }),
            None => None,
        },
        run: flight.run,
        pending,
        fault: match flight.fault {
            Some(fault) => Some(encode_fault(fault)),
            None => None,
        },
    }
}
pub(super) fn decode_flight(flight: w::TierFlight) -> d::TierFlight {
    let w::TierFlight { entry, start, since, waiting, next, run, pending, fault, report } = flight;
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
                let w::ReceiptRun { id, number, url } = run;
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
        d::TierBranch::Pending { entry } => w::TierBranch::Pending(w::TierBranchEntry { entry }),
    }
}
fn decode_branch(branch: w::TierBranch) -> d::TierBranch {
    match branch {
        w::TierBranch::Absent => d::TierBranch::Absent,
        w::TierBranch::Made => d::TierBranch::Made,
        w::TierBranch::Pending(entry) => d::TierBranch::Pending { entry: entry.entry },
    }
}

fn decode_probe(
    probe: Option<w::TierProbeBranch>,
    run: Option<&p::Run>,
) -> Result<Option<d::TierProbeBranch>, Undecodable> {
    match probe {
        Some(probe) => {
            let w::TierProbeBranch { landing, branch } = probe;
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
        flakes.push(w::TierFlake { test: flake.test.clone(), count: flake.count }).expect(FITS);
    }
    let judged = match record.judged {
        Some(j) => Some(w::TierJudged { tip: j.tip, key: j.key }),
        None => None,
    };
    let run = match &record.run {
        Some(run) => Some(encode_run(run)),
        None => None,
    };
    w::TierRecord {
        sequence: record.sequence,
        judged,
        failing: verification::encode_failures(&record.failing),
        flakes,
        run,
    }
}

fn decode_record(record: w::TierRecord) -> Result<p::Record, Undecodable> {
    let w::TierRecord { sequence, judged, failing, flakes, run } = record;
    let judged = match judged {
        Some(j) => {
            let w::TierJudged { tip, key } = j;
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
        let w::TierFlake { test, count } = flake;
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
            .push(w::TierGoal {
                goal: goal.goal,
                branch: goal.branch.clone(),
                tip: goal.tip,
                ran: goal.ran,
                failing: verification::encode_failures(&goal.failing),
                started: match goal.started {
                    Some(at) => Some(at.as_nanos()),
                    None => None,
                },
            })
            .expect(FITS);
    }
    values
}
fn decode_goals(goals: List<w::TierGoal>) -> Result<Box<[d::TierGoal]>, Undecodable> {
    let mut values = List::with_capacity(goals.len());
    for goal in goals.into_boxed() {
        let w::TierGoal { goal, branch, tip, ran, failing, started } = goal;
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
