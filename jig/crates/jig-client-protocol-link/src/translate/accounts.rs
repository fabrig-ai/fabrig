//! Accounts watch decoding and shared deployment source facts (client/domain/contract/documents.md, 1).
use super::{Limits, List, Reader, Wall, c, chip_piece, piece_problem, too_large, w};
use crate::generated::client;
use alloc::boxed::Box;
use jig_client_protocol_wire::Record as _;
use skein_lib::length::len_u64;
pub(crate) fn account_uses(value: Option<&List<w::AccountUse>>) -> Option<List<c::AccountUse>> {
    let value = value?;
    let mut uses = List::with_capacity(value.len());
    for item in value {
        uses.push(client::account_use_to_domain(item.clone())).expect("supplied uses fit");
    }
    Some(uses)
}
pub(crate) fn account_source(value: &w::AccountSource, room: &mut u64) -> Result<c::AccountSource, w::Problem> {
    let name = match &value.name {
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
        place: value.place,
        number: value.number,
        kind: client::account_kind_to_domain(value.kind.clone()),
        state: client::account_state_to_domain(value.state.clone()),
        usable: value.usable,
        expiring: value.expiring,
        attention: value.attention,
        spent_until: match &value.spent_until {
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
    charge(room, Some(len_u64(display.bytes.len())))?;
    charge(room, List::<c::AccountFact>::price(facts.len()))?;
    for fact in facts {
        charge(room, Some(len_u64(fact.label.len())))?;
        match &fact.value {
            w::AccountFactValue::Text(text) => charge(room, Some(len_u64(text.text.len())))?,
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
    charge_account(value.uses.as_ref(), &value.display, &value.facts, room)?;
    Ok(c::AccountLine {
        list: account_list(value.list.as_ref()),
        facts: account_facts(&value.facts)?,
        number: value.number,
        place: value.place,
        uses: account_uses(value.uses.as_ref()),
        plan: value.plan,
        provider: value.provider,
        provider_name: match &value.provider_name {
            Some(name) => Some(crate::translate::facts::document_text(name, limits)?),
            None => None,
        },
        manage_usage: checked_manage_usage(value.manage_usage.as_deref(), w::paths::ACCOUNT_LINE_MANAGE_USAGE)?,
        display: crate::translate::facts::document_text(&value.display, limits)?,
        holder: client::account_holder_to_domain(value.holder.clone()),
        sources: account_source_piece(&value.sources, limits, room)?,
        cut: value.cut,
    })
}
pub(crate) fn accounts(value: w::Accounts, limits: &Limits) -> Result<c::Accounts, w::Problem> {
    let mut room = document_room(limits);
    charge(&mut room, Some(u64::try_from(size_of::<c::Accounts>()).expect("a record's size fits u64")))?;
    Ok(c::Accounts {
        readiness: super::catalogue::readiness(&value.readiness, limits, &mut room)?,
        readiness_cut: value.readiness_cut,
        program_hosts: super::super::programs::hosts(&value.program_hosts, limits, &mut room)?,
        accounts: account_line_piece(&value.accounts, limits, &mut room)?,
        signing: account_sign_in_piece(&value.signing, limits, &mut room)?,
        lendable: value.lendable,
        lending: terminal_login(value.lending.as_ref(), &mut room)?,
        lent_end: match value.lent_end {
            Some(row) => Some(client::account_lent_end_to_domain(row)),
            None => None,
        },
        last_end: match value.last_end {
            Some(row) => Some(client::account_last_end_to_domain(row)),
            None => None,
        },
        usable: value.usable,
        settled: value.settled,
        cut: value.cut,
    })
}
pub(crate) fn terminal(value: w::Terminal, limits: &Limits) -> Result<c::Terminal, w::Problem> {
    let mut room = document_room(limits);
    charge(&mut room, Some(u64::try_from(size_of::<c::Terminal>()).expect("a record's size fits u64")))?;
    Ok(c::Terminal { login: terminal_login(value.login.as_ref(), &mut room)? })
}
/// The terminal's variable name shares the owning document's native byte budget.
pub(crate) fn terminal_login(
    value: Option<&w::TerminalLogin>,
    room: &mut u64,
) -> Result<Option<c::TerminalLogin>, w::Problem> {
    let Some(value) = value else { return Ok(None) };
    if !c::valid_program_name(&value.name) {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    charge(room, Some(len_u64(value.name.len())))?;
    Ok(Some(c::TerminalLogin {
        number: value.number,
        provider: value.provider,
        name: skein_lib::bytes::copy_of(&value.name),
        cancelled: value.cancelled,
    }))
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
        rows.push(client::account_sign_in_to_domain(row)).expect("counted pieces fit");
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

pub(crate) fn account_fields(value: &w::Account, limits: &Limits, room: &mut u64) -> Result<c::Account, w::Problem> {
    charge_account(value.uses.as_ref(), &value.display, &value.facts, room)?;
    // An account's runs and its waiting tasks stay lists bounded by the rows.
    let chips = List::<c::Chip>::price(limits.client().rows);
    charge(room, chips)?;
    charge(room, chips)?;
    let opt0 = match &value.generation {
        Some(item) => Some(*item),
        None => None,
    };
    let opt1 = match &value.spent_until {
        Some(item) => Some(Wall::from_nanos(*item)),
        None => None,
    };
    Ok(c::Account {
        list: account_list(value.list.as_ref()),
        facts: account_facts(&value.facts)?,
        number: value.number,
        place: value.place,
        uses: account_uses(value.uses.as_ref()),
        plan: value.plan,
        provider: value.provider,
        provider_name: match &value.provider_name {
            Some(name) => Some(crate::translate::facts::document_text(name, limits)?),
            None => None,
        },
        manage_usage: checked_manage_usage(value.manage_usage.as_deref(), w::paths::ACCOUNT_MANAGE_USAGE)?,
        display: crate::translate::facts::document_text(&value.display, limits)?,
        holder: client::account_holder_to_domain(value.holder.clone()),
        sources: account_source_piece(&value.sources, limits, room)?,
        generation: opt0,
        state: client::account_state_to_domain(value.state.clone()),
        usable: value.usable,
        expiring: value.expiring,
        attention: value.attention,
        spent_until: opt1,
        runs: chip_piece(&value.runs, limits, limits.client().rows)?,
        waiting: chip_piece(&value.waiting, limits, limits.client().rows)?,
        cut: value.cut,
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
    Some(client::account_list_to_domain(value?.clone()))
}

fn account_facts(facts: &List<w::AccountFact>) -> Result<List<c::AccountFact>, w::Problem> {
    let mut rows = List::with_capacity(facts.len());
    for fact in facts {
        rows.push(client::account_fact_to_domain(fact.clone())?).expect("supplied facts fit");
    }
    Ok(rows)
}

#[cfg(test)]
pub(crate) fn account_state_fields(value: &w::AccountState) -> c::AccountState {
    client::account_state_to_domain(value.clone())
}
