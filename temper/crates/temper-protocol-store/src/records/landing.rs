//! Landing and receipt records (protocol/store.md, families 23 and 24).
use super::{FITS, names, w};
use alloc::boxed::Box;
use skein_lib::{List, Wall};
use temper_engine_domain_forge_change as change;
use temper_engine_domain_forge_landing as d;
use temper_engine_domain_forge_vocabulary as top;

fn encode_place(phase: &d::Phase) -> w::LandingPlace {
    match phase {
        d::Phase::Waiting => w::LandingPlace::Waiting,
        d::Phase::Preparing => w::LandingPlace::Preparing,
        d::Phase::Proving => w::LandingPlace::Proving,
        d::Phase::Proved { at } => w::LandingPlace::Proved(w::LandingAt { at: at.as_nanos() }),
        d::Phase::Turn { at } => w::LandingPlace::Turn(w::LandingAt { at: at.as_nanos() }),
        d::Phase::Landed => w::LandingPlace::Landed,
        d::Phase::Cancelled => w::LandingPlace::Cancelled,
        d::Phase::Held { .. } => unreachable!("one held wrapper"),
    }
}
fn decode_place(phase: w::LandingPlace) -> d::Phase {
    match phase {
        w::LandingPlace::Waiting => d::Phase::Waiting,
        w::LandingPlace::Preparing => d::Phase::Preparing,
        w::LandingPlace::Proving => d::Phase::Proving,
        w::LandingPlace::Proved(at) => d::Phase::Proved { at: Wall::from_nanos(at.at) },
        w::LandingPlace::Turn(at) => d::Phase::Turn { at: Wall::from_nanos(at.at) },
        w::LandingPlace::Landed => d::Phase::Landed,
        w::LandingPlace::Cancelled => d::Phase::Cancelled,
    }
}
fn encode_phase(phase: &d::Phase) -> w::LandingPhase {
    let (place, held) = match phase {
        d::Phase::Held { was, why: d::Hold::Faults { member, gate } } => {
            (encode_place(was), Some(w::LandingFaults { member: *member, gate: *gate }))
        }
        d::Phase::Held {
            was,
            why: d::Hold::Prepare | d::Hold::Record { .. } | d::Hold::Timeout { .. } | d::Hold::Turn,
        } => (encode_place(was), None),
        d::Phase::Waiting
        | d::Phase::Preparing
        | d::Phase::Proving
        | d::Phase::Proved { .. }
        | d::Phase::Turn { .. }
        | d::Phase::Landed
        | d::Phase::Cancelled => (encode_place(phase), None),
    };
    w::LandingPhase { place, held }
}
fn decode_phase(phase: w::LandingPhase) -> d::Phase {
    let w::LandingPhase { place, held } = phase;
    let phase = decode_place(place);
    match held {
        Some(held) => {
            let w::LandingFaults { member, gate } = held;
            d::Phase::Held { was: Box::new(phase), why: d::Hold::Faults { member, gate } }
        }
        None => phase,
    }
}
fn encode_proof(proof: &d::GateProof) -> w::GateProof {
    let state = match proof.proof {
        d::Proof::Unkeyed => w::GateProofState::Unkeyed,
        d::Proof::Dispatched { key, try_ } => w::GateProofState::Dispatched(w::DispatchedKey { key, try_ }),
        d::Proof::Keyed { key } => w::GateProofState::Keyed(w::GateKey { key }),
        d::Proof::Passed { key } => w::GateProofState::Passed(w::GateKey { key }),
        d::Proof::Failed { key } => w::GateProofState::Failed(w::GateKey { key }),
    };
    w::GateProof {
        proof: state,
        posted: match proof.posted {
            d::Posted::ByTest => w::GatePosted::ByTest,
            d::Posted::Nothing => w::GatePosted::Nothing,
            d::Posted::Pending => w::GatePosted::Pending,
            d::Posted::Verdict => w::GatePosted::Verdict,
        },
        faults: proof.faults,
    }
}
fn decode_proof(proof: w::GateProof) -> d::GateProof {
    let w::GateProof { proof, posted, faults } = proof;
    let proof = match proof {
        w::GateProofState::Unkeyed => d::Proof::Unkeyed,
        w::GateProofState::Dispatched(value) => {
            let w::DispatchedKey { key, try_ } = value;
            d::Proof::Dispatched { key, try_ }
        }
        w::GateProofState::Keyed(value) => d::Proof::Keyed { key: value.key },
        w::GateProofState::Passed(value) => d::Proof::Passed { key: value.key },
        w::GateProofState::Failed(value) => d::Proof::Failed { key: value.key },
    };
    d::GateProof {
        proof,
        posted: match posted {
            w::GatePosted::ByTest => d::Posted::ByTest,
            w::GatePosted::Nothing => d::Posted::Nothing,
            w::GatePosted::Pending => d::Posted::Pending,
            w::GatePosted::Verdict => d::Posted::Verdict,
        },
        faults,
    }
}
fn encode_member(member: &d::Member) -> w::LandingMember {
    let mut gates = List::with_capacity(u32::try_from(member.gates.len()).expect(FITS));
    for gate in &member.gates {
        gates.push(encode_proof(gate)).expect(FITS);
    }
    w::LandingMember {
        round: member.round,
        unit: member.unit,
        branch: member.branch.clone(),
        combined: encode_combine(member.combined),
        recorded: member.recorded,
        told: member.told,
        judging: member.judging,
        leave_told: member.leave_told,
        change: member.change,
        repository: names::encode_repository(skein_forge::api::Repository {
            forge: member.repository.forge,
            repository: member.repository.repository,
        }),
        pull: member.pull,
        head: member.head,
        base: member.base,
        candidate: member.candidate,
        gates,
        asked: member.asked,
        landed: member.landed,
        left: match member.left {
            Some(why) => Some(encode_leave(why)),
            None => None,
        },
    }
}
fn decode_member(member: w::LandingMember) -> d::Member {
    let w::LandingMember {
        round,
        change,
        repository,
        pull,
        head,
        base,
        candidate,
        gates,
        asked,
        landed,
        left,
        unit,
        branch,
        combined,
        recorded,
        told,
        judging,
        leave_told,
    } = member;
    let repository = names::decode_repository(repository);
    let mut proofs = List::with_capacity(gates.len());
    for gate in gates.into_boxed() {
        proofs.push(decode_proof(gate)).expect(FITS);
    }
    d::Member {
        round,
        unit,
        branch,
        combined: decode_combine(combined),
        recorded,
        told,
        judging,
        leave_told,
        change,
        repository: d::Repository { forge: repository.forge, repository: repository.repository },
        pull,
        head,
        base,
        candidate,
        gates: proofs.into_boxed(),
        asked,
        landed,
        left: match left {
            Some(why) => Some(decode_leave(why)),
            None => None,
        },
    }
}
fn encode_operation(op: top::Operation) -> w::LandingOperation {
    match op {
        top::Operation::Dispatch => w::LandingOperation::Dispatch,
        top::Operation::Rerun => w::LandingOperation::Rerun,
        top::Operation::Pending => w::LandingOperation::Pending,
        top::Operation::Verdict => w::LandingOperation::Verdict,
        top::Operation::Open => w::LandingOperation::Open,
        top::Operation::Close => w::LandingOperation::Close,
        top::Operation::Delete => w::LandingOperation::Delete,
        top::Operation::Record => w::LandingOperation::Record,
        top::Operation::Merge => w::LandingOperation::Merge,
    }
}
fn decode_operation(op: w::LandingOperation) -> top::Operation {
    match op {
        w::LandingOperation::Dispatch => top::Operation::Dispatch,
        w::LandingOperation::Rerun => top::Operation::Rerun,
        w::LandingOperation::Pending => top::Operation::Pending,
        w::LandingOperation::Verdict => top::Operation::Verdict,
        w::LandingOperation::Open => top::Operation::Open,
        w::LandingOperation::Close => top::Operation::Close,
        w::LandingOperation::Delete => top::Operation::Delete,
        w::LandingOperation::Record => top::Operation::Record,
        w::LandingOperation::Merge => top::Operation::Merge,
    }
}
pub(super) fn encode_ask(ask: change::Ask) -> w::LandingAsk {
    match ask {
        change::Ask::Judge(target) => w::LandingAsk::Judge(super::change::encode_judgement(&target)),
        change::Ask::Landed { prefix } => w::LandingAsk::Landed(w::LandingPrefix { prefix }),
        change::Ask::Resolve { base } => w::LandingAsk::Resolve(w::LandingResolve { base }),
        change::Ask::Prepare { head, main } => w::LandingAsk::Prepare(w::LandingPrepare { head, main }),
        change::Ask::Repair { gate, candidate, below } => {
            w::LandingAsk::Repair(w::LandingRepairAsk { gate, candidate, below: super::change::encode_below(below) })
        }
    }
}
pub(super) fn decode_ask(ask: w::LandingAsk) -> change::Ask {
    match ask {
        w::LandingAsk::Judge(value) => change::Ask::Judge(super::change::decode_judgement(value)),
        w::LandingAsk::Landed(value) => change::Ask::Landed { prefix: value.prefix },
        w::LandingAsk::Resolve(value) => change::Ask::Resolve { base: value.base },
        w::LandingAsk::Prepare(value) => {
            let w::LandingPrepare { head, main } = value;
            change::Ask::Prepare { head, main }
        }
        w::LandingAsk::Repair(value) => {
            let w::LandingRepairAsk { gate, candidate, below } = value;
            change::Ask::Repair { gate, candidate, below: super::change::decode_below(below) }
        }
    }
}
fn encode_round(round: &d::Round) -> w::LandingRound {
    w::LandingRound {
        merges: round.merges,
        search: match &round.search {
            Some(search) => Some(w::LandingSearch {
                green: search.green,
                red: search.red,
                tests: encode_search_tests(&search.tests),
            }),
            None => None,
        },
        changed: round.changed,
        branch: round.branch.clone(),
        pull: round.pull,
        upstream: round.upstream,
        combined: match round.combined {
            Some(d::Combine::Merge) => Some(w::LandingCombine::Merge),
            Some(d::Combine::Squash) => Some(w::LandingCombine::Squash),
            None => None,
        },
        recorded: round.recorded,
        told: round.told,
        failure: round.failure.clone(),
        judging: round.judging,
        ending: match round.ending {
            Some(d::RoundEnd::Stale) => Some(w::LandingRoundEnd::Stale),
            Some(d::RoundEnd::Leave) => Some(w::LandingRoundEnd::Leave),
            Some(d::RoundEnd::Conflict) => Some(w::LandingRoundEnd::Conflict),
            Some(d::RoundEnd::Cut) => Some(w::LandingRoundEnd::Cut),
            None => None,
        },
    }
}
fn decode_round(round: w::LandingRound) -> d::Round {
    let w::LandingRound {
        merges,
        branch,
        pull,
        upstream,
        combined,
        recorded,
        told,
        ending,
        failure,
        judging,
        changed,
        search,
    } = round;
    d::Round {
        merges,
        search: match search {
            Some(search) => {
                Some(d::Search { green: search.green, red: search.red, tests: decode_search_tests(search.tests) })
            }
            None => None,
        },
        changed,
        branch,
        pull,
        upstream,
        combined: match combined {
            Some(w::LandingCombine::Merge) => Some(d::Combine::Merge),
            Some(w::LandingCombine::Squash) => Some(d::Combine::Squash),
            None => None,
        },
        recorded,
        told,
        failure,
        judging,
        ending: match ending {
            Some(w::LandingRoundEnd::Stale) => Some(d::RoundEnd::Stale),
            Some(w::LandingRoundEnd::Leave) => Some(d::RoundEnd::Leave),
            Some(w::LandingRoundEnd::Conflict) => Some(d::RoundEnd::Conflict),
            Some(w::LandingRoundEnd::Cut) => Some(d::RoundEnd::Cut),
            None => None,
        },
    }
}
pub(super) fn encode(row: &top::LandingRow) -> w::LandingRow {
    let mut members = List::with_capacity(u32::try_from(row.landing.members.len()).expect(FITS));
    for member in &row.landing.members {
        members.push(encode_member(member)).expect(FITS);
    }
    let landing = w::LandingState {
        searched: encode_searched(row.landing.searched.as_ref()),
        task: row.landing.task,
        project: row.landing.project,
        phase: encode_phase(&row.landing.phase),
        members,
        preparation: row.landing.preparation,
        stale_proofs: row.landing.stale_proofs,
        last_left: row.landing.last_left,
        branch: row.landing.branch.clone(),
        round: if row.landing.round.branch.is_empty() { None } else { Some(encode_round(&row.landing.round)) },
        own_hold: match row.landing.phase {
            d::Phase::Held { why: d::Hold::Prepare, .. } => Some(w::LandingOwnHold::Prepare),
            d::Phase::Held { why: d::Hold::Record { member }, .. } => {
                Some(w::LandingOwnHold::Record(w::LandingRecordHold { member }))
            }
            d::Phase::Held { why: d::Hold::Timeout { member, gate, run }, .. } => {
                Some(w::LandingOwnHold::Timeout(w::LandingTimeout { member, gate, run }))
            }
            d::Phase::Held { why: d::Hold::Turn, .. } => Some(w::LandingOwnHold::Turn),
            d::Phase::Held { why: d::Hold::Faults { .. }, .. }
            | d::Phase::Waiting
            | d::Phase::Preparing
            | d::Phase::Proving
            | d::Phase::Proved { .. }
            | d::Phase::Turn { .. }
            | d::Phase::Landed
            | d::Phase::Cancelled => None,
        },
    };
    let mut asks = List::with_capacity(u32::try_from(row.asks.len()).expect(FITS));
    for ask in &row.asks {
        asks.push(w::LandingAskSlot {
            ask: match ask {
                Some(ask) => Some(encode_ask(ask.clone())),
                None => None,
            },
        })
        .expect(FITS);
    }
    let mut entries = List::with_capacity(u32::try_from(row.entries.len()).expect(FITS));
    for entry in &row.entries {
        entries
            .push(w::LandingEntry {
                preparation: entry.preparation,
                base: entry.base,
                run: match entry.run {
                    Some(run) => Some(encode_run(run)),
                    None => None,
                },
                entry: entry.entry,
                member: entry.member,
                gate: entry.gate,
                operation: encode_operation(entry.operation),
            })
            .expect(FITS);
    }
    w::LandingRow {
        landing,
        asks,
        entries,
        job: super::history::encode_job_record(row.job.as_ref()),
        flake_reruns: encode_flake_reruns(&row.flake_reruns),
        room: row.room,
        last_event: match row.last_event {
            Some(at) => Some(at.as_nanos()),
            None => None,
        },
        turn_since: match row.turn_since {
            Some(at) => Some(at.as_nanos()),
            None => None,
        },
        waiting: encode_wait(row.waiting),
    }
}
fn encode_flake_reruns(reruns: &[top::FlakeRerun]) -> List<w::LandingFlakeRerun> {
    let mut encoded = List::with_capacity(u32::try_from(reruns.len()).expect(FITS));
    for rerun in reruns {
        let mut findings = List::with_capacity(64);
        for number in &rerun.findings {
            findings.push(*number).expect(FITS);
        }
        let mut pending = List::with_capacity(16);
        for attempt in &rerun.pending {
            pending
                .push(w::JobAttempt {
                    head: attempt.head,
                    run: attempt.run,
                    job: attempt.job,
                    attempt: attempt.attempt,
                })
                .expect(FITS);
        }
        encoded
            .push(w::LandingFlakeRerun {
                key: encode_receipt_key(&rerun.key),
                run: rerun.run,
                findings,
                pending,
                redispatch: rerun.redispatch,
                ended: rerun.ended,
            })
            .expect(FITS);
    }
    encoded
}
fn decode_flake_reruns(reruns: List<w::LandingFlakeRerun>) -> Box<[top::FlakeRerun]> {
    let mut decoded = List::with_capacity(reruns.len());
    for rerun in reruns.into_boxed() {
        let w::LandingFlakeRerun { key, run, findings, pending, redispatch, ended } = rerun;
        let mut attempts = List::with_capacity(pending.len());
        for attempt in pending.into_boxed() {
            let w::JobAttempt { head, run, job, attempt } = attempt;
            attempts.push(skein_forge::api::JobAttempt { head, run, job, attempt }).expect(FITS);
        }
        decoded
            .push(top::FlakeRerun {
                key: decode_receipt_key(key),
                run,
                findings: findings.into_boxed(),
                pending: attempts.into_boxed(),
                redispatch,
                ended,
            })
            .expect(FITS);
    }
    decoded.into_boxed()
}
pub(super) fn decode_receipt_key(key: w::ReceiptKey) -> top::ReceiptKey {
    let w::ReceiptKey { project, repository, check_name, key } = key;
    let check = match check_name {
        w::ReceiptCheck::Gate(name) => top::Check::Gate(name.name),
        w::ReceiptCheck::Tier(name) => top::Check::Tier(name.name),
    };
    top::ReceiptKey { project, repository: names::decode_repository(repository), check, key }
}
fn encode_run(run: top::LandingRun) -> w::LandingRun {
    w::LandingRun { id: run.id, attempt: run.attempt, ended: run.ended, clock: super::change::encode_clock(run.clock) }
}
fn decode_run(run: w::LandingRun) -> top::LandingRun {
    let w::LandingRun { id, attempt, ended, clock } = run;
    top::LandingRun { id, attempt, ended, clock: super::change::decode_clock(clock) }
}
pub(super) fn decode(row: w::LandingRow) -> Result<top::LandingRow, super::Undecodable> {
    let w::LandingRow { landing, asks, entries, job, flake_reruns, room, last_event, turn_since, waiting } = row;
    let w::LandingState {
        task,
        project,
        phase,
        members,
        preparation,
        stale_proofs,
        branch,
        round,
        own_hold,
        last_left,
        searched,
    } = landing;
    if phase.held.is_some() && own_hold.is_some() {
        return Err(super::Undecodable::Malformed);
    }
    let mut decoded_members = List::with_capacity(members.len());
    for member in members.into_boxed() {
        decoded_members.push(decode_member(member)).expect(FITS);
    }
    let mut decoded_asks = List::with_capacity(asks.len());
    for ask in asks.into_boxed() {
        decoded_asks
            .push(match ask.ask {
                Some(ask) => Some(decode_ask(ask)),
                None => None,
            })
            .expect(FITS);
    }
    let mut decoded_entries = List::with_capacity(entries.len());
    for entry in entries.into_boxed() {
        let w::LandingEntry { entry, member, gate, operation, preparation, base, run } = entry;
        decoded_entries
            .push(top::LandingEntry {
                entry,
                member,
                gate,
                operation: decode_operation(operation),
                preparation,
                base,
                run: match run {
                    Some(run) => Some(decode_run(run)),
                    None => None,
                },
            })
            .expect(FITS);
    }
    Ok(top::LandingRow {
        last_event: match last_event {
            Some(at) => Some(Wall::from_nanos(at)),
            None => None,
        },
        turn_since: match turn_since {
            Some(at) => Some(Wall::from_nanos(at)),
            None => None,
        },
        waiting: decode_wait(waiting),
        flake_reruns: decode_flake_reruns(flake_reruns),
        room,
        job: super::history::decode_job_record(job),
        landing: d::Landing {
            searched: match searched {
                Some(value) => {
                    Some(d::Searched { red: value.red, green: value.green, changes: value.changes.into_boxed() })
                }
                None => None,
            },
            last_left,
            task,
            project,
            phase: match own_hold {
                Some(w::LandingOwnHold::Turn) => {
                    d::Phase::Held { was: Box::new(decode_phase(phase)), why: d::Hold::Turn }
                }
                Some(w::LandingOwnHold::Timeout(value)) => d::Phase::Held {
                    was: Box::new(decode_phase(phase)),
                    why: d::Hold::Timeout { member: value.member, gate: value.gate, run: value.run },
                },
                Some(w::LandingOwnHold::Prepare) => {
                    d::Phase::Held { was: Box::new(decode_phase(phase)), why: d::Hold::Prepare }
                }
                Some(w::LandingOwnHold::Record(value)) => {
                    d::Phase::Held { was: Box::new(decode_phase(phase)), why: d::Hold::Record { member: value.member } }
                }
                None => decode_phase(phase),
            },
            members: decoded_members.into_boxed(),
            preparation,
            stale_proofs,
            branch,
            round: match round {
                Some(round) => decode_round(round),
                None => d::Round::new(Box::new([])),
            },
        },
        asks: decoded_asks.into_boxed(),
        entries: decoded_entries.into_boxed(),
    })
}
pub(super) fn encode_receipt_key(key: &top::ReceiptKey) -> w::ReceiptKey {
    let (name, tier) = match &key.check {
        top::Check::Gate(name) => (name.clone(), false),
        top::Check::Tier(name) => (name.clone(), true),
    };
    let name = w::ReceiptName { name };
    w::ReceiptKey {
        project: key.project,
        repository: names::encode_repository(key.repository),
        check_name: if tier { w::ReceiptCheck::Tier(name) } else { w::ReceiptCheck::Gate(name) },
        key: key.key,
    }
}
pub(super) fn encode_receipt(key: &top::ReceiptKey, receipt: &top::Receipt) -> w::ReceiptRow {
    w::ReceiptRow {
        report: encode_report(&receipt.report),
        key: encode_receipt_key(key),
        receipt: encode_receipt_value(receipt),
    }
}
fn encode_report(report: &top::ReceiptReport) -> w::ReceiptReport {
    match report {
        top::ReceiptReport::Unset => w::ReceiptReport::Unset,
        top::ReceiptReport::Unread => w::ReceiptReport::Unread,
        top::ReceiptReport::Many { bound } => w::ReceiptReport::Many(w::ReceiptMany { bound: *bound }),
        top::ReceiptReport::Tests(tests) => {
            let mut kept = List::with_capacity(u32::try_from(tests.len()).expect(FITS));
            for test in tests {
                kept.push(w::ReceiptTest { key: test.key.clone(), fuzzy: test.fuzzy }).expect(FITS);
            }
            w::ReceiptReport::Tests(w::ReceiptTests { tests: kept })
        }
    }
}
fn decode_report(report: w::ReceiptReport) -> top::ReceiptReport {
    match report {
        w::ReceiptReport::Unset => top::ReceiptReport::Unset,
        w::ReceiptReport::Unread => top::ReceiptReport::Unread,
        w::ReceiptReport::Many(many) => top::ReceiptReport::Many { bound: many.bound },
        w::ReceiptReport::Tests(tests) => {
            let tests = tests.tests;
            let mut kept = List::with_capacity(tests.len());
            for test in tests.into_boxed() {
                let w::ReceiptTest { key, fuzzy } = test;
                kept.push(top::ReceiptTest { key, fuzzy }).expect(FITS);
            }
            top::ReceiptReport::Tests(kept.into_boxed())
        }
    }
}

