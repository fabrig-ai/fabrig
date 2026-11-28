//! Landing and receipt records (protocol/store.md, families 23 and 24).
use super::{FITS, names, w};
use alloc::boxed::Box;
use skein_codec::Record;
use skein_lib::{List, Reader, Wall, Writer};
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
            why:
                d::Hold::Cleanup
                | d::Hold::Prepare
                | d::Hold::Read
                | d::Hold::Record { .. }
                | d::Hold::Timeout { .. }
                | d::Hold::Turn
                | d::Hold::Claimed { .. }
                | d::Hold::Stuck(_),
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
    crate::generated::store::gate_proof_to_record(proof)
}
fn decode_proof(proof: w::GateProof) -> d::GateProof {
    crate::generated::store::gate_proof_to_domain(proof)
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
fn encode_round(round: &d::Round) -> w::LandingRound {
    crate::generated::store::landing_round_to_record(round)
}
fn decode_round(round: w::LandingRound) -> d::Round {
    crate::generated::store::landing_round_to_domain(round)
}
pub(super) fn encode(row: &top::LandingRow) -> w::LandingRow {
    crate::generated::store::landing_row_to_record(row).expect(FITS)
}
pub(super) fn decode_receipt_key(key: w::ReceiptKey) -> top::ReceiptKey {
    crate::generated::store::receipt_key_to_domain(key)
}
pub(super) fn decode(row: w::LandingRow, bounds: &crate::Bounds) -> Result<top::LandingRow, super::Undecodable> {
    crate::generated::store::landing_row_to_domain(row, &super::Context::bounded(bounds))
        .or(Err(super::Undecodable::Malformed))
}
/// Byte-bounded pieces let this row reuse `DispatchWrite` without multiplying
/// that record's independent input ceilings by the number of followed runs.
pub fn landing_follows_to_record(value: &[top::LandingFollow]) -> Result<w::LandingFollows, w::Problem> {
    let bound = w::paths::LANDING_FOLLOWS_ITEMS.problem(skein_codec::Reason::Bound);
    let slots = u32::try_from(value.len()).or(Err(bound))?;
    let mut pieces = List::with_capacity(slots);
    let mut bytes = 0_usize;
    for follow in value {
        let record = crate::generated::store::landing_follow_to_record(follow)?;
        let piece = record.encode(&w::CEILINGS)?;
        bytes = bytes.checked_add(piece_length(piece.len()).ok_or(bound)?).ok_or(bound)?;
        pieces.push(piece).expect("measured follow count");
    }
    if bytes > skein_lib::usize_of(w::CEILINGS.landing_follows_items) {
        return Err(bound);
    }
    let mut writer = Writer::new(bytes);
    for piece in &pieces {
        skein_codec::piece::write(&mut writer, piece).or(Err(bound))?;
    }
    Ok(w::LandingFollows { items: writer.finish() })
}
fn piece_length(bytes: usize) -> Option<usize> {
    bytes.checked_add(4)
}
pub fn landing_follows_to_domain(
    value: w::LandingFollows,
    context: &super::Context,
) -> Result<Box<[top::LandingFollow]>, w::Problem> {
    let bounds = context.bounds();
    let slots = top::landing::follow_slots(bounds.forge()).expect("checked follow bound");
    let path = w::paths::LANDING_FOLLOWS_ITEMS;
    let mut followed = List::with_capacity(slots);
    let mut pieces = Reader::new(&value.items);
    while let Some(piece) =
        skein_codec::piece::split(&mut pieces).or(Err(path.problem(skein_codec::Reason::Malformed)))?
    {
        let row = w::LandingFollow::decode(bounds.limits(), &mut Reader::new(&piece))?;
        let row = crate::generated::store::landing_follow_to_domain(row);
        followed.push(row).or(Err(path.problem(skein_codec::Reason::Bound)))?;
    }
    Ok(followed.into_boxed())
}
pub(super) fn encode_receipt_key(key: &top::ReceiptKey) -> w::ReceiptKey {
    crate::generated::store::receipt_key_to_record(key)
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
            w::ReceiptReport::Tests(crate::generated::store::receipt_tests_to_record(tests).expect(FITS))
        }
    }
}
fn decode_report(report: w::ReceiptReport) -> top::ReceiptReport {
    match report {
        w::ReceiptReport::Unset => top::ReceiptReport::Unset,
        w::ReceiptReport::Unread => top::ReceiptReport::Unread,
        w::ReceiptReport::Many(many) => top::ReceiptReport::Many { bound: many.bound },
        w::ReceiptReport::Tests(tests) => {
            top::ReceiptReport::Tests(crate::generated::store::receipt_tests_to_domain(tests))
        }
    }
}

