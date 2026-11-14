//! A goal's reds (temper/docs/design/protocol/store.md, 2).
use super::{FITS, w};
use crate::{Bounds, Undecodable};
use temper_engine_domain_forge_vocabulary as d;

pub(super) fn encode_reply(reply: &d::RedReply) -> w::RedReply {
    let result = match &reply.result {
        Ok(d::RedOutcome::Recorded { head, recorded }) => w::RedResult::Recorded(
            w::RedRecorded::new(
                &w::CEILINGS,
                w::RedRecordedParts {
                    head: head.clone(),
                    generation: recorded.generation,
                    replaced: match &recorded.replaced {
                        Some(red) => Some(encode(red)),
                        None => None,
                    },
                },
            )
            .expect(FITS),
        ),
        Ok(d::RedOutcome::Ended(red)) => w::RedResult::Ended(encode(red)),
        Err(why) => w::RedResult::Refused(
            w::RedRefused::new(
                &w::CEILINGS,
                w::RedRefusedParts {
                    why: match why {
                        d::RedRefusal::NoGoal => w::RedRefusal::NoGoal,
                        d::RedRefusal::Full { bound } => w::RedRefusal::Full(
                            w::RedFull::new(&w::CEILINGS, w::RedFullParts { bound: *bound }).expect(FITS),
                        ),
                        d::RedRefusal::NoSuch => w::RedRefusal::NoSuch,
                    },
                },
            )
            .expect(FITS),
        ),
    };
    w::RedReply::new(&w::CEILINGS, w::RedReplyParts { goal: reply.goal, key: reply.key.clone(), result }).expect(FITS)
}
pub(super) fn decode_reply(reply: w::RedReply, bounds: &Bounds) -> Result<d::RedReply, Undecodable> {
    let w::RedReplyParts { goal, key, result } = reply.into_parts();
    if !d::reds::key_fits(&key, bounds.forge().red_key_bytes) {
        return Err(Undecodable::Malformed);
    }
    let result = match result {
        w::RedResult::Recorded(recorded) => {
            let w::RedRecordedParts { head, generation, replaced } = recorded.into_parts();
            if head.len() != 20 && head.len() != 32 {
                return Err(Undecodable::Malformed);
            }
            let replaced = match replaced {
                Some(red) => Some(decode(red, bounds)?),
                None => None,
            };
            Ok(d::RedOutcome::Recorded { head, recorded: d::RedRecorded { generation, replaced } })
        }
        w::RedResult::Ended(red) => Ok(d::RedOutcome::Ended(decode(red, bounds)?)),
        w::RedResult::Refused(why) => Err(match why.into_parts().why {
            w::RedRefusal::NoGoal => d::RedRefusal::NoGoal,
            w::RedRefusal::Full(bound) => d::RedRefusal::Full { bound: bound.bound() },
            w::RedRefusal::NoSuch => d::RedRefusal::NoSuch,
        }),
    };
    Ok(d::RedReply { goal, key, result })
}

pub(super) fn encode_pass(row: &d::RedPass) -> w::RedPass {
    w::RedPass::new(
        &w::CEILINGS,
        w::RedPassParts { change: row.change, key: row.key.clone(), generation: row.generation },
    )
    .expect(FITS)
}
pub(super) fn decode_pass(row: w::RedPass, bounds: &Bounds) -> Result<d::RedPass, Undecodable> {
    let w::RedPassParts { change, key, generation } = row.into_parts();
    if !d::reds::key_fits(&key, bounds.forge().red_key_bytes) {
        return Err(Undecodable::Malformed);
    }
    Ok(d::RedPass { change, key, generation })
}
pub(super) fn encode(row: &d::Red) -> w::Red {
    let by = match row.by {
        d::Recorder::Task(number) => {
            w::Recorder::Task(w::Number::new(&w::CEILINGS, w::NumberParts { number }).expect(FITS))
        }
        d::Recorder::Party(number) => {
            w::Recorder::Party(w::Number::new(&w::CEILINGS, w::NumberParts { number }).expect(FITS))
        }
    };
    w::Red::new(
        &w::CEILINGS,
        w::RedParts {
            goal: row.goal,
            key: row.key.clone(),
            head: row.head.clone(),
            why: row.why.clone(),
            by,
            at: row.at,
            generation: row.generation,
        },
    )
    .expect(FITS)
}
pub(super) fn decode(row: w::Red, bounds: &Bounds) -> Result<d::Red, Undecodable> {
    let row = row.into_parts();
    if !d::reds::key_fits(&row.key, bounds.forge().red_key_bytes)
        || !d::settings::words_fit(&row.why, 256)
        || (row.head.len() != 20 && row.head.len() != 32)
    {
        return Err(Undecodable::Malformed);
    }
    let by = match row.by {
        w::Recorder::Task(number) => d::Recorder::Task(number.number()),
        w::Recorder::Party(number) => d::Recorder::Party(number.number()),
    };
    Ok(d::Red {
        goal: row.goal,
        key: row.key,
        head: row.head,
        why: row.why,
        by,
        at: row.at,
        generation: row.generation,
    })
}
