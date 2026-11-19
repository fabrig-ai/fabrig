//! Parts are made by the real root, from the durable note index and entries.
use super::*;
use jig_domain_core::{documents as d, reads};
use skein_lib::{List, Reader};
use temper_client_protocol_wire::v1::Record as CodecRecord;
use temper_client_protocol_wire::{self as wire, v1 as w};

fn fixture() -> (Driver, engine::Assignment) {
    let mut configs = [note_config(), note_config()];
    for config in &mut configs {
        config.core.parts = temper_engine_domain_forge::parts::kinds(0);
    }
    let mut bounds = limits();
    bounds.notes.description_bytes = 128;
    bounds.notes.entries_per_scope = 8;
    bounds.call_records = 8;
    let mut driver = Driver::configured(Store::new(), configs, &bounds);
    hello(&mut driver);
    driver.settle();
    driver.sign_in();
    driver.settle();
    chat(&mut driver, 12);
    driver.settle();
    let run = assigned(&driver);
    (driver, run)
}
pub(super) fn query(kind: u32) -> d::TaskQuery {
    let mut query = jig_test_profile::fixtures::own_tasks();
    query.project = Some(1);
    query.parts = List::with_capacity(1);
    query.parts.push(d::PartAsk { owner: wire::FORGE, kind, version: wire::CLIENT_VERSION }).expect("part fixture");
    query
}
pub(super) fn read(driver: &mut Driver, query: d::TaskQuery) -> Box<reads::TaskPage> {
    driver.send(engine::Event::Party(engine::PartyInput::Read {
        reply_to: ReplyTo::new(Token::new(800)),
        sign_in: driver.session(),
        read: Box::new(reads::Read::Tasks { query, live: None, ended: None }),
    }));
    driver.settle();
    driver
        .delivered
        .iter()
        .rev()
        .find_map(|delivery| {
            if let Delivery::Read { answer: reads::Answer::Tasks(page), .. } = delivery {
                Some(page.clone())
            } else {
                None
            }
        })
        .expect("task query answered")
}
pub(super) fn bytes(line: &d::TaskLine) -> &[u8] {
    let Some(d::RowPart::Made(bytes)) = line.parts.get(0) else { panic!("made part: {:?}", line.parts) };
    bytes
}
fn goal(line: &d::TaskLine) -> w::GoalPart {
    w::GoalPart::decode(&w::CEILINGS, &mut Reader::new(bytes(line))).expect("part fixture")
}
fn document(line: &d::TaskLine) -> Option<Box<[u8]>> {
    w::ChatPart::decode(&w::CEILINGS, &mut Reader::new(bytes(line))).expect("part fixture").document.clone()
}
pub(super) fn note(driver: &mut Driver, run: &engine::Assignment, call: u64, name: u64, description: &[u8]) -> u64 {
    let recalled = if name == 0 { None } else { Some(stored_note(driver, name).revision) };
    tool_call(
        driver,
        run,
        call,
        engine::Tool::Note {
            entry: notes::New {
                name,
                scope: notes::Scope::Goal { project: 1, goal: run.task },
                description: description.into(),
                body: b"body".as_slice().into(),
                references: List::with_capacity(0),
                author: notes::Author::Task { task: 0, attempt: 0 },
            },
            recalled,
        },
    );
    let Some(engine::CallAnswer::NoteWritten { name, .. }) = driver.answered(call) else {
        panic!(
            "note {call}: {:?}; phase={:?}; events={:?}; tail={:?}",
            driver.answered(call),
            driver.task(run.task).map(|row| &row.phase),
            driver.events,
            driver.delivered.iter().rev().take(5).collect::<Vec<_>>()
        )
    };
    *name
}
fn watch(driver: &mut Driver, kind: u32) {
    driver.send(engine::Event::Party(engine::PartyInput::Watch {
        watcher: Token::new(801),
        sign_in: driver.session(),
        subject: d::Watched::Query { query: query(kind) },
        window: d::Window { rows: 2, after: None },
    }));
    driver.settle();
}
fn delivered(driver: &mut Driver) {
    driver.send(engine::Event::Party(engine::PartyInput::Delivered { watcher: Token::new(801) }));
    driver.settle();
}
fn watched(driver: &Driver) -> &d::TaskLine {
    driver
        .delivered
        .iter()
        .rev()
        .find_map(|delivery| {
            if let Delivery::Document { document, .. } = delivery
                && let d::Document::Lines(lines) = document.as_ref()
            {
                lines.lines.get(0)
            } else {
                None
            }
        })
        .expect("query redelivered")
}
#[test]
fn goal_parts_choose_the_newest_report_and_invalidate_the_kept_entry() {
    let (mut driver, run) = fixture();
    let page = read(&mut driver, query(wire::GOAL_PART));
    assert!(goal(page.live.get(0).expect("part fixture")).report.is_none());
    note(&mut driver, &run, 710, 0, b"report: Landing slices");
    driver.env.wall = Wall::from_nanos(driver.env.wall.as_nanos() + 1);
    let name = note(&mut driver, &run, 711, 0, b"report: Two of three landed");
    let changed = stored_note(&driver, name).changed;
    watch(&mut driver, wire::GOAL_PART);
    let part = goal(watched(&driver));
    assert_eq!(part.report.as_ref().expect("part fixture").headline.as_ref(), b"Two of three landed");
    assert_eq!(part.report.as_ref().expect("part fixture").at, changed.as_nanos());
    assert_eq!(part.milestone, None);
    let loads = driver.loads.len();
    read(&mut driver, query(wire::GOAL_PART));
    assert_eq!(driver.loads.len(), loads, "kept notes avoid all loads");
    delivered(&mut driver);
    note(&mut driver, &run, 712, 0, b"report: All landed");
    assert_eq!(goal(watched(&driver)).report.as_ref().expect("part fixture").headline.as_ref(), b"All landed");
    assert!(driver.loads.len() > loads, "a saved scope note invalidates the entry");
}
#[test]
fn chat_parts_keep_the_first_non_work_note_and_follow_its_edits() {
    let (mut driver, run) = fixture();
    assert_eq!(document(read(&mut driver, query(wire::CHAT_PART)).live.get(0).expect("part fixture")), None);
    note(&mut driver, &run, 720, 0, b"work: scratch");
    let first = note(&mut driver, &run, 721, 0, b"The chat document");
    note(&mut driver, &run, 722, 0, b"Later document");
    watch(&mut driver, wire::CHAT_PART);
    assert_eq!(document(watched(&driver)).as_deref(), Some(b"The chat document".as_slice()));
    delivered(&mut driver);
    note(&mut driver, &run, 723, 0, b"work: later scratch");
    assert_eq!(document(watched(&driver)).as_deref(), Some(b"The chat document".as_slice()));
    delivered(&mut driver);
    note(&mut driver, &run, 724, first, b"Edited document");
    assert_eq!(document(watched(&driver)).as_deref(), Some(b"Edited document".as_slice()));
}
#[test]
fn milestone_encoding_keeps_a_sixteen_goal_prefix_within_its_bytes() {
    let (mut driver, _) = fixture();
    let page = read(&mut driver, query(wire::GOAL_PART));
    let mut line = page.live.get(0).expect("part fixture").clone();
    line.parts = List::with_capacity(0);
    let totals = temper_engine_domain_forge::MilestoneTotals {
        task: 99,
        done: 0,
        failed: 0,
        closed: 0,
        live: 17,
        ended: None,
        last_landing: None,
        spent: 0,
    };
    let goals = vec![(line.clone(), None); 17];
    let encoded =
        temper_engine_service_vocabulary::family::parts::milestone(None, &totals, &goals).expect("part fixture");
    let part = w::MilestonePart::decode(&w::CEILINGS, &mut Reader::new(&encoded)).expect("part fixture");
    assert_eq!((part.goals.len(), part.more), (16, 1));
    line.chip.title.bytes = vec![b'x'; 8000].into_boxed_slice();
    let goals = vec![(line, None); 17];
    let encoded =
        temper_engine_service_vocabulary::family::parts::milestone(None, &totals, &goals).expect("part fixture");
    let part = w::MilestonePart::decode(&w::CEILINGS, &mut Reader::new(&encoded)).expect("part fixture");
    assert!(encoded.len() <= usize::try_from(wire::MILESTONE_PART_BYTES).expect("part fixture"));
    assert!(!part.goals.is_empty() && part.goals.len() < 16);
    assert_eq!(part.goals.len() + part.more, 17);
}

