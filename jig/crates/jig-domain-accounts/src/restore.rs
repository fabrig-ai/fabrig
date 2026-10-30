//! Default restoration and the opted-in load/link start (jig/docs/design/domain/engine.md, 6).
//! Default restores apply the accounts' rules (engine.md, 5.4). A record that keeps
//! the accounts' rules is taken; any other is counted and left, never trusted, so that a provider
//! configured again finds its accounts. Every number a record carries moves the counter past it,
//! taken or not, so that no number is given twice.
use crate::domain::{Account, Domain, Hold, Source, settle};
use crate::records::{program_name, within};
use crate::{DISPLAY, Deadline, Fact, Held, Key, Kind, Limits, SUBJECT, Stored};
use skein_lib::{Duration, Env, List, Map, Queue, Wall};

pub(crate) fn restore(domain: &mut Domain, env: &Env<Limits>, record: Stored) {
    let taken = match record {
        Stored::Issued { number, holder, name, signed_out } => {
            domain.past(number);
            crate::programs::restore(domain, number, holder, name, signed_out)
        }
        Stored::Next { number } => {
            domain.next = domain.next.max(number);
            domain.kept = number;
            true
        }
        Stored::Account { number, provider, holder, place, uses, subject, display } => {
            domain.past(number);
            // An account of a provider no longer configured waits for it.
            domain.providers.get(provider).is_some()
                && account(
                    domain,
                    env,
                    number,
                    Account {
                        provider,
                        holder,
                        place,
                        uses,
                        plan: false,
                        subject,
                        display,
                        sources: Map::with_capacity(0),
                    },
                )
        }
        Stored::Source { account, number, place, kind, name, held } => {
            domain.past(number);
            let mut entry = Source::new(account, kind, name, &env.limits);
            hold(&mut entry, env, held);
            source(domain, env, number, place, entry)
        }
    };
    if !taken {
        domain.unrestored = domain.unrestored.saturating_add(1);
    }
}

/// The start drops each account left with no source: none of its records says how it is used.
pub(crate) fn finish(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<crate::Request>) {
    crate::programs::finish_restore(domain, out);
    let mut empty = List::with_capacity(env.limits.accounts);
    for (number, account) in &domain.accounts {
        if account.sources.is_empty() {
            empty.push(*number).expect("one per account");
        }
    }
    for number in &empty {
        drop(domain.accounts.remove(number).expect("a listed account"));
        domain.unrestored = domain.unrestored.saturating_add(1);
    }
}

/// A free number, a bounded subject and display name, and a subject no other account of its
/// provider holds (accounts.md, 3.1 and 3.2).
fn account(domain: &mut Domain, env: &Env<Limits>, number: u32, mut entry: Account) -> bool {
    if !free(domain, number)
        || domain.issued.contains_key(&number)
        || !within(&entry.subject, SUBJECT)
        || !within(&entry.display, DISPLAY)
        || !crate::records::valid_uses(entry.uses.as_deref())
    {
        return false;
    }
    if !entry.subject.is_empty() {
        for (_, other) in &domain.accounts {
            if other.provider == entry.provider && other.subject == entry.subject {
                return false;
            }
        }
    }
    entry.sources = Map::with_capacity(env.limits.sources);
    match domain.accounts.insert(number, entry) {
        Ok(old) => old.is_none(),
        Err(_) => false,
    }
}

/// A free number, a kept account with room at a free place, one signed-in source an account, and
/// a program's name valid and unique in the deployment (accounts.md, 3.4).
fn source(domain: &mut Domain, env: &Env<Limits>, number: u32, place: u32, entry: Source) -> bool {
    if !free(domain, number) {
        return false;
    }
    let Some(account) = domain.accounts.get(&entry.account) else { return false };
    if account.sources.contains_key(&place) || account.sources.len() >= account.sources.capacity() {
        return false;
    }
    match entry.kind {
        Kind::Program => {
            if !program_name(&entry.name) {
                return false;
            }
            for (_, other) in &domain.sources {
                if other.kind == Kind::Program && other.name == entry.name {
                    return false;
                }
            }
        }
        Kind::SignIn => {
            if !entry.name.is_empty() {
                return false;
            }
            for (_, other) in &account.sources {
                if domain.sources.get(other).expect("an account's sources are kept").kind == Kind::SignIn {
                    return false;
                }
            }
        }
        Kind::Key | Kind::None => {
            if !entry.name.is_empty() {
                return false;
            }
        }
    }
    let plan = domain.providers.get(account.provider).expect("a restored provider").plan(entry.kind);
    let account = entry.account;
    if domain.sources.insert(number, entry) != Ok(None) {
        return false;
    }
    let account = domain.accounts.get_mut(&account).expect("looked up above");
    account.plan |= plan;
    let places = &mut account.sources;
    assert!(places.insert(place, number) == Ok(None), "a free place with room, checked above");
    let _: Fact = settle(domain, env, number);
    true
}

