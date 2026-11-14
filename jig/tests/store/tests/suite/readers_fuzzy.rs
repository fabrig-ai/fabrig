//! Seeded pages against independently committed inputs (protocol/store/loads.md, 1, and protocol/store/README.md, 9).
use jig_domain_core::{self as core, reads};
use jig_domain_tasks as tasks;
use jig_protocol_store::{Restore, Row, Write, keys, restore::Item};
use jig_store_world::{World, root};
use skein_lib::Rng;
use skein_lib::{Token, Wall};
use std::collections::BTreeMap;

// Each page must hold the two-byte application key and largest 32-byte value.
const APPLICATION_BYTES: u64 = 34;
const INVENTORY_BYTES: u64 = 65_536;
const SEEDS: u64 = 32;
const RECORDS: u64 = 20;

#[test]
fn seeded_pages_match_committed_application_rows() {
    for seed in 0..SEEDS {
        pages_match(seed);
    }
}

fn pages_match(seed: u64) {
    let mut rng = Rng::new(seed);
    let mut world = World::unopened(seed);
    world.open().expect("open");
    world.commit(root::first()).expect("first commit");
    let expected: BTreeMap<Box<[u8]>, Box<[u8]>> = (0..RECORDS)
        .map(|number| {
            let key = Box::from([u8::try_from(number).expect("twenty rows"), 0]);
            let value =
                vec![u8::try_from(number).expect("twenty rows"); usize::try_from(rng.below(33)).expect("value")]
                    .into_boxed_slice();
            (key, value)
        })
        .collect();
    let mut writes = vec![Write::Save(root::header(2))];
    writes.extend(expected.iter().map(|(key, value)| Write::Put { key: key.clone(), value: value.clone() }));
    world.commit(writes.into_boxed_slice()).expect("source commits");
    let mut found = BTreeMap::new();
    let mut after = None;
    loop {
        let rows = u32::try_from(1 + rng.below(4)).expect("row budget");
        let page = world
            .restore_page(
                Restore::Application(Box::new([skein_kv::Range::prefix(b"")])),
                after.as_ref(),
                rows,
                APPLICATION_BYTES,
            )
            .expect("typed page");
        assert!(page.items.len() <= skein_lib::length::usize_of(rows));
        let mut bytes = 0;
        for item in page.items {
            let Item::Row(Row::Application { key, value }) = item else { panic!("application row") };
            bytes += key.len() + value.len();
            assert!(found.insert(key, value).is_none(), "exactly once, seed {seed}");
        }
        assert!(bytes <= usize::try_from(APPLICATION_BYTES).expect("bytes"));
        after = page.next;
        if after.is_none() {
            break;
        }
    }
    assert_eq!(found, expected, "typed source readback seed {seed}");
    let expected: BTreeMap<_, _> = expected
        .into_iter()
        .map(|(key, value)| (keys::application(&key, root::limits().store.key).expect("raw key"), value))
        .collect();
    let mut found = BTreeMap::new();
    let mut after = None;
    loop {
        let rows = u32::try_from(1 + rng.below(4)).expect("row budget");
        let page = world.retained_page(after.as_deref(), rows, INVENTORY_BYTES).expect("inventory page");
        assert!(page.rows.len() <= skein_lib::length::usize_of(rows.min(root::limits().store.page.rows)));
        for row in page.rows {
            assert!(after.as_deref().is_none_or(|after| row.key.as_ref() > after));
            if row.key.first() == Some(&keys::APPLICATION) {
                assert!(found.insert(row.key, row.value).is_none(), "exactly once, seed {seed}");
            }
        }
        after = page.next;
        if after.is_none() {
            break;
        }
    }
    assert_eq!(found, expected, "raw source readback seed {seed}");
    let complete: BTreeMap<_, _> =
        world.retained().into_iter().filter(|(key, _)| keys::of_application(key).is_some()).collect();
    assert_eq!(complete, expected, "complete collector source readback seed {seed}");
}

/// Mix retained records, summary-only owners and stale indexes, varying the
/// page boundaries independently (protocol/store/retention.md, 7).
#[test]
fn seeded_task_queries_list_summary_only_owners_and_other_reads_are_gone() {
    for seed in 0..SEEDS {
        summary_queries(seed);
    }
}

