//! Dev's build's layouts and task logs through a writing open (protocol/store/layouts.md, 8).
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "ordinary Rust disk step fixtures inspect complete registry and read answers"
)]
use super::{
    fixtures::{self, ALICE, disk::Disk},
    history_logs::append_operations,
};
use crate::{Event, Get, OwnerSets, Request, Restore, Row, Store, codec, keys, logs};
use jig_domain_core_vocabulary::{self as core, Record, RestoreRange};
use jig_domain_tasks as tasks;
use skein_codec::{
    Record as _,
    layout::{Mode, Plan, Workspace},
    schema,
    value::Value,
};
use skein_lib::{Buffer, Queue, Reader, Token};

// Verbatim from 13efa774615022299aea9ed57b2577df517494ad, under
// jig/crates/jig-protocol-store/schema/ and smith/crates/smith-transcript/schema/.
pub(super) const CORE_SCHEMA: &str = include_str!("fixtures/dev-core-v6.schema");
const TEXTS: [(&str, &str); 10] = [
    ("jig-store-component", include_str!("fixtures/dev-component-v3.schema")),
    ("jig-store-openings", include_str!("fixtures/dev-openings-v1.schema")),
    ("jig-store-registry", include_str!("fixtures/dev-registry-v1.schema")),
    ("jig-store-core", CORE_SCHEMA),
    ("jig-store-tasks", include_str!("fixtures/dev-tasks-v13.schema")),
    ("jig-store-people", include_str!("fixtures/dev-people-v17.schema")),
    ("jig-store-notes", include_str!("fixtures/dev-notes-v1.schema")),
    ("jig-store-accounts", include_str!("fixtures/dev-accounts-v2.schema")),
    ("jig-store-settings", include_str!("fixtures/dev-settings-v3.schema")),
    ("jig-store-secrets", include_str!("fixtures/dev-secrets-v1.schema")),
];
const SMITH: &str = include_str!("fixtures/dev-smith-transcript-v7.schema");
const TASKS: [u64; 2] = [7, 4097];
const PAGE: u32 = 65_536;
const OWNER: Token = Token::new(71);
const CURRENT: OwnerSets = OwnerSets { sets: &[crate::SKEIN_SET, crate::SMITH_SET, crate::JIG_STORE_SET] };

