//! Application row parts and ended-group positions at the root's durable client seam.
use crate::read_support::request;
use jig_core_world::{
    Fault,
    client_peer::ClientPeer,
    fixtures,
    world::{Setup, World, fixture},
};
use jig_domain_core::{self as core, documents as d, reads};
use jig_domain_notes as notes;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{List, ReplyTo, Token};

pub(super) fn world(seed: u64) -> World {
    configured_world(seed, root::parts::kinds())
}

fn configured_world(seed: u64, parts: Box<[d::parts::PartKind]>) -> World {
    let (mut config, mut limits) = fixture(seed, false);
    config.core.parts = parts;
    limits.core.load_slots = 2;
    limits.core.documents.rows = 4;
    limits.core.views.watchers = 4;
    limits.core.views.watchers_per_party = 4;
    limits.core.views.subjects_per_change = 4;
    let mut world = World::new(Setup { seed, configuration: Some(config), limits, ..Setup::default() });
    world.client_peer = Some(ClientPeer::default());
    world
}

pub(super) fn query(kinds: &[u32], live: bool) -> d::TaskQuery {
    let mut parts = List::with_capacity(4);
    for kind in kinds {
        parts.push(core::PartAsk { owner: 7, kind: *kind, version: 12 }).expect("configured fixture");
    }
    d::TaskQuery { parts, phase: live.then_some(d::PhaseClass::Working), ..jig_test_profile::fixtures::own_tasks() }
}

fn read(world: &mut World, number: u64, query: d::TaskQuery) {
    world.send(root::Event::Core(core::Event::Read {
        reply: ReplyTo::new(Token::new(number)),
        sign_in: world.sign_in.expect("configured fixture"),
        read: reads::Read::Tasks { query, live: None, ended: None },
    }));
}

fn page(world: &World, number: u64) -> &reads::TaskPage {
    deliveries(world)
        .iter()
        .rev()
        .find_map(|(_, delivery)| {
            if let root::Delivery::Core(core::Held::Read { reply, answer: reads::Answer::Tasks(page) }) = delivery
                && *reply == ReplyTo::new(Token::new(number))
            {
                Some(page.as_ref())
            } else {
                None
            }
        })
        .expect("task page reached its live reply")
}

fn deliveries(world: &World) -> &[(u64, root::Delivery)] {
    &world.client_peer.as_ref().expect("configured fixture").delivered
}

pub(super) fn lines(world: &World, watcher: u64) -> &d::Lines {
    deliveries(world)
        .iter()
        .rev()
        .find_map(|(_, delivery)| {
            if let root::Delivery::Core(core::Held::Delivery { watcher: name, document }) = delivery
                && name.raw() == watcher
                && let d::Document::Lines(lines) = document.as_ref()
            {
                Some(lines)
            } else {
                None
            }
        })
        .expect("query delivery")
}

fn count(world: &World, watcher: u64) -> usize {
    deliveries(world)
        .iter()
        .filter(|(_, delivery)| {
            matches!(delivery,
        root::Delivery::Core(core::Held::Delivery { watcher: name, .. }) if name.raw() == watcher)
        })
        .count()
}

pub(super) fn open(world: &mut World, watcher: u64, query: d::TaskQuery) {
    world.send(root::Event::Core(core::Event::Watch {
        watcher: Token::new(watcher),
        sign_in: world.sign_in.expect("configured fixture"),
        subject: d::Watched::Query { query },
        window: d::Window { rows: 4, after: None },
    }));
}

pub(super) fn ack(world: &mut World, watcher: u64) {
    world.send(root::Event::Core(core::Event::Delivered { watcher: Token::new(watcher) }));
}

fn answer(world: &mut World, connector: u16, ask: &core::PartsAsk) {
    let parts = root::parts::make(ask);
    world.send(root::Event::Core(core::Event::Parts { connector, token: ask.token, parts }));
}

pub(super) fn close(world: &mut World, watcher: u64) {
    world.send(root::Event::Core(core::Event::Unwatch { watcher: Token::new(watcher) }));
    ack(world, watcher);
}

