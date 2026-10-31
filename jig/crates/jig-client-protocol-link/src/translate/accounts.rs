#![expect(clippy::manual_map, reason = "step code translates optional facts without closures")]
//! Accounts watch decoding and shared deployment source facts (contract.md, 5.1).
use super::{Limits, List, Reader, Wall, c, piece_problem, too_large, w};
use alloc::boxed::Box;
pub(crate) fn account_use(value: &w::AccountUse) -> c::AccountUse {
    match value {
        w::AccountUse::Party(row) => c::AccountUse::Party(row.party()),
        w::AccountUse::Project(row) => c::AccountUse::Project(row.project()),
    }
}
pub(crate) fn account_holder(value: &w::AccountHolder) -> c::AccountHolder {
    match value {
        w::AccountHolder::Deployment => c::AccountHolder::Deployment,
        w::AccountHolder::Party(row) => c::AccountHolder::Party(row.party()),
    }
}
pub(crate) fn account_kind(value: &w::AccountKind) -> c::AccountKind {
    match value {
        w::AccountKind::SignIn => c::AccountKind::SignIn,
        w::AccountKind::Key => c::AccountKind::Key,
        w::AccountKind::Program => c::AccountKind::Program,
        w::AccountKind::None => c::AccountKind::None,
    }
}
pub(crate) fn account_bound(value: &w::AccountBound) -> c::AccountBound {
    match value {
        w::AccountBound::Accounts => c::AccountBound::Accounts,
        w::AccountBound::HolderAccounts => c::AccountBound::HolderAccounts,
        w::AccountBound::Sources => c::AccountBound::Sources,
        w::AccountBound::SignIns => c::AccountBound::SignIns,
    }
}
pub(crate) fn account_key_failure(value: &w::AccountKeyFailure) -> c::AccountKeyFailure {
    match value {
        w::AccountKeyFailure::TooLong => c::AccountKeyFailure::TooLong,
        w::AccountKeyFailure::NotStored => c::AccountKeyFailure::NotStored,
    }
}
pub(crate) fn account_refusal(value: &w::AccountRefusal) -> c::AccountRefusal {
    match value {
        w::AccountRefusal::NotLimited => c::AccountRefusal::NotLimited,
        w::AccountRefusal::Uses => c::AccountRefusal::Uses,
        w::AccountRefusal::NotKept(row) => c::AccountRefusal::NotKept { why: account_key_failure(row.why()) },
        w::AccountRefusal::Limit(row) => c::AccountRefusal::Limit { bound: account_bound(row.bound()) },
        w::AccountRefusal::Held(row) => c::AccountRefusal::Held { provider: row.provider() },
        w::AccountRefusal::Provider(row) => c::AccountRefusal::Provider { provider: row.provider() },
        w::AccountRefusal::Subject => c::AccountRefusal::Subject,
        w::AccountRefusal::Unknown => c::AccountRefusal::Unknown,
        w::AccountRefusal::Numbers => c::AccountRefusal::Numbers,
        w::AccountRefusal::Kind(row) => c::AccountRefusal::Kind { kind: account_kind(row.kind()) },
        w::AccountRefusal::Closing => c::AccountRefusal::Closing,
        w::AccountRefusal::NotSignedIn => c::AccountRefusal::NotSignedIn,
        w::AccountRefusal::NotOurs => c::AccountRefusal::NotOurs,
        w::AccountRefusal::Cancelled => c::AccountRefusal::Cancelled,
        w::AccountRefusal::Moving => c::AccountRefusal::Moving,
        w::AccountRefusal::NotStarted => c::AccountRefusal::NotStarted,
    }
}
pub(crate) fn account_failure(value: &w::AccountFailure) -> c::AccountFailure {
    match value {
        w::AccountFailure::Later(row) => c::AccountFailure::Later { after: *row.after() },
        w::AccountFailure::SignInAgain => c::AccountFailure::SignInAgain,
        w::AccountFailure::MissingScope(row) => c::AccountFailure::MissingScope { scope: row.scope() },
        w::AccountFailure::Misconfigured => c::AccountFailure::Misconfigured,
        w::AccountFailure::Unloaded(row) => {
            c::AccountFailure::Unloaded { at: account_place(row.at()), why: account_unread(row.why()) }
        }
        w::AccountFailure::Cancelled => c::AccountFailure::Cancelled,
    }
}
pub(crate) fn account_end(value: &w::AccountEnd) -> c::AccountEnd {
    match value {
        w::AccountEnd::Made(row) => c::AccountEnd::Made { account: row.account(), declined: row.declined() },
        w::AccountEnd::Renewed(row) => c::AccountEnd::Renewed { account: row.account(), declined: row.declined() },
        w::AccountEnd::Refused(row) => c::AccountEnd::Refused { refusal: account_refusal(row.refusal()) },
        w::AccountEnd::Failed(row) => c::AccountEnd::Failed { failure: account_failure(row.failure()) },
    }
}
fn account_place(value: &w::AccountPlace) -> c::AccountPlace {
    match value {
        w::AccountPlace::Directory => c::AccountPlace::Directory,
        w::AccountPlace::File => c::AccountPlace::File,
    }
}
fn account_unread(value: &w::AccountUnread) -> c::AccountUnread {
    match value {
        w::AccountUnread::Unreadable => c::AccountUnread::Unreadable,
        w::AccountUnread::Refused => c::AccountUnread::Refused,
        w::AccountUnread::Failed => c::AccountUnread::Failed,
        w::AccountUnread::Stalled => c::AccountUnread::Stalled,
        w::AccountUnread::TooLarge => c::AccountUnread::TooLarge,
    }
}
pub(crate) fn account_uses(value: Option<&List<w::AccountUse>>) -> Option<List<c::AccountUse>> {
    let value = value?;
    let mut uses = List::with_capacity(value.len());
    for item in value {
        uses.push(account_use(item)).expect("supplied uses fit");
    }
    Some(uses)
}
pub(crate) fn account_source(value: &w::AccountSource, room: &mut u64) -> Result<c::AccountSource, w::Problem> {
    let name = match value.name() {
        Some(name) => {
            if !c::valid_program_name(name) {
                return Err(piece_problem(skein_codec::Reason::Malformed));
            }
            charge(room, u64::try_from(name.len()).ok())?;
            Some(skein_lib::bytes::copy_of(name))
        }
        None => None,
    };
    Ok(c::AccountSource {
        name,
        place: value.place(),
        number: value.number(),
        kind: account_kind(value.kind()),
        state: super::account_state_fields(value.state()),
        usable: value.usable(),
        expiring: value.expiring(),
        attention: value.attention(),
        spent_until: match value.spent_until() {
            Some(at) => Some(Wall::from_nanos(*at)),
            None => None,
        },
    })
}
/// Charges an account's own heap, its uses and its display text, to the document's room.
pub(crate) fn charge_account(
    uses: Option<&List<w::AccountUse>>,
    display: &w::DocumentText,
    room: &mut u64,
) -> Result<(), w::Problem> {
    let uses = match uses {
        Some(value) => List::<c::AccountUse>::price(value.len()),
        None => Some(0),
    };
    charge(room, uses)?;
    charge(room, Some(u64::try_from(display.bytes().len()).expect("text fits u64")))
}
pub(crate) fn account_line(
    value: &w::AccountLine,
    limits: &Limits,
    room: &mut u64,
) -> Result<c::AccountLine, w::Problem> {
    charge_account(value.uses().as_ref(), value.display(), room)?;
    Ok(c::AccountLine {
        number: value.number(),
        place: value.place(),
        uses: account_uses(value.uses().as_ref()),
        plan: value.plan(),
        provider: *value.provider(),
        provider_name: match value.provider_name() {
            Some(name) => Some(crate::translate::facts::document_text(name, limits)?),
            None => None,
        },
        manage_usage: checked_manage_usage(value.manage_usage().as_deref(), w::Path::AccountLineManageUsage)?,
        display: crate::translate::facts::document_text(value.display(), limits)?,
        holder: account_holder(value.holder()),
        sources: account_source_piece(value.sources(), limits, room)?,
        cut: value.cut(),
    })
}
pub(crate) fn account_sign_in(value: &w::AccountSignIn) -> c::AccountSignIn {
    c::AccountSignIn { source: value.source(), provider: value.provider(), holder: account_holder(value.holder()) }
}
pub(crate) fn accounts(value: w::Accounts, limits: &Limits) -> Result<c::Accounts, w::Problem> {
    let mut room = document_room(limits);
    charge(&mut room, Some(u64::try_from(size_of::<c::Accounts>()).expect("a record's size fits u64")))?;
    Ok(c::Accounts {
        readiness: super::catalogue::readiness(value.readiness(), limits, &mut room)?,
        readiness_cut: value.readiness_cut(),
        program_hosts: super::super::programs::hosts(value.program_hosts(), limits, &mut room)?,
        accounts: account_line_piece(value.accounts(), limits, &mut room)?,
        signing: account_sign_in_piece(value.signing(), limits, &mut room)?,
        last_end: match value.last_end() {
            Some(row) => Some(c::AccountLastEnd { source: row.source(), end: account_end(row.end()) }),
            None => None,
        },
        usable: value.usable(),
        settled: value.settled(),
        cut: value.cut(),
    })
}
/// The bytes a decoded account list may hold, with the record that holds it. The client bounds a
/// document by its bytes and account counts (client/domain/client.md, 14), and its domain prices
/// each document it holds at twice its byte limit (`jig_client_domain::limits`).
pub(crate) fn document_room(limits: &Limits) -> u64 {
    limits.document_room()
}
/// Takes `bytes` from the document's room; a document they would pass is too large.
pub(crate) fn charge(room: &mut u64, bytes: Option<u64>) -> Result<(), w::Problem> {
    let Some(bytes) = bytes else {
        return Err(too_large());
    };
    match room.checked_sub(bytes) {
        Some(left) => {
            *room = left;
            Ok(())
        }
        None => Err(too_large()),
    }
}
/// The pieces a list holds, each its four length bytes and then its record.
pub(crate) fn piece_count(bytes: &[u8]) -> Result<u32, w::Problem> {
    let mut reader = Reader::new(bytes);
    let mut count = 0_u32;
    for _ in 0..=reader.remaining() / 4 {
        if reader.is_empty() {
            return Ok(count);
        }
        let Some(length) = reader.u32() else {
            return Err(piece_problem(skein_codec::Reason::Malformed));
        };
        if reader.skip(length).is_none() {
            return Err(piece_problem(skein_codec::Reason::Malformed));
        }
        count = count.checked_add(1).expect("pieces are fewer than their bytes");
    }
    unreachable!("each piece takes at least its four length bytes")
}
pub(crate) fn account_source_piece(
    bytes: &[u8],
    limits: &Limits,
    room: &mut u64,
) -> Result<List<c::AccountSource>, w::Problem> {
    let count = piece_count(bytes)?;
    if count > limits.client().accounts.sources {
        return Err(too_large());
    }
    charge(room, List::<c::AccountSource>::price(count))?;
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(count);
    for _ in 0..count {
        let Ok(Some(item)) = skein_codec::piece::split(&mut reader) else { unreachable!("counted pieces split") };
        let row = w::AccountSource::decode(limits.receiving(), &mut Reader::new(&item))?;
        rows.push(account_source(&row, room)?).expect("counted pieces fit");
    }
    Ok(rows)
}
pub(crate) fn account_line_piece(
    bytes: &[u8],
    limits: &Limits,
    room: &mut u64,
) -> Result<List<c::AccountLine>, w::Problem> {
    let count = piece_count(bytes)?;
    if count > limits.client().accounts.accounts {
        return Err(too_large());
    }
    charge(room, List::<c::AccountLine>::price(count))?;
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(count);
    for _ in 0..count {
        let Ok(Some(item)) = skein_codec::piece::split(&mut reader) else { unreachable!("counted pieces split") };
        let row = w::AccountLine::decode(limits.receiving(), &mut Reader::new(&item))?;
        rows.push(account_line(&row, limits, room)?).expect("counted pieces fit");
    }
    Ok(rows)
}
pub(crate) fn account_sign_in_piece(
    bytes: &[u8],
    limits: &Limits,
    room: &mut u64,
) -> Result<List<c::AccountSignIn>, w::Problem> {
    let count = piece_count(bytes)?;
    if count > limits.client().accounts.sign_ins {
        return Err(too_large());
    }
    charge(room, List::<c::AccountSignIn>::price(count))?;
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(count);
    for _ in 0..count {
        let Ok(Some(item)) = skein_codec::piece::split(&mut reader) else { unreachable!("counted pieces split") };
        let row = w::AccountSignIn::decode(limits.receiving(), &mut Reader::new(&item))?;
        rows.push(account_sign_in(&row)).expect("counted pieces fit");
    }
    Ok(rows)
}

/// Both account document forms reject targets outside the contract's renderable set.
pub(crate) fn checked_manage_usage(value: Option<&[u8]>, path: w::Path) -> Result<Option<Box<[u8]>>, w::Problem> {
    match value {
        Some(address) => {
            if !c::valid_manage_usage(address) {
                return Err(w::Problem { path, reason: skein_codec::Reason::Malformed });
            }
            Ok(Some(skein_lib::bytes::copy_of(address)))
        }
        None => Ok(None),
    }
}