fn old_sets() -> OwnerSets {
    let families = crate::STORE_FAMILIES.map(|family| crate::KeptFamily {
        current: TEXTS.iter().find(|(name, _)| *name == family.name).unwrap().1,
        // Dev's core predates parts: keep only its registered types and key range.
        types: if family.name == "jig-store-core" { &family.types[..8] } else { family.types },
        keys: match family.name {
            "jig-store-core" => &family.keys[..1],
            // The kept registry text does not declare Order.
            "jig-store-registry" => &family.keys[..3],
            _ => family.keys,
        },
        frames: None, // Dev has the map registry but no frame rows.
        ..family
    });
    let smith = crate::KeptFamily { current: SMITH, frames: None, ..crate::SMITH_SET.families[0] };
    OwnerSets {
        sets: Box::leak(Box::new([
            crate::SKEIN_SET,
            crate::OwnerSet { families: Box::leak(Box::new([smith])), level: Some(0), ..crate::SMITH_SET },
            crate::OwnerSet { families: Box::leak(Box::new(families)), ..crate::JIG_STORE_SET },
        ])),
    }
}
fn store(disk: &Disk, sets: OwnerSets) -> Store {
    let mut limits = fixtures::registered_store_limits();
    limits.core.tasks.message_bytes = PAGE;
    limits.store.page.bytes = PAGE;
    limits.store.record = PAGE;
    let mut store = disk.store_with_limits(&limits);
    store.owner_sets(sets);
    store
}
fn opened(events: &[Event], pending: bool) -> u64 {
    let [Event::Opened { last, pending: found, .. }, Event::Census { .. }] = events else {
        panic!("open at the binary's sets: {events:?}")
    };
    assert_eq!(*found, pending);
    *last
}
fn key(row: &Record) -> Box<[u8]> {
    keys::encode(&row.key(), fixtures::KEY_LIMIT).unwrap()
}
fn generic(text: &str, root: &str, bytes: &[u8]) {
    let schema = schema::parse(text.as_bytes()).unwrap();
    let mut value = Value::with_room(8192);
    value.decode(&schema, schema.root(root.as_bytes(), None).unwrap().decl, bytes.into()).unwrap();
}
/// Write a binary-encoded value at dev's layout by dropping only what dev's
/// texts do not hold.
fn at_dev(family: &crate::KeptFamily, ty: &crate::ValueType, value: &[u8]) -> Box<[u8]> {
    let parse = |text: &str| schema::parse(text.as_bytes()).unwrap();
    let dev = |name: &str| TEXTS.iter().find(|(held, _)| *held == name).unwrap().1;
    let binary = parse(family.current);
    let old = parse(dev(family.name));
    let held = crate::STORE_FAMILIES.iter().filter(|held| family.current.contains(&format!("holds {} ", held.name)));
    let binary_held: Vec<_> = held.clone().map(|held| parse(held.current)).collect();
    let old_held: Vec<_> = held.map(|held| parse(dev(held.name))).collect();
    let variant = ty.variant.map(str::as_bytes);
    let plan = Plan::between(
        Mode::Read,
        &binary,
        binary.root(ty.root.as_bytes(), variant).unwrap(),
        &binary_held.iter().collect::<Vec<_>>(),
        &old,
        old.root(ty.root.as_bytes(), variant).unwrap(),
        &old_held.iter().collect::<Vec<_>>(),
    )
    .expect("the binary's value drops to dev's layout");
    let mut work = Workspace::new();
    assert!(work.fit(&plan));
    let mut out = Buffer::with_capacity(1_048_576);
    plan.convert(value, &mut out, &mut work).unwrap();
    out.as_slice().into()
}
fn encoded(store: &Store, row: &Record) -> logs::record::Row {
    let key = key(row);
    let family = crate::STORE_FAMILIES
        .iter()
        .find(|family| {
            family
                .types
                .iter()
                .any(|ty| ty.keys.iter().any(|range| key.as_ref() >= range.from && key.as_ref() < range.to))
        })
        .unwrap();
    let text = TEXTS.iter().find(|(name, _)| *name == family.name).unwrap().1;
    let ty = family
        .types
        .iter()
        .find(|ty| ty.keys.iter().any(|range| key.as_ref() >= range.from && key.as_ref() < range.to))
        .unwrap();
    let value = at_dev(family, ty, &codec::encode(row, &store.codec_limits, &store.names).unwrap());
    // These values use only dev's variants, and the binary's added fields at
    // their defaults. Check the bytes against the kept schema.
    generic(text, ty.root, &value);
    assert_eq!(crate::registry::head(&value), Some(schema::parse(text.as_bytes()).unwrap().version()));
    (key, Some(value))
}
fn refused(task: u64, why: tasks::Refusal) -> Record {
    let Record::Core(core::CoreRecord::Call(mut call)) = fixtures::call(task, 1) else { panic!("call fixture") };
    call.part = core::CallPart::DelegationRefused(tasks::Problem::new(Some(task), why));
    Record::Core(core::CoreRecord::Call(call))
}
fn records(task: u64) -> Vec<Record> {
    vec![fixtures::ended_task(task), fixtures::terminal(task, 1), fixtures::history(task, 1)]
}
fn activity(store: &Store, task: u64) -> Record {
    let Record::Tasks(tasks::Stored::Activity(mut row)) = fixtures::activity_line(task, 1) else { panic!("activity") };
    let tasks::Line::Words { words, .. } = &mut row.line else { panic!("words") };
    *words = Box::new([]);
    let empty = Record::Tasks(tasks::Stored::Activity(row.clone()));
    let body = logs::record::encode(&[encoded(store, &empty)]).unwrap();
    let size = usize::try_from(PAGE).unwrap().checked_sub(body.len()).unwrap().checked_sub(32).unwrap();
    let tasks::Line::Words { words, .. } = &mut row.line else { panic!("words") };
    let mut rng = skein_lib::Rng::new(task);
    *words = (0..size).map(|_| u8::try_from(rng.below(256)).unwrap()).collect();
    let row = Record::Tasks(tasks::Stored::Activity(row));
    let body = logs::record::encode(&[encoded(store, &row)]).unwrap();
    assert_eq!(body.len(), usize::try_from(PAGE - 32).unwrap());
    let frame = store.stores.store.frame(&body).unwrap();
    assert!(
        frame.len() > u64::from(PAGE - 64) && frame.len() <= u64::from(PAGE),
        "the encoded frame too is near the fetch page"
    );
    row
}
fn raw(disk: &mut Disk, store: &mut Store, mut ops: Vec<skein_kv::Op>) {
    let number = store.stores.store.last().checked_add(1).unwrap();
    let (key, value) = encoded(store, &fixtures::header_record(number));
    ops.push(skein_kv::Op::Put { key, value: value.unwrap() });
    let env = crate::memory_env(store.limits.store);
    let mut above = Queue::uncapped();
    let mut below = fixtures::below();
    skein_kv::down(
        &mut store.stores.store,
        &env,
        skein_kv::Request::Commit { owner: OWNER, ops: ops.into_boxed_slice() },
        &mut above,
        &mut below.store,
    );
    disk.drive(&mut below, |io, below| {
        let crate::Io::Store(arrived) = io else { panic!("raw public-map commit") };
        skein_kv::up(&mut store.stores.store, &env, arrived.event, &mut above, &mut below.store);
    });
    let Some(skein_kv::Event::Committed { number: found, .. }) = above.pop() else { panic!("raw commit: {above:?}") };
    assert_eq!(found, number);
    assert!(above.is_empty());
    store.phase = crate::Phase::Ready { next: number.checked_add(1).unwrap(), format_due: false };
}
fn registry(store: &mut Store) {
    for (family, text) in TEXTS {
        let family = crate::STORE_FAMILIES.iter().find(|held| held.name == family).unwrap();
        let schema = schema::parse(text.as_bytes()).unwrap();
        for ty in family.types {
            if schema.root(ty.root.as_bytes(), ty.variant.map(str::as_bytes)).is_none() {
                continue;
            }
            let id = schema.version();
            let key = keys::encode_own(
                &keys::Own::Layout { family: family.name.into(), number: ty.number, id },
                fixtures::KEY_LIMIT,
            )
            .unwrap();
            let bytes = fixtures::get_kv(store, &key).unwrap();
            let row =
                crate::registry_v1::Layout::decode(&crate::registry_v1::CEILINGS, &mut Reader::new(&bytes)).unwrap();
            assert_eq!(row.root.as_ref(), ty.root.as_bytes());
            assert_eq!(row.variant.as_deref(), ty.variant.map(str::as_bytes));
            let key = keys::encode_own(&keys::Own::Text(row.text), fixtures::KEY_LIMIT).unwrap();
            let bytes = fixtures::get_kv(store, &key).unwrap();
            let held =
                crate::registry_v1::Text::decode(&crate::registry_v1::CEILINGS, &mut Reader::new(&bytes)).unwrap();
            assert_eq!(held.text.as_ref(), text.as_bytes());
            for digest in &row.held {
                let key = keys::encode_own(&keys::Own::Text(*digest), fixtures::KEY_LIMIT).unwrap();
                let bytes = fixtures::get_kv(store, &key).unwrap();
                let held =
                    crate::registry_v1::Text::decode(&crate::registry_v1::CEILINGS, &mut Reader::new(&bytes)).unwrap();
                assert!(
                    TEXTS.iter().any(|(_, text)| held.text.as_ref() == text.as_bytes())
                        || held.text.as_ref() == SMITH.as_bytes()
                );
            }
        }
    }
}
fn restore(disk: &mut Disk, store: &mut Store, range: Restore) -> Vec<Record> {
    let mut found = Vec::new();
    let mut after = None;
    for _ in 0_u32..128 {
        let events = disk.request(
            store,
            Request::Restore { owner: OWNER, range: range.clone(), after, rows: 4, bytes: u64::from(PAGE) },
        );
        let [Event::Page { items, next, .. }] = events.as_ref() else { panic!("restore page: {events:?}") };
        for item in items {
            let crate::restore::Item::Row(Row::Jig(row)) = item else { panic!("joined jig row: {item:?}") };
            found.push(row.clone());
        }
        after = next.clone();
        if after.is_none() {
            return found;
        }
    }
    panic!("bounded restore walk")
}
fn read_back(disk: &mut Disk, store: &mut Store, calls: &[Record], activities: &[Record]) {
    let live = restore(disk, store, Restore::Jig(Box::new(RestoreRange::LIVE)));
    for call in calls {
        assert!(live.contains(call), "refused delegation restored at the binary layout");
    }
    let census = restore(
        disk,
        store,
        Restore::Jig(Box::new([RestoreRange::Census(core::Census {
            questions: Box::new([]),
            results: Box::new([core::ReadResults { party: ALICE, after: 0 }]),
        })])),
    );
    let mut expected = Vec::new();
    for task in TASKS {
        expected.push(Record::People(jig_domain_people::Stored::EndedRead { task, party: ALICE, through: 1 }));
        expected.push(fixtures::ended_task(task));
    }
    expected.extend(TASKS.map(fixtures::ended_task));
    assert_eq!(census, expected, "both ended-read and result joins fetch ended owners from their logs");
    for (task, activity) in TASKS.into_iter().zip(activities) {
        let events = disk
            .request(store, Request::Get { owner: OWNER, key: Get::Jig(core::Key::Tasks(tasks::Key::Ended(task))) });
        let [Event::Got { row: Some(Row::Jig(row)), .. }] = events.as_ref() else { panic!("ended get: {events:?}") };
        assert_eq!(*row, fixtures::ended_task(task));
        let Record::Tasks(tasks::Stored::Ended(kept_task)) = row else { panic!("ended task") };
        assert_eq!((kept_task.suspension, kept_task.suspended, kept_task.restarted), (None, 0, false));
        let mut range = fixtures::read(ALICE, core::reads::Scope::TaskOwnRows { task });
        range.rows = 4;
        // The fetch's encoded page and the caller's owned, decoded rows have
        // separate bounds; leave the caller room for the typed row's overhead.
        range.bytes = u64::from(PAGE).checked_mul(2).unwrap();
        let mut found = Vec::new();
        for _ in 0_u32..32 {
            let events = disk.request(store, Request::Read { owner: OWNER, range: range.clone() });
            let [Event::Read { result: core::reads::Loaded::Page { rows, next }, .. }] = events.as_ref() else {
                panic!("own rows: {events:?}")
            };
            found.extend(rows.iter().map(|row| row.value.clone()));
            range.after = next.clone();
            if range.after.is_none() {
                break;
            }
        }
        assert!(
            range.after.is_none(),
            "own rows cursor {:?}, keys {:?}",
            range.after,
            found.iter().map(Record::key).collect::<Vec<_>>()
        );
        assert_eq!(
            found,
            [
                fixtures::terminal(task, 1),
                fixtures::history(task, 1),
                activity.clone(),
                Record::People(jig_domain_people::Stored::EndedRead { task, party: ALICE, through: 1 })
            ]
        );
    }
}
#[test]
fn dev_layouts_with_refusals_and_sharded_history_open_and_restore_twice() {
    let mut disk = Disk::new(&[]);
    let sets = old_sets();
    let mut old = store(&disk, sets);
    opened(&disk.open(&mut old), false);
    // Use the registry's first-commit writer with the kept texts, rather than
    // duplicating its layout-row and held-text construction in the fixture.
    let committed = disk.request(&mut old, fixtures::commit(1, Box::new([fixtures::header(1)])));
    let [Event::Committed { number: 1, .. }] = committed.as_ref() else { panic!("old registry: {committed:?}") };
    registry(&mut old);
    // The fixture uses today's first-commit writer. Drop the orders it wrote
    // so that the kept generation has the registry dev's build could write.
    let orders = sets
        .sets
        .iter()
        .flat_map(|set| set.families)
        .flat_map(|family| family.types.iter().map(move |ty| (family.name, ty.number)))
        .map(|(family, number)| skein_kv::Op::Erase { key: keys::order(family, number, fixtures::KEY_LIMIT).unwrap() });
    raw(&mut disk, &mut old, orders.collect());
    let calls = [refused(TASKS[0], tasks::Refusal::ResourceUnavailable), refused(TASKS[1], tasks::Refusal::Busy)];
    let mut activities = Vec::new();
    for (task, call) in TASKS.into_iter().zip(&calls) {
        let Record::Core(core::CoreRecord::Call(call_row)) = call else { panic!("refused call") };
        let core::CallPart::DelegationRefused(problem) = &call_row.part else { panic!("refusal") };
        let bytes = codec::tasks::carried::problem(problem, &old.codec_limits.tasks).unwrap();
        generic(TEXTS.iter().find(|(name, _)| *name == "jig-store-tasks").unwrap().1, "TaskProblem", &bytes);
        let mut rows: Vec<_> = records(task).iter().map(|row| encoded(&old, row)).collect();
        rows.sort_by(|left, right| left.0.cmp(&right.0));
        let mut ops = append_operations(&mut old, task, rows.into_boxed_slice(), true).into_vec();
        for row in [
            call.clone(),
            fixtures::result_index(ALICE, task, task),
            Record::People(jig_domain_people::Stored::EndedRead { task, party: ALICE, through: 1 }),
        ] {
            let (key, value) = encoded(&old, &row);
            ops.push(skein_kv::Op::Put { key, value: value.unwrap() });
        }
        raw(&mut disk, &mut old, ops);
        let activity = activity(&old, task);
        let row = encoded(&old, &activity);
        let ops = append_operations(&mut old, task, Box::new([row]), true).into_vec();
        raw(&mut disk, &mut old, ops);
        activities.push(activity);
    }
    let source: Vec<_> = disk.contents().into_iter().filter(|(path, ..)| path.starts_with(b"generations/1/")).collect();
    let mut disk = Disk::new(&disk.items());
    let mut current = store(&disk, CURRENT);
    let last = opened(&disk.open(&mut current), true);
    assert!(last > old.stores.store.last(), "open registers changed binary layouts and seals a new generation");
    read_back(&mut disk, &mut current, &calls, &activities);
    for task in TASKS {
        let path = skein_kv::log_path(logs::history(task).unwrap());
        let source = [b"generations/1/store/".as_slice(), path.as_ref()].concat();
        let target = [b"generations/2/store/".as_slice(), path.as_ref()].concat();
        assert_eq!(disk.bytes(&source), disk.bytes(&target));
        assert_eq!(disk.links(&source), 2, "a new generation links every shard's history file");
    }
    let after: Vec<_> = disk.contents().into_iter().filter(|(path, ..)| path.starts_with(b"generations/1/")).collect();
    for entry in &source {
        assert!(after.contains(entry), "source file unchanged: {:?}", entry.0);
    }
    // KV recovery creates an empty segment at open, before the registry plans
    // decide whether to write in a new generation. No source data changes.
    for (path, kind, _, bytes) in &after {
        if !source.iter().any(|(before, ..)| before == path) {
            assert_eq!(*kind, skein_io::kernel::Kind::File);
            assert!(path.starts_with(b"generations/1/store/log-"));
            assert!(bytes.is_empty());
        }
    }
    let switched = disk.request(&mut current, Request::Switch { owner: OWNER });
    let [Event::Switched { .. }] = switched.as_ref() else { panic!("switch: {switched:?}") };
    assert_eq!(disk.bytes(b"active").as_ref(), b"2\n");
    let mut disk = Disk::new(&disk.items());
    let mut again = store(&disk, CURRENT);
    assert_eq!(opened(&disk.open(&mut again), false), last);
    assert!(!disk.tree().iter().any(|(path, ..)| path.starts_with(b"generations/3")));
    read_back(&mut disk, &mut again, &calls, &activities);
}

