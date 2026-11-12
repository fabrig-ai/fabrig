//! A baseline row reaches the copy-and-patch path through the simulated disk.
use jig_protocol_store::{Family, OpenRefusal, Owner, PatchSet, PatchSets, codec, keys};
use jig_store_world::{World, patches, root};
use skein_fake_machine::Item;

// The production snapshot is hand-kept, not a generated codec or today's layouts.
mod fixed {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../crates/jig-protocol-store/src/baseline.rs"));
}

type Row = (Box<[u8]>, Box<[u8]>);

/// One source commit in skein's framed format (skein/docs/design/data/kv.md, 3).
fn source_commit(rows: &[Row]) -> Vec<u8> {
    let mut body = Vec::new();
    for (key, value) in rows {
        body.push(1);
        body.extend(u32::try_from(key.len()).expect("key length").to_be_bytes());
        body.extend(key.as_ref());
        body.extend(u32::try_from(value.len()).expect("value length").to_be_bytes());
        body.extend(value.as_ref());
    }
    let mut frame = b"SKVC".to_vec();
    frame.extend(1_u16.to_be_bytes());
    frame.extend(1_u64.to_be_bytes());
    frame.extend(u32::try_from(rows.len()).expect("row count").to_be_bytes());
    frame.extend(u32::try_from(body.len()).expect("body length").to_be_bytes());
    frame.extend(body);
    let mut crc = !0_u32;
    for byte in &frame {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ if crc & 1 == 0 { 0 } else { 0x82f6_3b78 };
        }
    }
    frame.extend((!crc).to_be_bytes());
    frame
}

fn source_world(format: codec::component::Format, extra: Vec<Row>, sets: jig_protocol_store::PatchSets) -> World {
    let baseline = &fixed::BASELINE;
    let limits = patches::limits();
    let own = |key| keys::encode_own(&key, limits.store.key).expect("own key");
    let header = root::header(1);
    let mut rows = vec![
        (own(keys::Own::Format), format.encode().expect("baseline format")),
        (own(keys::Own::Layouts), baseline.layouts.encode()),
        (
            keys::installation(limits.store.key).expect("installation key"),
            codec::accounts::Installation { id: root::DEPLOYMENT }.encode(),
        ),
        (
            keys::encode(&header.key(), limits.store.key).expect("header key"),
            codec::encode(&header, &codec::limits(&limits.core).expect("limits"), &[]).expect("header"),
        ),
    ];
    rows.extend(extra);
    let mut world = World::unopened_observed(0xba5e, limits, None);
    world.lay(&[
        Item::directory(b"transcripts").mode(0o700),
        Item::directory(b"generations"),
        Item::directory(b"generations/1"),
        Item::directory(b"generations/1/store"),
        Item::directory(b"generations/1/secrets").mode(0o700),
        Item::file(b"generations/1/store/log-00000000000000000001", &source_commit(&rows)),
        Item::file(b"active", b"1\n"),
    ]);
    world.patch_sets(sets);
    world
}

// Keep the fixture's changes and sites, with the real jig changes that its
// baseline format row now needs. The reviewed fixture patch files do not change.
fn baseline_fixture_sets() -> PatchSets {
    let jig = patches::SETS.sets[1];
    let families =
        jig.families.iter().chain(jig_protocol_accounts::JIG_PATCH_SET.families).copied().collect::<Vec<_>>();
    let (_, blocks) = jig_protocol_store::patches::PATCHES[0].split_once("\n\n").expect("real jig blocks");
    let first: &'static str = Box::leak(format!("{}\n{blocks}", jig.patches[0]).into_boxed_str());
    let texts = Box::leak(vec![first, jig.patches[1]].into_boxed_slice());
    PatchSets {
        sets: Box::leak(Box::new([
            jig_protocol_store::SKEIN_PATCH_SET,
            jig_protocol_store::SMITH_PATCH_SET,
            patches::SETS.sets[0],
            PatchSet { patches: texts, families: Box::leak(families.into_boxed_slice()), ..jig },
        ])),
    }
}

const REAL_SETS: PatchSets = PatchSets {
    sets: &[
        jig_protocol_store::SKEIN_PATCH_SET,
        jig_protocol_store::SMITH_PATCH_SET,
        jig_protocol_accounts::JIG_PATCH_SET,
    ],
};

