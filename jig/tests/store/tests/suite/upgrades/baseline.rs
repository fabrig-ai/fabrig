//! A baseline row reaches the copy-and-patch path through the simulated disk.
use jig_protocol_store::{Owner, codec, keys};
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

#[test]
fn baseline_store_is_patched_from_zero_and_opens() {
    let baseline = &fixed::BASELINE;
    let limits = patches::limits();
    let format = codec::component::Format {
        application: root::APPLICATION.version,
        deployment: root::DEPLOYMENT,
        ..baseline.format
    };
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
    let mut records = patches::Records::story();
    records.tokens.clear();
    rows.extend(
        records
            .rows(1)
            .into_iter()
            .map(|(key, value)| (keys::application(&key, limits.store.key).expect("application key"), value)),
    );
    let mut world = World::unopened_observed(0xba5e, limits, None);
    world.lay(&[
        Item::directory(b"generations"),
        Item::directory(b"generations/1"),
        Item::directory(b"generations/1/store"),
        Item::directory(b"generations/1/secrets").mode(0o700),
        Item::file(b"generations/1/store/log-00000000000000000001", &source_commit(&rows)),
        Item::file(b"active", b"1\n"),
    ]);
    world.patch_sets(patches::SETS);
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
        codec::component::Levels { jig: Some(0), application: Some(0), skein: None, smith: None }
    );
}
