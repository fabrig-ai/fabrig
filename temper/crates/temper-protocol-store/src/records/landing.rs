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
        d::Phase::Proved { at } => w::LandingPlace::Proved(
            w::LandingAt::new(&w::CEILINGS, w::LandingAtParts { at: at.as_nanos() }).expect(FITS),
        ),
        d::Phase::Turn { at } => {
            w::LandingPlace::Turn(w::LandingAt::new(&w::CEILINGS, w::LandingAtParts { at: at.as_nanos() }).expect(FITS))
        }
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
        w::LandingPlace::Proved(at) => d::Phase::Proved { at: Wall::from_nanos(at.at()) },
        w::LandingPlace::Turn(at) => d::Phase::Turn { at: Wall::from_nanos(at.at()) },
        w::LandingPlace::Landed => d::Phase::Landed,
        w::LandingPlace::Cancelled => d::Phase::Cancelled,
    }
}
fn encode_phase(phase: &d::Phase) -> w::LandingPhase {
    let (place, held) = match phase {
        d::Phase::Held { was, why: d::Hold::Faults { member, gate } } => (
            encode_place(was),
            Some(
                w::LandingFaults::new(&w::CEILINGS, w::LandingFaultsParts { member: *member, gate: *gate })
                    .expect(FITS),
            ),
        ),
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
    w::LandingPhase::new(&w::CEILINGS, w::LandingPhaseParts { place, held }).expect(FITS)
}
fn decode_phase(phase: w::LandingPhase) -> d::Phase {
    let w::LandingPhaseParts { place, held } = phase.into_parts();
    let phase = decode_place(place);
    match held {
        Some(held) => {
            let w::LandingFaultsParts { member, gate } = held.into_parts();
            d::Phase::Held { was: Box::new(phase), why: d::Hold::Faults { member, gate } }
        }
        None => phase,
    }
}
fn encode_proof(proof: &d::GateProof) -> w::GateProof {
    let state = match proof.proof {
        d::Proof::Unkeyed => w::GateProofState::Unkeyed,
        d::Proof::Dispatched { key, try_ } => w::GateProofState::Dispatched(
            w::DispatchedKey::new(&w::CEILINGS, w::DispatchedKeyParts { key, try_ }).expect(FITS),
        ),
        d::Proof::Keyed { key } => {
            w::GateProofState::Keyed(w::GateKey::new(&w::CEILINGS, w::GateKeyParts { key }).expect(FITS))
        }
        d::Proof::Passed { key } => {
            w::GateProofState::Passed(w::GateKey::new(&w::CEILINGS, w::GateKeyParts { key }).expect(FITS))
        }
        d::Proof::Failed { key } => {
            w::GateProofState::Failed(w::GateKey::new(&w::CEILINGS, w::GateKeyParts { key }).expect(FITS))
        }
    };
    w::GateProof::new(
        &w::CEILINGS,
        w::GateProofParts {
            proof: state,
            posted: match proof.posted {
                d::Posted::Nothing => w::GatePosted::Nothing,
                d::Posted::Pending => w::GatePosted::Pending,
                d::Posted::Verdict => w::GatePosted::Verdict,
            },
            faults: proof.faults,
        },
    )
    .expect(FITS)
}
fn decode_proof(proof: w::GateProof) -> d::GateProof {
    let w::GateProofParts { proof, posted, faults } = proof.into_parts();
    let proof = match proof {
        w::GateProofState::Unkeyed => d::Proof::Unkeyed,
        w::GateProofState::Dispatched(value) => {
            let w::DispatchedKeyParts { key, try_ } = value.into_parts();
            d::Proof::Dispatched { key, try_ }
        }
        w::GateProofState::Keyed(value) => d::Proof::Keyed { key: *value.key() },
        w::GateProofState::Passed(value) => d::Proof::Passed { key: *value.key() },
        w::GateProofState::Failed(value) => d::Proof::Failed { key: *value.key() },
    };
    d::GateProof {
        proof,
        posted: match posted {
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
    w::LandingMember::new(
        &w::CEILINGS,
        w::LandingMemberParts {
            round: member.round,            unit: member.unit,
            branch: member.branch.clone(),
            combined: encode_combine(member.combined),
            skein_bools: [member.recorded, member.told, member.judging, member.leave_told],
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
        },
    )
    .expect(FITS)
}
fn decode_member(member: w::LandingMember) -> d::Member {
    let w::LandingMemberParts {
        round,        change,
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
        skein_bools: [recorded, told, judging, leave_told],
    } = member.into_parts();
    let repository = names::decode_repository(repository);
    let mut proofs = List::with_capacity(gates.len());
    for gate in gates.into_boxed() {
        proofs.push(decode_proof(gate)).expect(FITS);
    }
    d::Member {
        round,        unit,
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
fn encode_ask(ask: change::Ask) -> w::LandingAsk {
    match ask {
        change::Ask::Judge(target) => w::LandingAsk::Judge(super::change::encode_judgement(&target)),
        change::Ask::Landed { prefix } => {
            w::LandingAsk::Landed(w::LandingPrefix::new(&w::CEILINGS, w::LandingPrefixParts { prefix }).expect(FITS))
        }
        change::Ask::Resolve { base } => {
            w::LandingAsk::Resolve(w::LandingResolve::new(&w::CEILINGS, w::LandingResolveParts { base }).expect(FITS))
        }
        change::Ask::Prepare { head, main } => w::LandingAsk::Prepare(
            w::LandingPrepare::new(&w::CEILINGS, w::LandingPrepareParts { head, main }).expect(FITS),
        ),
        change::Ask::Repair { gate, candidate, below } => w::LandingAsk::Repair(
            w::LandingRepairAsk::new(
                &w::CEILINGS,
                w::LandingRepairAskParts { gate, candidate, below: super::change::encode_below(below) },
            )
            .expect(FITS),
        ),
    }
}
fn decode_ask(ask: w::LandingAsk) -> change::Ask {
    match ask {
        w::LandingAsk::Judge(value) => change::Ask::Judge(super::change::decode_judgement(value)),
        w::LandingAsk::Landed(value) => change::Ask::Landed { prefix: *value.prefix() },
        w::LandingAsk::Resolve(value) => change::Ask::Resolve { base: *value.base() },
        w::LandingAsk::Prepare(value) => {
            let w::LandingPrepareParts { head, main } = value.into_parts();
            change::Ask::Prepare { head, main }
        }
        w::LandingAsk::Repair(value) => {
            let w::LandingRepairAskParts { gate, candidate, below } = value.into_parts();
            change::Ask::Repair { gate, candidate, below: super::change::decode_below(below) }
        }
    }
}
fn encode_round(round: &d::Round) -> w::LandingRound {
    w::LandingRound::new(
        &w::CEILINGS,
        w::LandingRoundParts {
            search: match round.search {
                Some(search) => Some(
                    w::LandingSearch::new(&w::CEILINGS, w::LandingSearchParts { green: search.green, red: search.red })
                        .expect(FITS),
                ),
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
        },
    )
    .expect(FITS)
}
fn decode_round(round: w::LandingRound) -> d::Round {
    let w::LandingRoundParts {
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
    } = round.into_parts();
    d::Round {
        search: match search {
            Some(search) => Some(d::Search { green: search.green(), red: search.red() }),
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
    let landing = w::LandingState::new(
        &w::CEILINGS,
        w::LandingStateParts {
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
                d::Phase::Held { why: d::Hold::Record { member }, .. } => Some(w::LandingOwnHold::Record(
                    w::LandingRecordHold::new(&w::CEILINGS, w::LandingRecordHoldParts { member }).expect(FITS),
                )),
                d::Phase::Held { why: d::Hold::Timeout { member, gate, run }, .. } => Some(w::LandingOwnHold::Timeout(
                    w::LandingTimeout::new(&w::CEILINGS, w::LandingTimeoutParts { member, gate, run }).expect(FITS),
                )),
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
        },
    )
    .expect(FITS);
    let mut asks = List::with_capacity(u32::try_from(row.asks.len()).expect(FITS));
    for ask in &row.asks {
        asks.push(
            w::LandingAskSlot::new(
                &w::CEILINGS,
                w::LandingAskSlotParts {
                    ask: match ask {
                        Some(ask) => Some(encode_ask(ask.clone())),
                        None => None,
                    },
                },
            )
            .expect(FITS),
        )
        .expect(FITS);
    }
    let mut entries = List::with_capacity(u32::try_from(row.entries.len()).expect(FITS));
    for entry in &row.entries {
        entries
            .push(
                w::LandingEntry::new(
                    &w::CEILINGS,
                    w::LandingEntryParts {
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
                    },
                )
                .expect(FITS),
            )
            .expect(FITS);
    }
    w::LandingRow::new(&w::CEILINGS, w::LandingRowParts { landing, asks, entries, job: super::history::encode_job_record(row.job.as_ref()) }).expect(FITS)
}
fn encode_run(run: top::LandingRun) -> w::LandingRun {
    w::LandingRun::new(&w::CEILINGS, w::LandingRunParts { id: run.id, attempt: run.attempt, ended: run.ended })
        .expect(FITS)
}
fn decode_run(run: w::LandingRun) -> top::LandingRun {
    let w::LandingRunParts { id, attempt, ended } = run.into_parts();
    top::LandingRun { id, attempt, ended }
}
pub(super) fn decode(row: w::LandingRow) -> top::LandingRow {
    let w::LandingRowParts { landing, asks, entries, job } = row.into_parts();
    let w::LandingStateParts { task, project, phase, members, preparation, stale_proofs, branch, round, own_hold, last_left } =
        landing.into_parts();
    let mut decoded_members = List::with_capacity(members.len());
    for member in members.into_boxed() {
        decoded_members.push(decode_member(member)).expect(FITS);
    }
    let mut decoded_asks = List::with_capacity(asks.len());
    for ask in asks.into_boxed() {
        decoded_asks
            .push(match ask.into_parts().ask {
                Some(ask) => Some(decode_ask(ask)),
                None => None,
            })
            .expect(FITS);
    }
    let mut decoded_entries = List::with_capacity(entries.len());
    for entry in entries.into_boxed() {
        let w::LandingEntryParts { entry, member, gate, operation, preparation, base, run } = entry.into_parts();
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
    top::LandingRow {
        job: super::history::decode_job_record(job),
        landing: d::Landing {
            last_left,
            task,
            project,
            phase: match own_hold {
                Some(w::LandingOwnHold::Turn) => {
                    d::Phase::Held { was: Box::new(decode_phase(phase)), why: d::Hold::Turn }
                }
                Some(w::LandingOwnHold::Timeout(value)) => d::Phase::Held {
                    was: Box::new(decode_phase(phase)),
                    why: d::Hold::Timeout { member: value.member(), gate: value.gate(), run: value.run() },
                },
                Some(w::LandingOwnHold::Prepare) => {
                    d::Phase::Held { was: Box::new(decode_phase(phase)), why: d::Hold::Prepare }
                }
                Some(w::LandingOwnHold::Record(value)) => d::Phase::Held {
                    was: Box::new(decode_phase(phase)),
                    why: d::Hold::Record { member: value.member() },
                },
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
    }
}
pub(super) fn encode_receipt(key: &top::ReceiptKey, receipt: &top::Receipt) -> w::ReceiptRow {
    let (name, tier) = match &key.check {
        top::Check::Gate(name) => (name.clone(), false),
        top::Check::Tier(name) => (name.clone(), true),
    };
    let name = w::ReceiptName::new(&w::CEILINGS, w::ReceiptNameParts { name }).expect(FITS);
    let key = w::ReceiptKey::new(
        &w::CEILINGS,
        w::ReceiptKeyParts {
            project: key.project,
            repository: names::encode_repository(key.repository),
            check_name: if tier { w::ReceiptCheck::Tier(name) } else { w::ReceiptCheck::Gate(name) },
            key: key.key,
        },
    )
    .expect(FITS);
    let receipt = encode_receipt_value(receipt);
    w::ReceiptRow::new(&w::CEILINGS, w::ReceiptRowParts { key, receipt }).expect(FITS)
}
pub(super) fn encode_receipt_value(receipt: &top::Receipt) -> w::Receipt {
    let run = w::ReceiptRun::new(
        &w::CEILINGS,
        w::ReceiptRunParts { id: receipt.run.id, number: receipt.run.number, url: receipt.run.url.clone() },
    )
    .expect(FITS);
    w::Receipt::new(
        &w::CEILINGS,
        w::ReceiptParts {
            failures: match &receipt.failures {
                Some(f) => Some(super::verification::encode_failures(f)),
                None => None,
            },
            passed: receipt.passed,
            run,
            candidate: receipt.candidate,
            ended: receipt.ended.as_nanos(),
            logs: receipt.logs.clone(),
        },
    )
    .expect(FITS)
}

pub(super) fn decode_receipt(row: w::ReceiptRow) -> Result<top::Stored, crate::Undecodable> {
    let w::ReceiptRowParts { key, receipt } = row.into_parts();
    let w::ReceiptKeyParts { project, repository, check_name, key } = key.into_parts();
    let check = match check_name {
        w::ReceiptCheck::Gate(name) => top::Check::Gate(name.into_parts().name),
        w::ReceiptCheck::Tier(name) => top::Check::Tier(name.into_parts().name),
    };
    Ok(top::Stored::Receipt {
        key: top::ReceiptKey { project, repository: names::decode_repository(repository), check, key },
        receipt: decode_receipt_value(receipt)?,
    })
}
pub(super) fn decode_receipt_value(receipt: w::Receipt) -> Result<top::Receipt, crate::Undecodable> {
    let w::ReceiptParts { failures, passed, run, candidate, ended, logs } = receipt.into_parts();
    let w::ReceiptRunParts { id, number, url } = run.into_parts();
    Ok(top::Receipt {
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
        d::Leave::Unit { change } => w::LandingLeave::Unit(
            w::LandingLeftChange::new(&w::CEILINGS, w::LandingLeftChangeParts { change }).expect(FITS),
        ),
        d::Leave::Dependency { change } => w::LandingLeave::Dependency(
            w::LandingLeftChange::new(&w::CEILINGS, w::LandingLeftChangeParts { change }).expect(FITS),
        ),
        d::Leave::Blamed { gate, run, red, green } => w::LandingLeave::Blamed(
            w::LandingBlamed::new(&w::CEILINGS, w::LandingBlamedParts { gate, run, red, green }).expect(FITS),
        ),
        d::Leave::HeadMoved { head } => {
            w::LandingLeave::HeadMoved(w::Commit::new(&w::CEILINGS, w::CommitParts { commit: head }).expect(FITS))
        }
        d::Leave::Cancelled => w::LandingLeave::Cancelled,
        d::Leave::Held => w::LandingLeave::Held,
        d::Leave::Refused { gate } => w::LandingLeave::Refused(
            w::LandingGateRefused::new(&w::CEILINGS, w::LandingGateRefusedParts { gate }).expect(FITS),
        ),
        d::Leave::Failed { gate, run } => w::LandingLeave::Failed(
            w::LandingGateFailed::new(&w::CEILINGS, w::LandingGateFailedParts { gate, run }).expect(FITS),
        ),
    }
}
pub(super) fn decode_leave(why: w::LandingLeave) -> d::Leave {
    match why {
        w::LandingLeave::Conflict => d::Leave::Conflict,
        w::LandingLeave::EarlierConflict => d::Leave::EarlierConflict,
        w::LandingLeave::Unit(value) => d::Leave::Unit { change: value.change() },
        w::LandingLeave::Dependency(value) => d::Leave::Dependency { change: value.change() },
        w::LandingLeave::Fallback(_) | w::LandingLeave::Held => d::Leave::Held,
        w::LandingLeave::Blamed(value) => {
            d::Leave::Blamed { gate: value.gate(), run: value.run(), red: *value.red(), green: *value.green() }
        }
        w::LandingLeave::HeadMoved(value) => d::Leave::HeadMoved { head: *value.commit() },
        w::LandingLeave::Cancelled => d::Leave::Cancelled,
        w::LandingLeave::Refused(value) => d::Leave::Refused { gate: value.gate() },
        w::LandingLeave::Failed(value) => d::Leave::Failed { gate: value.gate(), run: value.run() },
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
