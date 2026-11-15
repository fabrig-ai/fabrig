//! Dev's registered layouts settle before history enters logs (protocol/store/jobs.md, 6).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary Rust step tests"
)]
use super::*;
use crate::{JIG_STORE_PATCH_SET as SET, Owner, PatchSet, Unplanned, codec, keys, registry_v1 as wire};
use alloc::collections::VecDeque;
use jig_domain_core_vocabulary::{Key, Record};
use jig_domain_tasks as tasks;
use skein_lib::{List, Reader, Writer};

const TASKS: &str = include_str!("../fixtures/cb429613c6-tasks-v13.schema");
const CORE: &str = include_str!("../fixtures/cb429613c6-core-v6.schema");
const KEPT: &str = include_str!("../../../schema/kept/jig-store-tasks-v13.schema");
fn old_set() -> PatchSet {
    let families = SET
        .families
        .iter()
        .map(|family| crate::KeptFamily {
            current: match family.name {
                "jig-store-tasks" => TASKS,
                "jig-store-core" => CORE,
                _ => family.current,
            },
            ..*family
        })
        .collect::<Vec<_>>();
    PatchSet {
        families: Box::leak(families.into_boxed_slice()),
        jobs: Box::leak(
            SET.jobs
                .iter()
                .filter(|job| !["settle-layouts", "ended-talk", "task-logs"].contains(&job.name))
                .copied()
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        ),
        ..SET
    }
}
fn layout_key(family: &str, number: u16, id: u16) -> Box<[u8]> {
    keys::encode_own(&keys::Own::Layout { family: family.into(), number, id }, 1024).unwrap()
}
fn hash(text: &str) -> [u8; 32] {
    let mut hash = skein_lib::sha256::Sha256::new();
    hash.update(text.as_bytes());
    hash.finish()
}
fn ended_layout(text: &str, id: u16) -> Vec<skein_kv::Op> {
    let schema = skein_codec::schema::parse(text.as_bytes()).unwrap();
    let row = wire::Layout::new(
        &wire::CEILINGS,
        wire::LayoutParts {
            root: b"Row".as_slice().into(),
            variant: None,
            layout: schema.layout(schema.root(b"Row", None).unwrap(), &[]).unwrap().digest(),
            text: hash(text),
            held: List::with_capacity(0),
        },
    )
    .unwrap();
    let mut writer = Writer::new(usize::try_from(row.measure()).unwrap());
    row.encode(&mut writer).unwrap();
    let text_row = wire::Text::new(&wire::CEILINGS, wire::TextParts { text: text.as_bytes().into() }).unwrap();
    let mut text_writer = Writer::new(usize::try_from(text_row.measure()).unwrap());
    text_row.encode(&mut text_writer).unwrap();
    vec![
        skein_kv::Op::Put { key: layout_key("jig-store-tasks", 6, id), value: writer.finish() },
        skein_kv::Op::Put {
            key: keys::encode_own(&keys::Own::Text(hash(text)), 1024).unwrap(),
            value: text_writer.finish(),
        },
    ]
}
fn convert(bytes: &[u8], from: &str, to: &str, variant: Option<&[u8]>) -> Box<[u8]> {
    let from = skein_codec::schema::parse(from.as_bytes()).unwrap();
    let to = skein_codec::schema::parse(to.as_bytes()).unwrap();
    let tasks = skein_codec::schema::parse(TASKS.as_bytes()).unwrap();
    let plan = skein_codec::layout::Plan::derive_dropping(
        &from,
        from.root(b"Row", variant).unwrap(),
        &[&tasks],
        &to,
        to.root(b"Row", variant).unwrap(),
        &[&tasks],
    )
    .unwrap();
    let mut work = skein_codec::layout::Workspace::new();
    assert!(work.fit(&plan));
    let mut out = skein_lib::Buffer::with_capacity(1_048_576);
    plan.convert(bytes, &mut out, &mut work).unwrap();
    out.as_slice().into()
}
fn rehead(mut bytes: Box<[u8]>, id: u16) -> Box<[u8]> {
    bytes[..2].copy_from_slice(&id.to_be_bytes());
    bytes
}
fn expected() -> Vec<Record> {
    let mut rows = Vec::new();
    for number in 1..=4 {
        let row = crate::tests::ended_talk::row(number, number % 2 == 1);
        rows.push(Record::Tasks(codec::tasks::decode(&row.value, &crate::tasks_v13::CEILINGS, &[]).unwrap()));
    }
    let mut record = fixtures::task_record();
    record.number = 5;
    rows.push(Record::Tasks(tasks::Stored::Live(Box::new(record.clone()))));
    rows.push(Record::Tasks(tasks::Stored::LiveCold {
        number: 5,
        spec: Box::new(record.spec),
        contract: record.contract,
        authority: record.authority,
        instructions: record.instructions,
    }));
    rows.push(fixtures::attempt_row(1, 1));
    for projection in [
        fixtures::core_rows::goal(),
        jig_domain_core_vocabulary::ProjectionRecord::GoalWords {
            goal: 11,
            task: 11,
            words: b"goal".as_slice().into(),
        },
        fixtures::core_rows::projection_task(tasks::Party::Task(11)),
        jig_domain_core_vocabulary::ProjectionRecord::TaskWords {
            goal: 11,
            task: 12,
            words: b"task".as_slice().into(),
        },
    ] {
        rows.push(Record::Core(jig_domain_core_vocabulary::CoreRecord::Projection(projection)));
    }
    rows
}
fn baseline(extra: Option<(&[u8], &[u8])>, logs_done: bool) -> Vec<Item> {
    let mut disk = Disk::new(&[]);
    let old_set = old_set();
    let mut old = jig_store(&disk, old_set);
    assert_opened(&disk.open(&mut old), 0);
    let events = disk.request(&mut old, fixtures::commit(1, Box::new([fixtures::header(1)])));
    assert!({ if let [Event::Committed { .. }] = events.as_ref() { true } else { false } });
    let mut ops = ended_layout(KEPT, 13);
    ops.extend(ended_layout(TASKS, 14));
    let core_layout = fixtures::get_kv(&mut old, &layout_key("jig-store-core", 1, 6)).unwrap();
    ops.push(skein_kv::Op::Erase { key: layout_key("jig-store-core", 1, 6) });
    ops.push(skein_kv::Op::Put { key: layout_key("jig-store-core", 1, 7), value: core_layout });
    for record in expected() {
        let key = keys::encode(&record.key(), 1024).unwrap();
        let bytes = codec::encode(&record, &old.codec_limits, &[]).unwrap();
        let bytes = if key.starts_with(&[1, 1, 6]) {
            let Key::Tasks(tasks::Key::Ended(number)) = record.key() else { unreachable!() };
            let row = crate::tests::ended_talk::row(number, number % 2 == 1);
            let (text, id) = if number <= 2 { (KEPT, 13) } else { (TASKS, 14) };
            rehead(convert(&row.value, include_str!("../../../schema/tasks-v13.schema"), text, None), id)
        } else if key.starts_with(&[1, 1]) {
            convert(&bytes, include_str!("../../../schema/tasks-v13.schema"), TASKS, None)
        } else {
            rehead(convert(&bytes, include_str!("../../../schema/core-v6.schema"), CORE, Some(b"projection")), 7)
        };
        ops.push(skein_kv::Op::Put { key, value: bytes });
    }
    if let Some((key, value)) = extra {
        ops.push(skein_kv::Op::Put { key: key.into(), value: value.into() });
    }
    if logs_done {
        ops.push(crate::jobs::kept::operation(&old, Owner::Jig, SET.jobs.last().unwrap(), 2).unwrap());
    }
    for chunk in ops.chunks(16) {
        registry::raw(&mut disk, &mut old, chunk.to_vec());
    }
    disk.items()
}
type Contents = Vec<(Box<[u8]>, Kind, u32, Box<[u8]>)>;
fn contents(disk: &mut Disk) -> Contents {
    disk.contents().into_iter().filter(|(_, kind, _, bytes)| *kind != Kind::File || !bytes.is_empty()).collect()
}
fn open(disk: &mut Disk, store: &mut Store) -> Vec<Event> {
    let mut pending = VecDeque::from(disk.open(store).into_vec());
    let mut events = Vec::new();
    while let Some(event) = pending.pop_front() {
        if let Event::Job { owner, of: Owner::Jig, job, page } = event {
            let answer = crate::jobs::page(&job, &page);
            pending.extend(disk.request(store, Request::Job { owner, answer }));
        } else {
            events.push(event);
        }
    }
    events
}
#[test]
fn dev_layouts_13_14_and_7_settle_before_ended_talk_and_task_logs_and_reopen_in_place() {
    let mut disk = Disk::new(&baseline(None, false));
    let mut store = jig_store(&disk, SET);
    let events = open(&mut disk, &mut store);
    let ran = events
        .iter()
        .filter_map(|event| {
            if let Event::Ran { job, counts, .. } = event {
                Some((job.as_ref(), counts.iter().map(|count| (count.name.as_ref(), count.count)).collect::<Vec<_>>()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(ran.len(), 3, "{events:?}");
    assert_eq!(ran[0].0, "settle-layouts");
    assert_eq!(
        ran[0].1,
        vec![("put", 11), ("retired", 8), ("turns-6", 0), ("turns-7", 0), ("turns-other", 0), ("turns-unread", 0)],
        "{ran:?}"
    );
    assert_eq!(ran[1], ("ended-talk", vec![("ended", 2), ("ended-talk", 2)]));
    assert_eq!(ran[2], ("task-logs", vec![]));
    assert!({
        if let [
            Event::Ran { .. },
            Event::Ran { .. },
            Event::Ran { .. },
            Event::Opened { pending: true, .. },
            Event::Census { .. },
        ] = events.as_slice()
        {
            true
        } else {
            false
        }
    });
    for record in expected() {
        let events = disk.request(&mut store, Request::Get { owner: Token::new(9), key: Get::Jig(record.key()) });
        let [Event::Got { row: Some(Row::Jig(found)), .. }] = events.as_ref() else { panic!("{events:?}") };
        assert_eq!(found, &record);
    }
    for task in 1..=4 {
        let key = keys::encode(&Key::Tasks(tasks::Key::Ended(task)), 1024).unwrap();
        assert!(crate::view::raw_get(&mut store, key).unwrap().is_none());
        assert!(store.stores.store.log_end(crate::logs::history(task).unwrap()) > 0);
    }
    let range = [skein_kv::Range { start: Box::from([0, 6]), end: Some(Box::from([0, 7])) }];
    let mut after = None;
    let mut layouts = Vec::new();
    loop {
        let page = crate::pages::reader::walk(&mut store, &range, after.as_ref(), 128).unwrap();
        for row in page.rows {
            layouts.push((
                keys::decode_own(&row.row.key).unwrap(),
                wire::Layout::decode(&wire::CEILINGS, &mut Reader::new(&row.row.value)).unwrap(),
            ));
        }
        after = page.next;
        if after.is_none() {
            break;
        }
    }
    let schemas = [crate::SKEIN_PATCH_SET, crate::SMITH_PATCH_SET, SET]
        .iter()
        .flat_map(|set| set.families)
        .filter(|family| !family.texts.is_empty())
        .map(|family| skein_codec::schema::parse(family.current.as_bytes()).unwrap())
        .collect::<Vec<_>>();
    let held = schemas.iter().collect::<Vec<_>>();
    for family in SET.families.iter().filter(|family| !family.types.is_empty()) {
        for ty in family.types {
            let rows = layouts
                .iter()
                .filter(|(key, _)| {
                    if let keys::Own::Layout { family: name, number, .. } = key {
                        name.as_ref() == family.name && *number == ty.number
                    } else {
                        false
                    }
                })
                .collect::<Vec<_>>();
            assert_eq!(rows.len(), 1, "{} {}", family.name, ty.number);
            let schema = skein_codec::schema::parse(family.current.as_bytes()).unwrap();
            let fingerprint = schema
                .layout(schema.root(ty.root.as_bytes(), ty.variant.map(str::as_bytes)).unwrap(), &held)
                .unwrap()
                .digest();
            assert_eq!(*rows[0].1.layout(), fingerprint, "{} {}", family.name, ty.number);
        }
    }
    let last = store.stores.store.last();
    let switched = disk.request(&mut store, Request::Switch { owner: Token::new(1) });
    assert!({ if let [Event::Switched { .. }] = switched.as_ref() { true } else { false } });
    let mut again = jig_store(&disk, SET);
    let events = disk.open(&mut again);
    assert_opened(&events, last);
    assert!({ if let Event::Opened { pending: false, .. } = events[0] { true } else { false } });
    assert_eq!(disk.bytes(b"active").as_ref(), b"2\n");
}
#[test]
fn dev_log_held_layouts_refuse_at_the_check_when_task_logs_is_done() {
    let mut disk = Disk::new(&baseline(None, true));
    let before = contents(&mut disk);
    let mut store = jig_store(&disk, SET);
    let events = disk.open(&mut store);
    assert!(
        {
            if let [Event::OpenRefused { why: OpenRefusal::Layout { why: Unplanned::Underived(_), .. }, .. }] =
                events.as_ref()
            {
                true
            } else {
                false
            }
        },
        "{events:?}"
    );
    assert_eq!(contents(&mut disk), before);
    assert_eq!(disk.bytes(b"active").as_ref(), b"1\n");
}
#[test]
fn dev_whole_live_talk_refuses_in_settling_without_switching_active() {
    let schema = skein_codegen::parse(TASKS).unwrap();
    let value = skein_codegen::generate(&schema)
        .goldens
        .into_iter()
        .find(|g| g.name == "enum_value_live_talk_smallest.bin")
        .unwrap()
        .bytes;
    let key = keys::encode(&Key::Tasks(tasks::Key::Live(0)), 1024).unwrap();
    let value = [&[0, 13][..], &value, &[0, 0, 0, 0]].concat();
    let mut disk = Disk::new(&baseline(Some((&key, &value)), false));
    let mut store = jig_store(&disk, SET);
    let events = open(&mut disk, &mut store);
    let Event::OpenRefused { why: OpenRefusal::Job { owner: Owner::Jig, job, key: Some(found), why }, .. } =
        events.last().unwrap()
    else {
        panic!("{events:?}")
    };
    assert_eq!(job.as_ref(), "settle-layouts");
    assert_eq!(found, &key);
    assert_eq!(why.as_ref(), "holds the variant `live_talk` of `Value`, which this build no longer has");
    assert_eq!(disk.bytes(b"active").as_ref(), b"1\n");
}
#[test]
fn dev_retired_milestone_refuses_before_any_write() {
    let key = [&[1, 1, 10][..], &0_u64.to_be_bytes(), &0_u64.to_be_bytes()].concat();
    let schema = skein_codegen::parse(TASKS).unwrap();
    let value = skein_codegen::generate(&schema)
        .goldens
        .into_iter()
        .find(|g| g.name == "enum_value_milestone_smallest.bin")
        .unwrap()
        .bytes;
    let value = [&[0, 13][..], &value, &[0, 0, 0, 0]].concat();
    let mut disk = Disk::new(&baseline(Some((&key, &value)), false));
    let before = contents(&mut disk);
    let mut store = jig_store(&disk, SET);
    let events = disk.open(&mut store);
    assert!(
        {
            if let [
                Event::OpenRefused {
                    why: OpenRefusal::Layout { family, value_type: 10, id: 13, why: Unplanned::Retired, .. },
                    ..
                },
            ] = events.as_ref()
            {
                family.as_ref() == "jig-store-tasks"
            } else {
                false
            }
        },
        "{events:?}"
    );
    assert_eq!(contents(&mut disk), before);
    assert_eq!(disk.bytes(b"active").as_ref(), b"1\n");
}
