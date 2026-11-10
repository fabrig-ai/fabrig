//! Field-for-field document values (client/domain/contract/documents.md and client/domain/contract/README.md, 4–5).
use super::{Limits, List, Reader, Wall, c, task_link_fields, too_large, w};

pub(crate) fn store_fill_fields(value: &w::StoreFill) -> c::StoreFill {
    c::StoreFill { bytes: value.bytes(), mark: value.mark() }
}

pub(crate) fn host_kind_fields(value: &w::HostKind) -> c::HostKind {
    match value {
        w::HostKind::Worker => c::HostKind::Worker,
        w::HostKind::Engine => c::HostKind::Engine,
    }
}

pub(crate) fn attempt_state_fields(value: &w::AttemptState) -> c::AttemptState {
    match value {
        w::AttemptState::Waiting => c::AttemptState::Waiting,
        w::AttemptState::Adopted => c::AttemptState::Adopted,
        w::AttemptState::Claimed => c::AttemptState::Claimed,
        w::AttemptState::Cancelled => c::AttemptState::Cancelled,
        w::AttemptState::Handed => c::AttemptState::Handed,
        w::AttemptState::Acknowledged => c::AttemptState::Acknowledged,
        w::AttemptState::Stray => c::AttemptState::Stray,
        w::AttemptState::Kept => c::AttemptState::Kept,
        w::AttemptState::Fenced => c::AttemptState::Fenced,
        w::AttemptState::Closed => c::AttemptState::Closed,
    }
}

pub(crate) fn run_location_fields(value: &w::RunLocation) -> c::RunLocation {
    match value {
        w::RunLocation::None => c::RunLocation::None,
        w::RunLocation::Connected(row) => {
            c::RunLocation::Connected { channel: row.channel(), kind: host_kind_fields(row.kind()) }
        }
        w::RunLocation::Adrift(row) => {
            let opt0 = match row.channel() {
                Some(item) => Some(*item),
                None => None,
            };
            let opt1 = match row.until() {
                Some(item) => Some(Wall::from_nanos(*item)),
                None => None,
            };
            let opt2 = match row.remaining() {
                Some(item) => Some(*item),
                None => None,
            };
            c::RunLocation::Adrift { channel: opt0, until: opt1, remaining: opt2 }
        }
    }
}

pub(crate) fn host_run_fields(value: &w::HostRun, limits: &Limits) -> Result<c::HostRun, w::Problem> {
    let opt0 = match value.grace_until() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::HostRun {
        task: task_link_fields(value.task(), limits)?,
        attempt: value.attempt(),
        kept: value.kept(),
        listed: value.listed(),
        state: attempt_state_fields(value.state()),
        location: run_location_fields(value.location()),
        grace_until: opt0,
    })
}

pub(crate) fn host_run_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::HostRun>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let row = w::HostRun::decode(limits.wire(), &mut Reader::new(&item))?;
        let row = host_run_fields(&row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn host_fields(value: &w::Host, limits: &Limits, room: &mut u64) -> Result<c::Host, w::Problem> {
    let opt0 = match value.grace_until() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt1 = match value.grace_remaining() {
        Some(item) => Some(*item),
        None => None,
    };
    let name = match value.name() {
        Some(name) => Some(crate::translate::programs::name(name, room)?),
        None => None,
    };
    Ok(c::Host {
        name,
        awaiting_logins: value.awaiting_logins(),
        rule: match value.rule() {
            w::HostRule::Open => c::HostRule::Open,
            w::HostRule::Person(person) => c::HostRule::Person(person.party()),
            w::HostRule::Several => c::HostRule::Several,
            w::HostRule::Unknown => c::HostRule::Unknown,
        },
        programs: crate::translate::programs::logins(value.programs(), limits, room)?,
        channel: value.channel(),
        kind: host_kind_fields(value.kind()),
        slots: value.slots(),
        used: value.used(),
        free: value.free(),
        draining: value.draining(),
        connected: value.connected(),
        grace_until: opt0,
        grace_remaining: opt1,
        runs: host_run_piece(value.runs(), limits, limits.client().rows)?,
        cut: value.cut(),
    })
}

pub(crate) fn host_piece(
    bytes: &[u8],
    limits: &Limits,
    bound: u32,
    room: &mut u64,
) -> Result<List<c::Host>, w::Problem> {
    let count = super::piece_count(bytes)?;
    if count > bound {
        return Err(too_large());
    }
    super::charge(room, List::<c::Host>::price(bound))?;
    let mut rows = List::with_capacity(bound);
    let mut reader = Reader::new(bytes);
    for _ in 0..count {
        let item = skein_codec::piece::counted(&mut reader);
        let row = w::Host::decode(limits.wire(), &mut Reader::new(&item))?;
        rows.push(host_fields(&row, limits, room)?).expect("counted host pieces");
    }
    Ok(rows)
}
