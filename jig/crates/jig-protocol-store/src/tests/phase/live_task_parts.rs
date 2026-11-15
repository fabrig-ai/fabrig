//! The real jig set splits a store's whole live rows once (protocol/store/parts.md, 4).
use super::*;
use crate::{JobWrite, Owner, PatchSet, PatchSets};
use alloc::collections::VecDeque;
use jig_domain_core_vocabulary::Record;
use jig_domain_tasks::{self as tasks, Stored};

fn store(disk: &Disk, set: PatchSet) -> Store {
    let mut limits = fixtures::store_limits();
    limits.store.budget = 2_097_152;
    limits.store.ops = 128;
    limits.commits.writes = 3;
    let mut store = disk.store_with_limits(&limits);
    store.patch_sets(PatchSets { sets: Box::leak(Box::new([crate::SKEIN_PATCH_SET, crate::SMITH_PATCH_SET, set])) });
    store
}
fn row(number: u64) -> tasks::TaskRecord {
    let mut row = fixtures::task_record();
    row.number = number;
    row.proposal = None;
    row.recurring = None;
    row.instructions = Some(Box::from(b"instructions".as_slice()));
    if number != 8 {
        row.talk = Some(tasks::Talk {
            party: 31,
            opened: skein_lib::Wall::from_nanos(41),
            last: skein_lib::Wall::from_nanos(43),
        });
    }
    row
}
fn key(record: &Record) -> Box<[u8]> {
    crate::keys::encode(&record.key(), fixtures::KEY_LIMIT).unwrap()
}
fn whole(record: &tasks::TaskRecord) -> Box<[u8]> {
    let limits = &crate::tasks_v13::CEILINGS;
    let wire = crate::codec::tasks::record::task_record(record, limits, None).unwrap();
    let value = if record.number == 8 {
        crate::tasks_v13::Value::Live(wire.into_parts().record)
    } else {
        crate::tasks_v13::Value::LiveTalk(wire)
    };
    let row = crate::tasks_v13::Row::new(
        limits,
        crate::tasks_v13::RowParts { value, models: skein_lib::List::with_capacity(limits.row_models) },
    )
    .unwrap();
    let mut writer = skein_lib::Writer::new(usize::try_from(row.measure()).unwrap());
    row.encode(&mut writer).unwrap();
    writer.finish()
}
fn before() -> PatchSet {
    let set = crate::JIG_STORE_PATCH_SET;
    let families = set
        .families
        .iter()
        .map(|family| {
            if family.name != "jig-store-tasks" {
                return *family;
            }
            let types = family
                .types
                .iter()
                .filter(|ty| ty.number < 19)
                .map(|ty| {
                    if ty.number == 18 {
                        crate::ValueType { keys: &[crate::KeyRange { from: &[1, 1, 21], to: &[1, 2] }], ..*ty }
                    } else {
                        *ty
                    }
                })
                .collect::<Vec<_>>();
            crate::KeptFamily {
                current: family.texts.last().unwrap(),
                types: Box::leak(types.into_boxed_slice()),
                ..*family
            }
        })
        .collect::<Vec<_>>();
    let jobs = set.jobs.iter().filter(|job| job.name != "live-task-parts").copied().collect::<Vec<_>>();
    PatchSet { families: Box::leak(families.into_boxed_slice()), jobs: Box::leak(jobs.into_boxed_slice()), ..set }
}

