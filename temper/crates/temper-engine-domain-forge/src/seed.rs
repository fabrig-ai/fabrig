//! A document seeds only the store's first start (temper/docs/plan/playground-reset-r4/notes/contracts.md, D1).
use crate::{Domain, Request, Stored};
use alloc::boxed::Box;
use skein_lib::{Map, Queue};

pub(crate) fn seed(d: &mut Domain, out: &mut Queue<Request>) {
    let seed = core::mem::replace(&mut d.seed, Box::new([]));
    if d.next_forge.is_some() {
        // With no restored forge, even the provisional seed configuration goes.
        d.configured = Map::with_capacity(d.configured.capacity());
        d.identities = Map::with_capacity(d.identities.capacity());
        for (number, row) in &d.forge_records {
            d.configured.insert(*number, crate::Configured::from_record(row)).expect("kept forge capacity");
            d.identities.insert(*number, row.identity.number).expect("kept forge capacity");
        }
        return;
    }
    let mut next = 1;
    for row in &seed {
        next = next.max(row.number.checked_add(1).expect("seed number checked"));
        d.forge_records.insert(row.number, row.clone()).expect("seed capacity checked");
        out.push(Request::Save { record: Stored::Forge(row.clone()), release: None, read_afresh: None });
    }
    d.next_forge = Some(next);
    out.push(Request::Save { record: Stored::Forges { next }, release: None, read_afresh: None });
}

pub(crate) fn kept(d: &Domain, out: &mut Queue<Request>) {
    for (_, row) in &d.forge_records {
        out.push(Request::ForgeKept { forge: Box::new(row.clone()) });
    }
}
