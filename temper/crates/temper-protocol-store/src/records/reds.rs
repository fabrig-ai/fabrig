//! A goal's reds (temper/docs/design/protocol/store.md, 2).
use super::{FITS, w};
use crate::{Bounds, Undecodable};
use temper_engine_domain_forge_vocabulary as d;

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