#[test]
#[expect(clippy::too_many_lines, reason = "one real-set upgrade, both commit edges, and its no-op reopen")]
fn whole_live_tasks_open_in_parts_in_bounded_pages_and_never_run_the_job_again() {
    let mut disk = Disk::new(&[]);
    let mut old = store(&disk, before());
    assert_opened(&disk.open(&mut old), 0);
    let committed = disk.request(&mut old, fixtures::commit(1, Box::new([fixtures::header(1)])));
    assert!(match committed.as_ref() {
        [Event::Committed { .. }] => true,
        _ => false,
    });
    let records: Vec<_> =
        [7, 8, 9].into_iter().map(row).map(|row| Record::Tasks(Stored::Live(Box::new(row)))).collect();
    let ended = fixtures::ended_task(20);
    let ended_bytes = crate::codec::encode(&ended, &old.codec_limits, &[]).unwrap();
    let mut ops: Vec<_> = records
        .iter()
        .map(|record| {
            let Record::Tasks(Stored::Live(row)) = record else { unreachable!() };
            skein_kv::Op::Put { key: key(record), value: whole(row) }
        })
        .collect();
    ops.push(skein_kv::Op::Put { key: key(&ended), value: ended_bytes.clone() });
    registry::raw(&mut disk, &mut old, ops);
    assert_eq!(jobs::done(&mut old, Owner::Jig, "live-task-parts"), None);
    let mut disk = Disk::new(&disk.items());
    let mut current = store(&disk, crate::JIG_STORE_PATCH_SET);
    let mut events = VecDeque::from(disk.open(&mut current).into_vec());
    let mut pages = 0_u32;
    let mut ran = 0_u32;
    let mut last = None;
    while let Some(event) = events.pop_front() {
        match event {
            Event::Job { owner, of: Owner::Jig, job, page } => {
                assert_eq!(job.as_ref(), "live-task-parts");
                let answer = crate::jobs::page(&job, &page).unwrap();
                assert_eq!(answer.writes.len(), if page.end { 0 } else { 2 });
                let bytes: u32 = answer
                    .writes
                    .iter()
                    .map(|write| {
                        let JobWrite::Put { key, value } = write else { unreachable!() };
                        skein_kv::put_bytes(u32::try_from(key.len()).unwrap(), u32::try_from(value.len()).unwrap())
                            .unwrap()
                    })
                    .sum();
                assert!(u64::from(bytes) <= page.room.bytes);
                assert!(u32::try_from(answer.writes.len()).unwrap() <= page.room.writes);
                if page.rows.len() > 1 {
                    assert!(answer.through.is_some());
                    let tight = crate::JobPage {
                        rows: page
                            .rows
                            .iter()
                            .map(|row| crate::JobRow { key: row.key.clone(), value: row.value.clone() })
                            .collect(),
                        end: false,
                        layout: None,
                        room: crate::JobRoom { writes: 128, bytes: u64::from(bytes) },
                    };
                    let tight_answer = crate::jobs::page(&job, &tight).unwrap();
                    assert_eq!(tight_answer.writes.len(), 2, "byte edge consumes one whole row");
                    assert_eq!(tight_answer.through.as_ref(), page.rows.first().map(|row| &row.key));
                }
                pages += 1;
                events.extend(disk.request(&mut current, Request::Job { owner, answer: Ok(answer) }));
            }
            Event::Ran { of: Owner::Jig, job, .. } => {
                assert_eq!(job.as_ref(), "live-task-parts");
                ran += 1;
            }
            Event::Opened { last: number, pending: true, .. } => last = Some(number),
            Event::Census { .. } => {}
            other => panic!("open: {other:?}"),
        }
    }
    assert_eq!((ran, pages), (1, 4));
    assert!(jobs::done(&mut current, Owner::Jig, "live-task-parts").is_some());
    for expected in &records {
        let events = disk.request(&mut current, Request::Get { owner: Token::new(9), key: Get::Jig(expected.key()) });
        let [Event::Got { row: Some(Row::Jig(found)), .. }] = events.as_ref() else { panic!("read: {events:?}") };
        assert_eq!(found, expected);
        let bytes = crate::open::get(&mut current, Token::new(0), key(expected)).unwrap().unwrap();
        let row =
            crate::tasks_v13::Row::decode(&crate::tasks_v13::CEILINGS, &mut skein_lib::Reader::new(&bytes)).unwrap();
        assert!(match row.into_parts().value {
            crate::tasks_v13::Value::LiveHot(_) => true,
            _ => false,
        });
        let page = crate::JobPage {
            rows: Box::new([crate::JobRow { key: key(expected), value: bytes }]),
            end: false,
            layout: None,
            room: crate::JobRoom { writes: 0, bytes: 0 },
        };
        assert!(crate::jobs::page("live-task-parts", &page).unwrap().writes.is_empty(), "hot rows pass without writes");
    }
    assert_eq!(fixtures::get_kv(&mut current, &key(&ended)), Some(ended_bytes));
    let switched = disk.request(&mut current, Request::Switch { owner: Token::new(1) });
    assert!(match switched.as_ref() {
        [Event::Switched { .. }] => true,
        _ => false,
    });
    let mut again = store(&disk, crate::JIG_STORE_PATCH_SET);
    let opened = disk.open(&mut again);
    assert_opened(&opened, last.unwrap());
    assert!(match opened.first() {
        Some(Event::Opened { pending: false, .. }) => true,
        _ => false,
    });
}
