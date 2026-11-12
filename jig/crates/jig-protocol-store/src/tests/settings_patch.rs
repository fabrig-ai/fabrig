//! A retained catalogue is read only at the binary's levels (protocol/store/opening.md, 2).
use super::fixtures;
use crate::{Event, KeptFamily, KeptMap, Keys, Owner, PatchSet, PatchSets, codec, format, keys, patches};
use jig_domain_core_vocabulary::{self as core, settings as s};
use skein_codec::patch::{Capacity, Patcher, Step};
use skein_lib::{Queue, Token, Wall};

const SETS: PatchSets = PatchSets {
    sets: &[PatchSet {
        owner: Owner::Jig,
        name: "jig",
        fresh: None,
        families: &[KeptFamily {
            name: "jig-store-settings",
            texts: patches::kept::jig_store_settings::TEXTS,
            map: KeptMap::Store,
            keys: &[Keys { from: &[1, 0, 8, 1, 6], to: &[1, 0, 8, 1, 7], root: "Changed" }],
            frames: None,
            flights: &[],
        }],
        patches: patches::PATCHES,
    }],
};

fn key() -> core::Key {
    core::Key::Core(core::CoreKey::Settings(s::Key::Changed(s::Setting::Model(1))))
}

#[test]
fn a_changed_model_key_names_the_model_without_a_revision() {
    assert_eq!(keys::decode(&[1, 0, 8, 1, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]), Some(key()));
}

fn store(value: &[u8], level: u32) -> crate::Store {
    let mut stores = fixtures::stores();
    fixtures::plant(&mut stores, &rows(value, level));
    let mut store = fixtures::over(stores);
    store.patch_sets(SETS);
    store
}

// Format, layouts, header, installation and one changed model.
const ROWS: usize = 5;

fn rows(value: &[u8], level: u32) -> [skein_kv::Op; ROWS] {
    let mut row = format::current(fixtures::APPLICATION.version, fixtures::DEPLOYMENT);
    row.levels.jig = Some(level);
    if level == 0 {
        row.settings = 1;
    }
    [
        skein_kv::Op::Put {
            key: keys::encode_own(&keys::Own::Format, fixtures::KEY_LIMIT).expect("format key"),
            value: row.encode().expect("format row"),
        },
        fixtures::layouts_put(),
        fixtures::header_put(fixtures::DEPLOYMENT, 1),
        fixtures::installation_put(fixtures::INSTALLATION),
        skein_kv::Op::Put { key: keys::encode(&key(), fixtures::KEY_LIMIT).expect("model key"), value: value.into() },
    ]
}

fn open(store: &mut crate::Store) -> Box<[Event]> {
    let mut above = Queue::uncapped();
    crate::open::open(store, Token::new(1), &mut above, &mut fixtures::below());
    fixtures::events(&mut above)
}

#[test]
fn behind_open_defers_the_v1_model_catalogue_until_patching() {
    let mut store = store(OLD_MODEL, 0);
    let events = open(&mut store);
    let [Event::Opened { .. }] = events.as_ref() else {
        panic!("{events:?}; {:?}", store.start_open_failure());
    };
    assert!(store.directory.patch.is_some(), "the old generation enters patching");
    assert!(store.names.is_empty(), "the old model is not read by the serving decoder");
}

fn patched() -> Box<[u8]> {
    let mut patcher = Patcher::new(Capacity {
        versions: 3,
        patches: 2,
        text: skein_codec::schema::MAX_TEXT,
        bytes: 4096,
        nodes: 256,
    });
    for text in patches::kept::jig_store_settings::TEXTS {
        patcher.keep(text.as_bytes()).expect("kept settings schema");
    }
    let mut bytes = OLD_MODEL.into();
    let mut context = patches::Context::default();
    for (version, text) in [(1, patches::PATCHES[0]), (2, patches::PATCHES[1])] {
        let root = patcher.schema(version).expect("kept schema").find(b"Changed").expect("changed record");
        let plan = patcher.plan(text.as_bytes()).expect("valid patch").expect("settings block");
        patcher.start(plan, root, bytes).expect("kept model record");
        loop {
            match patcher.next().expect("patch step") {
                Step::Written { site, at } => {
                    let (old, new) = patcher.parts();
                    patches::written(site, old, new, at, &mut context).expect("written patch");
                }
                Step::Held { .. } => panic!("a catalogue model holds no other family"),
                Step::Done => break,
            }
        }
        bytes = patcher.finish().expect("patched model").into();
    }
    bytes
}

