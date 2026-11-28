//! Check the recorded room against its holders, not the domain's assertions
//! on mutations (smith/docs/design/testing.md, 6).

use skein_lib::{Id, Token};
use skein_world::inspect::{Inspection, Moment};
use smith_domain_session::{Domain, InspectionBatch, InspectionSession, InspectionShare};

use crate::{KINDS, session::session_key};

/// One session's recorded reservation and independently read holders, in bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoomSums {
    pub reserved: u64,
    pub provider: Option<u64>,
    pub batch: u64,
    pub program: u64,
}

/// Read the session's room holders without changing any reservation.
pub fn room_sums(domain: &Domain, owner: Token, moment: Moment) -> Result<Option<RoomSums>, String> {
    let id = Id::from_token(owner);
    domain.inspection_sessions().get(id).map(|session| sums(session, moment, id)).transpose()
}

/// Check that the sum of the holders is the reservation the session records.
pub fn check_room_sum(moment: Moment, owner: Token, sums: RoomSums) -> Result<(), String> {
    let RoomSums { reserved, provider, batch, program } = sums;
    let provider = provider.unwrap_or(0);
    if u128::from(reserved) == u128::from(provider) + u128::from(batch) + u128::from(program) {
        return Ok(());
    }
    Err(finding(
        moment,
        &format!("session {}", owner.raw()),
        &format!(
            "reserved room {reserved} bytes does not equal provider {provider} + batch {batch} + program {program} bytes"
        ),
    ))
}

/// Check transcript room, the ownership of each held share, and each kit's job cells.
pub fn room(domain: &Domain, moment: Moment) -> Result<(), String> {
    let calls = domain.inspection_calls();
    let sessions = domain.inspection_sessions();
    for (id, session) in sessions {
        let sums = sums(session, moment, id)?;
        check_room_sum(moment, id.token(), sums)?;
        let bytes = session.inspection_room().bytes;
        let limit = domain.inspection_history_bytes();
        if u128::from(bytes) + u128::from(sums.reserved) > u128::from(limit) {
            return Err(finding(
                moment,
                &session_key(id),
                &format!("transcript {bytes} + reserved {} bytes exceeds room {limit} bytes", sums.reserved),
            ));
        }
        if let Some(batch) = session.inspection_batch() {
            for (&block, share) in batch.inspection_shares() {
                let running = share.inspection_run().and_then(|run| {
                    let call = calls.get(run)?;
                    call.inspection_call()?;
                    let (position, slot) = call.inspection_position();
                    Some(
                        call.inspection_session() == id
                            && position == block
                            && session.inspection_running_slot(slot) == Some(run),
                    )
                });
                if !batch.inspection_range().contains(&block) || running != Some(true) {
                    return Err(share_finding(
                        moment,
                        id,
                        batch,
                        block,
                        "held share does not name its running call within the batch",
                    ));
                }
            }
        }
    }
    // Check the converse too: a running call must have its own held share.
    for (id, call) in calls {
        if call.inspection_call().is_none() {
            continue;
        }
        let owner = call.inspection_session();
        let (block, _) = call.inspection_position();
        let batch = sessions.get(owner).and_then(InspectionSession::inspection_batch);
        if batch.and_then(|batch| batch.inspection_shares().get(&block)).and_then(InspectionShare::inspection_run)
            != Some(id)
        {
            return Err(finding(
                moment,
                &format!("{} call {} block {block}", session_key(owner), id.token().raw()),
                "running call has no held share",
            ));
        }
    }
    jobs(domain, moment)
}

fn sums(session: &InspectionSession, moment: Moment, id: Id<InspectionSession>) -> Result<RoomSums, String> {
    let held = session.inspection_room();
    let batch = if let Some(batch) = session.inspection_batch() {
        batch.inspection_shares().iter().try_fold(0_u64, |sum, (&block, share)| {
            sum.checked_add(share.inspection_credit())
                .ok_or_else(|| share_finding(moment, id, batch, block, "held shares exceed u64 bytes"))
        })?
    } else {
        0
    };
    let program = session.inspection_program().iter().try_fold(0_u64, |sum, (_, claim)| {
        let Some((credit, occupied)) = claim.inspection_receiving() else { return Ok(sum) };
        let key = || format!("{} program call {}", session_key(id), claim.inspection_call().owner.raw());
        let unused = credit.checked_sub(occupied).ok_or_else(|| {
            finding(moment, &key(), &format!("program result {occupied} bytes exceeds its credit {credit} bytes"))
        })?;
        sum.checked_add(unused).ok_or_else(|| finding(moment, &key(), "unused program credit exceeds u64 bytes"))
    })?;
    Ok(RoomSums { reserved: held.reserved, provider: held.provider, batch, program })
}

fn jobs(domain: &Domain, moment: Moment) -> Result<(), String> {
    let tools = domain.inspection_tools();
    let flights = tools.inspection_flights();
    let sessions = domain.inspection_sessions();
    for (id, kit) in tools.inspection_kits() {
        let owner = kit.inspection_session();
        let jobs = kit.inspection_jobs();
        let held = jobs.inspection_held();
        let reserved = jobs.inspection_reserved();
        let used = u64::from(reserved.len()) + u64::from(held.len());
        let capacity = held.capacity();
        if used > u64::from(capacity) {
            return Err(finding(
                moment,
                &format!("session {} kit {} job cells", owner.raw(), id.token().raw()),
                &format!("{} reserved + {} held exceeds capacity {capacity}", reserved.len(), held.len()),
            ));
        }
        for (&number, job) in held {
            let live_session =
                sessions.get(Id::from_token(owner)).is_some_and(|session| session.inspection_kit() == id.token());
            let owns_job = flights.get(job.inspection_owner()).is_some_and(|flight| {
                flight.inspection_kit() == id && flight.inspection_job().is_some_and(|job| job.0 == number)
            });
            // An unread terminal owns its job cell independently of live IO.
            if !live_session || (!job.inspection_ended() && !owns_job) {
                return Err(finding(
                    moment,
                    &format!("session {} kit {} job {number}", owner.raw(), id.token().raw()),
                    &format!(
                        "held cell's owner {} is not this kit's run for the job",
                        job.inspection_owner().token().raw()
                    ),
                ));
            }
        }
    }
    Ok(())
}

fn share_finding(
    moment: Moment,
    session: Id<InspectionSession>,
    batch: &InspectionBatch,
    block: u32,
    what: &str,
) -> String {
    let range = batch.inspection_range();
    finding(moment, &format!("{} batch {}..{} block {block}", session_key(session), range.start, range.end), what)
}

fn finding(moment: Moment, object: &str, what: &str) -> String {
    Inspection::new("smith", KINDS).finding(moment, object, what)
}