#[test]
fn read_and_watch_parts_wait_for_each_connector_and_durability_in_query_order() {
    let mut w = world(1220);
    let ended = w.task();
    fixtures::finish_chat(&mut w, 0, b"ended");
    let chat = fixtures::chat(&w, 1, b"live".as_slice().into());
    request(&mut w, 40, chat);
    let live = w.task();
    w.domain.script_parts();
    read(&mut w, 500, query(&[4, 99, 1], false));
    assert!(deliveries(&w).iter().all(|(_, value)| !matches!(value, root::Delivery::Core(core::Held::Read { .. }))));
    let (first, a) = w.domain.take_parts().expect("connector one ask");
    let (second, b) = w.domain.take_parts().expect("connector two ask");
    assert_eq!((first, second), (1, 2));
    assert!(w.domain.take_parts().is_none(), "one ask per connector and answer");
    assert_eq!(a.rows.iter().map(|row| row.chip.number).collect::<Vec<_>>(), [live, ended]);
    assert_eq!(a.kinds.iter().map(|kind| kind.kind).collect::<Vec<_>>(), [1]);
    assert_eq!(b.kinds.iter().map(|kind| kind.kind).collect::<Vec<_>>(), [4]);
    answer(&mut w, second, &b);
    w.store.fault(Fault::Hold { commits: 1 });
    w.turn(1, 1, None, b"durable before a read");
    answer(&mut w, first, &a);
    assert!(deliveries(&w).iter().all(|(_, value)| !matches!(value, root::Delivery::Core(core::Held::Read { .. }))));
    let commit = w.store.release_held().expect("configured fixture");
    w.send(root::Event::Committed { number: commit, fill: 0 });
    let p = page(&w, 500);
    let reads::EndedGroup::Rows { rows, .. } = &p.ended else { panic!("ended rows") };
    for (row, number) in p.live.iter().chain(rows.iter()).zip([live, ended]) {
        assert_eq!(
            row.parts.as_slice(),
            [
                core::RowPart::Made(Box::from(12_u64.to_le_bytes())),
                core::RowPart::Unknown,
                core::RowPart::Made(Box::from(number.to_le_bytes()))
            ]
        );
    }
    open(&mut w, 600, query(&[1, 4], false));
    assert_eq!(count(&w, 600), 0);
    let (first, a) = w.domain.take_parts().expect("configured fixture");
    let (second, b) = w.domain.take_parts().expect("configured fixture");
    assert_eq!(a.rows.len(), 1, "deliveries make live rows only");
    answer(&mut w, second, &b);
    assert_eq!(count(&w, 600), 0, "the other connector still owns a part");
    w.store.fault(Fault::Hold { commits: 1 });
    w.turn(2, 2, None, b"durable before a delivery");
    answer(&mut w, first, &a);
    assert_eq!(count(&w, 600), 0);
    let commit = w.store.release_held().expect("configured fixture");
    w.send(root::Event::Committed { number: commit, fill: 0 });
    assert_eq!(
        lines(&w, 600).lines.get(0).expect("configured fixture").parts.get(0),
        Some(&core::RowPart::Made(Box::from(live.to_le_bytes())))
    );
    assert!(deliveries(&w).iter().any(|(durable, value)| matches!(value,
        root::Delivery::Core(core::Held::Delivery { watcher, .. }) if watcher.raw() == 600 && *durable >= commit)));
    // Closing a changed delivery does not start another construction.
    close(&mut w, 600);
}

fn all_rows(page: &reads::TaskPage) -> Vec<&d::TaskLine> {
    let mut all: Vec<_> = page.live.iter().collect();
    if let reads::EndedGroup::Rows { rows, .. } = &page.ended {
        all.extend(rows.iter());
    }
    all
}

#[test]
fn unknown_oversized_wrong_count_and_busy_parts_leave_every_row_whole() {
    let mut w = world(1221);
    fixtures::finish_chat(&mut w, 0, b"ended");
    let chat = fixtures::chat(&w, 1, b"live".as_slice().into());
    request(&mut w, 40, chat);
    w.domain.script_parts();
    read(&mut w, 500, query(&[99], false));
    assert!(w.domain.take_parts().is_none(), "unknown kinds never ask a connector");
    assert_eq!(all_rows(page(&w, 500)).len(), 2);
    for row in all_rows(page(&w, 500)) {
        assert_eq!(row.parts.as_slice(), [core::RowPart::Unknown]);
    }
    read(&mut w, 501, query(&[2, 3], false));
    for _ in 0..2 {
        let (connector, ask) = w.domain.take_parts().expect("configured fixture");
        answer(&mut w, connector, &ask);
    }
    for row in all_rows(page(&w, 501)) {
        assert_eq!(
            row.parts.as_slice(),
            [core::RowPart::Failed(core::PartFailure::TooLarge), core::RowPart::Failed(core::PartFailure::Busy)]
        );
    }
    read(&mut w, 502, query(&[1, 2], false));
    let (connector, ask) = w.domain.take_parts().expect("configured fixture");
    assert_eq!(ask.kinds.len(), 2);
    let mut wrong = List::with_capacity(1);
    wrong.push(core::MadePart::Made(Box::new([]))).expect("configured fixture");
    w.send(root::Event::Core(core::Event::Parts { connector, token: ask.token, parts: wrong }));
    for row in all_rows(page(&w, 502)) {
        assert_eq!(
            row.parts.as_slice(),
            [
                core::RowPart::Failed(core::PartFailure::Unreadable),
                core::RowPart::Failed(core::PartFailure::Unreadable)
            ]
        );
    }
    read(&mut w, 503, query(&[2, 1], false));
    let (connector, ask) = w.domain.take_parts().expect("one connector for both kinds");
    assert_eq!(ask.kinds.iter().map(|kind| kind.kind).collect::<Vec<_>>(), [2, 1]);
    answer(&mut w, connector, &ask);
    for row in all_rows(page(&w, 503)) {
        assert_eq!(
            row.parts.as_slice(),
            [
                core::RowPart::Failed(core::PartFailure::TooLarge),
                core::RowPart::Made(Box::from(row.chip.number.to_le_bytes()))
            ]
        );
    }
}