// Gate reports belong to ReceiptRow, not the value a tier result embeds.
pub(super) fn encode_receipt_value(receipt: &top::Receipt) -> w::Receipt {
    let run = w::ReceiptRun { id: receipt.run.id, number: receipt.run.number, url: receipt.run.url.clone() };
    let mut jobs = List::with_capacity(16);
    for job in &receipt.jobs {
        let job = w::ReceiptJob { job: job.job, attempt: job.attempt, name: job.name.clone() };
        jobs.push(job).expect(FITS);
    }
    w::Receipt {
        base: receipt.base,
        jobs,
        failures: match &receipt.failures {
            Some(f) => Some(super::verification::encode_failures(f)),
            None => None,
        },
        passed: receipt.passed,
        run,
        candidate: receipt.candidate,
        ended: receipt.ended.as_nanos(),
        logs: receipt.logs.clone(),
    }
}

pub(super) fn decode_receipt(row: w::ReceiptRow) -> Result<top::Stored, crate::Undecodable> {
    let w::ReceiptRow { report, key, receipt } = row;
    let mut receipt = decode_receipt_value(receipt)?;
    receipt.report = decode_report(report);
    Ok(top::Stored::Receipt { key: decode_receipt_key(key), receipt })
}
pub(super) fn decode_receipt_value(receipt: w::Receipt) -> Result<top::Receipt, crate::Undecodable> {
    let w::Receipt { base, jobs, failures, passed, run, candidate, ended, logs } = receipt;
    let w::ReceiptRun { id, number, url } = run;
    let mut kept = List::with_capacity(16);
    for job in jobs.into_boxed() {
        let w::ReceiptJob { job, attempt, name } = job;
        kept.push(top::ReceiptJob { job, attempt, name }).expect(FITS);
    }
    Ok(top::Receipt {
        report: top::ReceiptReport::Unset,
        base,
        jobs: kept.into_boxed(),
        logs,
        failures: match failures {
            Some(f) => Some(super::verification::decode_failures(f)?),
            None => None,
        },
        passed,
        run: top::ReceiptRun { id, number, url },
        candidate,
        ended: Wall::from_nanos(ended),
    })
}

