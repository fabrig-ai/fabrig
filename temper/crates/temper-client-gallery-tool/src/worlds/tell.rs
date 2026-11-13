//! Tell temper's wide region and phone page (client/web/gallery.md, 2).
use super::{Address, Domain, c, doc, task_line};
use crate::driver::{answer, deliver, down, step};
use jig_client::{Down, Readable, Up, Watchable};
use skein_lib::{List, Queue};
use temper_client_domain::{Action, Event, Place, pages::Act};
use temper_client_protocol_wire::{self as wire, v1 as w};

pub(super) fn world(page: bool) -> Domain {
    let mut root = super::team::world(if page { "tell-page" } else { "tell-area" });
    let reader = root.pages().me(root.client()).expect("Team's me").party.number;
    let mut out = Queue::uncapped();
    // The page's About is Team; the region's is the owner's talk.
    let address = if page { Address::Team { project: 1, role: None } } else { Address::Talk { project: 1, task: 52 } };
    step(&mut root, Event::Went { place: Place::Address(address) }, &mut out);
    fill(&mut root, reader, &mut out);
    let address = if page { Address::Tell { project: 1 } } else {
        Address::Inbox { filter: temper_client_domain::address::Filter { project: Some(1), ..temper_client_domain::address::Filter::ALL } }
    };
    step(&mut root, Event::Went { place: Place::Address(address) }, &mut out);
    fill(&mut root, reader, &mut out);
    step(&mut root, Event::Act { action: Action::Pages(Act::TellWords {
        words: b"The order email's courier link opens the tracking page in English for German customers.".as_slice().into(),
    }) }, &mut out);
    root
}
fn fill(root: &mut Domain, reader: u64, out: &mut Queue<temper_client_domain::Request>) {
    while let Some(request) = out.pop() {
        let Some(request) = down(request) else { continue };
        match request {
            Down::Open { watch, subject: Watchable::Jig(c::Subject::Task { task: 52 }) } => {
                deliver(root, watch, c::Delivery::Task(Box::new(super::team::prover(reader))), out);
            }
            Down::Open { watch, subject: Watchable::Jig(c::Subject::Tasks { query, .. }) } if query.label == Some(3) => {
                deliver(root, watch, c::Delivery::Tasks(Box::new(c::Tasks {
                    rows: List::from_boxed(Box::new([task_line(super::team::prover(reader))])), more: false,
                })), out);
            }
            Down::Read { read, what: Readable::Jig(c::Read::Project { project: 1 }) } => {
                answer(root, read, c::ReadValue::Project(Box::new(super::team::project(reader, true))), out);
            }
            Down::Open { watch, subject: Watchable::Jig(c::Subject::Inbox { .. }) } => {
                let mut inbox = temper_fixtures::client_documents::inbox(List::with_capacity(0), 0, c::Count { value: 0, limited: false }, 0, false);
                inbox.party = reader;
                deliver(root, watch, c::Delivery::Inbox(Box::new(inbox)), out);
            }
            Down::Read { read, what: Readable::Jig(c::Read::Inbox { .. }) } => {
                answer(root, read, c::ReadValue::Inbox(Box::new(c::InboxPage {
                    rows: List::with_capacity(0), before: None, after: None, cut: false, selection: None,
                })), out);
            }
            Down::Open { watch, subject: Watchable::Family { record, .. } } if wire::decode_watching(&record.bytes, &w::CEILINGS).is_some_and(|watching| matches!(watching.watch(), w::Watch::Findings(_))) => {
                step(root, Event::Client { event: jig_client::Event::Up { up: Up::FamilyDelivered {
                    watch, record: jig_client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::findings(&document()) },
                } } }, out);
            }
            _ => {}
        }
    }
}
fn document() -> w::Findings {
    let mut areas = List::with_capacity(2);
    for (name, number) in [("docs", 40), ("tests", 52)] {
        areas.push(w::AreaRow::new(&w::CEILINGS, w::AreaRowParts {
            name: name.as_bytes().into(), words: doc(name).bytes().into(),
            owner: w::AreaOwner::Task(w::AreaTask::new(&w::CEILINGS, w::AreaTaskParts { number }).expect("owner")),
            holder: Some(w::Holder::Task(w::HolderTask::new(&w::CEILINGS, w::HolderTaskParts { number }).expect("holder"))),
        }).expect("area")).expect("two areas");
    }
    w::Findings::new(&w::CEILINGS, w::FindingsParts {
        project: 1, rows: List::with_capacity(0), areas, cut: false,
        triager: Some(w::AreaOwner::Task(w::AreaTask::new(&w::CEILINGS, w::AreaTaskParts { number: 52 }).expect("triager"))),
        holder: w::Holder::Task(w::HolderTask::new(&w::CEILINGS, w::HolderTaskParts { number: 52 }).expect("holder")),
    }).expect("Tell's findings")
}
