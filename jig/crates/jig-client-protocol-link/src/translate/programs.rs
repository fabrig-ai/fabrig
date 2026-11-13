//! Whole program facts, admitted against the document's shared byte budget.
use super::documents::{charge, piece_count};
use super::piece_problem;
use crate::Limits;
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use jig_client_protocol_wire::v21 as w;
use skein_lib::{List, Reader, bytes, length::len_u64};
fn copy(value: &[u8], room: &mut u64) -> Result<Box<[u8]>, w::Problem> {
    charge(room, Some(len_u64(value.len())))?;
    Ok(bytes::copy_of(value))
}
fn failure(value: &w::ProgramFailure) -> c::ProgramFailure {
    match value {
        w::ProgramFailure::Missing => c::ProgramFailure::Missing,
        w::ProgramFailure::Deadline => c::ProgramFailure::Deadline,
        w::ProgramFailure::Unreadable => c::ProgramFailure::Unreadable,
        w::ProgramFailure::Exited => c::ProgramFailure::Exited,
        w::ProgramFailure::Unconfigured => c::ProgramFailure::Unconfigured,
        w::ProgramFailure::Unknown => c::ProgramFailure::Unknown,
        w::ProgramFailure::Closed => c::ProgramFailure::Closed,
    }
}
fn state(value: &w::ProgramState, room: &mut u64) -> Result<c::ProgramState, w::Problem> {
    Ok(match value {
        w::ProgramState::Usable => c::ProgramState::Usable,
        w::ProgramState::NotSignedIn => c::ProgramState::NotSignedIn,
        w::ProgramState::SignedInOtherwise(row) => {
            if !c::valid_program_method(row.method()) {
                return Err(piece_problem(skein_codec::Reason::Malformed));
            }
            c::ProgramState::SignedInOtherwise { method: copy(row.method(), room)? }
        }
        w::ProgramState::Failed(row) => {
            c::ProgramState::Failed { reason: failure(row.reason()), detail: copy(row.detail(), room)? }
        }
    })
}
fn login(value: &w::ProgramLogin, room: &mut u64) -> Result<c::ProgramLogin, w::Problem> {
    if !c::valid_program_name(value.name()) {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    let state = match value.state() {
        Some(value) => Some(state(value, room)?),
        None => None,
    };
    let sign_out = match value.sign_out() {
        Some(w::ProgramSignOut::SignedOut) => Some(c::ProgramSignOut::SignedOut),
        Some(w::ProgramSignOut::Failed(value)) => {
            Some(c::ProgramSignOut::Failed { reason: failure(value.reason()), detail: copy(value.detail(), room)? })
        }
        None => None,
    };
    Ok(c::ProgramLogin { name: copy(value.name(), room)?, state, sign_out, cleanup: value.cleanup(), cut: value.cut() })
}
pub(crate) fn logins(value: &[u8], limits: &Limits, room: &mut u64) -> Result<List<c::ProgramLogin>, w::Problem> {
    let count = piece_count(value)?;
    charge(room, List::<c::ProgramLogin>::price(count))?;
    let mut reader = Reader::new(value);
    let mut rows = List::with_capacity(count);
    for _ in 0..count {
        let item = skein_codec::piece::counted(&mut reader);
        let value = w::ProgramLogin::decode(limits.receiving(), &mut Reader::new(&item))?;
        rows.push(login(&value, room)?).expect("counted login pieces");
    }
    Ok(rows)
}
pub(crate) fn name(value: &[u8], room: &mut u64) -> Result<Box<[u8]>, w::Problem> {
    if !bytes::is_utf8(value) {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    copy(value, room)
}
pub(crate) fn hosts(value: &[u8], limits: &Limits, room: &mut u64) -> Result<List<c::ProgramHost>, w::Problem> {
    let count = piece_count(value)?;
    charge(room, List::<c::ProgramHost>::price(count))?;
    let mut reader = Reader::new(value);
    let mut rows = List::with_capacity(count);
    for _ in 0..count {
        let item = skein_codec::piece::counted(&mut reader);
        let row = w::ProgramHost::decode(limits.receiving(), &mut Reader::new(&item))?;
        rows.push(c::ProgramHost {
            name: name(row.name(), room)?,
            channel: row.channel(),
            connected: row.connected(),
            awaiting_logins: row.awaiting_logins(),
            programs: logins(row.programs(), limits, room)?,
            cut: row.cut(),
        })
        .expect("counted host pieces");
    }
    Ok(rows)
}