#[test]
fn real_jig_set_versions_match_each_level() {
    for (level, format) in [
        (0, fixed::BASELINE.format),
        (1, codec::component::Format { tasks: 10, people: 15, settings: 2, ..fixed::BASELINE.format }),
        (2, jig_protocol_store::format::current(root::APPLICATION.version, root::DEPLOYMENT)),
    ] {
        let mut expected = codec::component::Format {
            application: root::APPLICATION.version,
            deployment: root::DEPLOYMENT,
            ..format
        };
        expected.levels =
            codec::component::Levels { skein: Some(0), smith: Some(0), jig: Some(level), application: None };
        let mut valid = source_world(expected, Vec::new(), REAL_SETS);
        assert!(valid.open_switched().is_some(), "jig level {level}: {:?}", valid.refusal());
        for family in [
            Family::Component,
            Family::Core,
            Family::Tasks,
            Family::People,
            Family::Notes,
            Family::Accounts,
            Family::Settings,
        ] {
            let mut bad = expected;
            let version = match family {
                Family::Component => &mut bad.component,
                Family::Core => &mut bad.core,
                Family::Tasks => &mut bad.tasks,
                Family::People => &mut bad.people,
                Family::Notes => &mut bad.notes,
                Family::Accounts => &mut bad.accounts,
                Family::Settings => &mut bad.settings,
                _ => unreachable!("format family"),
            };
            let wanted = u32::from(*version);
            *version = version.checked_add(1).expect("one past this family's level");
            let found = u32::from(*version);
            let mut world = source_world(bad, Vec::new(), REAL_SETS);
            assert_eq!(world.open_switched(), None, "{family:?} at level {level}");
            assert_eq!(world.refusal(), Some(&OpenRefusal::Format { family, found, wanted }));
        }
    }
}

#[test]
fn baseline_rehearsal_decodes_first_kept_versions_before_patches() {
    let limits = patches::limits();
    let format = codec::component::Format {
        application: root::APPLICATION.version,
        deployment: root::DEPLOYMENT,
        ..fixed::BASELINE.format
    };
    let rows = patches::Records::story()
        .rows(1)
        .into_iter()
        .map(|(key, value)| (keys::application(&key, limits.store.key).expect("application key"), value))
        .collect();
    let mut world = source_world(format, rows, baseline_fixture_sets());
    let before = world.contents();
    let events = world.rehearse();
    assert!(matches!(events.get(1), Some(jig_protocol_store::Event::Decoded { records: 8, .. })), "{events:?}");
    assert!(matches!(events.last(), Some(jig_protocol_store::Event::Opened { pending: true, .. })), "{events:?}");
    assert_eq!(world.contents(), before);
    let closed = world.request(jig_protocol_store::Request::Close { owner: skein_lib::Token::new(900) });
    assert!(matches!(closed.as_slice(), [jig_protocol_store::Event::Closed { removed: true, .. }]));
    assert!(world.scratch_contents().is_empty());
    assert_eq!(world.contents(), before);
}

#[test]
fn baseline_store_is_patched_from_zero_and_opens() {
    let baseline = &fixed::BASELINE;
    let limits = patches::limits();
    let format = codec::component::Format {
        application: root::APPLICATION.version,
        deployment: root::DEPLOYMENT,
        ..baseline.format
    };
    let mut records = patches::Records::story();
    records.tokens.clear();
    let rows = records
        .rows(1)
        .into_iter()
        .map(|(key, value)| (keys::application(&key, limits.store.key).expect("application key"), value))
        .collect();
    let mut world = source_world(format, rows, baseline_fixture_sets());
    let opened = world.open_switched();
    assert!(opened.is_some(), "baseline opens through patching: {:?}", world.refusal());
    assert_eq!(world.active(), b"2\n");
    assert_eq!(
        world.patched().iter().map(|(owner, patch, _)| (*owner, *patch)).collect::<Vec<_>>(),
        [(Owner::Application, 1), (Owner::Jig, 1), (Owner::Application, 2), (Owner::Jig, 2),]
    );
    records.check(&mut world, 2, &"baseline patched from zero");
    let manifest = world.contents().remove(b"generations/2/manifest".as_slice()).flatten().expect("patched manifest");
    let manifest = codec::component::Manifest::decode(&manifest).expect("manifest");
    assert_eq!(
        manifest.before,
        codec::component::Levels { jig: Some(0), application: Some(0), skein: Some(0), smith: Some(0) }
    );
}