#[test]
fn dev_frame_heads_seed_at_core_six_and_smith_seven_and_eight() {
    use crate::transcripts;
    use skein_fake_machine::Item;

    fn smallest(text: &str) -> Box<[u8]> {
        skein_codegen::fixed_values(&skein_codegen::parse(text).expect("kept schema"))
            .into_iter()
            .find(|value| value.name == "record_turn_smallest.bin")
            .expect("smallest turn")
            .bytes
            .into_boxed_slice()
    }

    let written = [smallest(SMITH), smallest(smith_transcript::SCHEMA)];
    let task = TASKS[0];
    let mut disk = Disk::new(&[]);
    let mut old = store(&disk, old_sets());
    opened(&disk.open(&mut old), false);
    let events = disk.request(&mut old, fixtures::commit(1, Box::new([fixtures::header(1)])));
    let [Event::Committed { number: 1, .. }] = &*events else { panic!("first commit: {events:?}") };
    let mut bytes = Vec::new();
    for (index, turn) in written.iter().enumerate() {
        let turn = fixtures::window_turn(task, 1, u32::try_from(index + 1).expect("turn number"), turn);
        bytes.extend_from_slice(&transcripts::frame::encode(&turn, &old.codec_limits).expect("pre-registry frame"));
    }
    let own = |own| keys::encode_own(&own, fixtures::KEY_LIMIT).expect("component key");
    raw(
        &mut disk,
        &mut old,
        vec![
            skein_kv::Op::Put {
                key: own(keys::Own::Transcript { task }),
                value: codec::component::Transcript { length: skein_lib::len_u64(bytes.len()), window: 0 }
                    .encode()
                    .expect("places"),
            },
            skein_kv::Op::Put {
                key: own(keys::Own::Attempt { task, attempt: 1 }),
                value: codec::component::Attempt { first: 0 }.encode().expect("attempt"),
            },
        ],
    );
    let path = [b"transcripts/".as_slice(), &transcripts::path(task)].concat();
    let mut items = disk.items();
    items.push(Item::directory(b"transcripts/0000000000000000").mode(0o700));
    items.push(Item::file(&path, &bytes).mode(0o600));
    let mut disk = Disk::new(&items);
    let mut current = store(&disk, CURRENT);
    opened(&disk.open(&mut current), true);
    let events = disk.request(&mut current, Request::Switch { owner: OWNER });
    let [Event::Switched { .. }] = &*events else { panic!("switch: {events:?}") };
    let core = crate::STORE_FAMILIES.iter().find(|family| family.name == "jig-store-core").expect("core family");
    for (family, number, root, id, text) in [
        (core.name, 9, "TurnRecord", 6, core.current),
        ("smith-transcript", 0, "Turn", 7, SMITH),
        ("smith-transcript", 0, "Turn", 8, smith_transcript::SCHEMA),
    ] {
        let key = own(keys::Own::Layout { family: family.into(), number, id });
        let row = fixtures::get_kv(&mut current, &key).expect("seed row");
        let row = crate::registry_v1::Layout::decode(&crate::registry_v1::CEILINGS, &mut Reader::new(&row)).unwrap();
        let schema = schema::parse(text.as_bytes()).unwrap();
        assert_eq!(row.root.as_ref(), root.as_bytes());
        assert!(row.variant.is_none() && row.held.is_empty());
        assert_eq!(row.layout, schema.layout(schema.root(root.as_bytes(), None).unwrap(), &[]).unwrap().digest());
    }
    let events = disk.request(
        &mut current,
        Request::Transcript {
            owner: OWNER,
            read: crate::TranscriptRead::Page { task, backwards: false, after: None },
            rows: 2,
            bytes: u64::from(PAGE),
        },
    );
    let [Event::Turns { turns, cut: None, next: None, .. }] = events.as_ref() else { panic!("dev turns: {events:?}") };
    assert_eq!(turns.len(), 2);
    assert!(turns.iter().all(|turn| turn.transcript == written[1]));
    assert_eq!(disk.bytes(&path).as_ref(), bytes);
}

