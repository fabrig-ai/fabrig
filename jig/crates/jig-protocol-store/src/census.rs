//! Naming and reporting the map's census (protocol/store/README.md, 6.3).
use alloc::boxed::Box;
use jig_domain_core_vocabulary::documents::{CensusHead, CensusKeyspace, StoreCensus};
use skein_lib::{Duration, List, Queue, Time};

use crate::{Event, Store, keys};

pub const CENSUS_EVERY: Duration = Duration::from_secs(60);

/// The application's owner and full `OWNER.FAMILY` names, under tag 2 (6.3).
#[derive(Clone, Copy, Debug)]
pub struct Keyspaces {
    pub owner: &'static str,
    pub families: &'static [(u8, &'static str)],
}

impl Keyspaces {
    pub(crate) const NONE: Self = Self { owner: "", families: &[] };
}

#[derive(Debug)]
pub(crate) struct State {
    pub rule: skein_kv::CensusRule,
    pub names: Keyspaces,
    pub now: Time,
    pub reported: Option<Time>,
}

impl State {
    pub fn new() -> Self {
        let mut prefix = [1; 256];
        for &(tag, length) in keys::TAGS {
            *prefix.get_mut(usize::from(tag)).expect("a byte indexes the rule") = length;
        }
        Self { rule: skein_kv::CensusRule { prefix }, names: Keyspaces::NONE, now: Time::ZERO, reported: None }
    }
}

pub(crate) fn due(store: &mut Store, above: &mut Queue<Event>) {
    if let Some(reported) = store.census.reported
        && store.census.now.saturating_since(reported) < CENSUS_EVERY
    {
        return;
    }
    report(store, above);
}

pub(crate) fn report(store: &mut Store, above: &mut Queue<Event>) {
    let census = store.stores.store.census();
    let mut keyspaces = List::with_capacity(u32::try_from(census.keyspaces().len()).expect("bounded census"));
    for space in census.keyspaces() {
        let mut heads = List::with_capacity(u32::try_from(space.heads().len()).expect("bounded census heads"));
        for head in space.heads() {
            heads.push(CensusHead { head: head.head, rows: head.rows }).expect("measured census heads");
        }
        keyspaces
            .push(CensusKeyspace {
                prefix: Box::from(space.prefix()),
                name: Box::from(name(space.prefix(), store.census.names)),
                rows: space.rows,
                keys: space.keys,
                values: space.values,
                heads,
                puts: space.puts,
                erases: space.erases,
                written: space.written,
            })
            .expect("measured census keyspaces");
    }
    let census = Box::new(StoreCensus { commit: store.stores.store.last(), since: census.since(), keyspaces });
    store.census.reported = Some(store.census.now);
    above.push(Event::Census { census });
}

pub(crate) fn name(prefix: &[u8], application: Keyspaces) -> &'static str {
    for &(key, name) in keys::KEYSPACES {
        if prefix == key {
            return name;
        }
    }
    if let [keys::APPLICATION, family] = prefix {
        for &(tag, name) in application.families {
            if *family == tag {
                return name;
            }
        }
    }
    ""
}