// Gate reports belong to ReceiptRow, not the value a tier result embeds.
pub(super) fn encode_receipt_value(receipt: &top::Receipt) -> w::Receipt {
    let run = crate::generated::store::receipt_run_to_record(&receipt.run);
    let mut jobs = List::with_capacity(16);
    for job in &receipt.jobs {
        let job = crate::generated::store::receipt_job_to_record(job);
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
        kept.push(crate::generated::store::receipt_job_to_domain(job)).expect(FITS);
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
    let combined = combined?;
    Some(crate::generated::store::landing_combine_to_record(&combined))
}
fn decode_combine(combined: Option<w::LandingCombine>) -> Option<d::Combine> {
    let combined = combined?;
    Some(crate::generated::store::landing_combine_to_domain(combined))
}

pub(super) fn encode_search_tests(search: &d::SearchTests) -> w::LandingSearchTests {
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
                outcomes.push(crate::generated::store::landing_probed_to_record(outcome)).expect(FITS);
            }
            w::LandingSearchTests::ByTest(w::LandingByTest { gates: selected, probed: outcomes })
        }
    }
}
pub(super) fn decode_search_tests(search: w::LandingSearchTests) -> d::SearchTests {
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
                probed.push(crate::generated::store::landing_probed_to_domain(outcome)).expect(FITS);
            }
            d::SearchTests::ByTest { gates: gates.into_boxed(), probed: probed.into_boxed() }
        }
    }
}
fn encode_searched(searched: Option<&d::Searched>) -> Option<w::LandingSearched> {
    let searched = searched?;
    Some(crate::generated::store::landing_searched_to_record(searched).expect(FITS))
}

fn decode_searched(searched: Option<w::LandingSearched>) -> Option<d::Searched> {
    let searched = searched?;
    Some(crate::generated::store::landing_searched_to_domain(searched))
}

fn encode_last_prepare(last: Option<&d::LastPrepare>) -> Option<w::LandingLastPrepare> {
    let last = last?;
    Some(crate::generated::store::landing_last_prepare_to_record(last).expect(FITS))
}

fn decode_last_prepare(last: Option<w::LandingLastPrepare>) -> Option<d::LastPrepare> {
    let last = last?;
    Some(crate::generated::store::landing_last_prepare_to_domain(last))
}