#[test]
fn pending_parts_hold_load_slots_and_a_closed_watch_drops_its_late_answer() {
    let mut w = world(1222);
    w.domain.script_parts();
    open(&mut w, 600, query(&[1], true));
    read(&mut w, 500, query(&[1], true));
    let (one, watch) = w.domain.take_parts().expect("configured fixture");
    let (two, read_ask) = w.domain.take_parts().expect("configured fixture");
    read(&mut w, 501, query(&[1], true));
    assert!(w.domain.take_parts().is_none(), "two pending answers fill the two load slots");
    assert!(deliveries(&w).iter().any(|(_, value)| matches!(
        value,
        root::Delivery::Core(core::Held::Read { answer: reads::Answer::None(reads::Why::Busy), .. })
    )));
    open(&mut w, 601, query(&[1, 99], true));
    assert_eq!(
        lines(&w, 601).lines.get(0).expect("busy watch row").parts.as_slice(),
        [core::RowPart::Failed(core::PartFailure::Busy), core::RowPart::Unknown]
    );
    assert!(w.domain.take_parts().is_none(), "a busy watch retains its row without asking");
    close(&mut w, 601);
    w.send(root::Event::Core(core::Event::Unwatch { watcher: Token::new(600) }));
    answer(&mut w, one, &watch);
    assert_eq!(count(&w, 600), 0, "no late delivery for a closed watch");
    assert!(deliveries(&w).iter().any(|(_, value)| matches!(value, root::Delivery::Core(core::Held::WatchEnded { watcher, .. }) if watcher.raw() == 600)));
    answer(&mut w, two, &read_ask);
    assert_eq!(page(&w, 500).live.len(), 1);
    let before = deliveries(&w).len();
    answer(&mut w, one, &watch);
    assert_eq!(deliveries(&w).len(), before, "a duplicate answer cannot reuse a retired slot");
    read(&mut w, 502, query(&[1], true));
    let (connector, ask) = w.domain.take_parts().expect("configured fixture");
    answer(&mut w, connector, &ask);
    assert_eq!(page(&w, 502).live.len(), 1);
    read(&mut w, 503, query(&[1], true));
    let (connector, ask) = w.domain.take_parts().expect("pending read");
    w.client_peer.as_mut().expect("reply owner").end_read(Token::new(503));
    answer(&mut w, connector, &ask);
    assert!(
        deliveries(&w).iter().all(|(_, value)| !matches!(value,
        root::Delivery::Core(core::Held::Read { reply, .. }) if *reply == ReplyTo::new(Token::new(503)))),
        "an ended reply drops its late answer"
    );
    read(&mut w, 504, query(&[1], true));
    let (connector, ask) = w.domain.take_parts().expect("pending read");
    w.send(root::Event::Core(core::Event::People(people::Event::SignOut {
        reply_to: ReplyTo::new(Token::new(950)),
        sign_in: w.sign_in.expect("sign-in"),
    })));
    answer(&mut w, connector, &ask);
    assert!(deliveries(&w).iter().any(|(_, value)| matches!(value,
        root::Delivery::Core(core::Held::Read { reply, answer: reads::Answer::None(reads::Why::SignedOut) })
            if *reply == ReplyTo::new(Token::new(504)))));
}

pub(super) fn note(world: &mut World, task: u64, name: u64, recalled: Option<u32>) {
    world.send(root::Event::Core(core::Event::Notes(notes::Event::Write {
        owner: Token::new(800 + name),
        recalled,
        entry: notes::New {
            name,
            scope: notes::Scope::Goal { project: 1, goal: task },
            description: b"a part input".as_slice().into(),
            body: b"note".as_slice().into(),
            references: List::with_capacity(0),
            author: notes::Author::Party { party: 1 },
        },
    })));
}

