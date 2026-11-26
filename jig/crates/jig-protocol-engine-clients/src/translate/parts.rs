//! Query parts are checked at admission and restored in place on every row (client/domain/contract/parts.md, 2–3).
use super::w;
use crate::generated::client as mapped;
use jig_domain_core_vocabulary::{documents as d, reads};
use skein_lib::List;

#[derive(Debug)]
pub(crate) enum Slot {
    Served(d::PartAsk),
    NotServed(w::NotServed),
}

pub(crate) fn plan(asks: &List<d::PartAsk>, settings: &crate::Settings) -> List<Slot> {
    let mut slots = List::with_capacity(asks.len());
    for ask in asks {
        let slot = if crate::served::serves(settings, Some(ask.owner), ask.version) {
            Slot::Served(*ask)
        } else {
            let owner = w::Owner::Family(w::Family { number: ask.owner });
            Slot::NotServed(crate::served::not_served(settings, owner))
        };
        slots.push(slot).expect("one slot per ask");
    }
    slots
}

pub(crate) fn served(slots: &List<Slot>) -> List<d::PartAsk> {
    let mut asks = List::with_capacity(slots.len());
    for slot in slots {
        match slot {
            Slot::Served(ask) => asks.push(*ask).expect("subset of slots"),
            Slot::NotServed(_) => (),
        }
    }
    asks
}

/// Keep the original query for answering; only served parts and matching continuations reach the core.
pub(crate) fn read(value: &reads::Read, settings: &crate::Settings) -> Option<reads::Read> {
    let mut value = value.clone();
    if let reads::Read::Tasks { query, live, ended } = &mut value {
        let slots = plan(&query.parts, settings);
        for cursor in [live, ended].into_iter().flatten() {
            let (reads::Cursor::Live { scope, .. } | reads::Cursor::Store { scope, .. }) = cursor else {
                return None;
            };
            if let reads::Scope::Tasks { query: held } = scope {
                if held != query {
                    return None;
                }
                held.parts = served(&slots);
            } else {
                return None;
            }
        }
        query.parts = served(&slots);
    }
    Some(value)
}

pub(crate) fn row(value: &List<d::RowPart>, slots: &List<Slot>) -> Result<List<w::RowPart>, w::Problem> {
    let invalid = super::problem(skein_codec::Reason::Tag);
    let mut supplied = value.iter();
    let mut parts = List::with_capacity(slots.len());
    for slot in slots {
        let part = match slot {
            Slot::NotServed(why) => w::RowPart::NotServed(why.clone()),
            Slot::Served(ask) => match supplied.next().ok_or(invalid)? {
                d::RowPart::Made(bytes) => w::RowPart::Value(w::Owned {
                    owner: w::Owner::Family(w::Family { number: ask.owner }),
                    version: ask.version,
                    record: skein_lib::bytes::copy_of(bytes),
                }),
                d::RowPart::Unknown => w::RowPart::Unknown,
                d::RowPart::Failed(why) => {
                    w::RowPart::Failed(w::PartFailed { why: mapped::part_failure_to_record(why) })
                }
            },
        };
        parts.push(part).expect("one part per slot");
    }
    if supplied.next().is_some() {
        return Err(invalid);
    }
    Ok(parts)
}
