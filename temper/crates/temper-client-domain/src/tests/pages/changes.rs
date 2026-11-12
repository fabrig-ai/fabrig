//! A project's changes own one subject and the reads of the rows it shows (client/pages/README.md).
use super::welcome::{deliver, event};
use crate::{
    Address, Domain, Event, Place, Request, forge,
    pages::changes::{From, State},
    tests::fixtures as f,
};
use jig_client::{Down, Readable, Readout, Up, Watchable, contract as c};
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
    assert_eq!(&*closed(&out), &[watch]);
    assert!(root.pages().changes().is_none());
}
#[test]
fn reads_follow_distinct_shown_rows_without_reading_more_or_rereading_kept_rows() {
    let (mut root, watch) = fixture();
    let out = show(&mut root, watch, &panel(&[FIRST], &[SECOND]));
    let rows = reads(&out);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows.first().unwrap().0, FIRST);
    assert_eq!(rows.get(1).unwrap().0, SECOND);
    assert_eq!(root.pages().changes().unwrap().from(root.client(), FIRST), None);
    match root.pages().changes().unwrap().state() {
        State::Shown(_) => {}
        State::Loading | State::Ended => panic!("panel shown"),
    }
    assert!(reads(&show(&mut root, watch, &panel(&[FIRST], &[SECOND]))).is_empty());
    let out = show(&mut root, watch, &panel(&[SECOND], &[SECOND]));
    assert!(reads(&out).is_empty());
    // Replacing more rows than the client's read capacity proves departed reads release their places.
    for task in 20..40 {
        let out = show(&mut root, watch, &panel(&[task], &[]));
        assert_eq!(reads(&out).len(), 1, "row {task} has a read place");
    }
    let out = go(&mut root, Address::Changes { project: f::PROJECT });
    assert!(out.is_empty(), "same address keeps its watch and reads");
    let out = go(&mut root, Address::Settings { project: f::PROJECT });
    assert!(closed(&out).contains(&watch));
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
fn row_origin_uses_its_goal_chip_or_a_chat_root_and_waits_for_its_read() {
    for (goal, root_chat, has_goal) in
        [(true, false, true), (false, true, false), (true, true, false), (false, false, false)]
    {
        let (mut root, watch) = fixture();
        let out = show(&mut root, watch, &panel(&[FIRST], &[]));
        let read = reads(&out).first().unwrap().1;
        let mut task = f::chat::task();
        task.chip.task = FIRST;
        task.goal = if goal { Some(GOAL) } else { None };
        task.lineage_chips = List::uncapped();
        let mut chip = task.chip.clone();
        chip.task = if has_goal { GOAL } else { CHAT };
        chip.title = c::DocumentText::complete(f::acts::text(b"Goal title"));
        chip.executor =
            if root_chat { c::ChipExecutor::Agent { charter: 1 } } else { c::ChipExecutor::Party { party: f::PARTY } };
        task.lineage_chips.push(chip).unwrap();
        up(
            &mut root,
            Up::ReadDone { read, result: Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(Box::new(task)))) },
        );
        let expected = if has_goal {
            Some(From::Goal(b"Goal title"))
        } else if !goal && root_chat {
            Some(From::Chat(CHAT))
        } else {
            None
        };
        assert_eq!(root.pages().changes().unwrap().from(root.client(), FIRST), expected);
    }
}

#[test]
fn default_branch_rows_get_task_reads_without_rereading_shared_or_kept_rows() {
    let (mut root, watch) = fixture();
    let mut parts = panel(&[], &[SECOND]).into_parts();
    parts.landed_default = list_for_default(&[SECOND, 99]);
    let panel = w::ChangesPanel::new(&w::CEILINGS, parts).unwrap();
    let got = reads(&show(&mut root, watch, &panel));
    let mut tasks = List::uncapped();
    for (task, _) in &got {
        tasks.push(*task).unwrap();
    }
    assert_eq!(tasks.as_slice(), &[SECOND, 99]);
    assert!(reads(&show(&mut root, watch, &panel)).is_empty());
    let mut parts = panel.into_parts();
    parts.landed = List::with_capacity(0);
    let panel = w::ChangesPanel::new(&w::CEILINGS, parts).unwrap();
    assert!(reads(&show(&mut root, watch, &panel)).is_empty(), "default history keeps its reads");
}
fn list_for_default(tasks: &[u64]) -> List<w::ChangesLanded> {
    let mut rows = List::with_capacity(32);
    for task in tasks {
        rows.push(
            w::ChangesLanded::new(
                &w::CEILINGS,
                w::ChangesLandedParts {
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