pub(super) fn encode_leave(why: d::Leave) -> w::LandingLeave {
    match why {
        d::Leave::Conflict => w::LandingLeave::Conflict,
        d::Leave::EarlierConflict => w::LandingLeave::EarlierConflict,
        d::Leave::Unit { change } => w::LandingLeave::Unit(w::LandingLeftChange { change }),
        d::Leave::Dependency { change } => w::LandingLeave::Dependency(w::LandingLeftChange { change }),
        d::Leave::Blamed { gate, run, red, green } => {
            w::LandingLeave::Blamed(w::LandingBlamed { gate, run, red, green })
        }
        d::Leave::HeadMoved { head } => w::LandingLeave::HeadMoved(w::Commit { commit: head }),
        d::Leave::Cancelled => w::LandingLeave::Cancelled,
        d::Leave::Held => w::LandingLeave::Held,
        d::Leave::Refused { gate } => w::LandingLeave::Refused(w::LandingGateRefused { gate }),
        d::Leave::Failed { gate, run } => w::LandingLeave::Failed(w::LandingGateFailed { gate, run }),
    }
}
pub(super) fn decode_leave(why: w::LandingLeave) -> d::Leave {
    match why {
        w::LandingLeave::Conflict => d::Leave::Conflict,
        w::LandingLeave::EarlierConflict => d::Leave::EarlierConflict,
        w::LandingLeave::Unit(value) => d::Leave::Unit { change: value.change },
        w::LandingLeave::Dependency(value) => d::Leave::Dependency { change: value.change },
        w::LandingLeave::Fallback(_) | w::LandingLeave::Held => d::Leave::Held,
        w::LandingLeave::Blamed(value) => {
            d::Leave::Blamed { gate: value.gate, run: value.run, red: value.red, green: value.green }
        }
        w::LandingLeave::HeadMoved(value) => d::Leave::HeadMoved { head: value.commit },
        w::LandingLeave::Cancelled => d::Leave::Cancelled,
        w::LandingLeave::Refused(value) => d::Leave::Refused { gate: value.gate },
        w::LandingLeave::Failed(value) => d::Leave::Failed { gate: value.gate, run: value.run },
    }
}

