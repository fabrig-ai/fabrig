//! Idle program accounts and limited windows (jig/docs/design/domain/accounts.md, 3.7).
//! Clocks, reset reads and flights live only for this process.
use crate::{Domain, FactValue, Kind, Request, SourceName};
use alloc::boxed::Box;
use jig_domain_link::UsageOutcome;
use skein_lib::{Duration, List, Queue, Time, Wall};

pub(crate) const QUIET: Duration = Duration::from_secs(30 * 60);

#[derive(Hash, Debug)]
pub(crate) struct Reading {
    quiet: Time,
    source: Option<u32>,
    due: bool,
    read: Option<(u64, u32)>,
    resets: List<(Box<[u8]>, Wall)>,
    spent: bool,
}

fn first(domain: &Domain, account: u32) -> Option<(u32, SourceName)> {
    for (_, number) in &domain.accounts.get(&account)?.sources {
        let source = domain.sources.get(number).expect("an account's source");
        if source.kind == Kind::Program && domain.source_grant(*number, domain.now).is_some() {
            let name = domain.issued.get(number).expect("a usable program has a reservation").name;
            return Some((*number, name));
        }
    }
    None
}

fn has_program(domain: &Domain, account: u32) -> bool {
    let Some(entry) = domain.accounts.get(&account) else { return false };
    for (_, source) in &entry.sources {
        if domain.sources.get(source).expect("an account's source").kind == Kind::Program {
            return true;
        }
    }
    false
}

pub(crate) fn start(domain: &mut Domain, out: &mut Queue<Request>) {
    let mut accounts = List::with_capacity(domain.accounts.capacity());
    for (account, _) in &domain.accounts {
        accounts.push(*account).expect("one per account");
    }
    for account in &accounts {
        settled(domain, *account, out);
    }
}

/// A report moves the quiet clock even when it leaves the fact unchanged.
pub(crate) fn reported(domain: &mut Domain, account: u32) {
    if let Some(state) = domain.usage_reads.get_mut(&account) {
        state.quiet = domain.now.saturating_add(QUIET);
        state.due = false;
    }
}

pub(crate) fn settled(domain: &mut Domain, account: u32, out: &mut Queue<Request>) {
    if !domain.wanted() {
        return;
    }
    if !has_program(domain, account) {
        domain.usage_alarms.cancel(account);
        if let Some(state) = domain.usage_reads.get_mut(&account) {
            state.source = None;
        }
        return;
    }
    let source = first(domain, account);
    if !domain.usage_reads.contains_key(&account) {
        domain
            .usage_reads
            .insert(
                account,
                Reading {
                    quiet: domain.now.saturating_add(QUIET),
                    source: None,
                    due: false,
                    read: None,
                    resets: List::uncapped(),
                    spent: false,
                },
            )
            .expect("one usage state per account");
    }
    let state = domain.usage_reads.get_mut(&account).expect("usage state admitted");
    state.due |= state.quiet <= domain.now;
    let selected = match source {
        Some((number, _)) => Some(number),
        None => None,
    };
    if selected.is_some() && selected != state.source && !state.due {
        state.quiet = domain.now.saturating_add(QUIET);
    }
    state.source = selected;
    domain.usage_alarms.cancel(account);
    if state.spent || state.read.is_some() {
        return;
    }
    let mut at = state.quiet;
    let mut resets = List::uncapped();
    for fact in domain.account_facts(account) {
        if fact.kind != crate::KIND_USAGE {
            continue;
        }
        if let FactValue::Window(window) = fact.value
            && window.state == crate::WINDOW_LIMITED
            && let Some(reset) = window.resets
        {
            resets.push((fact.label.clone(), reset)).expect("one per held fact");
        }
    }
    let state = domain.usage_reads.get_mut(&account).expect("usage state admitted");
    for reset in &resets {
        if !state.resets.as_slice().contains(reset) {
            let left = Duration::from_nanos(reset.1.as_nanos().saturating_sub(domain.wall.as_nanos()));
            at = at.min(domain.now.saturating_add(left));
        }
    }
    if at > domain.now && !state.due {
        domain.usage_alarms.arm(account, at).expect("one usage alarm per account");
        return;
    }
    state.due = true;
    let Some((number, source)) = source else { return };
    for reset in resets.into_boxed() {
        if reset.1 <= domain.wall && !state.resets.as_slice().contains(&reset) {
            state.resets.push(reset).expect("reset history is process memory");
        }
    }
    let read = domain.next_read;
    domain.next_read = domain.next_read.checked_add(1).expect("account read counter lasts this run");
    state.read = Some((read, number));
    state.due = false;
    out.push(Request::ReadUsage { read, source });
}

pub(crate) fn answered(domain: &mut Domain, read: u64, outcome: UsageOutcome, out: &mut Queue<Request>) {
    let mut held = None;
    for (account, state) in &domain.usage_reads {
        if let Some((number, source)) = state.read
            && number == read
        {
            held = Some((*account, source));
            break;
        }
    }
    let Some((account, source)) = held else { return };
    let state = domain.usage_reads.get_mut(&account).expect("held usage read");
    state.read = None;
    state.quiet = domain.now.saturating_add(QUIET);
    state.due = false;
    match outcome {
        UsageOutcome::Read(windows) => {
            let mut changed = false;
            for window in &windows {
                changed |=
                    crate::facts::report_window(domain, account, source, crate::ORIGIN_USAGE_READ, window, domain.wall);
            }
            if changed {
                out.push(Request::Save {
                    record: crate::Stored::Facts { account, facts: domain.account_facts(account).into() },
                });
            }
        }
        UsageOutcome::Spent => state.spent = true,
        UsageOutcome::NotSignedIn | UsageOutcome::Failed { .. } => {}
    }
    settled(domain, account, out);
}

pub(crate) fn source_gone(domain: &mut Domain, account: u32, source: u32, out: &mut Queue<Request>) {
    if let Some(state) = domain.usage_reads.get_mut(&account)
        && let Some((read, reading)) = state.read
        && reading == source
    {
        out.push(Request::CancelUsage { read });
        state.read = None;
        state.quiet = domain.now.saturating_add(QUIET);
        state.due = false;
    }
}

pub(crate) fn removed(domain: &mut Domain, account: u32, out: &mut Queue<Request>) {
    domain.usage_alarms.cancel(account);
    if let Some(state) = domain.usage_reads.remove(&account)
        && let Some((read, _)) = state.read
    {
        out.push(Request::CancelUsage { read });
    }
}
