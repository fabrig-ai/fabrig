//! The durable account of a named limit hold (protocol/configuration/limit-settings.md, 6).
use super::{Bounds, FITS, Undecodable, d, names, w};

pub(super) fn encode(row: &d::LimitHold) -> w::LimitHold {
    w::LimitHold::new(
        &w::CEILINGS,
        w::LimitHoldParts {
            task: row.task,
            member: row.member.clone(),
            configured: row.configured,
            needed: row.needed,
            revision: row.revision,
            at: row.at.as_nanos(),
            room: row.room,
            set: encode_set(&row.set),
            order: row.order,
        },
    )
    .expect(FITS)
}
pub(super) fn decode(value: w::LimitHold, bounds: &Bounds) -> Result<d::LimitHold, Undecodable> {
    let w::LimitHoldParts { task, member, configured, needed, revision, at, room, set, order } = value.into_parts();
    Ok(d::LimitHold {
        task,
        member,
        configured,
        needed,
        revision,
        at: skein_lib::Wall::from_nanos(at),
        order,
        room,
        set: decode_set(set, bounds)?,
    })
}
fn encode_set(set: &d::LimitSet) -> w::LimitSet {
    match set {
        d::LimitSet::Operation => w::LimitSet::Operation,
        d::LimitSet::Entries => w::LimitSet::Entries,
        d::LimitSet::Holds => w::LimitSet::Holds,
        d::LimitSet::Tasks => w::LimitSet::Tasks,
        d::LimitSet::Changes => w::LimitSet::Changes,
        d::LimitSet::Steps => w::LimitSet::Steps,
        d::LimitSet::Subscriptions => w::LimitSet::Subscriptions,
        d::LimitSet::Watches => w::LimitSet::Watches,
        d::LimitSet::Heads => w::LimitSet::Heads,
        d::LimitSet::Repositories => w::LimitSet::Repositories,
        d::LimitSet::DesignWriters(name) => w::LimitSet::DesignWriters(names::encode_name(name)),
    }
}
fn decode_set(set: w::LimitSet, bounds: &Bounds) -> Result<d::LimitSet, Undecodable> {
    Ok(match set {
        w::LimitSet::Operation => d::LimitSet::Operation,
        w::LimitSet::Entries => d::LimitSet::Entries,
        w::LimitSet::Holds => d::LimitSet::Holds,
        w::LimitSet::Tasks => d::LimitSet::Tasks,
        w::LimitSet::Changes => d::LimitSet::Changes,
        w::LimitSet::Steps => d::LimitSet::Steps,
        w::LimitSet::Subscriptions => d::LimitSet::Subscriptions,
        w::LimitSet::Watches => d::LimitSet::Watches,
        w::LimitSet::Heads => d::LimitSet::Heads,
        w::LimitSet::Repositories => d::LimitSet::Repositories,
        w::LimitSet::DesignWriters(name) => d::LimitSet::DesignWriters(names::decode_name(name, bounds)?),
    })
}