#[test]
fn a_registered_person_wait_opens_with_its_offer_or_the_older_default() {
    use skein_lib::{Duration, Wall};
    const WAITING: u64 = 7;
    const ADDRESS: u64 = 6;
    const HOLDER: u64 = 5;
    let offer = tasks::PersonOffer { holder: tasks::PersonAddress::Person(HOLDER), since: Wall::from_nanos(20) };
    for (sets, expected, older) in [(old_sets(), None, true), (CURRENT, Some(offer), false)] {
        let mut disk = Disk::new(&[]);
        let mut writer = store(&disk, sets);
        opened(&disk.open(&mut writer), false);
        let commit = disk.request(&mut writer, fixtures::commit(1, Box::new([fixtures::header(1)])));
        let [Event::Committed { number: 1, .. }] = commit.as_ref() else { panic!("registry commit: {commit:?}") };
        let Record::Tasks(tasks::Stored::Live(mut row)) = fixtures::live_task(WAITING) else { panic!("live row") };
        row.executor = tasks::Executor::Person(tasks::PersonAddress::Person(ADDRESS));
        row.phase = tasks::Phase::Active(tasks::Active::Due);
        row.taken_by = None;
        row.person_answer = None;
        let wait = tasks::PersonWait {
            since: Wall::from_nanos(10),
            stall: Duration::from_nanos(10),
            deadline: Wall::from_nanos(20),
            notice: Some(tasks::PersonStallNotice { message: 4, at: Wall::from_nanos(20), activity: 4 }),
            offer: expected,
        };
        row.person_wait = Some(wait);
        let cold = fixtures::cold_task(&row);
        let hot = Record::Tasks(tasks::Stored::Live(row));
        if older {
            let ops = [hot, cold]
                .iter()
                .map(|row| {
                    let (key, value) = encoded(&writer, row);
                    skein_kv::Op::Put { key, value: value.unwrap() }
                })
                .collect();
            raw(&mut disk, &mut writer, ops);
        } else {
            let commit = disk.request(
                &mut writer,
                fixtures::commit(2, Box::new([fixtures::header(2), fixtures::save(hot), fixtures::save(cold)])),
            );
            let [Event::Committed { number: 2, .. }] = commit.as_ref() else {
                panic!("offered wait commit: {commit:?}")
            };
        }
        let mut disk = Disk::new(&disk.items());
        let mut reader = store(&disk, CURRENT);
        let events = disk.open(&mut reader);
        assert!(events.len() == 2, "registered person wait from older={older}: {events:?}");
        opened(&events, older);
        let live = restore(&mut disk, &mut reader, Restore::Jig(Box::new(RestoreRange::LIVE)));
        let row = live
            .iter()
            .find_map(|row| {
                if let Record::Tasks(tasks::Stored::Live(row)) = row
                    && row.number == WAITING
                {
                    Some(row)
                } else {
                    None
                }
            })
            .expect("restored person task");
        assert_eq!(row.person_wait, Some(wait));
    }
}

