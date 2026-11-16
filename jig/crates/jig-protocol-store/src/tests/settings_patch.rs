//! Current catalogue keys and serving decoding (protocol/store/opening.md, 2).
use super::fixtures;
use crate::{Event, Write, codec, keys};
use jig_domain_core_vocabulary::{self as core, settings as s};
use skein_lib::Wall;
fn key() -> core::Key {
    core::Key::Core(core::CoreKey::Settings(s::Key::Changed(s::Setting::Model(1))))
}

#[test]
fn a_changed_model_key_names_the_model_without_a_revision() {
    assert_eq!(keys::decode(&[1, 0, 8, 1, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]), Some(key()));
}

#[test]
fn open_restores_the_current_catalogue() {
    let mut disk = fixtures::disk::Disk::new(&[]);
    let mut store = store(&disk);
    let events = disk.open(&mut store);
    let [Event::Opened { last: 0, .. }, Event::Census { .. }] = events.as_ref() else { panic!("{events:?}") };
    let expected = model();
    let bytes = codec::settings::encode(&expected, &store.codec_limits.settings, &[]).expect("model encodes");
    let events = disk.request(
        &mut store,
        fixtures::commit(
            1,
            Box::new([
                fixtures::header(1),
                Write::Save(core::Record::Core(core::CoreRecord::Settings(expected.clone()))),
            ]),
        ),
    );
    let [Event::Committed { .. }] = events.as_ref() else { panic!("{events:?}") };
    let items = disk.items();
    let mut disk = fixtures::disk::Disk::new(&items);
    let mut store = self::store(&disk);
    let events = disk.open(&mut store);
    let [Event::Opened { pending: false, .. }, Event::Census { .. }] = events.as_ref() else { panic!("{events:?}") };
    assert_eq!(
        store.names.as_ref(),
        &[s::ModelName { name: b"kept".as_slice().into(), efforts: Box::new([b"high".as_slice().into()]) }]
    );
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

fn store(disk: &fixtures::disk::Disk) -> crate::Store {
    let mut limits = fixtures::store_limits();
    limits.store.budget = 2_097_152;
    limits.store.ops = 128;
    let mut store = disk.store_with_limits(&limits);
    store.patch_sets(crate::PatchSets {
        sets: &[crate::SKEIN_PATCH_SET, crate::SMITH_PATCH_SET, crate::JIG_STORE_PATCH_SET],
    });
    store
}
