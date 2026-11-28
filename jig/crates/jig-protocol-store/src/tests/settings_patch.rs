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
    let mut expected = model();
    let s::Record::Changed { changed, .. } = &mut expected else { panic!("changed model fixture") };
    let Some(s::Value::Model(model)) = &mut changed.value else { panic!("model fixture") };
    model.model.speeds = Box::new([core::Speed {
        name: b"fast".as_slice().into(),
        also: Box::new([b"priority".as_slice().into()]),
        input_price: 22,
        cached_price: 14,
        cache_write_price: None,
        output_price: 34,
    }]);
    model.model.speed = Some(b"fast".as_slice().into());
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
                    speeds: Box::default(),
                    speed: None,
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
#[expect(clippy::disallowed_methods, reason = "derive the former settings layout from the current schema text")]
fn a_model_at_the_older_settings_layout_has_no_speeds() {
    use skein_codec::layout::{Mode, Plan, Workspace};
    use skein_codec::schema;
    use skein_lib::Buffer;

    let family = crate::STORE_FAMILIES.iter().find(|family| family.name == "jig-store-settings").unwrap();
    let text = family.current;
    let speed_record = text.lines().find(|line| line.starts_with("record Speed ")).unwrap();
    let old_text =
        text.replace(speed_record, "").replace(" speeds: list 4 Speed = [] speed: option bytes 32 = none", "");
    let current = schema::parse(text.as_bytes()).unwrap();
    let old = schema::parse(old_text.as_bytes()).unwrap();
    assert!(current.find(b"Speed").is_some());
    assert!(old.find(b"Speed").is_none());
    let current_root = current.root(b"Changed", None).unwrap();
    let old_root = old.root(b"Changed", None).unwrap();
    let limits = &codec::limits(&fixtures::CORE_LIMITS).unwrap().settings;
    let expected = model();
    let current_bytes = codec::settings::encode(&expected, limits, &[]).unwrap();
    let write_old = Plan::between(Mode::Read, &current, current_root, &[], &old, old_root, &[]).unwrap();
    let read_old = Plan::between(Mode::Keep, &old, old_root, &[], &current, current_root, &[]).unwrap();
    let mut workspace = Workspace::new();
    let mut old_bytes = Buffer::with_capacity(8192);
    assert!(workspace.fit(&write_old));
    write_old.convert(&current_bytes, &mut old_bytes, &mut workspace).unwrap();
    let mut converted = Buffer::with_capacity(8192);
    assert!(workspace.fit(&read_old));
    read_old.convert(old_bytes.as_slice(), &mut converted, &mut workspace).unwrap();
    assert_eq!(codec::settings::decode(&expected.key(), converted.as_slice(), limits, &[]), Ok(expected));
}

fn store(disk: &fixtures::disk::Disk) -> crate::Store {
    let mut store = disk.store_with_limits(&fixtures::registered_store_limits());
    store.owner_sets(crate::OwnerSets { sets: &[crate::SKEIN_SET, crate::SMITH_SET, crate::JIG_STORE_SET] });
    store
}
#[test]
#[expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "a layout test compares the former schema fixtures with their current family"
)]
fn a_model_choice_at_the_previous_layout_has_no_speed() {
    use skein_codec::layout::{Mode, Plan, Workspace};
    use skein_codec::schema;
    use skein_lib::Buffer;

    const OLD: &[u8] = &[7, 1, 2]; // Model seven, effort Some(two).
    const CURRENT: &[u8] = &[7, 1, 2, 0]; // The extra zero is the absent speed.
    for (family, old) in [
        ("jig-store-tasks", include_str!("fixtures/dev-tasks-v13.schema")),
        ("jig-store-people", include_str!("fixtures/dev-people-v17.schema")),
        ("jig-store-settings", include_str!("fixtures/dev-settings-v3.schema")),
    ] {
        let latest = crate::STORE_FAMILIES.iter().find(|found| found.name == family).unwrap();
        let old = schema::parse(old.as_bytes()).unwrap();
        let current = schema::parse(latest.current.as_bytes()).unwrap();
        let root = b"ModelEffort";
        let old_root = old.root(root, None).unwrap();
        let current_root = current.root(root, None).unwrap();
        let read = Plan::between(Mode::Keep, &old, old_root, &[], &current, current_root, &[]).unwrap();
        let mut workspace = Workspace::new();
        let mut converted = Buffer::with_capacity(u32::try_from(CURRENT.len()).unwrap());
        assert!(workspace.fit(&read));
        read.convert(OLD, &mut converted, &mut workspace).unwrap();
        assert_eq!(converted.as_slice(), CURRENT, "{family}");
    }
}