fn encode_combine(combined: Option<d::Combine>) -> Option<w::LandingCombine> {
    match combined {
        Some(d::Combine::Merge) => Some(w::LandingCombine::Merge),
        Some(d::Combine::Squash) => Some(w::LandingCombine::Squash),
        None => None,
    }
}
fn decode_combine(combined: Option<w::LandingCombine>) -> Option<d::Combine> {
    match combined {
        Some(w::LandingCombine::Merge) => Some(d::Combine::Merge),
        Some(w::LandingCombine::Squash) => Some(d::Combine::Squash),
        None => None,
    }
}

fn encode_search_tests(search: &d::SearchTests) -> w::LandingSearchTests {
    match search {
        d::SearchTests::Whole => w::LandingSearchTests::Whole,
        d::SearchTests::WholeAgain => w::LandingSearchTests::WholeAgain,
        d::SearchTests::ByTest { gates, probed } => {
            let mut selected = List::with_capacity(u32::try_from(gates.len()).expect(FITS));
            for gate in gates {
                let selection = match gate {
                    Some(ids) => {
                        let mut tests = List::with_capacity(u32::try_from(ids.len()).expect(FITS));
                        for id in ids {
                            tests.push(w::FailingTest { test_id: id.clone() }).expect(FITS);
                        }
                        Some(w::LandingSelection { tests })
                    }
                    None => None,
                };
                selected.push(w::LandingSelectedGate { selection }).expect(FITS);
            }
            let mut outcomes = List::with_capacity(u32::try_from(probed.len()).expect(FITS));
            for outcome in probed {
                outcomes
                    .push(w::LandingProbed {
                        member: outcome.member,
                        gate: outcome.gate,
                        run: outcome.run,
                        passed: outcome.passed,
                    })
                    .expect(FITS);
            }
            w::LandingSearchTests::ByTest(w::LandingByTest { gates: selected, probed: outcomes })
        }
    }
}
fn decode_search_tests(search: w::LandingSearchTests) -> d::SearchTests {
    match search {
        w::LandingSearchTests::Whole => d::SearchTests::Whole,
        w::LandingSearchTests::WholeAgain => d::SearchTests::WholeAgain,
        w::LandingSearchTests::ByTest(value) => {
            let mut gates = List::with_capacity(value.gates.len());
            for gate in value.gates.into_boxed() {
                let selection = match gate.selection {
                    Some(selection) => {
                        let mut ids = List::with_capacity(selection.tests.len());
                        for id in selection.tests.into_boxed() {
                            ids.push(id.test_id).expect(FITS);
                        }
                        Some(ids.into_boxed())
                    }
                    None => None,
                };
                gates.push(selection).expect(FITS);
            }
            let mut probed = List::with_capacity(value.probed.len());
            for outcome in value.probed.into_boxed() {
                probed
                    .push(d::Probed {
                        member: outcome.member,
                        gate: outcome.gate,
                        run: outcome.run,
                        passed: outcome.passed,
                    })
                    .expect(FITS);
            }
            d::SearchTests::ByTest { gates: gates.into_boxed(), probed: probed.into_boxed() }
        }
    }
}
fn encode_searched(searched: Option<&d::Searched>) -> Option<w::LandingSearched> {
    let searched = searched?;
    let mut changes = List::with_capacity(u32::try_from(searched.changes.len()).expect(FITS));
    for change in &searched.changes {
        changes.push(*change).expect(FITS);
    }
    Some(w::LandingSearched { red: searched.red, green: searched.green, changes })
}