#[test]
fn notes_and_connector_records_mark_only_the_queries_naming_their_inputs_without_activity() {
    let mut w = world(1223);
    let task = w.task();
    open(&mut w, 600, query(&[1], true));
    open(&mut w, 601, query(&[4], true));
    ack(&mut w, 600);
    ack(&mut w, 601);
    let activity = fixtures::live_row(&w, task).last_activity;
    note(&mut w, task, 700, None);
    assert_eq!(count(&w, 600), 2);
    assert_eq!(count(&w, 601), 1, "no notes input");
    ack(&mut w, 600);
    note(&mut w, task, 700, Some(1));
    assert_eq!(count(&w, 600), 3, "edited notes mark too");
    ack(&mut w, 600);
    w.send(root::Event::Core(core::Event::Notes(notes::Event::Edit {
        owner: Token::new(1500),
        party: 1,
        name: 700,
        scope: notes::Scope::Goal { project: 1, goal: task },
        change: notes::Change::Delete { recalled: 2 },
    })));
    assert_eq!(count(&w, 600), 4, "deleted notes mark too");
    ack(&mut w, 600);
    let mut numbers = List::with_capacity(1);
    numbers.push(task).expect("configured fixture");
    w.send(root::Event::Core(core::Event::PartsMarked { connector: 2, tasks: numbers.clone() }));
    assert_eq!(count(&w, 600), 4, "another connector cannot mark this kind's records");
    w.send(root::Event::Core(core::Event::PartsMarked { connector: 1, tasks: numbers }));
    assert_eq!(count(&w, 600), 5);
    assert_eq!(count(&w, 601), 1);
    assert_eq!(fixtures::live_row(&w, task).last_activity, activity, "notes never move task activity");
    close(&mut w, 600);
    close(&mut w, 601);
}

#[test]
fn delegate_notes_mark_the_requester_and_delegate_lines_touch_its_row() {
    let mut kinds = root::parts::kinds();
    kinds[0].inputs.notes = false;
    kinds[0].inputs.delegates = false;
    kinds[3].inputs.delegates = true;
    let mut w = configured_world(1225, kinds);
    let assigned = w.assigned.as_ref().expect("parent");
    let run = (assigned.task, assigned.attempt);
    let mut member = fixtures::member(&w, tasks::Executor::Agent { charter: 1 }, b"delegate");
    member.tracked = Some(1);
    member.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let child = fixtures::delegate(&mut w, run, 1, member);
    open(&mut w, 600, query(&[1], true));
    open(&mut w, 601, query(&[4], true));
    assert_eq!(lines(&w, 600).lines.get(0).expect("requester's row").chip.number, run.0);
    ack(&mut w, 600);
    ack(&mut w, 601);
    note(&mut w, child, 701, None);
    assert_eq!(count(&w, 600), 2, "the requester declares delegate_notes, not notes");
    assert_eq!(count(&w, 601), 1, "a delegate's notes are not its line");
    ack(&mut w, 600);
    note(&mut w, child, 701, Some(1));
    assert_eq!(count(&w, 600), 3);
    ack(&mut w, 600);
    w.send(root::Event::Core(core::Event::Notes(notes::Event::Edit {
        owner: Token::new(1501),
        party: 1,
        name: 701,
        scope: notes::Scope::Goal { project: 1, goal: child },
        change: notes::Change::Delete { recalled: 2 },
    })));
    assert_eq!(count(&w, 600), 4);
    ack(&mut w, 600);
    request(&mut w, 42, people::Ask::Stop { project: 1, task: child });
    assert_eq!(count(&w, 601), 2, "a delegate touched marks its requester's row for delegates");
    ack(&mut w, 600);
    ack(&mut w, 601);
    request(
        &mut w,
        43,
        people::Ask::Cancel { project: 1, task: child, reason: b"end delegate".as_slice().into(), grace: None },
    );
    assert!(w.store.rows.contains_key(&root::Key::Core(core::Key::Tasks(tasks::Key::Ended(child)))));
    ack(&mut w, 600);
    ack(&mut w, 601);
    let received = count(&w, 600);
    note(&mut w, child, 702, None);
    assert_eq!(count(&w, 600), received + 1, "an ended delegate's notes still mark its live requester");
    close(&mut w, 600);
    close(&mut w, 601);
}