#[test]
fn task_parts_read_a_nonzero_former_question_watermark_without_keeping_it() {
    const DROPPED: &str = "dropped question_woken: u64 = 0";
    let family = crate::STORE_FAMILIES.iter().find(|family| family.name == "jig-store-tasks").expect("tasks family");
    let previous = TEXTS.iter().find(|(name, _)| *name == family.name).expect("dev tasks schema").1;
    let current = schema::parse(family.current.as_bytes()).expect("current tasks schema");
    let old_layout = schema::parse(previous.as_bytes()).expect("previous layout");
    let nonzero = schema::parse(family.current.replace(DROPPED, "dropped question_woken: u64 = 37").as_bytes())
        .expect("fixture writer supplies the previous field at a nonzero value");
    let names = fixtures::configured_model_names(&[b"first", b"second", b"third"]);
    let limits = codec::tasks::limits(&fixtures::TASKS_LIMITS).expect("within the schema's ceilings");
    let mut record = fixtures::task_record();
    record.proposal = None;
    record.recurring = None;
    for (variant, row) in [
        (b"live_hot".as_slice(), tasks::Stored::Live(Box::new(record.clone()))),
        (b"ended_talk".as_slice(), tasks::Stored::Ended(Box::new(record))),
    ] {
        let current_bytes = codec::tasks::encode(&row, &limits, &names).expect("current record fits");
        let old_root = old_layout.root(b"Row", Some(variant)).expect("old variant");
        let new_root = current.root(b"Row", Some(variant)).expect("new variant");
        let back = Plan::between(Mode::Keep, &current, new_root, &[], &old_layout, old_root, &[])
            .expect("the tombstone supplies a downgrade default");
        let write_nonzero = Plan::between(
            Mode::Keep,
            &nonzero,
            nonzero.root(b"Row", Some(variant)).expect("fixture variant"),
            &[],
            &old_layout,
            old_root,
            &[],
        )
        .expect("encode an old value with the field present and nonzero");
        let zero_bytes = convert_question_layout(&back, &current_bytes);
        let old_bytes = convert_question_layout(&write_nonzero, &current_bytes);
        assert_ne!(old_bytes, zero_bytes, "the fixture really stores 37 rather than 0");
        assert_eq!(old_bytes.len(), zero_bytes.len());
        let upgrade = Plan::derive(&old_layout, old_root, &[], &current, new_root, &[])
            .expect("the store can read the previous layout");
        assert_eq!(convert_question_layout(&upgrade, &old_bytes), current_bytes);
    }
}

