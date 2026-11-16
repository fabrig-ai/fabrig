//! A baseline row refuses at the floor through the simulated disk.
use jig_protocol_store::{OpenRefusal, PatchSets, codec, keys};
use jig_store_world::{World, patches, root};
use skein_fake_machine::Item;

// The production snapshot is hand-kept, not a generated codec or today's layouts.
mod fixed {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../crates/jig-protocol-store/src/baseline.rs"));
}

type Row = (Box<[u8]>, Box<[u8]>);

/// One source commit in skein's framed format (skein/docs/design/data/kv.md, 3).
pub(crate) fn source_commit(rows: &[Row], position: u64) -> Vec<u8> {
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
    frame.extend(position.to_be_bytes());
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
    let mut limits = patches::limits();
    // Registry Text rows retain the real families' kept schemas as whole values.
    for set in sets.sets {
        for family in set.families {
            for text in family.texts.iter().copied().chain([family.current]) {
                limits.store.value = limits.store.value.max(u32::try_from(text.len()).expect("schema bytes") + 6);
            }
        }
    }
    limits.store.page.bytes = limits.store.key + limits.store.value;
    limits.store.chunk = skein_kv::chunk_bytes(limits.store.key, limits.store.value).expect("registry row chunk");
    limits.store.commit = limits
        .store
        .commit
        .max(skein_kv::FRAME_BYTES + skein_kv::put_bytes(limits.store.key, limits.store.value).expect("schema put"));
    limits.store.queued_bytes = u64::from(limits.store.commit) * 2;
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
        Item::file(b"generations/1/store/log-00000000000000000001", &source_commit(&rows, 1)),
        Item::file(b"active", b"1\n"),
    ]);
    world.patch_sets(sets);
    world
}

const REAL_SETS: PatchSets = PatchSets {
    sets: &[
        jig_protocol_store::SKEIN_PATCH_SET,
        jig_protocol_store::SMITH_PATCH_SET,
        jig_protocol_accounts::JIG_PATCH_SET,
    ],
};

#[test]
fn baseline_rehearsal_refuses_the_floor_before_decoding() {
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
    let mut world = source_world(format, rows, REAL_SETS);
    let before = world.contents();
    let events = world.rehearse();
    assert!(
        matches!(
            events.as_slice(),
            [
                jig_protocol_store::Event::Rehearsing { fresh: false, .. },
                jig_protocol_store::Event::OpenRefused {
                    why: OpenRefusal::OlderBuild { why: jig_protocol_store::Older::Floor },
                    ..
                }
            ]
        ),
        "{events:?}"
    );
    assert_eq!(world.contents(), before);
    let closed = world.request(jig_protocol_store::Request::Close { owner: skein_lib::Token::new(900) });
    assert!(matches!(closed.as_slice(), [jig_protocol_store::Event::Closed { removed: true, .. }]));
    assert!(world.scratch_contents().is_empty());
    assert_eq!(world.contents(), before);
}
