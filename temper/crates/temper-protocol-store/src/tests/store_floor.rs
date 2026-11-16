//! Dev's layouts settle at F0 (jig protocol/store/jobs.md, 6).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary Rust disk fixtures"
)]
use super::{
    fixtures, jobs,
    unseeded_upgrade::{SETS, component, disk::Disk, header},
};
use crate::{PATCH_SET, records};
use jig_protocol_store::{self as store, Event, Get, Request, Row, Write, registry_v1 as w};
use skein_lib::{List, Reader, Token, Wall, Writer};
use temper_engine_domain_forge_vocabulary::{self as d, Stored, history as h};

const DEV: &str = include_str!("cb429613c6-store-v25.schema");
const KEPT: &str = include_str!("../../schema/kept/temper-store-v25.schema");
fn old_sets() -> store::PatchSets {
    store::PatchSets {
        sets: Box::leak(
            SETS.sets
                .iter()
                .map(|set| {
                    if set.name != "temper" {
                        return *set;
                    }
                    store::PatchSet {
                        families: Box::leak(
                            set.families
                                .iter()
                                .map(|family| store::KeptFamily {
                                    current: if family.name == "temper-store" { DEV } else { family.current },
                                    ..*family
                                })
                                .collect::<Vec<_>>()
                                .into_boxed_slice(),
                        ),
                        jobs: &PATCH_SET.jobs[..1],
                        ..*set
                    }
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        ),
    }
}
fn layout_key(number: u16, id: u16) -> Box<[u8]> {
    store::keys::encode_own(&store::keys::Own::Layout { family: "temper-store".into(), number, id }, 1024).unwrap()
}
fn layouts(text: &str, id: u16) -> Vec<skein_kv::Op> {
    let schema = skein_codec::schema::parse(text.as_bytes()).unwrap();
    let mut hash = skein_lib::sha256::Sha256::new();
    hash.update(text.as_bytes());
    let hash = hash.finish();
    let text_row = w::Text::new(&w::CEILINGS, w::TextParts { text: text.as_bytes().into() }).unwrap();
    let mut out = Writer::new(usize::try_from(text_row.measure()).unwrap());
    text_row.encode(&mut out).unwrap();
    let mut ops = vec![skein_kv::Op::Put {
        key: store::keys::encode_own(&store::keys::Own::Text(hash), 1024).unwrap(),
        value: out.finish(),
    }];
    for ty in PATCH_SET.families.iter().flat_map(|family| family.types).filter(|ty| (41..=45).contains(&ty.number)) {
        let variant = ty.variant.map(str::as_bytes);
        let row = w::Layout::new(
            &w::CEILINGS,
            w::LayoutParts {
                root: b"Row".as_slice().into(),
                variant: variant.map(Into::into),
                layout: schema.layout(schema.root(b"Row", variant).unwrap(), &[]).unwrap().digest(),
                text: hash,
                held: List::with_capacity(0),
            },
        )
        .unwrap();
        let mut out = Writer::new(usize::try_from(row.measure()).unwrap());
        row.encode(&mut out).unwrap();
        ops.push(skein_kv::Op::Put { key: layout_key(ty.number, id), value: out.finish() });
    }
    ops
}
fn history(project: u32) -> [Stored; 5] {
    [
        Stored::HistoryProject(h::Project {
            project,
            next: 2,
            kept: 1,
            whole_since: Some(Wall::from_nanos(3)),
            marks: Box::new([]),
            sweeping: None,
            removing: false,
        }),
        Stored::HistoryEvent(h::Event {
            project,
            number: 1,
            at: Wall::from_nanos(3),
            source_at: None,
            change: 7,
            repository: fixtures::REPO,
            landing: None,
            head: Some([8; 32]),
            base: None,
            operation: 9,
            attempt: 1,
            what: h::What::Ready,
        }),
        Stored::HistoryChange(h::ChangeIndex { project, change: 7, number: 1, landing: None }),
        Stored::HistoryLanding(h::LandingIndex { project, landing: 11, number: 1, change: 7 }),
        Stored::HistoryLast(h::Last { project, number: 1, change: 7, at: Wall::from_nanos(3) }),
    ]
}
fn expected() -> Vec<Stored> {
    history(1).into_iter().chain(history(2)).chain([jobs::landing(11, b"")]).collect()
}
fn old_bytes(row: &Stored, text: &str, id: u16) -> Box<[u8]> {
    let current = skein_codec::schema::parse(include_str!("../../schema/store-v25.schema").as_bytes()).unwrap();
    let old = skein_codec::schema::parse(text.as_bytes()).unwrap();
    let key = jobs::encoded(row).key;
    let ty = PATCH_SET
        .families
        .iter()
        .flat_map(|family| family.types)
        .find(|ty| ty.keys.iter().any(|range| key.as_ref() >= range.from && key.as_ref() < range.to))
        .unwrap();
    let variant = ty.variant.map(str::as_bytes);
    let plan = skein_codec::layout::Plan::derive_dropping(
        &current,
        current.root(b"Row", variant).unwrap(),
        &[],
        &old,
        old.root(b"Row", variant).unwrap(),
        &[],
    )
    .unwrap();
    let mut work = skein_codec::layout::Workspace::new();
    assert!(work.fit(&plan));
    let mut out = skein_lib::Buffer::with_capacity(131_072);
    plan.convert(&records::encode(row), &mut out, &mut work).unwrap();
    let mut bytes = out.as_slice().to_vec();
    bytes[..2].copy_from_slice(&id.to_be_bytes());
    bytes.into()
}
fn baseline(retired: bool) -> Disk {
    let mut disk = Disk::new();
    let mut old = component(&disk, old_sets());
    let events = disk.open(&mut old);
    let [Event::Opened { last: 0, .. }, Event::Census { .. }] = events.as_slice() else { panic!("{events:?}") };
    let events = disk.request(&mut old, Request::Commit { number: 1, writes: Box::new([Write::Save(header(1))]) });
    let [Event::Committed { .. }] = events.as_slice() else { panic!("{events:?}") };
    drop(old);
    disk.restart();
    let mut map = disk.map(1);
    let codec = store::codec::limits(&disk.limits().core).unwrap();
    let mut ops = vec![skein_kv::Op::Put {
        key: store::keys::encode(&header(2).key(), 1024).unwrap(),
        value: store::codec::encode(&header(2), &codec, &[]).unwrap(),
    }];
    ops.extend(layouts(KEPT, 25));
    ops.extend(layouts(DEV, 26));
    for (at, row) in expected().iter().enumerate() {
        let (text, id) = if at < 5 {
            (KEPT, 25)
        } else if at < 10 {
            (DEV, 26)
        } else {
            (DEV, 25)
        };
        ops.push(skein_kv::Op::Put {
            key: store::keys::application(&jobs::encoded(row).key, 1024).unwrap(),
            value: old_bytes(row, text, id),
        });
    }
    if retired {
        let value = super::catch_up::kept_golden(25, "enum_value_change_smallest.bin");
        ops.push(skein_kv::Op::Put {
            key: store::keys::application(&crate::keys::encode(&d::Key::Change(0), 1024).unwrap(), 1024).unwrap(),
            value: [&25_u16.to_be_bytes()[..], &value].concat().into(),
        });
    }
    let events = disk.kv(&mut map, skein_kv::Request::Commit { owner: Token::new(1), ops: ops.into() });
    let [skein_kv::Event::Committed { .. }] = events.as_slice() else { panic!("{events:?}") };
    disk.kv(&mut map, skein_kv::Request::Close { owner: Token::new(1) });
    drop(map);
    disk.restart();
    disk
}
fn open(disk: &mut Disk, current: &mut store::Store) -> Vec<Event> {
    let mut pending = alloc::collections::VecDeque::from(disk.open(current));
    let mut found = Vec::new();
    while let Some(event) = pending.pop_front() {
        if let Event::Job { owner, of: store::Owner::Application, job, page } = event {
            let bounds = crate::bounds(&fixtures::LIMITS, disk.limits().core.tasks.spec_bytes).unwrap();
            let answer = crate::jobs::page(&job, &page, &crate::jobs::Present::default(), &bounds);
            pending.extend(disk.request(current, Request::Job { owner, answer }));
        } else {
            found.push(event);
        }
    }
    found
}
#[test]
fn dev_layouts_25_and_26_settle_and_round_branches_counts_and_reopen_writes_nothing() {
    let mut disk = baseline(false);
    let mut current = component(&disk, SETS);
    let events = open(&mut disk, &mut current);
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
    assert_eq!(ran.len(), 2, "{events:?}");
    assert_eq!(ran[0].0, "settle-layouts");
    assert_eq!(ran[0].1[0], ("put", 11));
    assert_eq!(ran[1], ("round-branches", vec![("empty", 1), ("named", 0)]));
    let bounds = crate::bounds(&fixtures::LIMITS, disk.limits().core.tasks.spec_bytes).unwrap();
    for row in expected() {
        let events = disk.request(
            &mut current,
            Request::Get { owner: Token::new(1), key: Get::Application(jobs::encoded(&row).key) },
        );
        let [Event::Got { row: Some(Row::Application { key, value }), .. }] = events.as_slice() else {
            panic!("{events:?}")
        };
        assert_eq!(records::row(key, value, &bounds), Ok(row));
    }
    let events = disk.request(&mut current, Request::Switch { owner: Token::new(1) });
    let [Event::Switched { .. }] = events.as_slice() else { panic!("{events:?}") };
    drop(current);
    disk.restart();
    let mut map = disk.map(2);
    let schemas = SETS
        .sets
        .iter()
        .flat_map(|set| set.families)
        .filter(|family| !family.texts.is_empty())
        .map(|family| skein_codec::schema::parse(family.current.as_bytes()).unwrap())
        .collect::<Vec<_>>();
    let held = schemas.iter().collect::<Vec<_>>();
    let schema = skein_codec::schema::parse(include_str!("../../schema/store-v25.schema").as_bytes()).unwrap();
    for ty in PATCH_SET.families.iter().flat_map(|family| family.types) {
        let prefix = layout_key(ty.number, 0);
        let prefix = &prefix[..prefix.len() - 2];
        let events = disk.kv(
            &mut map,
            skein_kv::Request::Load {
                owner: Token::new(1),
                range: skein_kv::Range::prefix(prefix),
                max: disk.limits().store.page,
            },
        );
        let [skein_kv::Event::Loaded { rows, next: None, .. }] = events.as_slice() else { panic!("{events:?}") };
        let layouts = rows;
        assert_eq!(layouts.len(), 1, "value type {}", ty.number);
        let layout = w::Layout::decode(&w::CEILINGS, &mut Reader::new(&layouts[0].value)).unwrap();
        assert_eq!(layout.root(), b"Row");
        assert_eq!(layout.variant().as_deref(), ty.variant.map(str::as_bytes));
        assert_eq!(
            *layout.layout(),
            schema.layout(schema.root(b"Row", ty.variant.map(str::as_bytes)).unwrap(), &held).unwrap().digest(),
            "value type {}",
            ty.number
        );
    }
    disk.kv(&mut map, skein_kv::Request::Close { owner: Token::new(1) });
    drop(map);
    disk.restart();
    let before = disk.contents();
    let mut again = component(&disk, SETS);
    let events = disk.open(&mut again);
    let [Event::Opened { pending: false, .. }, Event::Census { .. }] = events.as_slice() else { panic!("{events:?}") };
    assert_eq!(disk.contents(), before);
}
#[test]
fn a_retired_change_refuses_layout_before_any_write() {
    let mut disk = baseline(true);
    let before = disk.contents();
    let mut current = component(&disk, SETS);
    let events = disk.open(&mut current);
    let [
        Event::OpenRefused {
            why: store::OpenRefusal::Layout { family, value_type: 12, id: 25, why: store::Unplanned::Retired, .. },
            ..
        },
    ] = events.as_slice()
    else {
        panic!("{events:?}")
    };
    assert_eq!(family.as_ref(), "temper-store");
    assert_eq!(disk.contents(), before);
}
