//! Accounts watch decoding and shared deployment source facts (client/domain/contract/documents.md, 1).
use super::{Limits, List, Reader, Wall, c, chip_piece, piece_problem, too_large, w};
use alloc::boxed::Box;
use skein_lib::length::len_u64;
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
        w::AccountRefusal::ProgramNameReserved => c::AccountRefusal::ProgramNameReserved,
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
            charge(room, Some(len_u64(name.len())))?;
            Some(skein_lib::bytes::copy_of(name))
        }
        None => None,
    };
    Ok(c::AccountSource {
        name,
        place: value.place(),
        number: value.number(),
        kind: account_kind(value.kind()),
        state: account_state_fields(value.state()),
        usable: value.usable(),
        expiring: value.expiring(),
        attention: value.attention(),
        spent_until: match value.spent_until() {
            Some(at) => Some(Wall::from_nanos(*at)),
            None => None,
        },
    })
}
/// Charges an account's uses, display and whole facts to the document's room.
pub(crate) fn charge_account(
    uses: Option<&List<w::AccountUse>>,
    display: &w::DocumentText,
    facts: &List<w::AccountFact>,
    room: &mut u64,
) -> Result<(), w::Problem> {
    let uses = match uses {
        Some(value) => List::<c::AccountUse>::price(value.len()),
        None => Some(0),
    };
    charge(room, uses)?;
    charge(room, Some(len_u64(display.bytes().len())))?;
    charge(room, List::<c::AccountFact>::price(facts.len()))?;
    for fact in facts {
        charge(room, Some(len_u64(fact.label().len())))?;
        match fact.value() {
            w::AccountFactValue::Text(text) => charge(room, Some(len_u64(text.text().len())))?,
            w::AccountFactValue::Count(_)
            | w::AccountFactValue::Instant(_)
            | w::AccountFactValue::Span(_)
            | w::AccountFactValue::Share(_)
            | w::AccountFactValue::Window(_) => {}
        }
    }
    Ok(())
}
pub(crate) fn account_line(
    value: &w::AccountLine,
    limits: &Limits,
    room: &mut u64,
) -> Result<c::AccountLine, w::Problem> {
    charge_account(value.uses().as_ref(), value.display(), value.facts(), room)?;
    Ok(c::AccountLine {
        list: account_list(value.list().as_ref()),
        facts: account_facts(value.facts())?,
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
        lendable: value.lendable(),
        lending: terminal_login(value.lending().as_ref(), &mut room)?,
        lent_end: match value.lent_end() {
            Some(row) => Some(c::AccountLentEnd { number: row.number(), end: login_end(row.end()) }),
            None => None,
        },
        last_end: match value.last_end() {
            Some(row) => Some(c::AccountLastEnd { source: row.source(), end: account_end(row.end()) }),
            None => None,
        },
        usable: value.usable(),
        settled: value.settled(),
        cut: value.cut(),
    })
}
pub(crate) fn terminal(value: w::Terminal, limits: &Limits) -> Result<c::Terminal, w::Problem> {
    let mut room = document_room(limits);
    charge(&mut room, Some(u64::try_from(size_of::<c::Terminal>()).expect("a record's size fits u64")))?;
    Ok(c::Terminal { login: terminal_login(value.login().as_ref(), &mut room)? })
}
/// The terminal's variable name shares the owning document's native byte budget.
pub(crate) fn terminal_login(value: Option<&w::TerminalLogin>, room: &mut u64) -> Result<Option<c::TerminalLogin>, w::Problem> {
    let Some(value) = value else { return Ok(None) };
    if !c::valid_program_name(value.name()) {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    charge(room, Some(len_u64(value.name().len())))?;
    Ok(Some(c::TerminalLogin {
        number: value.number(), provider: value.provider(), name: skein_lib::bytes::copy_of(value.name()), cancelled: value.cancelled(),
    }))
}
pub(crate) fn login_exit(value: &w::LoginExit) -> c::LoginExit {
    match value {
        w::LoginExit::Exited(row) => c::LoginExit::Exited { code: row.code() },
        w::LoginExit::Killed(row) => c::LoginExit::Killed { signal: row.signal() },
        w::LoginExit::NotStarted => c::LoginExit::NotStarted,
    }
}
pub(crate) fn login_end(value: &w::LoginEnd) -> c::LoginEnd {
    match value {
        w::LoginEnd::SignedIn(row) => c::LoginEnd::SignedIn { source: row.source() },
        w::LoginEnd::NotFinished(row) => c::LoginEnd::NotFinished { exit: match row.exit() {
            Some(exit) => Some(login_exit(exit)), None => None,
        } },
        w::LoginEnd::Cancelled => c::LoginEnd::Cancelled,
        w::LoginEnd::Refused(row) => c::LoginEnd::Refused { refusal: account_refusal(row.refusal()) },
    }
}
/// The bytes a decoded account list may hold, with the record that holds it. The client bounds a
/// document by its bytes and account counts (client/domain/client/README.md, 11), and its domain prices
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
    match skein_codec::piece::count(bytes) {
        Ok(count) => Ok(count),
        Err(reason) => Err(piece_problem(reason)),
    }
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
        let item = skein_codec::piece::counted(&mut reader);
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
        let item = skein_codec::piece::counted(&mut reader);
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
        let item = skein_codec::piece::counted(&mut reader);
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

pub(crate) fn account_place_fields(value: &w::AccountPlace) -> c::AccountPlace {
    match value {
        w::AccountPlace::Directory => c::AccountPlace::Directory,
        w::AccountPlace::File => c::AccountPlace::File,
    }
}

pub(crate) fn account_unread_fields(value: &w::AccountUnread) -> c::AccountUnread {
    match value {
        w::AccountUnread::Unreadable => c::AccountUnread::Unreadable,
        w::AccountUnread::Refused => c::AccountUnread::Refused,
        w::AccountUnread::Failed => c::AccountUnread::Failed,
        w::AccountUnread::Stalled => c::AccountUnread::Stalled,
        w::AccountUnread::TooLarge => c::AccountUnread::TooLarge,
    }
}

pub(crate) fn account_state_fields(value: &w::AccountState) -> c::AccountState {
    match value {
        w::AccountState::Asking => c::AccountState::Asking,
        w::AccountState::Granted => c::AccountState::Granted,
        w::AccountState::Later(row) => {
            let opt0 = match row.until() {
                Some(item) => Some(Wall::from_nanos(*item)),
                None => None,
            };
            c::AccountState::Later { until: opt0 }
        }
        w::AccountState::SignInAgain => c::AccountState::SignInAgain,
        w::AccountState::Misconfigured => c::AccountState::Misconfigured,
        w::AccountState::Unloaded(row) => {
            c::AccountState::Unloaded { at: account_place_fields(row.at()), why: account_unread_fields(row.why()) }
        }
        w::AccountState::Cancelled => c::AccountState::Cancelled,
        w::AccountState::Closing => c::AccountState::Closing,
        w::AccountState::Moving => c::AccountState::Moving,
    }
}

pub(crate) fn account_fields(value: &w::Account, limits: &Limits, room: &mut u64) -> Result<c::Account, w::Problem> {
    charge_account(value.uses().as_ref(), value.display(), value.facts(), room)?;
    // An account's runs and its waiting tasks stay lists bounded by the rows.
    let chips = List::<c::Chip>::price(limits.client().rows);
    charge(room, chips)?;
    charge(room, chips)?;
    let opt0 = match value.generation() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt1 = match value.spent_until() {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::Account {
        list: account_list(value.list().as_ref()),
        facts: account_facts(value.facts())?,
        number: value.number(),
        place: value.place(),
        uses: account_uses(value.uses().as_ref()),
        plan: value.plan(),
        provider: *value.provider(),
        provider_name: match value.provider_name() {
            Some(name) => Some(crate::translate::facts::document_text(name, limits)?),
            None => None,
        },
        manage_usage: checked_manage_usage(value.manage_usage().as_deref(), w::Path::AccountManageUsage)?,
        display: crate::translate::facts::document_text(value.display(), limits)?,
        holder: account_holder(value.holder()),
        sources: account_source_piece(value.sources(), limits, room)?,
        generation: opt0,
        state: account_state_fields(value.state()),
        usable: value.usable(),
        expiring: value.expiring(),
        attention: value.attention(),
        spent_until: opt1,
        runs: chip_piece(value.runs(), limits, limits.client().rows)?,
        waiting: chip_piece(value.waiting(), limits, limits.client().rows)?,
        cut: value.cut(),
    })
}

pub(crate) fn account_piece(bytes: &[u8], limits: &Limits, room: &mut u64) -> Result<List<c::Account>, w::Problem> {
    let count = piece_count(bytes)?;
    if count > limits.client().accounts.accounts {
        return Err(too_large());
    }
    charge(room, List::<c::Account>::price(count))?;
    let mut reader = Reader::new(bytes);
    let mut rows = List::with_capacity(count);
    for _ in 0..count {
        let item = skein_codec::piece::counted(&mut reader);
        let row = w::Account::decode(limits.receiving(), &mut Reader::new(&item))?;
        rows.push(account_fields(&row, limits, room)?).expect("counted pieces fit");
    }
    Ok(rows)
}

pub(crate) fn account_list(value: Option<&w::AccountList>) -> Option<c::AccountList> {
    let value = value?;
    Some(c::AccountList {
        read: match value.read() {
            Some(at) => Some(Wall::from_nanos(*at)),
            None => None,
        },
        tried: match value.tried() {
            Some(at) => Some(Wall::from_nanos(*at)),
            None => None,
        },
        failure: match value.failure() {
            Some(w::AccountListFailure::Refused(row)) => Some(c::AccountListFailure::Refused { status: row.status() }),
            Some(w::AccountListFailure::Unreachable) => Some(c::AccountListFailure::Unreachable),
            Some(w::AccountListFailure::Unreadable) => Some(c::AccountListFailure::Unreadable),
            Some(w::AccountListFailure::Deadline) => Some(c::AccountListFailure::Deadline),
            Some(w::AccountListFailure::Agent) => Some(c::AccountListFailure::Agent),
            Some(w::AccountListFailure::Room) => Some(c::AccountListFailure::Room),
            None => None,
        },
        left_out: value.left_out(),
    })
}

fn account_facts(facts: &List<w::AccountFact>) -> Result<List<c::AccountFact>, w::Problem> {
    let mut rows = List::with_capacity(facts.len());
    for fact in facts {
        let value = match fact.value() {
            w::AccountFactValue::Text(text) => c::AccountFactValue::Text(skein_lib::bytes::copy_of(text.text())),
            w::AccountFactValue::Count(count) => c::AccountFactValue::Count(count.count()),
            w::AccountFactValue::Instant(at) => c::AccountFactValue::Instant(Wall::from_nanos(at.at())),
            w::AccountFactValue::Span(span) => c::AccountFactValue::Span(span.nanos()),
            w::AccountFactValue::Share(share) => {
                if share.basis() > 10_000 {
                    return Err(w::Problem {
                        path: w::Path::AccountFactShareBasis,
                        reason: skein_codec::Reason::Bound,
                    });
                }
                c::AccountFactValue::Share(share.basis())
            }
            w::AccountFactValue::Window(window) => {
                if let Some(used) = window.used()
                    && *used > 10_000
                {
                    return Err(w::Problem { path: w::Path::AccountWindowUsed, reason: skein_codec::Reason::Bound });
                }
                c::AccountFactValue::Window(c::AccountWindow {
                    length: *window.length(),
                    used: *window.used(),
                    resets: match window.resets() {
                        Some(at) => Some(Wall::from_nanos(*at)),
                        None => None,
                    },
                    state: window.state(),
                })
            }
        };
        rows.push(c::AccountFact {
            kind: fact.kind(),
            label: skein_lib::bytes::copy_of(fact.label()),
            value,
            seen: Wall::from_nanos(fact.seen()),
            source: *fact.source(),
            origin: fact.origin(),
        })
        .expect("supplied facts fit");
    }
    Ok(rows)
}