fn summary_owners(
    seed: u64,
    rng: &mut Rng,
    stand: &jig_core_kit::world::World,
) -> (World, BTreeMap<u64, core::documents::TaskLine>, Vec<u64>) {
    let mut world = World::new(seed);
    world.commit(root::first()).expect("first commit");
    let mut expected = BTreeMap::new();
    let mut summaries = Vec::new();
    let mut writes = vec![Write::Save(root::header(2))];
    for number in 100..109 {
        let core::Record::Tasks(tasks::Stored::Live(mut task)) = root::live_task(number) else { panic!("fixture") };
        task.spec.words = format!("Ship {seed}:{number}").into_bytes().into_boxed_slice();
        task.flags.listed = true;
        task.label = u32::try_from(rng.below(3)).expect("three labels");
        task.phase =
            tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words: b"done".as_slice().into() }));
        task.ended_at = Some(Wall::from_nanos(rng.below(10)));
        let summary = tasks::TaskSummary::of(&task);
        let env = stand.env();
        match (number + seed) % 3 {
            0 => {
                expected.insert(
                    number,
                    core::documents::task_line(stand.domain.core(), &env.limits.core, 1, &task, None)
                        .expect("record line"),
                );
                writes.push(Write::Save(core::Record::Tasks(tasks::Stored::Ended(task))));
            }
            1 => {
                expected.insert(
                    number,
                    core::documents::task_line_from_summary(stand.domain.core(), &env.limits.core, 1, &summary)
                        .expect("summary line"),
                );
                writes.push(Write::Save(core::Record::Tasks(tasks::Stored::Summary(Box::new(summary.clone())))));
                summaries.push(number);
                writes.push(Write::Save(core::Record::Tasks(tasks::Stored::Milestone(tasks::Milestone {
                    task: number,
                    position: 1,
                    phase: tasks::Phase::Waiting,
                    at: Wall::EPOCH,
                }))));
                writes.push(Write::Save(core::Record::Tasks(tasks::Stored::Activity(tasks::ActivityLine {
                    task: number,
                    position: 1,
                    at: Wall::EPOCH,
                    line: tasks::Line::Words {
                        message: 1,
                        by: summary.requester,
                        kind: tasks::MessageKind::Words,
                        words: b"kept".as_slice().into(),
                    },
                }))));
            }
            2 => {}
            _ => unreachable!("three owner states"),
        }
        for index in [
            tasks::Stored::EndedByParty(tasks::EndedByParty {
                party: 1,
                label: summary.label,
                ended_at: summary.ended_at,
                task: number,
            }),
            tasks::Stored::EndedByProject(tasks::EndedByProject {
                project: 1,
                label: summary.label,
                ended_at: summary.ended_at,
                task: number,
            }),
        ] {
            writes.push(Write::Save(core::Record::Tasks(index)));
        }
    }
    world.commit(writes.into_boxed_slice()).expect("mixed ended owners");
    (world, expected, summaries)
}

fn summary_queries(seed: u64) {
    let mut rng = Rng::new(seed);
    let mut stand = jig_core_kit::world::World::new(jig_core_kit::world::Setup { seed, ..Default::default() });
    let (mut world, expected, summaries) = summary_owners(seed, &mut rng, &stand);
    for requester in [None, Some(tasks::Party::Person(1))] {
        let query = core::documents::TaskQuery {
            project: Some(1),
            requester,
            root: None,
            parent: None,
            label: None,
            phase: Some(core::documents::PhaseClass::Ended),
            executor: None,
            tracked: None,
            order: core::documents::QueryOrder::Ended,
        };
        let mut after = None;
        let mut found = BTreeMap::new();
        loop {
            let answer = joined_answer(
                &mut stand,
                &mut world,
                reads::Read::Tasks { query, live: None, ended: after.clone() },
                u32::try_from(1 + rng.below(4)).expect("page bound"),
            );
            let reads::Answer::Tasks(page) = answer else { panic!("task query seed {seed}") };
            let reads::EndedGroup::Rows { rows, next, .. } = page.ended else { panic!("ended group") };
            for line in rows.into_boxed() {
                assert!(found.insert(line.chip.number, line).is_none(), "once, seed {seed}");
            }
            if next.is_none() {
                break;
            }
            assert_ne!(next, after, "continuation advances, seed {seed}");
            after = next;
        }
        assert_eq!(
            found.keys().collect::<Vec<_>>(),
            expected.keys().collect::<Vec<_>>(),
            "mixed query seed {seed}, {requester:?}"
        );
        assert_eq!(found, expected, "mixed query seed {seed}, {requester:?}");
    }
    for task in summaries {
        for read in [
            reads::Read::Task { task },
            reads::Read::Result { task },
            reads::Read::TaskText { task, field: reads::TaskTextField::Specification, from: 0 },
        ] {
            assert_eq!(
                joined_answer(&mut stand, &mut world, read, 1),
                reads::Answer::None(reads::Why::Gone),
                "point seed {seed}, task {task}"
            );
        }
        for read in
            [reads::Read::History { task, subtree: false, from: None }, reads::Read::Activity { task, before: None }]
        {
            let reads::Answer::Page(page) = joined_answer(&mut stand, &mut world, read, 2) else {
                panic!("empty page")
            };
            assert!(page.rows.is_empty(), "erased owner seed {seed}");
            assert_eq!(page.next, None);
        }
    }
}

fn joined_answer(
    stand: &mut jig_core_kit::world::World,
    world: &mut World,
    read: reads::Read,
    rows: u32,
) -> reads::Answer {
    let env = stand.env().with(stand.env().limits.core);
    let sign_in = stand.sign_in.expect("signed in");
    stand.domain.core_mut().reclaim();
    let mut output = core::step(
        stand.domain.core_mut(),
        &env,
        core::Event::Read { reply: skein_lib::ReplyTo::new(Token::new(900)), sign_in, read },
    );
    let Some(core::Request::Held(held)) = output.pop() else { panic!("one load") };
    assert!(output.is_empty());
    let core::Held::ReadLoad { load, mut range } = *held else { panic!("retained read load: {held:?}") };
    range.rows = range.rows.min(rows);
    let mut events = world.request(jig_protocol_store::Request::Read { owner: load, range: *range }).into_iter();
    let Some(jig_protocol_store::Event::Read { result, .. }) = events.next() else { panic!("store read") };
    assert!(events.next().is_none());
    let mut output = core::step(stand.domain.core_mut(), &env, core::Event::ReadLoaded { load, result });
    let Some(core::Request::Held(held)) = output.pop() else { panic!("one answer") };
    assert!(output.is_empty());
    let core::Held::Read { answer, .. } = *held else { panic!("read answer") };
    answer
}
