//! Store load admission through the public domain API.
#![cfg(test)]

use temper_engine_domain::{Record, loads};

#[test]
fn load_validity_refuses_invalid_entrance_limits() {
    let limits = temper_limits_fixtures::small().root.loads;
    let record_bytes = u32::try_from(size_of::<Record>()).expect("record slot fits u32");
    for (name, limits) in [
        ("read bytes", loads::Limits { read_bytes: 0, ..limits }),
        ("load slots", loads::Limits { loads: 0, ..limits }),
        ("page rows", loads::Limits { rows: 0, ..limits }),
        ("reply below prefix", loads::Limits { reply_bytes: limits.bytes - 1, ..limits }),
        ("prefix below record slot", loads::Limits { bytes: record_bytes - 1, ..limits }),
    ] {
        assert!(!loads::valid(&limits), "{name}");
    }
}

#[test]
fn load_validity_accepts_one_record_slot_and_large_counts() {
    let limits = temper_limits_fixtures::small().root.loads;
    let record_bytes = u32::try_from(size_of::<Record>()).expect("record slot fits u32");
    for (name, limits) in [
        ("fixture", limits),
        (
            "minimum entrance",
            loads::Limits {
                read_bytes: 1,
                loads: 1,
                rows: 1,
                bytes: record_bytes,
                reply_bytes: record_bytes,
                transcript_bytes: 0,
            },
        ),
        (
            "maximum fields",
            loads::Limits {
                read_bytes: u64::MAX,
                loads: u32::MAX,
                rows: u32::MAX,
                bytes: u32::MAX,
                reply_bytes: u32::MAX,
                transcript_bytes: u32::MAX,
            },
        ),
    ] {
        assert!(loads::valid(&limits), "{name}");
    }
}