fn convert_question_layout(plan: &Plan, bytes: &[u8]) -> Box<[u8]> {
    const ROW_BYTES: u32 = 2048;
    let mut work = Workspace::new();
    assert!(work.fit(plan));
    let mut out = Buffer::with_capacity(ROW_BYTES);
    plan.convert(bytes, &mut out, &mut work).expect("convert layout");
    out.as_slice().into()
}

#[test]
fn old_stop_compound_and_leave_answers_open_with_the_default_reason() {
    use jig_domain_people as people;
    let mut disk = Disk::new(&[]);
    let mut old = store(&disk, old_sets());
    opened(&disk.open(&mut old), false);
    let events = disk.request(&mut old, fixtures::commit(1, Box::new([fixtures::header(1)])));
    let [Event::Committed { number: 1, .. }] = events.as_ref() else { panic!("first commit: {events:?}") };
    let mut operations = skein_lib::List::with_capacity(1);
    operations.push(people::Operation::Stop { task: 8, reason: Box::new([]) }).unwrap();
    let old_asks = [
        people::Ask::Stop { project: 3, task: 8, reason: Box::new([]) },
        people::Ask::Compound { project: 3, operations },
        people::Ask::Leave { project: 3, task: 8 },
    ];
    let rows = old_asks
        .into_iter()
        .enumerate()
        .map(|(index, ask)| {
            Record::People(people::Stored::Answer {
                key: people::RequestKey { person: ALICE, key: [u8::try_from(index).unwrap(); 16] },
                ask: Box::new(ask),
                models: Box::new([]),
                outcome: people::Outcome::Stopped { task: 8 },
                at: skein_lib::Wall::EPOCH,
            })
        })
        .collect::<Vec<_>>();
    let ops = rows
        .iter()
        .map(|row| {
            let (key, value) = encoded(&old, row);
            skein_kv::Op::Put { key, value: value.unwrap() }
        })
        .collect();
    raw(&mut disk, &mut old, ops);
    let mut disk = Disk::new(&disk.items());
    let mut current = store(&disk, CURRENT);
    opened(&disk.open(&mut current), true);
    assert_eq!(restore(&mut disk, &mut current, Restore::Jig(Box::new([RestoreRange::People]))), rows);
}
