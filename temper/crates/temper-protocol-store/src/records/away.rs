//! Away and decision families (temper/docs/design/protocol/store.md, 2).
use super::{FITS, w};
use crate::Undecodable;
use skein_lib::{List, Wall};
use temper_engine_domain_forge as d;

fn wall(value: Option<u64>) -> Option<Wall> {
    match value {
        Some(value) => Some(Wall::from_nanos(value)),
        None => None,
    }
}
fn nanos(value: Option<Wall>) -> Option<u64> {
    match value {
        Some(value) => Some(value.as_nanos()),
        None => None,
    }
}
pub(super) fn encode_away(row: &d::Away) -> w::Away {
    w::Away::new(
        &w::CEILINGS,
        w::AwayParts {
            party: row.party,
            project: row.project,
            since: row.since.as_nanos(),
            until: nanos(row.until),
            kinds: row.kinds,
            landed: row.landed,
            off: nanos(row.off),
        },
    )
    .expect(FITS)
}
pub(super) fn decode_away(row: w::Away) -> Result<d::Away, Undecodable> {
    let w::AwayParts { party, project, since, until, kinds, landed, off } = row.into_parts();
    if kinds > 15 {
        return Err(Undecodable);
    }
    Ok(d::Away { party, project, since: Wall::from_nanos(since), until: wall(until), kinds, landed, off: wall(off) })
}
pub(super) fn encode_decided(row: &d::Decided) -> w::Decided {
    let mut options = List::with_capacity(u32::try_from(row.options.len()).expect("bounded options"));
    for option in &row.options {
        options.push(option.clone()).expect("sized option list");
    }
    w::Decided::new(
        &w::CEILINGS,
        w::DecidedParts {
            project: row.project,
            number: row.number,
            party: row.party,
            task: row.task,
            at: row.at.as_nanos(),
            question: row.question.clone(),
            options,
            took: row.took,
            why: row.why.clone(),
            overturned: row.overturned,
        },
    )
    .expect(FITS)
}
pub(super) fn decode_decided(row: w::Decided) -> Result<d::Decided, Undecodable> {
    let w::DecidedParts { project, number, party, task, at, question, options, took, why, overturned } =
        row.into_parts();
    let options = options.into_boxed();
    if !d::DECIDED_OPTIONS.contains(&options.len()) || options.get(usize::from(took)).is_none() {
        return Err(Undecodable);
    }
    Ok(d::Decided { project, number, party, task, at: Wall::from_nanos(at), question, options, took, why, overturned })
}