/// A number neither an account nor a source holds; the counter never gives `u32::MAX`.
fn free(domain: &Domain, number: u32) -> bool {
    number != u32::MAX && !domain.accounts.contains_key(&number) && !domain.sources.contains_key(&number)
}

/// A kept hold means the same after a restart: a reset passed while the child was down has
/// lifted it, and a later one is projected onto this run's clock (engine.md, 5.4).
fn hold(entry: &mut Source, env: &Env<Limits>, kept: Option<Held>) {
    let (hold, attention) = match kept {
        None => return,
        Some(Held::Refused) => (Hold::Refused, true),
        Some(Held::Moving) => (Hold::Moving, false),
        Some(Held::Limited { reset: None }) => (Hold::Limited { until: None }, true),
        Some(Held::Limited { reset: Some(reset) }) => {
            let Some(left) = left(env.wall, reset) else { return };
            (
                Hold::Limited { until: Some(Deadline { at: env.now.saturating_add(left), wall: reset }) },
                left >= env.limits.spent_attention,
            )
        }
    };
    entry.hold = Some(hold);
    entry.hold_attention = attention;
}

/// The time from now to a later wall time; none once it has passed.
fn left(now: Wall, reset: Wall) -> Option<Duration> {
    match reset.as_nanos().checked_sub(now.as_nanos()) {
        Some(0) | None => None,
        Some(nanos) => Some(Duration::from_nanos(nanos)),
    }
}

/// Representation and retained-set fit (jig/docs/design/domain/accounts.md, 3.6;
/// jig/docs/design/domain/engine.md, 6).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RestoreSet {
    Accounts,
    Sources { account: u32 },
    ProgramNames,
    TotalSources,
    NameBytes,
}

/// The first fit failure of an opted-in start (jig/docs/design/domain/accounts.md, 3.6;
/// jig/docs/design/domain/engine.md, 6), retained until the next cold start.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RestoreLimit {
    pub key: Key,
    pub set: RestoreSet,
    pub limit: u32,
}

fn refuse(domain: &mut Domain, key: Key, set: RestoreSet, limit: u32) {
    if domain.restore_failure.is_none() {
        domain.restore_failure = Some(RestoreLimit { key, set, limit });
        domain.unrestored = domain.unrestored.saturating_add(1);
    }
}

