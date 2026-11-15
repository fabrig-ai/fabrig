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
    let ops = vec![
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
                    store::Owner::Application => crate::jobs::page(&job, &page),
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
