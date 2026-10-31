//! Secret-free program facts (accounts.md, 3.4; client/domain/contract.md, 5.1).
use super::{pieces, w};
use alloc::boxed::Box;
use jig_domain_core::documents as d;
use jig_domain_fleet as fleet;
use skein_lib::{List, Writer};
fn failure(value: fleet::SourceFailure) -> w::ProgramFailure {
    match value {
        fleet::SourceFailure::Missing => w::ProgramFailure::Missing,
        fleet::SourceFailure::Deadline => w::ProgramFailure::Deadline,
        fleet::SourceFailure::Unreadable => w::ProgramFailure::Unreadable,
        fleet::SourceFailure::Exited => w::ProgramFailure::Exited,
        fleet::SourceFailure::Unconfigured => w::ProgramFailure::Unconfigured,
        fleet::SourceFailure::Unknown => w::ProgramFailure::Unknown,
        fleet::SourceFailure::Closed => w::ProgramFailure::Closed,
    }
}
fn state(value: &fleet::LoginState, limits: &w::Limits) -> Result<w::ProgramState, w::Problem> {
    Ok(match value {
        fleet::LoginState::Usable => w::ProgramState::Usable,
        fleet::LoginState::NotSignedIn => w::ProgramState::NotSignedIn,
        fleet::LoginState::SignedInOtherwise(method) => w::ProgramState::SignedInOtherwise(w::ProgramOtherwise::new(
            limits,
            w::ProgramOtherwiseParts { method: skein_lib::bytes::copy_of(method.as_bytes()) },
        )?),
        fleet::LoginState::Failed { reason, detail } => w::ProgramState::Failed(w::ProgramFailed::new(
            limits,
            w::ProgramFailedParts { reason: failure(*reason), detail: detail.clone() },
        )?),
    })
}
pub fn login(value: &d::ProgramLogin, limits: &w::Limits) -> Result<w::ProgramLogin, w::Problem> {
    let state = match &value.state {
        Some(value) => Some(state(value, limits)?),
        None => None,
    };
    let sign_out = match &value.sign_out {
        Some(fleet::SignOutResult::SignedOut) => Some(w::ProgramSignOut::SignedOut),
        Some(fleet::SignOutResult::Failed { reason, detail }) => Some(w::ProgramSignOut::Failed(
            w::ProgramFailed::new(limits, w::ProgramFailedParts { reason: failure(*reason), detail: detail.clone() })?,
        )),
        None => None,
    };
    w::ProgramLogin::new(
        limits,
        w::ProgramLoginParts { name: value.name.clone(), state, sign_out, cleanup: value.cleanup, cut: value.cut },
    )
}
pub fn logins(
    value: &List<d::ProgramLogin>,
    limits: &w::Limits,
    bound: u32,
    path: w::Path,
) -> Result<Box<[u8]>, w::Problem> {
    let mut rows = List::with_capacity(value.len());
    for row in value {
        let row = login(row, limits)?;
        let mut out = Writer::new(usize::try_from(row.measure()).expect("measured row fits"));
        row.encode(&mut out).expect("measured login row");
        rows.push(out.finish()).expect("one piece per supplied row");
    }
    pieces(&rows, bound, path)
}
pub fn host(value: &d::ProgramHost, limits: &w::Limits) -> Result<w::ProgramHost, w::Problem> {
    w::ProgramHost::new(
        limits,
        w::ProgramHostParts {
            name: value.name.clone(),
            channel: value.channel,
            connected: value.connected,
            awaiting_logins: value.awaiting_logins,
            programs: logins(&value.programs, limits, limits.program_host_programs, w::Path::ProgramHostPrograms)?,
            cut: value.cut,
        },
    )
}
pub fn hosts(value: &List<d::ProgramHost>, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
    let mut rows = List::with_capacity(value.len());
    for row in value {
        let row = host(row, limits)?;
        let mut out = Writer::new(usize::try_from(row.measure()).expect("measured row fits"));
        row.encode(&mut out).expect("measured program host");
        rows.push(out.finish()).expect("one piece per supplied row");
    }
    pieces(&rows, limits.accounts_program_hosts, w::Path::AccountsProgramHosts)
}