/// Load decoded records without applying the writers' rules (engine.md, 6).
/// A source keeps its position until every account and reservation has arrived.
pub(crate) fn load(domain: &mut Domain, env: &Env<Limits>, record: Stored) {
    let key = record.key();
    match &record {
        Stored::Next { number } => {
            domain.next = domain.next.max(*number);
            domain.kept = *number;
        }
        Stored::Account { number, .. } | Stored::Source { number, .. } | Stored::Issued { number, .. } => {
            domain.past(*number);
        }
    }
    if domain.restore_failure.is_some() {
        return;
    }
    match record {
        Stored::Next { .. } => {}
        Stored::Account { number, provider, holder, place, uses, subject, display } => {
            assert!(!domain.accounts.contains_key(&number), "duplicate account record {key:?}");
            if domain.accounts.len() >= env.limits.accounts.min(domain.accounts.capacity()) {
                refuse(domain, key, RestoreSet::Accounts, env.limits.accounts.min(domain.accounts.capacity()));
                return;
            }
            let entry = Account {
                provider,
                holder,
                place,
                uses,
                subject,
                display,
                plan: false,
                sources: Map::with_capacity(env.limits.sources),
            };
            assert!(domain.accounts.insert(number, entry).is_ok(), "loaded account fits {key:?}");
        }
        Stored::Source { account, number, place, kind, name, held } => {
            assert!(
                !domain.sources.contains_key(&number),
                "duplicate source identity {key:?}, already owned by {:?}",
                domain.sources.get(&number)
            );
            if !within(&name, crate::NAME) {
                refuse(domain, key, RestoreSet::NameBytes, crate::NAME);
                return;
            }
            if domain.sources.len() == domain.sources.capacity() {
                refuse(domain, key, RestoreSet::TotalSources, domain.sources.capacity());
                return;
            }
            let mut entry = Source::new(account, kind, name, &env.limits);
            entry.restore_place = place;
            hold(&mut entry, env, held);
            assert!(domain.sources.insert(number, entry).is_ok(), "loaded source fits {key:?}");
        }
        Stored::Issued { number, holder, name, signed_out } => {
            let Some(name) = crate::SourceName::from_stored(&name) else {
                refuse(domain, key, RestoreSet::NameBytes, crate::NAME);
                return;
            };
            assert!(!domain.issued.contains_key(&number), "duplicate reservation {key:?}");
            if domain.issued.len() >= env.limits.program_names.min(domain.issued.capacity()) {
                refuse(domain, key, RestoreSet::ProgramNames, env.limits.program_names.min(domain.issued.capacity()));
                return;
            }
            let issued = crate::programs::Issued { holder, name, signed_out };
            assert!(domain.issued.insert(number, issued).is_ok(), "loaded reservation fits {key:?}");
        }
    }
}

/// Attach every retained source and directory before activation (engine.md, 6).
/// Missing owners and conflicting positions or reservations are writer bugs.
pub(crate) fn link(domain: &mut Domain, env: &Env<Limits>) {
    if domain.restore_failure.is_some() {
        return;
    }
    for (number, issued) in &domain.issued {
        for (other, reservation) in &domain.issued {
            assert!(
                number == other || issued.name != reservation.name,
                "reservation {:?} conflicts with {:?} for {:?}",
                Key::Issued { number: *number },
                Key::Issued { number: *other },
                issued.name
            );
        }
    }
    for (number, source) in &domain.sources {
        let key = Key::Source { account: source.account, number: *number };
        let account = domain.accounts.get_mut(&source.account);
        assert!(account.is_some(), "{key:?} requires account {:?}", Key::Account { number: source.account });
        let account = account.expect("source owner linked above");
        assert!(
            !account.sources.contains_key(&source.restore_place),
            "{key:?} conflicts at place {} with source {:?}",
            source.restore_place,
            account.sources.get(&source.restore_place)
        );
        if account.sources.len() == account.sources.capacity() {
            let limit = account.sources.capacity();
            let set = RestoreSet::Sources { account: source.account };
            refuse(domain, key, set, limit);
            return;
        }
        assert!(account.sources.insert(source.restore_place, *number) == Ok(None), "linked source fits {key:?}");
        match source.kind {
            Kind::Program => {
                let issued = domain.issued.get(number);
                assert!(issued.is_some(), "{key:?} requires reservation {:?}", Key::Issued { number: *number });
                let issued = issued.expect("program reservation linked above");
                assert!(
                    issued.holder == account.holder && issued.name.as_bytes() == source.name.as_ref(),
                    "{key:?} disagrees with reservation {:?} owner/name {:?}",
                    Key::Issued { number: *number },
                    issued
                );
                account.plan = true;
            }
            Kind::SignIn | Kind::Key | Kind::None => {
                assert!(
                    !domain.issued.contains_key(number),
                    "{key:?} conflicts with program reservation {:?}",
                    Key::Issued { number: *number }
                );
            }
        }
        if let Some(provider) = domain.providers.get(account.provider) {
            account.plan |= provider.plan(source.kind);
        }
    }
    let mut numbers = List::with_capacity(domain.sources.capacity());
    for (number, _) in &domain.sources {
        numbers.push(*number).expect("one number per source");
    }
    for number in &numbers {
        let _: Fact = settle(domain, env, *number);
    }
}
