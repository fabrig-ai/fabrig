//! Load and link stored accounts (jig/docs/design/domain/engine/restart.md).
use crate::domain::{Account, Domain, Hold, Source, settle};
use crate::records::within;
use crate::{Deadline, Fact, Held, Key, Kind, Limits, Stored};
use skein_lib::{Duration, Env, List, Map, Wall};

/// A kept hold means the same after a restart: a reset passed while the child was down has
/// lifted it, and a later one is projected onto this run's clock (domain/engine/README.md, 5.4).
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
pub(crate) fn left(now: Wall, reset: Wall) -> Option<Duration> {
    match reset.as_nanos().checked_sub(now.as_nanos()) {
        Some(0) | None => None,
        Some(nanos) => Some(Duration::from_nanos(nanos)),
    }
}

/// Representation and retained-set fit (jig/docs/design/domain/accounts.md, 3.6;
/// jig/docs/design/domain/engine/restart.md).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RestoreSet {
    Accounts,
    Sources { account: u32 },
    ProgramNames,
    TotalSources,
    NameBytes,
}

/// The first fit failure of a start (jig/docs/design/domain/accounts.md, 3.6;
/// jig/docs/design/domain/engine/restart.md), retained until the next cold start.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct RestoreLimit {
    pub key: Key,
    pub set: RestoreSet,
    pub limit: u32,
}

/// The first stored reference that cannot be linked (domain/engine/restart.md).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct RestoreLink {
    pub key: Key,
    pub target: Key,
}

fn refuse(domain: &mut Domain, key: Key, set: RestoreSet, limit: u32) {
    if domain.restore_failure.is_none() {
        domain.restore_failure = Some(RestoreLimit { key, set, limit });
    }
}

/// Load decoded records without applying the writers' rules (domain/engine/restart.md).
/// A source keeps its position until every account and reservation has arrived.
pub(crate) fn load(domain: &mut Domain, env: &Env<Limits>, record: Stored) {
    let key = record.key();
    match &record {
        Stored::Facts { .. } => {}
        Stored::Next { number } => {
            domain.next = domain.next.max(*number);
            domain.kept = *number;
        }
        Stored::Account { number, .. } | Stored::Source { number, .. } | Stored::Issued { number, .. } => {
            domain.past(*number);
        }
    }
    if domain.restore_failure.is_some() || domain.restore_link.is_some() {
        return;
    }
    match record {
        Stored::Next { .. } => {}
        Stored::Facts { account, facts } => {
            assert!(!domain.said.contains_key(&account), "duplicate facts record {key:?}");
            let limit = env.limits.accounts.min(domain.said.capacity());
            if domain.said.len() >= limit {
                refuse(domain, key, RestoreSet::Accounts, limit);
                return;
            }
            assert!(domain.said.insert(account, facts).is_ok(), "loaded facts fit {key:?}");
        }
        Stored::Account { number, provider, holder, place, uses, subject, display, list } => {
            assert!(!domain.accounts.contains_key(&number), "duplicate account record {key:?}");
            if domain.accounts.len() >= env.limits.accounts.min(domain.accounts.capacity()) {
                refuse(domain, key, RestoreSet::Accounts, env.limits.accounts.min(domain.accounts.capacity()));
                return;
            }
            let entry = Account {
                list,
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

/// Attach every retained source and directory before activation (domain/engine/restart.md).
/// Missing owners and conflicting positions or reservations refuse the start.
pub(crate) fn link(domain: &mut Domain, env: &Env<Limits>) {
    if domain.restore_failure.is_some() || domain.restore_link.is_some() {
        return;
    }
    let mut orphans = List::with_capacity(domain.said.capacity());
    for (account, _) in &domain.said {
        if !domain.accounts.contains_key(account) {
            orphans.push(*account).expect("one number per facts row");
        }
    }
    for account in &orphans {
        drop(domain.said.remove(account).expect("orphan facts are held"));
    }
    for (number, issued) in &domain.issued {
        for (other, reservation) in &domain.issued {
            if number != other && issued.name == reservation.name {
                domain.restore_link =
                    Some(RestoreLink { key: Key::Issued { number: *number }, target: Key::Issued { number: *other } });
                return;
            }
        }
    }
    for (number, source) in &domain.sources {
        let key = Key::Source { account: source.account, number: *number };
        let Some(account) = domain.accounts.get_mut(&source.account) else {
            domain.restore_link = Some(RestoreLink { key, target: Key::Account { number: source.account } });
            return;
        };
        if let Some(other) = account.sources.get(&source.restore_place) {
            domain.restore_link =
                Some(RestoreLink { key, target: Key::Source { account: source.account, number: *other } });
            return;
        }
        if account.sources.len() == account.sources.capacity() {
            let limit = account.sources.capacity();
            let set = RestoreSet::Sources { account: source.account };
            refuse(domain, key, set, limit);
            return;
        }
        assert!(account.sources.insert(source.restore_place, *number) == Ok(None), "linked source fits {key:?}");
        match source.kind {
            Kind::Program => {
                let target = Key::Issued { number: *number };
                let Some(issued) = domain.issued.get(number) else {
                    domain.restore_link = Some(RestoreLink { key, target });
                    return;
                };
                if issued.holder != account.holder || issued.name.as_bytes() != source.name.as_ref() {
                    domain.restore_link = Some(RestoreLink { key, target });
                    return;
                }
                account.plan = true;
            }
            Kind::SignIn | Kind::Key | Kind::None => {
                if domain.issued.contains_key(number) {
                    domain.restore_link = Some(RestoreLink { key, target: Key::Issued { number: *number } });
                    return;
                }
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