#[test]
fn u1_patches_a_v1_model_then_open_restores_its_catalogue() {
    let bytes = patched();
    let mut store = store(&bytes, 2);
    let events = open(&mut store);
    let [Event::Opened { .. }] = events.as_ref() else {
        panic!("{events:?}");
    };
    assert_eq!(
        store.names.as_ref(),
        &[s::ModelName { name: b"kept".as_slice().into(), efforts: Box::new([b"high".as_slice().into()]) }]
    );
    let expected = model();
    assert_eq!(codec::settings::decode(&expected.key(), &bytes, &store.codec_limits.settings, &[]), Ok(expected));
}

fn model() -> s::Record {
    s::Record::Changed {
        setting: s::Setting::Model(1),
        changed: s::Changed {
            revision: 10,
            value: Some(s::Value::Model(Box::new(s::CatalogueModel {
                number: 1,
                model: core::Model {
                    provider: 1,
                    dialect: 2,
                    endpoint: 3,
                    program: None,
                    name: b"kept".as_slice().into(),
                    window: 32768,
                    cache_life_ms: 0,
                    output: 4096,
                    input_price: 11,
                    cached_price: 7,
                    cache_write_price: Some(9),
                    output_price: 17,
                    price_unit: 1_000_000,
                    efforts: Box::new([b"high".as_slice().into()]),
                    effort: Some(b"high".as_slice().into()),
                },
                shown: Some(b"Kept".as_slice().into()),
                description: Some(b"Old model".as_slice().into()),
                picker: true,
                withdrawn: false,
                source: s::ModelSource::Person,
                estimated: true,
                first_read: Some(Wall::from_nanos(123)),
            }))),
        },
    }
}

#[test]
fn a_disk_upgrade_patches_the_model_before_loading_its_catalogue() {
    use fixtures::disk::Disk;
    // Write v1 rows directly through KV, independently of today's typed encoder.
    let mut disk = Disk::new(&[]);
    let mut source = disk.store();
    disk.open(&mut source);
    let mut above = Queue::uncapped();
    let mut below = fixtures::below();
    let env = crate::directory::env(&source, &fixtures::env(), false);
    skein_kv::down(
        &mut source.stores.store,
        &env,
        skein_kv::Request::Commit { owner: Token::new(1), ops: Box::new(rows(OLD_MODEL, 0)) },
        &mut Queue::uncapped(),
        &mut below.store,
    );
    disk.drive(&mut below, |io, below| crate::up(&mut source, &fixtures::env(), io, &mut above, below));
    let mut disk = Disk::new(&disk.items());
    let mut store = disk.store();
    store.patch_sets(SETS);
    let events = disk.open(&mut store);
    let Some(Event::Opened { pending: true, .. }) = events.last() else {
        panic!("{events:?}; {:?}", store.start_open_failure());
    };
    assert_eq!(disk.bytes(b"active").as_ref(), b"1\n", "the patched copy awaits its switch");
    let events = disk.request(&mut store, crate::Request::Get { owner: Token::new(2), key: crate::Get::Jig(key()) });
    let [Event::Got { row: Some(crate::Row::Jig(core::Record::Core(core::CoreRecord::Settings(found)))), .. }] =
        events.as_ref()
    else {
        panic!("patched model row: {events:?}");
    };
    assert_eq!(*found, model());
    assert_eq!(
        store.names.as_ref(),
        &[s::ModelName { name: b"kept".as_slice().into(), efforts: Box::new([b"high".as_slice().into()]) }]
    );
}

// Literal v1 Changed bytes, following the kept baseline schema; no current encoder.
const OLD_MODEL: &[u8] = &[
    // Changed: version 1, empty vocabulary, present model value.
    0, 1, 0, 0, 0, 0, 1, 4, // Catalogue model 1; provider, dialect, endpoint, no program.
    0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, // Model name, window, output (no cache life field).
    0, 0, 0, 4, 107, 101, 112, 116, 0, 0, 128, 0, 0, 0, 16, 0,
    // Input, cached, cache-write and output prices; price unit.
    0, 0, 0, 0, 0, 0, 0, 11, 0, 0, 0, 0, 0, 0, 0, 7, 1, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 0, 0, 0, 0, 0, 17, 0, 15, 66, 64,
    // Efforts and chosen effort.
    0, 0, 0, 1, 0, 0, 0, 4, 104, 105, 103, 104, 1, 0, 0, 0, 4, 104, 105, 103, 104,
    // Shown name, description, picker, withdrawn, person, estimated, first read.
    1, 0, 0, 0, 4, 75, 101, 112, 116, 1, 0, 0, 0, 9, 79, 108, 100, 32, 109, 111, 100, 101, 108, 1, 0, 2, 1, 1, 0, 0, 0,
    0, 0, 0, 0, 123, // Settings revision 10.
    0, 0, 0, 10,
];