pub(super) fn encode_landing_state(landing: &d::Landing) -> w::LandingState {
    let mut members = List::with_capacity(u32::try_from(landing.members.len()).expect(FITS));
    for member in &landing.members {
        members.push(encode_member(member)).expect(FITS);
    }
    w::LandingState {
        searched: encode_searched(landing.searched.as_ref()),
        last_prepare: encode_last_prepare(landing.last_prepare.as_ref()),
        task: landing.task,
        project: landing.project,
        phase: encode_phase(&landing.phase),
        members,
        preparation: landing.preparation,
        stale_proofs: landing.stale_proofs,
        last_left: landing.last_left,
        branch: landing.branch.clone(),
        round: if landing.round.branch.is_empty() { None } else { Some(encode_round(&landing.round)) },
        own_hold: match landing.phase {
            d::Phase::Held { why: d::Hold::Cleanup, .. } => Some(w::LandingOwnHold::Cleanup),
            d::Phase::Held { why: d::Hold::Prepare, .. } => Some(w::LandingOwnHold::Prepare),
            d::Phase::Held { why: d::Hold::Read, .. } => Some(w::LandingOwnHold::Read),
            d::Phase::Held { why: d::Hold::Record { member }, .. } => {
                Some(w::LandingOwnHold::Record(w::LandingRecordHold { member }))
            }
            d::Phase::Held { why: d::Hold::Timeout { member, gate, run }, .. } => {
                Some(w::LandingOwnHold::Timeout(w::LandingTimeout { member, gate, run }))
            }
            d::Phase::Held { why: d::Hold::Turn, .. } => Some(w::LandingOwnHold::Turn),
            d::Phase::Held { why: d::Hold::Claimed { change }, .. } => {
                Some(w::LandingOwnHold::Claimed(w::LandingClaimed { change }))
            }
            d::Phase::Held { why: d::Hold::Stuck(cause), .. } => Some(w::LandingOwnHold::Stuck(w::LandingStuckHold {
                cause: crate::generated::store::landing_stuck_to_record(&cause),
            })),
            d::Phase::Held { why: d::Hold::Faults { .. }, .. }
            | d::Phase::Waiting
            | d::Phase::Preparing
            | d::Phase::Proving
            | d::Phase::Proved { .. }
            | d::Phase::Turn { .. }
            | d::Phase::Landed
            | d::Phase::Cancelled => None,
        },
    }
}

pub(super) fn decode_landing_state(landing: w::LandingState) -> Result<d::Landing, super::Undecodable> {
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
        last_prepare,
    } = landing;
    if phase.held.is_some() && own_hold.is_some() {
        return Err(super::Undecodable::Malformed);
    }
    let mut decoded_members = List::with_capacity(members.len());
    for member in members.into_boxed() {
        decoded_members.push(decode_member(member)).expect(FITS);
    }
    Ok(d::Landing {
        searched: decode_searched(searched),
        last_left,
        task,
        project,
        phase: match own_hold {
            Some(w::LandingOwnHold::Cleanup) => {
                d::Phase::Held { was: Box::new(decode_phase(phase)), why: d::Hold::Cleanup }
            }
            Some(w::LandingOwnHold::Claimed(value)) => {
                d::Phase::Held { was: Box::new(decode_phase(phase)), why: d::Hold::Claimed { change: value.change } }
            }
            Some(w::LandingOwnHold::Stuck(value)) => d::Phase::Held {
                was: Box::new(decode_phase(phase)),
                why: d::Hold::Stuck(crate::generated::store::landing_stuck_to_domain(value.cause)),
            },
            Some(w::LandingOwnHold::Read) => d::Phase::Held { was: Box::new(decode_phase(phase)), why: d::Hold::Read },
            Some(w::LandingOwnHold::Turn) => d::Phase::Held { was: Box::new(decode_phase(phase)), why: d::Hold::Turn },
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
        last_prepare: decode_last_prepare(last_prepare),
    })
}

#[must_use]
pub fn landing_leave_to_record(value: &d::Leave) -> w::LandingLeave {
    encode_leave(*value)
}

#[must_use]
pub fn landing_leave_to_domain(value: w::LandingLeave) -> d::Leave {
    decode_leave(value)
}

#[must_use]
pub fn landing_search_tests_to_record(value: &d::SearchTests) -> w::LandingSearchTests {
    encode_search_tests(value)
}

#[must_use]
pub fn landing_search_tests_to_domain(value: w::LandingSearchTests) -> d::SearchTests {
    decode_search_tests(value)
}

#[must_use]
pub fn landing_state_to_record(value: &d::Landing) -> w::LandingState {
    encode_landing_state(value)
}

pub fn landing_state_to_domain(value: w::LandingState) -> Result<d::Landing, w::Problem> {
    decode_landing_state(value).or(Err(w::paths::LANDING_STATE_OWN_HOLD.problem(skein_codec::Reason::Malformed)))
}
