//! Seed retained transcript places for tests of the file writer (protocol/store/logs.md, 5).
use jig_protocol_store::{Limits, Write, codec, format, keys};
use jig_store_world::{World, root};
use skein_fake_machine::Item;

pub(super) fn world(seed: u64, limits: &Limits, tasks: &[u64]) -> World {
    let own = |key| keys::encode_own(&key, limits.store.key).expect("component key");
    let wires = jig_protocol_store::component_v3::Wires::new(
        &jig_protocol_store::component_v3::CEILINGS,
        jig_protocol_store::component_v3::WiresParts { wires: skein_lib::List::with_capacity(0) },
    )
    .expect("empty wires");
    let mut encoded = skein_lib::Writer::new(skein_lib::usize_of(wires.measure()));
    wires.encode(&mut encoded).expect("measured wires");
    let mut rows = vec![
        (
            own(keys::Own::Format),
            format::current(root::APPLICATION.version, root::DEPLOYMENT).encode().expect("format"),
        ),
        (own(keys::Own::Layouts), format::layouts().encode()),
        (
            keys::installation(limits.store.key).expect("installation key"),
            codec::accounts::Installation { id: root::DEPLOYMENT }.encode(),
        ),
        (own(keys::Own::Wires), encoded.finish()),
    ];
    let mut writes = root::first().into_vec();
    if let Write::Save(jig_domain_core::Record::Tasks(jig_domain_tasks::Stored::Live(task))) = &writes[2] {
        writes.push(Write::Save(root::cold_task(task)));
    }
    let codec = codec::limits(&limits.core).expect("fixture codec");
    for write in &writes {
        if let Write::Save(record) = write {
            rows.push((
                keys::encode(&record.key(), limits.store.key).expect("record key"),
                codec::encode(record, &codec, &[]).expect("record"),
            ));
        }
    }
    for task in tasks {
        rows.push((
            own(keys::Own::Transcript { task: *task }),
            codec::component::Transcript { length: 0, window: 0 }.encode().expect("file place"),
        ));
    }
    let frame = super::upgrades::baseline::source_commit(&rows, 1);
    let mut world = World::unopened_observed(seed, *limits, None);
    world.lay(&[
        Item::directory(b"transcripts").mode(0o700),
        Item::directory(b"generations").mode(0o700),
        Item::directory(b"generations/1").mode(0o700),
        Item::directory(b"generations/1/store").mode(0o700),
        Item::directory(b"generations/1/secrets").mode(0o700),
        Item::file(b"generations/1/store/log-00000000000000000001", &frame).mode(0o600),
        Item::file(b"active", b"1\n").mode(0o600),
    ]);
    world.model.stage(1, &writes);
    world.model.acknowledge(1);
    assert_eq!(world.reopen().last, 1);
    world
}
