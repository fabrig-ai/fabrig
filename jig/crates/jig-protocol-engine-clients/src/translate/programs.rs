//! Secret-free program facts (domain/accounts-sources.md, 1; client/domain/contract/documents.md, 1).
use super::{pieces, w};
use crate::generated::client as mapped;
use alloc::boxed::Box;
use jig_client_protocol_wire::Record as _;
use jig_domain_core_vocabulary::documents as d;
use skein_lib::{List, Writer};
#[must_use]
pub fn login(value: &d::ProgramLogin) -> w::ProgramLogin {
    mapped::program_login_to_record(value)
}
pub fn logins(
    value: &List<d::ProgramLogin>,
    limits: &w::Limits,
    bound: u32,
    path: w::Path,
) -> Result<Box<[u8]>, w::Problem> {
    let mut rows = List::with_capacity(value.len());
    for row in value {
        let row = login(row);
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        row.write(limits, &mut out).expect("measured login row");
        rows.push(out.finish()).expect("one piece per supplied row");
    }
    pieces(&rows, bound, path)
}
pub fn host(value: &d::ProgramHost, limits: &w::Limits) -> Result<w::ProgramHost, w::Problem> {
    Ok(w::ProgramHost {
        name: value.name.clone(),
        channel: value.channel,
        connected: value.connected,
        awaiting_logins: value.awaiting_logins,
        programs: logins(&value.programs, limits, limits.program_host_programs, w::paths::PROGRAM_HOST_PROGRAMS)?,
        cut: value.cut,
    })
}
pub fn hosts(value: &List<d::ProgramHost>, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
    let mut rows = List::with_capacity(value.len());
    for row in value {
        let row = host(row, limits)?;
        let mut out = Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        row.write(limits, &mut out).expect("measured program host");
        rows.push(out.finish()).expect("one piece per supplied row");
    }
    pieces(&rows, limits.accounts_program_hosts, w::paths::ACCOUNTS_PROGRAM_HOSTS)
}
