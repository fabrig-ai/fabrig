//! Prepared fixtures retain the fresh values (jig/docs/design/protocol/channel/README.md, 11).
use jig_protocol_worker_wire as wire;
use jig_worker_link_world::fixtures;
use skein_lib::{List, Writer};

fn fresh_schema() -> skein_channel::Schema {
    let codec = wire::sizes::limits(&fixtures::SIZES).expect("small sizes");
    wire::schema(&codec).expect("small schema")
}

fn fresh_agreement() -> skein_channel::Frame {
    let limits = wire::sizes::limits(&fixtures::SIZES).expect("small codec limits");
    let mut wires = List::with_capacity(5);
    for spoken in &wire::spoken() {
        wires
            .push(
                wire::AgreedWire::new(
                    &limits,
                    wire::AgreedWireParts { name: spoken.name.as_bytes().into(), version: spoken.highest() },
                )
                .expect("wire"),
            )
            .expect("four jig wires");
    }
    wires
        .push(
            wire::AgreedWire::new(
                &limits,
                wire::AgreedWireParts { name: b"test-worker-wire".as_slice().into(), version: 3 },
            )
            .expect("wire"),
        )
        .expect("application wire");
    let record = wire::v14::Agreed::new(&limits, wire::v14::AgreedParts { wires }).expect("agreement");
    let mut body = Writer::new(skein_lib::length::usize_of(record.measure()));
    record.encode(&mut body).expect("measured agreement");
    let mut frame = skein_channel::frame_writer(0x018d, record.measure()).expect("frame");
    frame.put(&body.finish()).expect("body");
    frame.finish().expect("measured frame")
}

fn schema_after_changed_terms() -> skein_channel::Schema {
    let _engine = fixtures::engine_with_terms(0x0101, 1);
    fixtures::schema()
}

#[test]
fn prepared_values_equal_fresh_builds() {
    for (row, equal) in [
        ("schema", fixtures::schema() == fresh_schema()),
        (
            "channel limits",
            fixtures::channel_limits()
                == jig_protocol_worker_link::channel_limits(&fixtures::LIMITS).expect("small channel"),
        ),
        ("kept wires", fixtures::spoken() == wire::spoken()),
        ("agreement bytes", fixtures::agreement().bytes() == fresh_agreement().bytes()),
        ("schema copy after changed terms", schema_after_changed_terms() == fresh_schema()),
    ] {
        assert!(equal, "prepared fixture differs from fresh build: {row}");
    }
}
