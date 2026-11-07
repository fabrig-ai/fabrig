//! When an account's list is read, and what an answer changes
//! (jig/docs/design/domain/accounts.md, 3.7).
//! Keeps the read in flight per account and each account's alarm; never a value or a row.
use crate::{Domain, Holder, Limits, ListRecord, ReadFailure, Request, State};
use skein_lib::{Duration, Env, List, Queue, Wall};

pub(crate) const EVERY: Duration = Duration::from_secs(86_400);

#[derive(Debug)]
pub(crate) struct Reading {
    first: u32,
    usable: bool,
    restoring: bool,
    due: bool,
    read: Option<u64>,
}

impl Domain {
    /// The account still holding this volatile list read (accounts.md, 3.7).
    #[must_use]
    pub fn reading_list(&self, read: u64) -> Option<u32> {
        for (account, state) in &self.list_reads {
            if state.read == Some(read) && self.accounts.contains_key(account) {
                return Some(*account);
            }
        }
        None
    }
}

fn first(domain: &Domain, account: u32) -> Option<u32> {
    if !domain.has_list(account) {
        return None;
    }
    let (_, source) = domain.accounts.get(&account)?.sources.first()?;
    Some(*source)
}

/// Project attempts' wall times once, before the restored component grants arrive.
pub(crate) fn start(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut accounts = List::with_capacity(domain.accounts.capacity());
    for (number, _) in &domain.accounts {
        accounts.push(*number).expect("one per account");
    }
    for account in &accounts {
        let Some(source) = first(domain, *account) else { continue };
        let left = match domain.list(*account) {
            Some(record) => match record.tried {
                Some(tried) => {
                    let at = Wall::from_nanos(tried.as_nanos().saturating_add(EVERY.as_nanos()));
                    crate::restore::left(env.wall, at)
                }
                None => None,
            },
            None => None,
        };
        domain
            .list_reads
            .insert(
                *account,
                Reading { first: source, usable: false, restoring: true, due: left.is_none(), read: None },
            )
            .expect("one list state per account");
        if let Some(left) = left {
            domain.list_alarms.arm(*account, env.now.saturating_add(left)).expect("one alarm per account");
        }
        settled(domain, *account, out);
    }
}

pub(crate) fn settled(domain: &mut Domain, account: u32, out: &mut Queue<Request>) {
    if !domain.wanted() {
        return;
    }
    let Some(source) = first(domain, account) else {
        domain.list_alarms.cancel(account);
        if let Some(state) = domain.list_reads.get_mut(&account) {
            state.usable = false;
            state.restoring = false;
        }
        return;
    };
    let usable = domain.source_grant(source, domain.now).is_some();
    let fact = domain.source(source).expect("the first source is kept");
    if !domain.list_reads.contains_key(&account) {
        domain
            .list_reads
            .insert(account, Reading { first: source, usable: false, restoring: false, due: true, read: None })
            .expect("one list state per account");
    }
    let state = domain.list_reads.get_mut(&account).expect("list state admitted above");
    if state.first != source {
        state.first = source;
        state.restoring = false;
        state.due = true;
        state.usable = false;
    }
    if usable && !state.usable && !state.restoring {
        state.due = true;
    }
    if fact.state != State::Asking {
        state.restoring = false;
    }
    state.usable = usable;
    ask(domain, account, out);
}

fn ask(domain: &mut Domain, account: u32, out: &mut Queue<Request>) {
    let Some(source) = first(domain, account) else { return };
    let Some(grant) = domain.source_grant(source, domain.now) else { return };
    let state = domain.list_reads.get_mut(&account).expect("settled list state");
    if !state.due || state.read.is_some() {
        return;
    }
    let read = domain.next_read;
    domain.next_read = domain.next_read.checked_add(1).expect("list read counter lasts this run");
    state.read = Some(read);
    state.due = false;
    domain.list_alarms.cancel(account);
    let provider = domain.accounts.get(&account).expect("a list's account is kept").provider;
    out.push(Request::ReadList { account, read, provider, grant });
}

pub(crate) fn due(domain: &mut Domain, account: u32, out: &mut Queue<Request>) {
    if let Some(state) = domain.list_reads.get_mut(&account) {
        state.due = true;
        settled(domain, account, out);
    }
}

pub(crate) fn again(domain: &mut Domain, holder: Holder, provider: u32, out: &mut Queue<Request>) -> u32 {
    let mut accounts = List::with_capacity(domain.accounts.capacity());
    for (number, entry) in &domain.accounts {
        if entry.holder == holder
            && entry.provider == provider
            && let Some(source) = first(domain, *number)
            && domain.source_grant(source, domain.now).is_some()
        {
            accounts.push(*number).expect("one per account");
        }
    }
    for account in &accounts {
        settled(domain, *account, out);
        due(domain, *account, out);
    }
    accounts.len()
}

fn answered(domain: &mut Domain, read: u64) -> Option<u32> {
    let mut account = None;
    for (number, state) in &domain.list_reads {
        if state.read == Some(read) {
            account = Some(*number);
            break;
        }
    }
    let account = account?;
    let state = domain.list_reads.get_mut(&account).expect("the read's account is held");
    state.read = None;
    state.due = false;
    Some(account)
}

fn save(domain: &mut Domain, account: u32, out: &mut Queue<Request>) {
    domain.list_alarms.arm(account, domain.now.saturating_add(EVERY)).expect("one alarm per account");
    out.push(Request::Save { record: crate::ordering::account_record(domain, account) });
}

pub(crate) fn listed(domain: &mut Domain, read: u64, names: u64, left_out: u32, at: Wall, out: &mut Queue<Request>) {
    let Some(account) = answered(domain, read) else { return };
    domain.accounts.get_mut(&account).expect("a held read's account is kept").list =
        Some(ListRecord { names, read: Some(at), tried: Some(at), failure: None, left_out });
    save(domain, account, out);
}

pub(crate) fn failed(domain: &mut Domain, read: u64, failure: ReadFailure, at: Wall, out: &mut Queue<Request>) {
    let Some(account) = answered(domain, read) else { return };
    let entry = domain.accounts.get_mut(&account).expect("a held read's account is kept");
    let record = entry.list.get_or_insert(ListRecord { names: 0, read: None, tried: None, failure: None, left_out: 0 });
    record.tried = Some(at);
    record.failure = Some(failure);
    save(domain, account, out);
}

pub(crate) fn removed(domain: &mut Domain, account: u32) {
    domain.list_alarms.cancel(account);
    let _removed = domain.list_reads.remove(&account);
}
