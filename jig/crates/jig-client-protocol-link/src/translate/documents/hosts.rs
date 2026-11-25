//! Field-for-field document values (client/domain/contract/documents.md and client/domain/contract/reads.md, 1, and client/domain/contract/watches.md, 1).
use super::{Limits, List, Reader, Wall, c, too_large, w};
use crate::generated::client;
use jig_client_protocol_wire::Record as _;

pub(crate) fn host_run_piece(bytes: &[u8], limits: &Limits, bound: u32) -> Result<List<c::HostRun>, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(bound);
    let mut sum = 0_u64;
    for _ in 0..=bound {
        let Some(item) = crate::translate::next_piece(&mut reader, &mut sum, limits)? else { return Ok(rows) };
        let row = w::HostRun::decode(limits.wire(), &mut Reader::new(&item))?;
        let row = client::host_run_to_domain(row, limits)?;
        if rows.push(row).is_err() {
            return Err(too_large());
        }
    }
    Err(too_large())
}

pub(crate) fn host_fields(value: &w::Host, limits: &Limits, room: &mut u64) -> Result<c::Host, w::Problem> {
    let opt0 = match &value.grace_until {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    let opt1 = match &value.grace_remaining {
        Some(item) => Some(*item),
        None => None,
    };
    let name = match &value.name {
        Some(name) => Some(crate::translate::programs::name(name, room)?),
        None => None,
    };
    Ok(c::Host {
        name,
        awaiting_logins: value.awaiting_logins,
        rule: client::host_rule_to_domain(value.rule.clone()),
        programs: crate::translate::programs::logins(&value.programs, limits, room)?,
        channel: value.channel,
        kind: client::host_kind_to_domain(value.kind.clone()),
        slots: value.slots,
        used: value.used,
        free: value.free,
        draining: value.draining,
        connected: value.connected,
        grace_until: opt0,
        grace_remaining: opt1,
        runs: host_run_piece(&value.runs, limits, limits.client().rows)?,
        cut: value.cut,
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
