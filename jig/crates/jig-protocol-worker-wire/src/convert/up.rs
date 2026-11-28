use super::walk::Walk;
use super::{Halt, codec_error};
use crate::layouts::{Nested, Peer, Unconverted};
use crate::{Carried, Kind, v22 as wire};
use alloc::boxed::Box;
use skein_codec::layout::{Misconverted, Mode};
use skein_lib::{Buffer, Reader};
use wire::Record;

pub(super) fn decode(
    peer: &Peer,
    source: u16,
    kind: Kind,
    body: &[u8],
    limits: &wire::Limits,
) -> Result<wire::FromWorker, Unconverted> {
    let number = kind.number();
    let offset = number.checked_sub(0x0100).expect("engine receives only worker kinds");
    let tag = u8::try_from(offset).expect("worker kind has a one-byte tag");
    let capacity = u32::try_from(body.len()).expect("the record plan's buffer is bounded by u32");
    let Some(capacity) = capacity.checked_add(1) else {
        return Err(peer.frame_problem(Mode::Read, source, Misconverted::Room));
    };
    let mut tagged = Buffer::with_capacity(capacity);
    tagged.put(&[tag]).expect("room for the tag");
    tagged.put(body).expect("room for the body");
    match wire::FromWorker::decode(limits, &mut Reader::new(tagged.as_slice())) {
        Ok(record) => Ok(record),
        Err(error) => Err(codec_error(peer, Mode::Read, source, error)),
    }
}

pub(super) fn parts(walk: &mut Walk<'_>, record: &mut wire::FromWorker, limits: &wire::Limits) -> Result<bool, Halt> {
    match record {
        wire::FromWorker::Ask(record) => {
            walk.part(
                Nested::Carried(Carried::Ask),
                &mut record.bytes,
                limits.ask_bytes.min(wire::CEILINGS.ask_bytes),
            )?;
        }
        wire::FromWorker::Turn(record) => {
            walk.part(Nested::Turn, &mut record.body, limits.turn_body.min(wire::CEILINGS.turn_body))?;
        }
        wire::FromWorker::Part(record) => {
            walk.part(Nested::Turn, &mut record.body, limits.part_body.min(wire::CEILINGS.part_body))?;
        }
        wire::FromWorker::Answer(record) => ending(walk, &mut record.ending, limits)?,
        wire::FromWorker::JobAnswer(record) => job_ending(walk, &mut record.ending, limits)?,
        wire::FromWorker::Hello(_)
        | wire::FromWorker::Call(_)
        | wire::FromWorker::Fact(_)
        | wire::FromWorker::Unauthorized(_)
        | wire::FromWorker::Limited(_)
        | wire::FromWorker::AccountRefused(_)
        | wire::FromWorker::Logins(_)
        | wire::FromWorker::SignedOut(_)
        | wire::FromWorker::Usage(_)
        | wire::FromWorker::UsageRead(_)
        | wire::FromWorker::ListRead(_)
        | wire::FromWorker::Paused(_)
        | wire::FromWorker::Draining(_) => return Ok(false),
    }
    Ok(true)
}

fn ending(walk: &mut Walk<'_>, ending: &mut wire::Ending, limits: &wire::Limits) -> Result<(), Halt> {
    match ending {
        wire::Ending::Ended(record) => {
            walk.part(Nested::Result, &mut record.outcome, limits.ended_outcome.min(wire::CEILINGS.ended_outcome))?;
            work(walk, &mut record.work, limits)?;
        }
        wire::Ending::Parked(record) => work(walk, &mut record.work, limits)?,
        wire::Ending::Failed(record) => {
            work(walk, &mut record.work, limits)?;
            match &mut record.failure {
                wire::Failure::Unprepared(record) => preparation(walk, &mut record.value, limits)?,
                wire::Failure::Run(_) | wire::Failure::Agent(_) | wire::Failure::Cancelled(_) => {}
            }
        }
        wire::Ending::Refused(_) => {}
    }
    Ok(())
}

fn job_ending(walk: &mut Walk<'_>, ending: &mut wire::JobEnding, limits: &wire::Limits) -> Result<(), Halt> {
    match ending {
        wire::JobEnding::Finished(record) => {
            work(walk, &mut record.work, limits)?;
            walk.part(
                Nested::Carried(Carried::JobResult),
                &mut record.result,
                limits.job_finished_result.min(wire::CEILINGS.job_finished_result),
            )?;
        }
        wire::JobEnding::Failed(record) => {
            work(walk, &mut record.work, limits)?;
            match &mut record.failure {
                wire::JobFailure::Unprepared(record) => preparation(walk, &mut record.value, limits)?,
                wire::JobFailure::Failed
                | wire::JobFailure::Deadline
                | wire::JobFailure::Result
                | wire::JobFailure::Unstarted
                | wire::JobFailure::Undelivered(_)
                | wire::JobFailure::Cancelled(_) => {}
            }
        }
        wire::JobEnding::Refused(_) => {}
    }
    Ok(())
}

fn work(walk: &mut Walk<'_>, work: &mut wire::Work, limits: &wire::Limits) -> Result<(), Halt> {
    optional(walk, Carried::Left, &mut work.left, limits.work_left_some.min(wire::CEILINGS.work_left_some))?;
    optional(walk, Carried::Saved, &mut work.saved, limits.work_saved_some.min(wire::CEILINGS.work_saved_some))
}

fn preparation(walk: &mut Walk<'_>, value: &mut wire::Preparation, limits: &wire::Limits) -> Result<(), Halt> {
    match value {
        wire::Preparation::Permanent(record) => optional(
            walk,
            Carried::Preparation,
            &mut record.resource,
            limits.resource_resource_some.min(wire::CEILINGS.resource_resource_some),
        ),
        wire::Preparation::Transient => Ok(()),
    }
}

fn optional(walk: &mut Walk<'_>, carried: Carried, bytes: &mut Option<Box<[u8]>>, bound: u32) -> Result<(), Halt> {
    if let Some(bytes) = bytes {
        walk.part(Nested::Carried(carried), bytes, bound)?;
    }
    Ok(())
}