#[test]
fn a_read_keeps_the_mark_from_before_an_ending_its_store_snapshot_missed() {
    let mut w = world(1226);
    w.client_peer = None;
    w.reads = Some(jig_core_world::reads::Parties::new(w.store.applied));
    jig_core_world::reads::ask(&mut w, 500, reads::Read::Tasks { query: query(&[1], false), live: None, ended: None });
    let reads::Answer::Tasks(first) = jig_core_world::reads::latest(&w) else { panic!("first page") };
    let before = first.ended_mark;
    w.reads.as_mut().expect("read party").hold_loads = true;
    jig_core_world::reads::ask(&mut w, 501, reads::Read::Tasks { query: query(&[1], false), live: None, ended: None });
    for request in w.reads.as_mut().expect("read party").complete() {
        w.store.ask(request);
    }
    // Take the real store's answer, then let the ending commit before that answer reaches the core.
    let mut snapshot = w.store.answers();
    assert_eq!(snapshot.len(), 1);
    let jig_test_store::Up::Root(answer) = snapshot.pop().expect("store snapshot") else { panic!("read terminal") };
    fixtures::finish_chat(&mut w, 0, b"ended after the snapshot");
    w.send(answer);
    let reads::Answer::Tasks(page) = jig_core_world::reads::latest(&w) else { panic!("snapshot page") };
    assert_eq!(page.ended_mark, before);
    let reads::EndedGroup::Rows { rows, .. } = &page.ended else { panic!("ended snapshot") };
    assert!(rows.is_empty(), "this load missed the ending");
    w.reads.as_mut().expect("read party").hold_loads = false;
    jig_core_world::reads::ask(&mut w, 502, reads::Read::Tasks { query: query(&[1], false), live: None, ended: None });
    let reads::Answer::Tasks(after) = jig_core_world::reads::latest(&w) else { panic!("current page") };
    assert!(after.ended_mark > before);
}

#[test]
fn forgetting_and_retention_advance_the_ended_mark_when_the_index_rows_go() {
    for forgotten in [true, false] {
        let mut w = crate::retention::world(1227, forgotten);
        let record = crate::retention::finish(&mut w);
        w.client_peer = Some(ClientPeer::default());
        open(&mut w, 600, query(&[], false));
        let before = lines(&w, 600).ended_mark;
        ack(&mut w, 600);
        crate::retention::read(&mut w, &record);
        if !forgotten {
            crate::retention::at(&mut w, &record, 31);
            crate::retention::pass(&mut w);
            crate::retention::at(&mut w, &record, 366);
            crate::retention::pass(&mut w);
        }
        assert_eq!(crate::retention::indexes(&w, record.number), 0);
        assert!(lines(&w, 600).ended_mark > before, "erased ended index: forgotten={forgotten}");
        close(&mut w, 600);
    }
}

#[test]
fn ended_marks_cover_endings_between_deliveries_ended_inputs_and_restart() {
    let mut w = world(1224);
    open(&mut w, 600, query(&[1], false));
    let first = lines(&w, 600).ended_mark;
    let ended = w.task();
    fixtures::finish_chat(&mut w, 0, b"ended");
    let chat = fixtures::chat(&w, 1, b"transient".as_slice().into());
    request(&mut w, 41, chat);
    fixtures::finish_chat(&mut w, 0, b"ended between deliveries");
    assert_eq!(count(&w, 600), 1, "changes coalesce behind the first delivery");
    ack(&mut w, 600);
    let ended_mark = lines(&w, 600).ended_mark;
    assert!(ended_mark > first);
    assert!(lines(&w, 600).lines.is_empty());
    ack(&mut w, 600);
    note(&mut w, ended, 700, None);
    let input_mark = lines(&w, 600).ended_mark;
    assert!(input_mark > ended_mark, "an ended row's notes move its group mark");
    ack(&mut w, 600);
    w.send(root::Event::Core(core::Event::PartsMarked { connector: 1, tasks: List::from_boxed(Box::new([ended])) }));
    let records_mark = lines(&w, 600).ended_mark;
    assert!(records_mark > input_mark, "an ended row's records move its group mark");
    close(&mut w, 600);
    read(&mut w, 500, query(&[], true));
    assert_eq!(page(&w, 500).ended_mark, 0, "no ended group");
    let floor = w.domain.core().counters.deployment().commits + 1;
    let (mut config, _) = fixture(1224, false);
    config.core.parts = root::parts::kinds();
    w.restart_with(config);
    read(&mut w, 501, query(&[1], false));
    assert!(page(&w, 501).ended_mark >= floor);
    assert!(page(&w, 501).ended_mark >= records_mark);
}
