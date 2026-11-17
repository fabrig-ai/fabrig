//! A project's changes own one subject whose rows name their origins (client/pages/README.md).
use super::welcome::{deliver, event};
use crate::{
    Address, Domain, Event, Place, Request, forge,
    pages::changes::{From, State},
    tests::fixtures as f,
};
use jig_client::{Down, Readable, Up, Watchable, contract as c};
use skein_lib::{List, Queue, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};
const FIRST: u64 = 9;
const SECOND: u64 = 10;
const GOAL: u64 = 7;
const CHAT: u64 = 6;

fn fixture() -> (Domain, Token) {
    let mut root = crate::unlinked(&f::limits(), f::SEED);
    let me = f::start(&mut root, List::with_capacity(0));
    deliver(&mut root, me, c::Delivery::Me(Box::new(f::chats::me())));
    let out = go(&mut root, Address::Changes { project: f::PROJECT });
    let subject = forge::changes::watch(f::PROJECT, &f::limits().forge).unwrap();
    let watch = opened(&out, &subject).expect("changes subject opened");
    up(&mut root, Up::Watched { watch, event: c::WatchEvent::Opened });
    (root, watch)
}
fn go(root: &mut Domain, address: Address) -> Queue<Request> {
    event(root, Event::Went { place: Place::Address(address) })
}
fn opened(out: &Queue<Request>, subject: &Watchable) -> Option<Token> {
    for request in out {
        if let Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: Down::Open { watch, subject: found } },
        } = request
            && found == subject
        {
            return Some(*watch);
        }
    }
    None
}
fn up(root: &mut Domain, up: Up) -> Queue<Request> {
    event(root, Event::Client { event: jig_client::Event::Up { up } })
}
fn panel(tasks: &[u64], landed: &[u64]) -> w::ChangesPanel {
    let mut changes = List::with_capacity(32);
    for task in tasks {
        changes
            .push(
                w::ChangesQueued::new(
                    &w::CEILINGS,
                    w::ChangesQueuedParts {
                        goal: None,
                        chat: None,
                        milestone: None,
                        task: *task,
                        title: b"Queued change".as_slice().into(),
                        ready_since: 0,
                        checks_since: None,
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    let mut queues = List::with_capacity(32);
    queues
        .push(
            w::ChangesQueue::new(
                &w::CEILINGS,
                w::ChangesQueueParts {
                    way: w::Way::Queue,
                    in_flight: None,
                    repository: b"owner/repo".as_slice().into(),
                    branch: b"main".as_slice().into(),
                    red: false,
                    changes,
                },
            )
            .unwrap(),
        )
        .unwrap();
    let mut rows = List::with_capacity(32);
    for task in landed {
        rows.push(
            w::ChangesLanded::new(
                &w::CEILINGS,
                w::ChangesLandedParts {
                    goal: None,
                    chat: None,
                    milestone: None,
                    signed: List::with_capacity(8),
                    commit: [0xab; 32],
                    default_branch: true,
                    task: *task,
                    title: b"Landed change".as_slice().into(),
                    at: 0,
                },
            )
            .unwrap(),
        )
        .unwrap();
    }
    w::ChangesPanel::new(
        &w::CEILINGS,
        w::ChangesPanelParts {
            landed_default: List::with_capacity(0),
            landed_default_more: 0,
            project: f::PROJECT,
            landing: b"main".as_slice().into(),
            queues,
            more: 19,
            landed: rows,
            landed_more: 13,
        },
    )
    .unwrap()
}
fn show(root: &mut Domain, watch: Token, panel: &w::ChangesPanel) -> Queue<Request> {
    up(
        root,
        Up::FamilyDelivered {
            watch,
            record: jig_client::Family {
                family: wire::FORGE,
                version: wire::CLIENT_VERSION,
                bytes: wire::changes_panel(panel),
            },
        },
    )
}
fn reads(out: &Queue<Request>) -> Box<[(u64, Token)]> {
    let mut reads = List::uncapped();
    for request in out {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down { down: Down::Read { read, what: Readable::Jig(c::Read::Task { task }) } },
        } = request
        {
            reads.push((*task, *read)).unwrap();
        }
    }
    reads.into_boxed()
}
fn closed(out: &Queue<Request>) -> Box<[Token]> {
    let mut closed = List::uncapped();
    for request in out {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Close { watch } } } =
            request
        {
            closed.push(*watch).unwrap();
        }
    }
    closed.into_boxed()
}
#[test]
fn changes_watch_is_project_scoped_and_leaves_with_the_page() {
    let subject = forge::changes::watch(f::PROJECT, &f::limits().forge).unwrap();
    let Watchable::Family { scope, window, record } = subject else { panic!("family watch") };
    assert_eq!(scope, c::Scope::Project(f::PROJECT));
    assert_eq!(window, c::Window { rows: 0, after: None });
    assert_eq!(
        wire::decode_watching(&record.bytes, &w::CEILINGS).unwrap().watch(),
        &w::Watch::Changes(
            w::ChangesSubject::new(&w::CEILINGS, w::ChangesSubjectParts { project: f::PROJECT }).unwrap()
        )
    );
    let (mut root, watch) = fixture();
    match root.pages().changes().unwrap().state() {
        State::Loading => {}
        State::Shown(_) | State::Ended => panic!("page starts loading"),
    }
    let out = go(&mut root, Address::Settings { project: f::PROJECT });
    assert!(closed(&out).contains(&watch));
    assert!(root.pages().changes().is_none());
}
#[test]
fn shown_rows_need_no_task_reads() {
    let (mut root, watch) = fixture();
    for panel in [panel(&[FIRST], &[SECOND]), panel(&[SECOND], &[SECOND]), panel(&[41], &[])] {
        assert!(reads(&show(&mut root, watch, &panel)).is_empty());
    }
    assert!(go(&mut root, Address::Changes { project: f::PROJECT }).is_empty());
    assert!(closed(&go(&mut root, Address::Settings { project: f::PROJECT })).contains(&watch));
}
#[test]
fn ended_or_invalid_changes_stays_ended_until_leaving() {
    for invalid in [false, true] {
        let (mut root, watch) = fixture();
        if invalid {
            up(
                &mut root,
                Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: b"invalid".as_slice().into(),
                    },
                },
            );
        } else {
            up(&mut root, Up::WatchEnded { watch, end: c::WatchEnd::NotServed });
        }
        match root.pages().changes().unwrap().state() {
            State::Ended => {}
            State::Loading | State::Shown(_) => panic!("page stays ended"),
        }
        let out = go(&mut root, Address::Changes { project: f::PROJECT });
        assert!(out.is_empty());
        go(&mut root, Address::Settings { project: f::PROJECT });
        let out = go(&mut root, Address::Changes { project: f::PROJECT });
        assert!(opened(&out, &forge::changes::watch(f::PROJECT, &f::limits().forge).unwrap()).is_some());
    }
}
#[test]
fn row_origins_arrive_in_the_panel_without_task_reads() {
    let (mut root, watch) = fixture();
    assert!(reads(&show(&mut root, watch, &panel(&[FIRST], &[]))).is_empty());
    assert_eq!(root.pages().changes().unwrap().from(FIRST), None);
    assert_eq!(root.pages().changes().unwrap().goal(FIRST), None);
    for (goal, chat, expected) in [
        (Some(origin(GOAL)), Some(origin(CHAT)), Some(From::Goal(b"Store shape".as_slice()))),
        (None, Some(origin(CHAT)), Some(From::Chat(CHAT))),
        (None, None, None),
    ] {
        let mut parts = panel(&[FIRST], &[SECOND]).into_parts();
        let mut queue = parts.queues.get(0).unwrap().clone().into_parts();
        let mut row = queue.changes.get(0).unwrap().clone().into_parts();
        row.goal = goal.clone();
        row.chat = chat.clone();
        queue.changes = List::from_boxed(Box::new([w::ChangesQueued::new(&w::CEILINGS, row).unwrap()]));
        queue.in_flight = Some(
            w::TrainLanding::new(
                &w::CEILINGS,
                w::TrainLandingParts {
                    task: 100,
                    member: 42,
                    round: 1,
                    step: w::LandingStep::Proving,
                    title: b"In flight".as_slice().into(),
                    ready_since: 0,
                    probe: None,
                    goal: goal.clone(),
                    chat: chat.clone(),
                    milestone: None,
                },
            )
            .unwrap(),
        );
        parts.queues = List::from_boxed(Box::new([w::ChangesQueue::new(&w::CEILINGS, queue).unwrap()]));
        let mut row = parts.landed.get(0).unwrap().clone().into_parts();
        row.goal = goal.clone();
        row.chat = chat;
        parts.landed = List::from_boxed(Box::new([w::ChangesLanded::new(&w::CEILINGS, row).unwrap()]));
        let mut row = parts.landed.get(0).unwrap().clone().into_parts();
        row.task = 99;
        parts.landed_default = List::from_boxed(Box::new([w::ChangesLanded::new(&w::CEILINGS, row).unwrap()]));
        assert!(reads(&show(&mut root, watch, &w::ChangesPanel::new(&w::CEILINGS, parts).unwrap())).is_empty());
        for task in [FIRST, SECOND, 42, 99] {
            assert_eq!(root.pages().changes().unwrap().from(task), expected);
            assert_eq!(
                root.pages().changes().unwrap().goal(task),
                match &goal {
                    Some(goal) => Some(goal.task()),
                    None => None,
                }
            );
        }
        assert_eq!(root.pages().changes().unwrap().from(123), None);
    }
}
fn origin(task: u64) -> w::ChangeFrom {
    w::ChangeFrom::new(&w::CEILINGS, w::ChangeFromParts { task, title: b"Store shape".as_slice().into() }).unwrap()
}

