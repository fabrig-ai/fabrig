//! Tell temper's two placements and single filing (client/pages/needs-you.md, 1).
#![expect(
    clippy::disallowed_methods,
    reason = "ordinary tests inspect semantic subtrees and hand-built deliveries"
)]
use super::fixtures as f;


use jig_client::{Down, Readable, Up, Watchable, contract as c};

use skein_lib::{List, Queue, Token};


use temper_client_domain::{
    self as d, Address, Event, Place, Request,
    pages::{team::Holder, tell::Goes},
};

use temper_client_protocol_wire::{self as wire, v1 as w};


// These navigation fixtures hold the rail, Team's card, and both Tell placements' reads.
fn limits() -> d::Limits {
    let mut limits = f::domain_limits();
    limits.client.domain.handles = 40;
    limits.client.domain.watches = 40;
    limits.client.domain.reads = 12;
    limits
}

fn event(root: &mut d::Domain, event: Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    d::step(root, &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: limits() }, event, &mut out);
    out
}

fn go(root: &mut d::Domain, address: Address) -> Queue<Request> {
    event(root, Event::Went { place: Place::Address(address) })
}

fn task() -> c::Task {
    let mut task = super::chat::asking_task();
    task.chip.task = 52;
    task.project = 1;
    task.label = 3;
    task.chip.title = c::DocumentText::complete(c::Text::new(b"Prover".as_slice().into(), 32).unwrap());
    task
}

fn findings() -> w::Findings {
    let mut areas = List::with_capacity(2);
    for (name, number) in [(b"docs".as_slice(), 40), (b"tests", 52)] {
        areas
            .push(
                w::AreaRow::new(
                    &w::CEILINGS,
                    w::AreaRowParts {
                        name: name.into(),
                        words: name.into(),
                        owner: w::AreaOwner::Task(w::AreaTask::new(&w::CEILINGS, w::AreaTaskParts { number }).unwrap()),
                        holder: Some(w::Holder::Task(
                            w::HolderTask::new(&w::CEILINGS, w::HolderTaskParts { number }).unwrap(),
                        )),
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    w::Findings::new(
        &w::CEILINGS,
        w::FindingsParts {
            project: 1,
            areas,
            rows: List::with_capacity(0),
            cut: false,
            triager: Some(w::AreaOwner::Task(w::AreaTask::new(&w::CEILINGS, w::AreaTaskParts { number: 52 }).unwrap())),
            holder: w::Holder::Task(w::HolderTask::new(&w::CEILINGS, w::HolderTaskParts { number: 52 }).unwrap()),
        },
    )
    .unwrap()
}

fn deliver(root: &mut d::Domain, watch: Token, delivery: c::Delivery) -> Queue<Request> {
    event(
        root,
        Event::Client {
            event: jig_client::Event::Up { up: Up::Watched { watch, event: c::WatchEvent::Delivered(delivery) } },
        },
    )
}

fn family(root: &mut d::Domain, watch: Token, bytes: Box<[u8]>) -> Queue<Request> {
    event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes },
                },
            },
        },
    )
}