#[test]
fn part_loads_answer_busy_or_unreadable_without_losing_the_query() {
    let (mut driver, _) = fixture();
    let mut terminals = Vec::new();
    for number in 0..=driver.env.limits.loads.loads {
        driver.send(engine::Event::Party(engine::PartyInput::Watch {
            watcher: Token::new(820 + u64::from(number)),
            sign_in: driver.session(),
            subject: d::Watched::Query { query: query(wire::GOAL_PART) },
            window: d::Window { rows: 1, after: None },
        }));
        for _ in 0..30 {
            if let Some(at) = driver
                .events
                .iter()
                .position(|event| matches!(event, engine::Event::Store(engine::StoreInput::Loaded { .. })))
            {
                let engine::Event::Store(engine::StoreInput::Loaded { owner, .. }) =
                    driver.events.remove(at).expect("part fixture")
                else {
                    unreachable!()
                };
                terminals.push(engine::Event::Store(engine::StoreInput::Unloaded { owner }));
                break;
            }
            driver.advance(true);
        }
    }
    driver.events.extend(terminals);
    driver.settle();
    let failures: Vec<_> = driver
        .delivered
        .iter()
        .filter_map(|delivery| {
            if let Delivery::Document { document, .. } = delivery
                && let d::Document::Lines(lines) = document.as_ref()
            {
                Some(lines.lines.get(0).expect("part fixture").parts.get(0).expect("part fixture"))
            } else {
                None
            }
        })
        .collect();
    assert!(failures.contains(&&d::RowPart::Failed(d::PartFailure::Busy)), "{failures:?}");
    assert!(failures.contains(&&d::RowPart::Failed(d::PartFailure::Unreadable)), "{failures:?}");
}