#[test]
fn default_branch_rows_need_no_task_reads() {
    let (mut root, watch) = fixture();
    let mut parts = panel(&[], &[SECOND]).into_parts();
    parts.landed_default = list_for_default(&[SECOND, 99]);
    let panel = w::ChangesPanel::new(&w::CEILINGS, parts).unwrap();
    assert!(reads(&show(&mut root, watch, &panel)).is_empty());
    let mut parts = panel.into_parts();
    parts.landed = List::with_capacity(0);
    let panel = w::ChangesPanel::new(&w::CEILINGS, parts).unwrap();
    assert!(reads(&show(&mut root, watch, &panel)).is_empty(), "default history needs no reads");
}
fn list_for_default(tasks: &[u64]) -> List<w::ChangesLanded> {
    let mut rows = List::with_capacity(32);
    for task in tasks {
        rows.push(
            w::ChangesLanded::new(
                &w::CEILINGS,
                w::ChangesLandedParts {
                    goal: None,
                    chat: None,
                    milestone: None,
                    signed: List::with_capacity(8),
                    task: *task,
                    title: b"Main landing".as_slice().into(),
                    at: 0,
                    commit: [0xab; 32],
                    default_branch: true,
                },
            )
            .unwrap(),
        )
        .unwrap();
    }
    rows
}
