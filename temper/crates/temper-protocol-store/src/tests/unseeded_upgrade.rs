//! Landed patches run before a real store's registry seed (jig protocol/store/layouts.md, 5.1).
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::wildcard_enum_match_arm,
    reason = "ordinary Rust disk fixtures"
)]
use jig_domain_core_vocabulary::{CoreRecord, Deployment, Record};
use jig_protocol_store::{self as store, Event, Get, KeptFamily, PatchSet, PatchSets, Request, Row, Store, Write};
use skein_lib::{Reader, Token};
#[path = "upgrade_disk.rs"]
mod disk;
use disk::Disk;

const SETS: PatchSets = PatchSets {
    sets: &[store::SKEIN_PATCH_SET, store::SMITH_PATCH_SET, jig_protocol_accounts::JIG_PATCH_SET, crate::PATCH_SET],
};
fn old_sets() -> PatchSets {
    PatchSets {
        sets: Box::leak(
            SETS.sets
                .iter()
                .map(|set| {
                    let patches = match set.name {
                        "jig" => &set.patches[..5],
                        "temper" => &set.patches[..12],
                        _ => set.patches,
                    };
                    let families = Box::leak(
                        set.families
                            .iter()
                            .map(|f| {
                                let texts = match f.name {
                                    "jig-store-tasks" => f.texts.split_last().unwrap().1,
                                    "temper-store" => &f.texts[..13], // v11 through the source fixture's v23.
                                    _ => f.texts,
                                };
                                KeptFamily { texts, current: texts.last().unwrap(), types: &[], ..*f }
                            })
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                    );
                    PatchSet { patches, families, jobs: &[], ..*set }
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        ),
    }
}
fn component(disk: &Disk, sets: PatchSets) -> Store {
    let mut app = crate::application(&super::fixtures::LIMITS, disk.limits().core.tasks.spec_bytes).unwrap();
    app.version = if sets.sets.last().unwrap().patches.len() == 12 { 23 } else { crate::STORE_VERSION };
    let mut store = Store::on_directory(&disk.limits(), app, &[], disk.seeded());
    store.patch_sets(sets);
    store
}
fn header(number: u64) -> Record {
    Record::Core(CoreRecord::Deployment(Deployment {
        id: [7; 16],
        tasks: 1,
        people: 0,
        sign_ins: 0,
        messages: 0,
        runs: 0,
        calls: 0,
        connector_rows: 1,
        projects: 0,
        applications: 0,
        commits: number,
    }))
}
fn golden(text: &str, name: &str) -> Box<[u8]> {
    let schema = skein_codegen::parse(text).unwrap();
    skein_codegen::generate(&schema).goldens.into_iter().find(|g| g.name == name).unwrap().bytes.into()
}
fn put(key: Box<[u8]>, value: Box<[u8]>) -> skein_kv::Op {
    skein_kv::Op::Put { key, value }
}
fn kv_get(disk: &mut Disk, map: &mut skein_kv::Store, key: Box<[u8]>) -> Option<Box<[u8]>> {
    let events = disk.kv(map, skein_kv::Request::Get { owner: Token::new(1), key });
    let [skein_kv::Event::Got { value, .. }] = events.as_slice() else { panic!("get: {events:?}") };
    value.clone()
}
#[test]
#[expect(clippy::too_many_lines, reason = "one open story checks both landed patches, readback and the durable seed")]
fn unseeded_v23_and_v12_rows_patch_before_the_current_registry_is_seeded() {
    let mut disk = Disk::new();
    let old = old_sets();
    let mut before = component(&disk, old);
    let events = disk.open(&mut before);
    let [Event::Opened { last: 0, .. }, Event::Census { .. }] = events.as_slice() else { panic!("open: {events:?}") };
    let events = disk.request(&mut before, Request::Commit { number: 1, writes: Box::new([Write::Save(header(1))]) });
    let [Event::Committed { number: 1, .. }] = events.as_slice() else { panic!("commit: {events:?}") };
    drop(before);
    disk.restart();

    // Plant the previous binary's encodings without passing them through today's codecs.
    let mut map = disk.map(1);
    let mut format = store::format::current(23, [7; 16]);
    format.tasks = 12;
    format.levels =
        store::codec::component::Levels { skein: Some(0), smith: Some(0), jig: Some(5), application: Some(12) };
    // The smallest golden change names task zero.
    let change_key = crate::keys::encode(&temper_engine_domain_forge_vocabulary::Key::Change(0), 1024).unwrap();
    let tagged_change = store::keys::application(&change_key, disk.limits().store.key).unwrap();
    let task_key = store::keys::encode(
        &jig_domain_core_vocabulary::Key::Tasks(jig_domain_tasks::Key::Live(0)),
        disk.limits().store.key,
    )
    .unwrap();
    let change_text = old.sets.last().unwrap().families.iter().find(|f| f.name == "temper-store").unwrap().current;
    let tasks_text = old.sets[2].families.iter().find(|f| f.name == "jig-store-tasks").unwrap().current;
    let change = [&[0, 23, 12][..], &golden(change_text, "record_change_row_smallest.bin")].concat().into_boxed_slice();
    let task = [&[0, 12, 9][..], &golden(tasks_text, "record_before_talk_smallest.bin"), &[0, 0, 0, 0]]
        .concat()
        .into_boxed_slice();
    let codec = store::codec::limits(&disk.limits().core).unwrap();
    // The smallest change names repository 0/0; keep its owner in the fixture too.
    let repository_key = crate::keys::encode(
        &temper_engine_domain_forge_vocabulary::Key::Repository(skein_forge::api::Repository {
            forge: 0,
            repository: 0,
        }),
        1024,
    )
    .unwrap();
    let repository =
        [&[0, 23, 2][..], &golden(change_text, "record_repository_smallest.bin")].concat().into_boxed_slice();
    let ops = vec![
        put(store::keys::application(&repository_key, 1024).unwrap(), repository),
        put(store::keys::encode_own(&store::keys::Own::Format, 1024).unwrap(), format.encode().unwrap()),
        put(
            store::keys::encode(&header(2).key(), 1024).unwrap(),
            store::codec::encode(&header(2), &codec, &[]).unwrap(),
        ),
        put(tagged_change.clone(), change),
        put(task_key.clone(), task),
    ];
    let events = disk.kv(&mut map, skein_kv::Request::Commit { owner: Token::new(1), ops: ops.into() });
    let [skein_kv::Event::Committed { number: 2, .. }] = events.as_slice() else { panic!("plant: {events:?}") };
    let layout_key = |family: &str, number, id| {
        store::keys::encode_own(&store::keys::Own::Layout { family: family.into(), number, id }, 1024).unwrap()
    };
    assert!(kv_get(&mut disk, &mut map, layout_key("temper-store", 12, 25)).is_none());
    assert!(kv_get(&mut disk, &mut map, layout_key("jig-store-tasks", 0, 13)).is_none());
    disk.kv(&mut map, skein_kv::Request::Close { owner: Token::new(1) });

    drop(map);
    disk.restart();
    let mut current = component(&disk, SETS);
    let mut events = alloc::collections::VecDeque::from(disk.open(&mut current));
    let mut patched = Vec::new();
    let mut opened = false;
    while let Some(event) = events.pop_front() {
        match event {
            Event::Patched { of, patch, .. } => patched.push((of, patch)),
            Event::Job { owner, of, job, page } => {
                let answer = match of {
                    store::Owner::Application => {
                        let bounds =
                            crate::bounds(&super::fixtures::LIMITS, disk.limits().core.tasks.spec_bytes).unwrap();
                        let present = crate::jobs::read_present(&mut current, &bounds).unwrap();
                        crate::jobs::page(&job, &page, &present, &bounds)
                    }
                    store::Owner::Jig => store::jobs::page(&job, &page),
                    store::Owner::Skein | store::Owner::Smith => panic!("no upstream jobs"),
                };
                events.extend(disk.request(&mut current, Request::Job { owner, answer }));
            }
            Event::Opened { pending: true, .. } => opened = true,
            Event::Census { .. } | Event::Ran { .. } => {}
            other => panic!("upgrade: {other:?}; {:?}", current.start_open_failure()),
        }
    }
    assert!(opened);
    assert_eq!(patched, [(store::Owner::Application, 13), (store::Owner::Jig, 6), (store::Owner::Application, 14)]);
    let events = disk.request(&mut current, Request::Get { owner: Token::new(1), key: Get::Application(change_key) });
    let [Event::Got { row: None, .. }] = events.as_slice() else { panic!("legacy change erased: {events:?}") };
    let hot_key = crate::keys::encode(&temper_engine_domain_forge_vocabulary::Key::ChangeHot(0), 1024).unwrap();
    let tagged_hot = store::keys::application(&hot_key, disk.limits().store.key).unwrap();
    let events = disk.request(&mut current, Request::Get { owner: Token::new(1), key: Get::Application(hot_key) });
    let [Event::Got { row: Some(Row::Application { value, .. }), .. }] = events.as_slice() else {
        panic!("change: {events:?}")
    };
    let change = crate::v25::Row::decode(&crate::v25::CEILINGS, &mut Reader::new(value)).unwrap();
    let crate::v25::Value::ChangeHot(change) = change.into_parts().value else { panic!("change row") };
    assert_eq!(change.task_release(), &None);
    assert!(!change.change().released_resolution());
    assert_eq!(change.change().last_checked(), &None);
    let events = disk.request(
        &mut current,
        Request::Get {
            owner: Token::new(1),
            key: Get::Jig(jig_domain_core_vocabulary::Key::Tasks(jig_domain_tasks::Key::Live(0))),
        },
    );
    let [Event::Got { row: Some(Row::Jig(Record::Tasks(jig_domain_tasks::Stored::Live(task)))), .. }] =
        events.as_slice()
    else {
        panic!("task: {events:?}")
    };
    assert_eq!(task.question_woken, 0);
    let switched = disk.request(&mut current, Request::Switch { owner: Token::new(1) });
    let [Event::Switched { .. }] = switched.as_slice() else { panic!("switch: {switched:?}") };
    drop(current);
    disk.restart();

    let mut map = disk.map(2);
    let cold_key =
        store::keys::encode(&jig_domain_core_vocabulary::Key::Tasks(jig_domain_tasks::Key::LiveCold(0)), 64).unwrap();
    for (family, number, id, key, binary) in [
        ("temper-store", 48, 25, Some(tagged_hot), true),
        ("jig-store-tasks", 0, 14, Some(task_key), true),
        ("jig-store-tasks", 21, 13, Some(cold_key), true),
    ] {
        let bytes = kv_get(&mut disk, &mut map, layout_key(family, number, id)).unwrap();
        let layout =
            store::registry_v1::Layout::decode(&store::registry_v1::CEILINGS, &mut Reader::new(&bytes)).unwrap();
        let family = SETS.sets.iter().flat_map(|s| s.families).find(|f| f.name == family).unwrap();
        let expected_text = if binary { family.current } else { family.texts.last().unwrap() };
        let schema = skein_codec::schema::parse(expected_text.as_bytes()).unwrap();
        let ty = family.types.iter().find(|t| t.number == number).unwrap();
        let expected = schema
            .layout(schema.root(ty.root.as_bytes(), ty.variant.map(str::as_bytes)).unwrap(), &[])
            .unwrap()
            .digest();
        assert_eq!(*layout.layout(), expected);
        let text_key = store::keys::encode_own(&store::keys::Own::Text(*layout.text()), 1024).unwrap();
        let text = kv_get(&mut disk, &mut map, text_key).unwrap();
        let text = store::registry_v1::Text::decode(&store::registry_v1::CEILINGS, &mut Reader::new(&text)).unwrap();
        assert_eq!(text.text(), expected_text.as_bytes());
        if let Some(key) = key {
            assert_eq!(&kv_get(&mut disk, &mut map, key).unwrap()[..2], &id.to_be_bytes());
        }
    }
}

fn without_deleted_repository_job() -> PatchSets {
    PatchSets {
        sets: Box::leak(
            SETS.sets
                .iter()
                .map(
                    |set| {
                        if set.name == "temper" { PatchSet { jobs: &crate::PATCH_SET.jobs[..1], ..*set } } else { *set }
                    },
                )
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        ),
    }
}

fn pull(repository: skein_forge::api::Repository, number: u64) -> temper_engine_domain_forge_vocabulary::Stored {
    use temper_engine_domain_forge_vocabulary::{Name, PullState, Stored, What};
    Stored::PullState(PullState {
        name: Name { forge: repository.forge, repository: repository.repository, what: What::Pull(number) },
        head: [1; 32],
        ci: skein_forge::api::Ci::Passed,
        state: skein_forge::api::State::Closed,
    })
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one open story checks pages, stored facts, readback and the durable job marker"
)]
fn deleted_repository_job_runs_over_directory_pages_once_and_preserves_present_and_ended_rows() {
    use super::jobs::{A, B, JOB, application, encoded, names};
    use skein_forge::api::Repository;
    use temper_engine_domain_forge_vocabulary as d;
    let mut disk = Disk::new();
    let mut before = component(&disk, without_deleted_repository_job());
    let events = disk.open(&mut before);
    let [Event::Opened { last: 0, .. }, Event::Census { .. }] = events.as_slice() else {
        panic!("first open: {events:?}")
    };
    let orphans = disk.limits().store.ops + 3;
    let mut rows: Vec<_> = (0..u64::from(orphans)).map(|n| pull(B, n)).collect();
    rows.extend([application(B), names(1, &[A, B])]);
    let erased = rows[..usize::try_from(orphans + 1).unwrap()].to_vec();
    let d::Stored::Repository(mut repository) = super::fixtures::adopted(d::Protection::Absent) else {
        panic!("repository")
    };
    repository.provider = A;
    let mut kept_forge = super::fixtures::forge_record();
    kept_forge.number = 1;
    let mut ended_forge = kept_forge.clone();
    ended_forge.number = 2;
    ended_forge.state = d::ForgeState::Ended { secrets_erased: true, box_removed: true };
    let mut ended_application = application(Repository { forge: 3, ..B });
    let d::Stored::Application { token, .. } = &mut ended_application else { panic!("application") };
    *token = 2;
    let kept = [
        d::Stored::Repository(repository),
        d::Stored::Forge(kept_forge),
        d::Stored::Forge(ended_forge),
        d::Stored::ForgeNumber { last: 3 },
        pull(A, 1),
        pull(A, 2),
        pull(Repository { forge: 2, ..B }, 1),
        ended_application,
    ];
    rows.extend(kept.clone());
    // Seed the registry and the old set's job markers before the fixture rows.
    let events = disk.request(&mut before, Request::Commit { number: 1, writes: Box::new([Write::Save(header(1))]) });
    let [Event::Committed { .. }] = events.as_slice() else { panic!("seed: {events:?}") };
    let mut number = 1;
    for chunk in rows.chunks(usize::try_from(disk.limits().store.ops / 2).unwrap()) {
        number += 1;
        let mut writes = vec![Write::Save(header(number))];
        for row in chunk {
            let row = encoded(row);
            writes.push(Write::Put { key: row.key, value: row.value });
        }
        let events = disk.request(&mut before, Request::Commit { number, writes: writes.into() });
        let [Event::Committed { .. }] = events.as_slice() else { panic!("plant: {events:?}") };
    }
    drop(before);
    disk.restart();
    let mut current = component(&disk, SETS);
    let bounds = crate::bounds(&super::fixtures::LIMITS, disk.limits().core.tasks.spec_bytes).unwrap();
    let mut events = alloc::collections::VecDeque::from(disk.open(&mut current));
    let mut pages = 0_u32;
    let mut ran = 0_u32;
    let mut opened = false;
    while let Some(event) = events.pop_front() {
        match event {
            Event::Job { owner, of: store::Owner::Application, job, page } => {
                assert_eq!(&*job, JOB);
                pages += 1;
                assert!(page.rows.len() <= usize::try_from(disk.limits().store.ops).unwrap());
                let present = crate::jobs::read_present(&mut current, &bounds).unwrap();
                assert_eq!(present.repositories.as_ref(), [A]);
                assert_eq!(present.forges.as_ref(), [(1, false), (2, true)]);
                assert_eq!(present.last, 3);
                let answer = crate::jobs::page(&job, &page, &present, &bounds);
                events.extend(disk.request(&mut current, Request::Job { owner, answer }));
            }
            Event::Ran { of: store::Owner::Application, job, counts, .. } => {
                assert_eq!(&*job, JOB);
                ran += 1;
                let names: Vec<_> = counts.iter().map(|c| &*c.name).collect();
                assert_eq!(names, crate::PATCH_SET.jobs[1].counts);
                let counts: Vec<_> = counts.iter().map(|c| c.count).collect();
                assert_eq!(counts, [u64::from(orphans + 2), 0, u64::from(orphans), 0, 0, 0, 0, 0, 0, 1, 1]);
            }
            Event::Opened { pending: true, .. } => opened = true,
            Event::Census { .. } => {}
            other => panic!("open: {other:?}; {:?}", current.start_open_failure()),
        }
    }
    assert!(opened);
    assert!(pages > 2, "more than one data page, then the empty end page");
    assert_eq!(ran, 1);
    for row in erased {
        let events =
            disk.request(&mut current, Request::Get { owner: Token::new(1), key: Get::Application(encoded(&row).key) });
        let [Event::Got { row: None, .. }] = events.as_slice() else { panic!("orphan gone: {events:?}") };
    }
    for row in kept.into_iter().chain([names(1, &[A])]) {
        let row = encoded(&row);
        let events =
            disk.request(&mut current, Request::Get { owner: Token::new(1), key: Get::Application(row.key.clone()) });
        let [Event::Got { row: Some(Row::Application { key, value }), .. }] = events.as_slice() else {
            panic!("kept: {events:?}")
        };
        assert_eq!((key, value), (&row.key, &row.value));
    }
    let events = disk.request(&mut current, Request::Switch { owner: Token::new(1) });
    let [Event::Switched { .. }] = events.as_slice() else { panic!("switch: {events:?}") };
    drop(current);
    disk.restart();
    let mut again = component(&disk, SETS);
    let events = disk.open(&mut again);
    let [Event::Opened { .. }, Event::Census { .. }] = events.as_slice() else {
        panic!("second open has no Job or Ran: {events:?}")
    };
}

#[test]
fn deleted_repository_job_refuses_unreadable_stored_facts_by_key() {
    use super::jobs::{B, JOB, encoded};
    use temper_engine_domain_forge_vocabulary as d;
    let repository = encoded(&super::fixtures::adopted(d::Protection::Absent));
    let mut forge = super::fixtures::forge_record();
    forge.number = 2;
    let forge = encoded(&d::Stored::Forge(forge));
    let mut number = encoded(&d::Stored::ForgeNumber { last: 2 });
    number.value = [&number.value[..], &[0]].concat().into_boxed_slice();
    for (family, bad) in [
        ("Repository", store::JobRow { key: Box::new([2]), value: repository.value }),
        ("Forge", store::JobRow { key: crate::keys::encode(&d::Key::Forge(1), 1024).unwrap(), value: forge.value }),
        ("ForgeNumber", number),
    ] {
        let mut disk = Disk::new();
        let mut before = component(&disk, without_deleted_repository_job());
        let events = disk.open(&mut before);
        let [Event::Opened { last: 0, .. }, Event::Census { .. }] = events.as_slice() else {
            panic!("open: {events:?}")
        };
        let events =
            disk.request(&mut before, Request::Commit { number: 1, writes: Box::new([Write::Save(header(1))]) });
        let [Event::Committed { .. }] = events.as_slice() else { panic!("seed: {events:?}") };
        let orphan = encoded(&pull(B, 1));
        let events = disk.request(
            &mut before,
            Request::Commit {
                number: 2,
                writes: Box::new([
                    Write::Save(header(2)),
                    Write::Put { key: bad.key.clone(), value: bad.value },
                    Write::Put { key: orphan.key, value: orphan.value },
                ]),
            },
        );
        let [Event::Committed { .. }] = events.as_slice() else { panic!("plant: {events:?}") };
        drop(before);
        disk.restart();
        let mut current = component(&disk, SETS);
        let events = disk.open(&mut current);
        let [Event::Job { owner, of: store::Owner::Application, job, .. }] = events.as_slice() else {
            panic!("job: {events:?}")
        };
        assert_eq!(&**job, JOB);
        let bounds = crate::bounds(&super::fixtures::LIMITS, disk.limits().core.tasks.spec_bytes).unwrap();
        let why = crate::jobs::read_present(&mut current, &bounds).unwrap_err();
        assert_eq!(why.key.as_ref(), Some(&bad.key), "{family}");
        assert_eq!(&*why.why, format!("unreadable {family} row"));
        let events = disk.request(&mut current, Request::Job { owner: *owner, answer: Err(why) });
        let [Event::OpenRefused { why: store::OpenRefusal::Job { owner: of, job, key, .. }, .. }] = events.as_slice()
        else {
            panic!("refusal: {events:?}")
        };
        assert_eq!(of, &store::Owner::Application);
        assert_eq!(&**job, JOB);
        assert_eq!(key.as_ref(), Some(&bad.key));
    }
}
