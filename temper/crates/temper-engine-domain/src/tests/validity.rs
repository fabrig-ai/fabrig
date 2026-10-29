use crate::{JournalLimits, Record, loads};

// All journal fields at their type's maximum; validity allocates no state.
const MAXIMUM_JOURNAL_LIMITS: JournalLimits = JournalLimits {
    commits: u32::MAX,
    held: u32::MAX,
    writes: u32::MAX,
    deliveries: u32::MAX,
    transcript_bytes: u32::MAX,
    result_bytes: u32::MAX,
    run_bytes: u32::MAX,
};

#[test]
fn journal_validity_refuses_zero_entrance_limits() {
    let limits = super::LIMITS;
    for (name, limits) in [
        ("commits", JournalLimits { commits: 0, ..limits }),
        ("writes", JournalLimits { writes: 0, ..limits }),
        ("deliveries", JournalLimits { deliveries: 0, ..limits }),
        ("run bytes", JournalLimits { run_bytes: 0, ..limits }),
    ] {
        assert!(!crate::journal_valid(&limits), "{name}");
    }
}

#[test]
fn journal_validity_accepts_the_header_slot_and_large_counts() {
    let limits = super::LIMITS;
    for (name, limits) in [
        ("fixture", limits),
        ("header only", JournalLimits { writes: 1, held: 0, transcript_bytes: 0, result_bytes: 0, ..limits }),
        ("maximum fields", MAXIMUM_JOURNAL_LIMITS),
    ] {
        assert!(crate::journal_valid(&limits), "{name}");
    }
}

#[test]
fn load_validity_refuses_invalid_entrance_limits() {
    let limits = super::resume_fixture::limits().loads;
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
    let limits = super::resume_fixture::limits().loads;
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

#[test]
fn root_validity_refuses_invalid_child_limits() {
    let limits = super::resume_fixture::limits();
    for (name, limits) in [
        ("journal", crate::Limits { journal: JournalLimits { writes: 0, ..limits.journal }, ..limits }),
        ("loads", crate::Limits { loads: loads::Limits { read_bytes: 0, ..limits.loads }, ..limits }),
        (
            "reply below prefix",
            crate::Limits { loads: loads::Limits { reply_bytes: limits.loads.bytes - 1, ..limits.loads }, ..limits },
        ),
        (
            "forge",
            crate::Limits { forge: temper_engine_domain_forge::Limits { repositories: 0, ..limits.forge }, ..limits },
        ),
    ] {
        assert!(!crate::valid(&limits), "{name}");
    }
}

#[test]
fn root_validity_accepts_dynamic_internal_counts() {
    let limits = super::resume_fixture::limits();
    for (name, limits) in [
        (
            "uncapped forge output",
            crate::Limits { forge: temper_engine_domain_forge::Limits { output: 0, ..limits.forge }, ..limits },
        ),
        ("maximum journal fields", crate::Limits { journal: MAXIMUM_JOURNAL_LIMITS, ..limits }),
    ] {
        assert!(crate::valid(&limits), "{name}");
    }
}
