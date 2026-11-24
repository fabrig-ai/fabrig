//! A goal's reds (temper/docs/design/protocol/store.md, 2).
use super::w;
use crate::{Bounds, Undecodable};
use temper_engine_domain_forge_vocabulary as d;

pub(super) fn encode_reply(reply: &d::RedReply) -> w::RedReply {
    let result = match &reply.result {
        Ok(d::RedOutcome::Recorded { head, recorded }) => w::RedResult::Recorded(w::RedRecorded {
            head: head.clone(),
            generation: recorded.generation,
            replaced: match &recorded.replaced {
                Some(red) => Some(encode(red)),
                None => None,
            },
        }),
        Ok(d::RedOutcome::Ended(red)) => w::RedResult::Ended(encode(red)),
        Err(why) => w::RedResult::Refused(w::RedRefused { why: crate::generated::store::red_refusal_to_record(why) }),
    };
    w::RedReply { goal: reply.goal, key: reply.key.clone(), result }
}
pub(super) fn decode_reply(reply: w::RedReply, bounds: &Bounds) -> Result<d::RedReply, Undecodable> {
    let w::RedReply { goal, key, result } = reply;
    if !d::reds::key_fits(&key, bounds.forge().red_key_bytes) {
        return Err(Undecodable::Malformed);
    }
    let result = match result {
        w::RedResult::Recorded(recorded) => {
            let w::RedRecorded { head, generation, replaced } = recorded;
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
        w::RedResult::Refused(why) => Err(crate::generated::store::red_refusal_to_domain(why.why)),
    };
    Ok(d::RedReply { goal, key, result })
}

pub(super) fn encode_pass(row: &d::RedPass) -> w::RedPass {
    w::RedPass { change: row.change, key: row.key.clone(), generation: row.generation }
}
pub(super) fn decode_pass(row: w::RedPass, bounds: &Bounds) -> Result<d::RedPass, Undecodable> {
    let w::RedPass { change, key, generation } = row;
    if !d::reds::key_fits(&key, bounds.forge().red_key_bytes) {
        return Err(Undecodable::Malformed);
    }
    Ok(d::RedPass { change, key, generation })
}
pub(super) fn encode(row: &d::Red) -> w::Red {
    let by = crate::generated::store::recorder_to_record(&row.by);
    w::Red {
        goal: row.goal,
        key: row.key.clone(),
        head: row.head.clone(),
        why: row.why.clone(),
        by,
        at: row.at,
        generation: row.generation,
    }
}
pub(super) fn decode(row: w::Red, bounds: &Bounds) -> Result<d::Red, Undecodable> {
    if !d::reds::key_fits(&row.key, bounds.forge().red_key_bytes)
        || !d::settings::words_fit(&row.why, 256)
        || (row.head.len() != 20 && row.head.len() != 32)
    {
        return Err(Undecodable::Malformed);
    }
    let by = crate::generated::store::recorder_to_domain(row.by);
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

/// Owner-shaped `Red` (skein/docs/design/data/codec/maps.md, 3.3).
#[must_use]
pub fn red_to_record(value: &d::Red) -> w::Red {
    encode(value)
}
pub fn red_to_domain(value: w::Red, context: &super::Context) -> Result<d::Red, w::Problem> {
    decode(value, context.bounds()).or(Err(w::paths::RED_GOAL.problem(skein_codec::Reason::Malformed)))
}

/// Owner-shaped `RedReply` (skein/docs/design/data/codec/maps.md, 3.3).
#[must_use]
pub fn red_reply_to_record(value: &d::RedReply) -> w::RedReply {
    encode_reply(value)
}
pub fn red_reply_to_domain(value: w::RedReply, context: &super::Context) -> Result<d::RedReply, w::Problem> {
    decode_reply(value, context.bounds()).or(Err(w::paths::RED_REPLY_GOAL.problem(skein_codec::Reason::Malformed)))
}
