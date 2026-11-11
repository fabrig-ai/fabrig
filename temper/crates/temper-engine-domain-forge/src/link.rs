//! A box's person on its forges (temper/docs/design/domain/forge/adoption.md, 6).
//! Only a connector built under a box keeps links. A closed pipe stops new links but clears none.
use crate::{Domain, ForgeState, Kind, Link, LinkNote, Mode, PersonAnswer, Request, forges};
use skein_lib::{List, Queue};

/// Hand the sole owner: clear every link off-box, else those whose person is not that party.
pub(crate) fn owner(d: &mut Domain, party: Option<u64>, out: &mut Queue<Request>) {
    d.owner = party;
    let mut cleared = List::with_capacity(d.records.len());
    for (_, record) in &d.records {
        if let Some(link) = record.person
            && (!d.built_under_box || Some(link.person) != party)
        {
            let mut record = record.clone();
            record.person = None;
            cleared.push(record).expect("one cleared link per kept record");
        }
    }
    for record in &cleared {
        let forge = record.number;
        forges::save(d, record.clone(), out);
        out.push(Request::LinkNoted { forge, note: LinkNote::Cleared });
    }
}

pub(crate) fn person_of(d: &mut Domain, forge: u16, answer: PersonAnswer, out: &mut Queue<Request>) {
    if !d.under_box {
        return;
    }
    let Some(person) = d.owner else { return };
    let Some(record) = d.records.get(&forge) else { return };
    match record.state {
        ForgeState::Kept => {}
        ForgeState::Ended { .. } => return,
    }
    match record.kind {
        Kind::Forgejo => {
            if record.mode != Mode::Owned {
                return;
            }
        }
        Kind::GitHub => {}
    }
    let mut record = record.clone();
    let note = match answer {
        PersonAnswer::Is(user) => {
            if d.is_writer(forge, user) {
                out.push(Request::LinkNoted { forge, note: LinkNote::Writer });
                return;
            }
            let link = Some(Link { user, person });
            if record.person == link {
                return;
            }
            let note = match record.person {
                Some(_) => Some(LinkNote::Replaced),
                None => None,
            };
            record.person = link;
            note
        }
        PersonAnswer::Gone => {
            out.push(Request::LinkNoted { forge, note: LinkNote::Gone });
            return;
        }
        PersonAnswer::NotTheirs => {
            if record.person.take().is_none() {
                return;
            }
            Some(LinkNote::Cleared)
        }
    };
    forges::save(d, record, out);
    if let Some(note) = note {
        out.push(Request::LinkNoted { forge, note });
    }
}
