//! Prepared fixtures retain the fresh values (jig/docs/design/protocol/channel/testing.md, 1).
use jig_protocol_worker_wire as wire;
use jig_worker_link_world::fixtures;

fn fresh_schema() -> skein_channel::Schema {
    let codec = wire::sizes::limits(&fixtures::SIZES).expect("small sizes");
    wire::schema(&codec).expect("small schema")
}

fn schema_after_changed_terms() -> skein_channel::Schema {
    let _engine = fixtures::engine_with_terms(0x0101, 1);
    fixtures::schema()
}

#[test]
fn a_prepared_schema_keeps_changed_part_bounds() {
    let mut sizes = fixtures::SIZES;
    sizes.turn += 1;
    let codec = wire::sizes::limits(&sizes).expect("changed part bound");
    assert_eq!(fixtures::schema_for_sizes(&sizes), wire::schema(&codec).expect("changed schema"));
    assert_eq!(fixtures::schema_for_sizes(&fixtures::SIZES), fresh_schema());
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
        ("schema copy after changed terms", schema_after_changed_terms() == fresh_schema()),
    ] {
        assert!(equal, "prepared fixture differs from fresh build: {row}");
    }
}
