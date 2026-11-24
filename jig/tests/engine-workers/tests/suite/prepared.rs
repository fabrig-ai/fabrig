//! Prepared fixtures equal fresh builds (protocol/channel/testing.md, 1).
use jig_engine_workers_world::fixtures::{self, APPLICATION_VERSION, OTHER_VERSION};
use jig_protocol_worker_wire as wire;
use jig_protocol_worker_wire::Record;
use skein_lib::List;

fn fresh_statement(version: u32) -> List<wire::Wire> {
    let mut wires = List::with_capacity(16);
    for spoken in &wire::spoken() {
        wires
            .push(wire::Wire {
                name: spoken.name.as_bytes().into(),
                lowest: spoken.lowest,
                layouts: spoken.layouts.clone(),
            })
            .expect("wire slot");
    }
    let current = wire::Spoken::current("test-worker-wire", u16::try_from(version).expect("fixture version"), [3; 32]);
    wires
        .push(wire::Wire { name: current.name.as_bytes().into(), lowest: current.lowest, layouts: current.layouts })
        .expect("wire slot");
    wires
}

fn fresh_channel_limits() -> skein_channel::Limits {
    let codec = wire::sizes::limits(&fixtures::SIZES).expect("small codec");
    let cap = wire::sizes::worker_cap(&fixtures::SIZES).expect("small cap").cap;
    skein_channel::Limits {
        chunk: 64,
        credential: u32::try_from(wire::Credential::encoded_bound(&codec).expect("small credential")).expect("fits"),
        skip: 64,
        output_bytes: cap.bytes,
        output_frames: cap.frames,
        kinds: u32::try_from(wire::KINDS.len()).expect("kinds fit"),
    }
}

#[test]
fn a_prepared_schema_keeps_changed_part_bounds() {
    let mut sizes = fixtures::SIZES;
    sizes.turn += 1;
    let codec = wire::sizes::limits(&sizes).expect("changed part bound");
    assert_eq!(fixtures::schema_for_sizes(&sizes), wire::schema(&codec).expect("changed schema"));
    let codec = wire::sizes::limits(&fixtures::SIZES).expect("fixture bounds");
    assert_eq!(fixtures::schema_for_sizes(&fixtures::SIZES), wire::schema(&codec).expect("fixture schema"));
}

#[derive(Clone, Copy, Debug)]
enum Row {
    Schema,
    ChannelLimits,
    KeptWires,
    Statement(u32),
    ChangedStatementCopy,
}

#[test]
fn prepared_values_equal_fresh_builds_and_copies_are_independent() {
    for row in [
        Row::Schema,
        Row::ChannelLimits,
        Row::KeptWires,
        Row::Statement(APPLICATION_VERSION),
        Row::Statement(OTHER_VERSION),
        Row::ChangedStatementCopy,
    ] {
        let equal = match row {
            Row::Schema => {
                let codec = wire::sizes::limits(&fixtures::SIZES).expect("small codec");
                fixtures::schema() == wire::schema(&codec).expect("small schema")
            }
            Row::ChannelLimits => fixtures::channel_limits() == fresh_channel_limits(),
            Row::KeptWires => fixtures::kept_wires() == wire::spoken(),
            Row::Statement(version) => fixtures::statement(version) == fresh_statement(version),
            Row::ChangedStatementCopy => {
                let mut changed = fixtures::statement(APPLICATION_VERSION);
                let extra = changed.get(0).expect("first kept wire").clone();
                changed.push(extra).expect("wire slot");
                fixtures::statement(APPLICATION_VERSION) == fresh_statement(APPLICATION_VERSION)
            }
        };
        assert!(equal, "{row:?}: prepared value differs from fresh build");
    }
}