fn fill(root: &mut d::Domain, mut out: Queue<Request>) {
    while let Some(request) = out.pop() {
        let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down } } = request else { continue };
        let mut more = match down {
            Down::Open { watch, subject: Watchable::Jig(c::Subject::Task { task: 52 }) } => {
                deliver(root, watch, c::Delivery::Task(Box::new(task())))
            }
            Down::Open { watch, subject: Watchable::Jig(c::Subject::Tasks { query, .. }) }
                if query.label == Some(3) =>
            {
                let task = task();
                let mut line = super::chats::line();
                line.chip = task.chip;
                line.project = 1;
                line.label = 3;
                deliver(
                    root,
                    watch,
                    c::Delivery::Tasks(Box::new(c::Tasks { rows: List::from_boxed(Box::new([line])), more: false })),
                )
            }
            Down::Open { watch, subject: Watchable::Family { record, .. } } => {
                match wire::decode_watching(&record.bytes, &w::CEILINGS).map(|w| w.watch().clone()) {
                    Some(w::Watch::Findings(_)) => family(root, watch, wire::findings(&findings())),
                    Some(w::Watch::Decided(_)) => {
                        let row = w::Decision::new(
                            &w::CEILINGS,
                            w::DecisionParts {
                                number: 1,
                                task: 99,
                                at: 0,
                                question: b"Which tests?".as_slice().into(),
                                took: b"The focused suite".as_slice().into(),
                                why: b"It covers the rule.".as_slice().into(),
                                overturned: false,
                                reach: temper_protocol_reaches::reach_record(&temper_protocol_reaches::Reach {
                                    kind: temper_protocol_reaches::Kind::Tests,
                                    class: temper_protocol_reaches::Class::Decide,
                                    reason: temper_protocol_reaches::Reason::Kind,
                                    under: 0,
                                    sets: 0,
                                    matched: 0,
                                    away: false,
                                }),
                            },
                        )
                        .unwrap();
                        family(
                            root,
                            watch,
                            wire::decisions(
                                &w::Decisions::new(
                                    &w::CEILINGS,
                                    w::DecisionsParts {
                                        project: 1,
                                        rows: List::from_boxed(Box::new([row])),
                                        asked: List::with_capacity(0),
                                        cut: false,
                                    },
                                )
                                .unwrap(),
                            ),
                        )
                    }
                    _ => Queue::uncapped(),
                }
            }
            Down::Read { read, what: Readable::Jig(_) } => event(
                root,
                Event::Client {
                    event: jig_client::Event::Up {
                        up: Up::ReadDone {
                            read,
                            result: jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Unreachable)),
                        },
                    },
                },
            ),
            Down::Save { .. }
            | Down::Erase { .. }
            | Down::Send { .. }
            | Down::Open { .. }
            | Down::Continue { .. }
            | Down::Close { .. }
            | Down::Read { .. } => Queue::uncapped(),
        };
        while let Some(request) = more.pop() {
            out.push(request);
        }
    }
}

fn fixture(address: Address) -> d::Domain {
    let (mut root, watch, _, _, _) =
        f::root_at_full(Address::Work { project: 2, filter: d::WorkFilter::All }, &limits(), b"shop");
    let me = f::asking_me(root.pages().me(root.client()).unwrap().clone());
    f::deliver_me(&mut root, watch, me);
    let out = go(&mut root, Address::Talk { project: 1, task: 52 });
    fill(&mut root, out);
    let out = go(&mut root, address);
    fill(&mut root, out);
    root
}

fn filtered() -> Address {
    Address::Inbox { filter: d::address::Filter { project: Some(1), ..d::address::Filter::ALL } }
}

#[test]
fn tell_destinations_name_the_owner_triager_and_owners() {
    let root = fixture(filtered());
    let w = f::browser_words();
    let you = root.pages().me(root.client()).unwrap().party.number;
    for (goes, words, toast) in [
        (
            Goes::Area { name: b"tests".as_slice().into(), holder: Holder::Task(52) },
            "Goes to Prover, tests's owner",
            "Filed for Prover.",
        ),
        (
            Goes::Area { name: b"tests".as_slice().into(), holder: Holder::Party(you) },
            "Goes to you, as tests's owner",
            "Filed for you.",
        ),
        (
            Goes::Triager { holder: Holder::Task(52), areas: false },
            "Goes to Prover, shop's triager",
            "Filed for Prover.",
        ),
        (
            Goes::Triager { holder: Holder::Task(52), areas: true },
            "Goes to Prover, shop's triager, who hands it to its area's owner",
            "Filed for Prover.",
        ),
        (Goes::Triager { holder: Holder::Party(you), areas: false }, "Goes to you, the triager", "Filed for you."),
        (
            Goes::Triager { holder: Holder::Party(you), areas: true },
            "Goes to you, the triager, who hands it to its area's owner",
            "Filed for you.",
        ),
        (Goes::Owners { areas: false }, "Goes to shop's owners", "Filed for shop's owners."),
        (
            Goes::Owners { areas: true },
            "Goes to shop's owners, who hand it to its area's owner",
            "Filed for shop's owners.",
        ),
        (
            Goes::Triager { holder: Holder::Task(99), areas: false },
            "Goes to Task 99, shop's triager",
            "Filed for Task 99.",
        ),
        (
            Goes::Triager { holder: Holder::Party(999), areas: false },
            "Goes to Party 999, shop's triager",
            "Filed for Party 999.",
        ),
    ] {
        assert_eq!(crate::pages::tell::destination(&root, &w, 1, &goes).as_ref(), words.as_bytes());
        assert_eq!(crate::pages::tell::filed(&root, &w, 1, Some(&goes)).as_ref(), toast.as_bytes());
    }
    assert_eq!(crate::pages::tell::filed(&root, &w, 1, None).as_ref(), b"Filed.");
}
