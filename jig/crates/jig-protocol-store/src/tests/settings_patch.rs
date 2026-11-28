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

fn settings_schema() -> &'static str {
    for family in crate::STORE_FAMILIES {
        if family.name == "jig-store-settings" {
            return family.current;
        }
    }
    unreachable!("settings family")
}

#[test]
#[expect(clippy::disallowed_methods, reason = "derive the former settings layout from the current schema text")]
fn a_model_at_the_older_settings_layout_has_no_speeds() {
    use skein_codec::layout::{Mode, Plan, Workspace};
    use skein_codec::schema;
    use skein_lib::Buffer;

    let text = settings_schema();
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
#[expect(clippy::disallowed_macros, reason = "the test derives the plan from a fixture of the previous schema")]
fn stored_setting_names_default_to_none_at_the_previous_layout_and_round_trip_now() {
    use skein_codec::layout::{Mode, Plan, Workspace};
    use skein_codec::schema;
    use skein_lib::Buffer;

    let current = schema::parse(settings_schema().as_bytes()).unwrap();
    let old = schema::parse(include_str!("fixtures/settings-before-names-v3.schema").as_bytes()).unwrap();
    let current_root = current.root(b"Changed", None).unwrap();
    let old_root = old.root(b"Changed", None).unwrap();
    let write_old = Plan::between(Mode::Read, &current, current_root, &[], &old, old_root, &[]).unwrap();
    let read_old = Plan::between(Mode::Keep, &old, old_root, &[], &current, current_root, &[]).unwrap();
    let (limits, catalogue) = named_settings_codec();
    let limits = &limits;
    let mut workspace = Workspace::new();
    for (setting, mut value) in unnamed_settings() {
        let unnamed = s::Record::Changed { setting, changed: s::Changed { value: Some(value.clone()), revision: 10 } };
        value.set_name(Some(b"my-setting"));
        let named = s::Record::Changed { setting, changed: s::Changed { value: Some(value), revision: 10 } };
        let current_bytes = codec::settings::encode(&named, limits, &catalogue).unwrap();
        assert_eq!(codec::settings::decode(&named.key(), &current_bytes, limits, &catalogue), Ok(named.clone()));
        let mut old_bytes = Buffer::with_capacity(8192);
        let unnamed_bytes = codec::settings::encode(&unnamed, limits, &catalogue).unwrap();
        assert!(workspace.fit(&write_old));
        write_old.convert(&unnamed_bytes, &mut old_bytes, &mut workspace).unwrap();
        let mut converted = Buffer::with_capacity(8192);
        assert!(workspace.fit(&read_old));
        read_old.convert(old_bytes.as_slice(), &mut converted, &mut workspace).unwrap();
        assert_eq!(codec::settings::decode(&named.key(), converted.as_slice(), limits, &catalogue), Ok(unnamed));
    }
}

fn named_settings_codec() -> (crate::settings_v3::Limits, [s::ModelName; 2]) {
    let mut domain = fixtures::CORE_LIMITS;
    domain.authority.executors = 3;
    let limits = codec::settings::limits(&domain).unwrap();
    (limits, [super::settings::catalogue_name(b"model"), super::settings::catalogue_name(b"other")])
}

fn unnamed_settings() -> [(s::Setting, s::Value); 3] {
    [
        (s::Setting::Charter(1), s::Value::Charter(Box::new(super::settings::charter()))),
        (s::Setting::Template(1), s::Value::Template(Box::new(super::settings::template()))),
        (s::Setting::Policy(1), s::Value::Policy(Box::new(super::settings::policy()))),
    ]
}

#[test]
#[expect(clippy::disallowed_macros, reason = "derive the conversion plans from the retained dev layout")]
fn dev_defaults_and_changes_read_without_setting_names() {
    use skein_codec::{
        layout::{Mode, Plan, Workspace},
        schema,
    };
    use skein_lib::Buffer;

    const RECORD_BYTES: u32 = 8192; // Room for any of the three settings fixtures.
    const REVISION: u32 = 10;
    let current = schema::parse(settings_schema().as_bytes()).unwrap();
    let dev = schema::parse(include_str!("fixtures/dev-settings-v3.schema").as_bytes()).unwrap();
    let (limits, catalogue) = named_settings_codec();
    let limits = &limits;
    let mut workspace = Workspace::new();
    for (root, records) in [
        (
            b"Default".as_slice(),
            unnamed_settings().map(|(setting, value)| s::Record::Default {
                setting,
                default: s::Default { value, revision: REVISION },
            }),
        ),
        (
            b"Changed".as_slice(),
            unnamed_settings().map(|(setting, value)| s::Record::Changed {
                setting,
                changed: s::Changed { value: Some(value), revision: REVISION },
            }),
        ),
    ] {
        let current_root = current.root(root, None).unwrap();
        let dev_root = dev.root(root, None).unwrap();
        let write_dev = Plan::between(Mode::Read, &current, current_root, &[], &dev, dev_root, &[]).unwrap();
        let read_dev = Plan::between(Mode::Keep, &dev, dev_root, &[], &current, current_root, &[]).unwrap();
        for record in records {
            let bytes = codec::settings::encode(&record, limits, &catalogue).unwrap();
            let mut retained = Buffer::with_capacity(RECORD_BYTES);
            assert!(workspace.fit(&write_dev));
            write_dev.convert(&bytes, &mut retained, &mut workspace).unwrap();
            let mut converted = Buffer::with_capacity(RECORD_BYTES);
            assert!(workspace.fit(&read_dev));
            read_dev.convert(retained.as_slice(), &mut converted, &mut workspace).unwrap();
            assert_eq!(codec::settings::decode(&record.key(), converted.as_slice(), limits, &catalogue), Ok(record));
        }
    }
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