fn encode_wait(wait: Option<d::Awaited>) -> Option<w::LandingAwaited> {
    let wait = wait?;
    Some(match wait {
        d::Awaited::Release => w::LandingAwaited::Release,
        d::Awaited::Turn => w::LandingAwaited::Turn,
        d::Awaited::Prepare => w::LandingAwaited::Prepare,
        d::Awaited::Prefixes => w::LandingAwaited::Prefixes,
        d::Awaited::Open => w::LandingAwaited::Open,
        d::Awaited::Close => w::LandingAwaited::Close,
        d::Awaited::Merge => w::LandingAwaited::Merge,
        d::Awaited::Read => w::LandingAwaited::Read,
        d::Awaited::Ready { member } => w::LandingAwaited::Ready(w::LandingWaitMember { member }),
        d::Awaited::Delete { member } => w::LandingAwaited::Delete(w::LandingWaitMember { member }),
        d::Awaited::Judgement { member } => w::LandingAwaited::Judgement(w::LandingWaitMember { member }),
        d::Awaited::Record { member } => w::LandingAwaited::Record(w::LandingWaitMember { member }),
        d::Awaited::Ancestry { member } => w::LandingAwaited::Ancestry(w::LandingWaitMember { member }),
        d::Awaited::Post { member, gate } => w::LandingAwaited::Post(w::LandingWaitGate { member, gate }),
        d::Awaited::Run { member, gate } => w::LandingAwaited::Run(w::LandingWaitGate { member, gate }),
        d::Awaited::Receipt { member, gate } => w::LandingAwaited::Receipt(w::LandingWaitGate { member, gate }),
    })
}
fn decode_wait(wait: Option<w::LandingAwaited>) -> Option<d::Awaited> {
    let wait = wait?;
    Some(match wait {
        w::LandingAwaited::Release => d::Awaited::Release,
        w::LandingAwaited::Turn => d::Awaited::Turn,
        w::LandingAwaited::Prepare => d::Awaited::Prepare,
        w::LandingAwaited::Prefixes => d::Awaited::Prefixes,
        w::LandingAwaited::Open => d::Awaited::Open,
        w::LandingAwaited::Close => d::Awaited::Close,
        w::LandingAwaited::Merge => d::Awaited::Merge,
        w::LandingAwaited::Read => d::Awaited::Read,
        w::LandingAwaited::Ready(value) => d::Awaited::Ready { member: value.member },
        w::LandingAwaited::Delete(value) => d::Awaited::Delete { member: value.member },
        w::LandingAwaited::Judgement(value) => d::Awaited::Judgement { member: value.member },
        w::LandingAwaited::Record(value) => d::Awaited::Record { member: value.member },
        w::LandingAwaited::Ancestry(value) => d::Awaited::Ancestry { member: value.member },
        w::LandingAwaited::Post(value) => d::Awaited::Post { member: value.member, gate: value.gate },
        w::LandingAwaited::Run(value) => d::Awaited::Run { member: value.member, gate: value.gate },
        w::LandingAwaited::Receipt(value) => d::Awaited::Receipt { member: value.member, gate: value.gate },
    })
}
